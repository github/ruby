// This file is a fragment of the yjit.o compilation unit. See yjit.c.
#include "internal.h"
#include "gc.h"
#include "internal/compile.h"
#include "internal/class.h"
#include "internal/hash.h"
#include "internal/object.h"
#include "internal/sanitizers.h"
#include "internal/string.h"
#include "internal/struct.h"
#include "internal/variable.h"
#include "internal/re.h"
#include "probes.h"
#include "probes_helper.h"
#include "yjit.h"
#include "yjit_iface.h"
#include "yjit_core.h"
#include "yjit_codegen.h"
#include "yjit_asm.h"

// Map from YARV opcodes to code generation functions
static codegen_fn gen_fns[VM_INSTRUCTION_SIZE] = { NULL };

// Map from method entries to code generation functions
static st_table *yjit_method_codegen_table = NULL;

// Code for exiting back to the interpreter from the leave instruction
static void *leave_exit_code;

// Code for full logic of returning from C method and exiting to the interpreter
static uint32_t outline_full_cfunc_return_pos;

// For implementing global code invalidation
struct codepage_patch {
    uint32_t inline_patch_pos;
    uint32_t outlined_target_pos;
};

typedef rb_darray(struct codepage_patch) patch_array_t;

static patch_array_t global_inval_patches = NULL;

// Print the current source location for debugging purposes
RBIMPL_ATTR_MAYBE_UNUSED()
static void
jit_print_loc(jitstate_t *jit, const char *msg)
{
    char *ptr;
    long len;
    VALUE path = rb_iseq_path(jit->iseq);
    RSTRING_GETMEM(path, ptr, len);
    fprintf(stderr, "%s %.*s:%u\n", msg, (int)len, ptr, rb_iseq_line_no(jit->iseq, jit->insn_idx));
}

// dump an object for debugging purposes
RBIMPL_ATTR_MAYBE_UNUSED()
static void
jit_obj_info_dump(codeblock_t *cb, x86opnd_t opnd) {
    push_regs(cb);
    mov(cb, C_ARG_REGS[0], opnd);
    call_ptr(cb, REG0, (void *)rb_obj_info_dump);
    pop_regs(cb);
}

// Get the current instruction's opcode
static int
jit_get_opcode(jitstate_t *jit)
{
    return jit->opcode;
}

// Get the index of the next instruction
static uint32_t
jit_next_insn_idx(jitstate_t *jit)
{
    return jit->insn_idx + insn_len(jit_get_opcode(jit));
}

// Get an instruction argument by index
static VALUE
jit_get_arg(jitstate_t *jit, size_t arg_idx)
{
    RUBY_ASSERT(arg_idx + 1 < (size_t)insn_len(jit_get_opcode(jit)));
    return *(jit->pc + arg_idx + 1);
}

// Load a VALUE into a register and keep track of the reference if it is on the GC heap.
static void
jit_mov_gc_ptr(jitstate_t *jit, codeblock_t *cb, x86opnd_t reg, VALUE ptr)
{
    RUBY_ASSERT(reg.type == OPND_REG && reg.num_bits == 64);

    // Load the pointer constant into the specified register
    mov(cb, reg, const_ptr_opnd((void*)ptr));

    // The pointer immediate is encoded as the last part of the mov written out
    uint32_t ptr_offset = cb->write_pos - sizeof(VALUE);

    if (!SPECIAL_CONST_P(ptr)) {
        if (!rb_darray_append(&jit->block->gc_object_offsets, ptr_offset)) {
            rb_bug("allocation failed");
        }
    }
}

// Check if we are compiling the instruction at the stub PC
// Meaning we are compiling the instruction that is next to execute
static bool
jit_at_current_insn(jitstate_t *jit)
{
    const VALUE *ec_pc = jit->ec->cfp->pc;
    return (ec_pc == jit->pc);
}

// Peek at the nth topmost value on the Ruby stack.
// Returns the topmost value when n == 0.
static VALUE
jit_peek_at_stack(jitstate_t *jit, ctx_t *ctx, int n)
{
    RUBY_ASSERT(jit_at_current_insn(jit));

    // Note: this does not account for ctx->sp_offset because
    // this is only available when hitting a stub, and while
    // hitting a stub, cfp->sp needs to be up to date in case
    // codegen functions trigger GC. See :stub-sp-flush:.
    VALUE *sp = jit->ec->cfp->sp;

    return *(sp - 1 - n);
}

static VALUE
jit_peek_at_self(jitstate_t *jit, ctx_t *ctx)
{
    return jit->ec->cfp->self;
}

RBIMPL_ATTR_MAYBE_UNUSED()
static VALUE
jit_peek_at_local(jitstate_t *jit, ctx_t *ctx, int n)
{
    RUBY_ASSERT(jit_at_current_insn(jit));

    int32_t local_table_size = jit->iseq->body->local_table_size;
    RUBY_ASSERT(n < (int)jit->iseq->body->local_table_size);

    const VALUE *ep = jit->ec->cfp->ep;
    return ep[-VM_ENV_DATA_SIZE - local_table_size + n + 1];
}

// Save the incremented PC on the CFP
// This is necessary when calleees can raise or allocate
static void
jit_save_pc(jitstate_t *jit, x86opnd_t scratch_reg)
{
    codeblock_t *cb = jit->cb;
    mov(cb, scratch_reg, const_ptr_opnd(jit->pc + insn_len(jit->opcode)));
    mov(cb, mem_opnd(64, REG_CFP, offsetof(rb_control_frame_t, pc)), scratch_reg);
}

// Save the current SP on the CFP
// This realigns the interpreter SP with the JIT SP
// Note: this will change the current value of REG_SP,
//       which could invalidate memory operands
static void
jit_save_sp(jitstate_t *jit, ctx_t *ctx)
{
    if (ctx->sp_offset != 0) {
        x86opnd_t stack_pointer = ctx_sp_opnd(ctx, 0);
        codeblock_t *cb = jit->cb;
        lea(cb, REG_SP, stack_pointer);
        mov(cb, member_opnd(REG_CFP, rb_control_frame_t, sp), REG_SP);
        ctx->sp_offset = 0;
    }
}

// jit_save_pc() + jit_save_sp(). Should be used before calling a routine that
// could:
//  - Perform GC allocation
//  - Take the VM lock through RB_VM_LOCK_ENTER()
//  - Perform Ruby method call
static void
jit_prepare_routine_call(jitstate_t *jit, ctx_t *ctx, x86opnd_t scratch_reg)
{
    jit->record_boundary_patch_point = true;
    jit_save_pc(jit, scratch_reg);
    jit_save_sp(jit, ctx);
}

// Record the current codeblock write position for rewriting into a jump into
// the outlined block later. Used to implement global code invalidation.
static void
record_global_inval_patch(const codeblock_t *cb, uint32_t outline_block_target_pos)
{
    struct codepage_patch patch_point = { cb->write_pos, outline_block_target_pos };
    if (!rb_darray_append(&global_inval_patches, patch_point)) rb_bug("allocation failed");
}

static bool jit_guard_known_klass(jitstate_t *jit, ctx_t *ctx, VALUE known_klass, insn_opnd_t insn_opnd, VALUE sample_instance, const int max_chain_depth, uint8_t *side_exit);

#if YJIT_STATS

// Add a comment at the current position in the code block
static void
_add_comment(codeblock_t *cb, const char *comment_str)
{
    // We can't add comments to the outlined code block
    if (cb == ocb)
        return;

    // Avoid adding duplicate comment strings (can happen due to deferred codegen)
    size_t num_comments = rb_darray_size(yjit_code_comments);
    if (num_comments > 0) {
        struct yjit_comment last_comment = rb_darray_get(yjit_code_comments, num_comments - 1);
        if (last_comment.offset == cb->write_pos && strcmp(last_comment.comment, comment_str) == 0) {
            return;
        }
    }

    struct yjit_comment new_comment = (struct yjit_comment){ cb->write_pos, comment_str };
    rb_darray_append(&yjit_code_comments, new_comment);
}

// Comments for generated machine code
#define ADD_COMMENT(cb, comment) _add_comment((cb), (comment))

// Verify the ctx's types and mappings against the compile-time stack, self,
// and locals.
static void
verify_ctx(jitstate_t *jit, ctx_t *ctx)
{
    // Only able to check types when at current insn
    RUBY_ASSERT(jit_at_current_insn(jit));

    VALUE self_val = jit_peek_at_self(jit, ctx);
    if (type_diff(yjit_type_of_value(self_val), ctx->self_type) == INT_MAX) {
        rb_bug("verify_ctx: ctx type (%s) incompatible with actual value of self: %s", yjit_type_name(ctx->self_type), rb_obj_info(self_val));
    }

    for (int i = 0; i < ctx->stack_size && i < MAX_TEMP_TYPES; i++) {
        temp_type_mapping_t learned = ctx_get_opnd_mapping(ctx, OPND_STACK(i));
        VALUE val = jit_peek_at_stack(jit, ctx, i);
        val_type_t detected = yjit_type_of_value(val);

        if (learned.mapping.kind == TEMP_SELF) {
            if (self_val != val) {
                rb_bug("verify_ctx: stack value was mapped to self, but values did not match\n"
                        "  stack: %s\n"
                        "  self: %s",
                        rb_obj_info(val),
                        rb_obj_info(self_val));
            }
        }

        if (learned.mapping.kind == TEMP_LOCAL) {
            int local_idx = learned.mapping.idx;
            VALUE local_val = jit_peek_at_local(jit, ctx, local_idx);
            if (local_val != val) {
                rb_bug("verify_ctx: stack value was mapped to local, but values did not match\n"
                        "  stack: %s\n"
                        "  local %i: %s",
                        rb_obj_info(val),
                        local_idx,
                        rb_obj_info(local_val));
            }
        }

        if (type_diff(detected, learned.type) == INT_MAX) {
            rb_bug("verify_ctx: ctx type (%s) incompatible with actual value on stack: %s", yjit_type_name(learned.type), rb_obj_info(val));
        }
    }

    int32_t local_table_size = jit->iseq->body->local_table_size;
    for (int i = 0; i < local_table_size && i < MAX_TEMP_TYPES; i++) {
        val_type_t learned = ctx->local_types[i];
        VALUE val = jit_peek_at_local(jit, ctx, i);
        val_type_t detected = yjit_type_of_value(val);

        if (type_diff(detected, learned) == INT_MAX) {
            rb_bug("verify_ctx: ctx type (%s) incompatible with actual value of local: %s", yjit_type_name(learned), rb_obj_info(val));
        }
    }
}

#else

#define ADD_COMMENT(cb, comment) ((void)0)
#define verify_ctx(jit, ctx) ((void)0)

#endif // if YJIT_STATS

#if YJIT_STATS

// Increment a profiling counter with counter_name
#define GEN_COUNTER_INC(cb, counter_name) _gen_counter_inc(cb, &(yjit_runtime_counters . counter_name))
static void
_gen_counter_inc(codeblock_t *cb, int64_t *counter)
{
    if (!rb_yjit_opts.gen_stats) return;

    // Use REG1 because there might be return value in REG0
    mov(cb, REG1, const_ptr_opnd(counter));
    cb_write_lock_prefix(cb); // for ractors.
    add(cb, mem_opnd(64, REG1, 0), imm_opnd(1));
}

// Increment a counter then take an existing side exit.
#define COUNTED_EXIT(jit, side_exit, counter_name) _counted_side_exit(jit, side_exit, &(yjit_runtime_counters . counter_name))
static uint8_t *
_counted_side_exit(jitstate_t* jit, uint8_t *existing_side_exit, int64_t *counter)
{
    if (!rb_yjit_opts.gen_stats) return existing_side_exit;

    uint8_t *start = cb_get_ptr(jit->ocb, jit->ocb->write_pos);
    _gen_counter_inc(jit->ocb, counter);
    jmp_ptr(jit->ocb, existing_side_exit);
    return start;
}

#else

#define GEN_COUNTER_INC(cb, counter_name) ((void)0)
#define COUNTED_EXIT(jit, side_exit, counter_name) side_exit

#endif // if YJIT_STATS

// Generate an exit to return to the interpreter
static uint32_t
yjit_gen_exit(VALUE *exit_pc, ctx_t *ctx, codeblock_t *cb)
{
    const uint32_t code_pos = cb->write_pos;

    ADD_COMMENT(cb, "exit to interpreter");

    // Generate the code to exit to the interpreters
    // Write the adjusted SP back into the CFP
    if (ctx->sp_offset != 0) {
        x86opnd_t stack_pointer = ctx_sp_opnd(ctx, 0);
        lea(cb, REG_SP, stack_pointer);
        mov(cb, member_opnd(REG_CFP, rb_control_frame_t, sp), REG_SP);
    }

    // Update CFP->PC
    mov(cb, RAX, const_ptr_opnd(exit_pc));
    mov(cb, member_opnd(REG_CFP, rb_control_frame_t, pc), RAX);

    // Accumulate stats about interpreter exits
#if YJIT_STATS
    if (rb_yjit_opts.gen_stats) {
        mov(cb, RDI, const_ptr_opnd(exit_pc));
        call_ptr(cb, RSI, (void *)&yjit_count_side_exit_op);
    }
#endif

    pop(cb, REG_SP);
    pop(cb, REG_EC);
    pop(cb, REG_CFP);

    mov(cb, RAX, imm_opnd(Qundef));
    ret(cb);

    return code_pos;
}

// Generate a continuation for gen_leave() that exits to the interpreter at REG_CFP->pc.
static uint8_t *
yjit_gen_leave_exit(codeblock_t *cb)
{
    uint8_t *code_ptr = cb_get_ptr(cb, cb->write_pos);

    // Note, gen_leave() fully reconstructs interpreter state and leaves the
    // return value in RAX before coming here.

    // Every exit to the interpreter should be counted
    GEN_COUNTER_INC(cb, leave_interp_return);

    pop(cb, REG_SP);
    pop(cb, REG_EC);
    pop(cb, REG_CFP);

    ret(cb);

    return code_ptr;
}

// Fill code_for_exit_from_stub. This is used by branch_stub_hit() to exit
// to the interpreter when it cannot service a stub by generating new code.
// Before coming here, branch_stub_hit() takes care of fully reconstructing
// interpreter state.
static void
gen_code_for_exit_from_stub(void)
{
    codeblock_t *cb = ocb;
    code_for_exit_from_stub = cb_get_ptr(cb, cb->write_pos);

    GEN_COUNTER_INC(cb, exit_from_branch_stub);

    pop(cb, REG_SP);
    pop(cb, REG_EC);
    pop(cb, REG_CFP);

    mov(cb, RAX, imm_opnd(Qundef));
    ret(cb);
}

// :side-exit:
// Get an exit for the current instruction in the outlined block. The code
// for each instruction often begins with several guards before proceeding
// to do work. When guards fail, an option we have is to exit to the
// interpreter at an instruction boundary. The piece of code that takes
// care of reconstructing interpreter state and exiting out of generated
// code is called the side exit.
//
// No guards change the logic for reconstructing interpreter state at the
// moment, so there is one unique side exit for each context. Note that
// it's incorrect to jump to the side exit after any ctx stack push/pop operations
// since they change the logic required for reconstructing interpreter state.
static uint8_t *
yjit_side_exit(jitstate_t *jit, ctx_t *ctx)
{
    if (!jit->side_exit_for_pc) {
        codeblock_t *ocb = jit->ocb;
        uint32_t pos = yjit_gen_exit(jit->pc, ctx, ocb);
        jit->side_exit_for_pc = cb_get_ptr(ocb, pos);
    }

    return jit->side_exit_for_pc;
}

// Ensure that there is an exit for the start of the block being compiled.
// Block invalidation uses this exit.
static void
jit_ensure_block_entry_exit(jitstate_t *jit)
{
    block_t *block = jit->block;
    if (block->entry_exit) return;

    if (jit->insn_idx == block->blockid.idx) {
        // We are compiling the first instruction in the block.
        // Generate the exit with the cache in jitstate.
        block->entry_exit = yjit_side_exit(jit, &block->ctx);
    }
    else {
        VALUE *pc = yjit_iseq_pc_at_idx(block->blockid.iseq, block->blockid.idx);
        uint32_t pos = yjit_gen_exit(pc, &block->ctx, ocb);
        block->entry_exit = cb_get_ptr(ocb, pos);
    }
}

// Generate a runtime guard that ensures the PC is at the start of the iseq,
// otherwise take a side exit.  This is to handle the situation of optional
// parameters.  When a function with optional parameters is called, the entry
// PC for the method isn't necessarily 0, but we always generated code that
// assumes the entry point is 0.
static void
yjit_pc_guard(codeblock_t *cb, const rb_iseq_t *iseq)
{
    RUBY_ASSERT(cb != NULL);

    mov(cb, REG0, member_opnd(REG_CFP, rb_control_frame_t, pc));
    mov(cb, REG1, const_ptr_opnd(iseq->body->iseq_encoded));
    xor(cb, REG0, REG1);

    // xor should impact ZF, so we can jz here
    uint32_t pc_is_zero = cb_new_label(cb, "pc_is_zero");
    jz_label(cb, pc_is_zero);

    // We're not starting at the first PC, so we need to exit.
    GEN_COUNTER_INC(cb, leave_start_pc_non_zero);

    pop(cb, REG_SP);
    pop(cb, REG_EC);
    pop(cb, REG_CFP);

    mov(cb, RAX, imm_opnd(Qundef));
    ret(cb);

    // PC should be at the beginning
    cb_write_label(cb, pc_is_zero);
    cb_link_labels(cb);
}

// The code we generate in gen_send_cfunc() doesn't fire the c_return TracePoint event
// like the interpreter. When tracing for c_return is enabled, we patch the code after
// the C method return to call into this to fire the event.
static void
full_cfunc_return(rb_execution_context_t *ec, VALUE return_value)
{
    rb_control_frame_t *cfp = ec->cfp;
    RUBY_ASSERT_ALWAYS(cfp == GET_EC()->cfp);
    const rb_callable_method_entry_t *me = rb_vm_frame_method_entry(cfp);

    RUBY_ASSERT_ALWAYS(RUBYVM_CFUNC_FRAME_P(cfp));
    RUBY_ASSERT_ALWAYS(me->def->type == VM_METHOD_TYPE_CFUNC);

    // CHECK_CFP_CONSISTENCY("full_cfunc_return"); TODO revive this

    // Pop the C func's frame and fire the c_return TracePoint event
    // Note that this is the same order as vm_call_cfunc_with_frame().
    rb_vm_pop_frame(ec);
    EXEC_EVENT_HOOK(ec, RUBY_EVENT_C_RETURN, cfp->self, me->def->original_id, me->called_id, me->owner, return_value);
    // Note, this deviates from the interpreter in that users need to enable
    // a c_return TracePoint for this DTrace hook to work. A reasonable change
    // since the Ruby return event works this way as well.
    RUBY_DTRACE_CMETHOD_RETURN_HOOK(ec, me->owner, me->def->original_id);

    // Push return value into the caller's stack. We know that it's a frame that
    // uses cfp->sp because we are patching a call done with gen_send_cfunc().
    ec->cfp->sp[0] = return_value;
    ec->cfp->sp++;
}

// Landing code for when c_return tracing is enabled. See full_cfunc_return().
static void
gen_full_cfunc_return(void)
{
    codeblock_t *cb = ocb;
    outline_full_cfunc_return_pos = ocb->write_pos;

    // This chunk of code expect REG_EC to be filled properly and
    // RAX to contain the return value of the C method.

    // Call full_cfunc_return()
    mov(cb, C_ARG_REGS[0], REG_EC);
    mov(cb, C_ARG_REGS[1], RAX);
    call_ptr(cb, REG0, (void *)full_cfunc_return);

    // Count the exit
    GEN_COUNTER_INC(cb, traced_cfunc_return);

    // Return to the interpreter
    pop(cb, REG_SP);
    pop(cb, REG_EC);
    pop(cb, REG_CFP);

    mov(cb, RAX, imm_opnd(Qundef));
    ret(cb);
}

/*
Compile an interpreter entry block to be inserted into an iseq
Returns `NULL` if compilation fails.
*/
static uint8_t *
yjit_entry_prologue(codeblock_t *cb, const rb_iseq_t *iseq)
{
    RUBY_ASSERT(cb != NULL);

    enum { MAX_PROLOGUE_SIZE = 1024 };

    // Check if we have enough executable memory
    if (cb->write_pos + MAX_PROLOGUE_SIZE >= cb->mem_size) {
        return NULL;
    }

    const uint32_t old_write_pos = cb->write_pos;

    // Align the current write position to cache line boundaries
    cb_align_pos(cb, 64);

    uint8_t *code_ptr = cb_get_ptr(cb, cb->write_pos);
    ADD_COMMENT(cb, "yjit entry");

    push(cb, REG_CFP);
    push(cb, REG_EC);
    push(cb, REG_SP);

    // We are passed EC and CFP
    mov(cb, REG_EC, C_ARG_REGS[0]);
    mov(cb, REG_CFP, C_ARG_REGS[1]);

    // Load the current SP from the CFP into REG_SP
    mov(cb, REG_SP, member_opnd(REG_CFP, rb_control_frame_t, sp));

    // Setup cfp->jit_return
    // TODO: this could use an IP relative LEA instead of an 8 byte immediate
    mov(cb, REG0, const_ptr_opnd(leave_exit_code));
    mov(cb, member_opnd(REG_CFP, rb_control_frame_t, jit_return), REG0);

    // We're compiling iseqs that we *expect* to start at `insn_idx`. But in
    // the case of optional parameters, the interpreter can set the pc to a
    // different location depending on the optional parameters.  If an iseq
    // has optional parameters, we'll add a runtime check that the PC we've
    // compiled for is the same PC that the interpreter wants us to run with.
    // If they don't match, then we'll take a side exit.
    if (iseq->body->param.flags.has_opt) {
        yjit_pc_guard(cb, iseq);
    }

    // Verify MAX_PROLOGUE_SIZE
    RUBY_ASSERT_ALWAYS(cb->write_pos - old_write_pos <= MAX_PROLOGUE_SIZE);

    return code_ptr;
}




#if 0

static int tracing_invalidate_all_i(void *vstart, void *vend, size_t stride, void *data);
static void invalidate_all_blocks_for_tracing(const rb_iseq_t *iseq);

// Invalidate all generated code and patch C method return code to contain
// logic for firing the c_return TracePoint event. Once rb_vm_barrier()
// returns, all other ractors are pausing inside RB_VM_LOCK_ENTER(), which
// means they are inside a C routine. If there are any generated code on-stack,
// they are waiting for a return from a C routine. For every routine call, we
// patch in an exit after the body of the containing VM instruction. This makes
// it so all the invalidated code exit as soon as execution logically reaches
// the next VM instruction. The interpreter takes care of firing the tracing
// event if it so happens that the next VM instruction has one attached.
//
// The c_return event needs special handling as our codegen never outputs code
// that contains tracing logic. If we let the normal output code run until the
// start of the next VM instruction by relying on the patching scheme above, we
// would fail to fire the c_return event. The interpreter doesn't fire the
// event at an instruction boundary, so simply exiting to the interpreter isn't
// enough. To handle it, we patch in the full logic at the return address. See
// full_cfunc_return().
//
// In addition to patching, we prevent future entries into invalidated code by
// removing all live blocks from their iseq.
void
rb_yjit_tracing_invalidate_all(void)
{
    if (!rb_yjit_enabled_p()) return;

    // Stop other ractors since we are going to patch machine code.
    RB_VM_LOCK_ENTER();
    rb_vm_barrier();

    // Make it so all live block versions are no longer valid branch targets
    rb_objspace_each_objects(tracing_invalidate_all_i, NULL);

    // Apply patches
    const uint32_t old_pos = cb->write_pos;
    rb_darray_for(global_inval_patches, patch_idx) {
        struct codepage_patch patch = rb_darray_get(global_inval_patches, patch_idx);
        cb_set_pos(cb, patch.inline_patch_pos);
        uint8_t *jump_target = cb_get_ptr(ocb, patch.outlined_target_pos);
        jmp_ptr(cb, jump_target);
    }
    cb_set_pos(cb, old_pos);

    // Freeze invalidated part of the codepage. We only want to wait for
    // running instances of the code to exit from now on, so we shouldn't
    // change the code. There could be other ractors sleeping in
    // branch_stub_hit(), for example. We could harden this by changing memory
    // protection on the frozen range.
    RUBY_ASSERT_ALWAYS(yjit_codepage_frozen_bytes <= old_pos && "frozen bytes should increase monotonically");
    yjit_codepage_frozen_bytes = old_pos;

    cb_mark_all_executable(ocb);
    cb_mark_all_executable(cb);
    RB_VM_LOCK_LEAVE();
}

static int
tracing_invalidate_all_i(void *vstart, void *vend, size_t stride, void *data)
{
    VALUE v = (VALUE)vstart;
    for (; v != (VALUE)vend; v += stride) {
        void *ptr = asan_poisoned_object_p(v);
        asan_unpoison_object(v, false);

        if (rb_obj_is_iseq(v)) {
            rb_iseq_t *iseq = (rb_iseq_t *)v;
            invalidate_all_blocks_for_tracing(iseq);
        }

        asan_poison_object_if(ptr, v);
    }
    return 0;
}

static void
invalidate_all_blocks_for_tracing(const rb_iseq_t *iseq)
{
    struct rb_iseq_constant_body *body = iseq->body;
    if (!body) return; // iseq yet to be initialized

    ASSERT_vm_locking();

    // Empty all blocks on the iseq so we don't compile new blocks that jump to the
    // invalidted region.
    // TODO Leaking the blocks for now since we might have situations where
    // a different ractor is waiting in branch_stub_hit(). If we free the block
    // that ractor can wake up with a dangling block.
    rb_darray_for(body->yjit_blocks, version_array_idx) {
        rb_yjit_block_array_t version_array = rb_darray_get(body->yjit_blocks, version_array_idx);
        rb_darray_for(version_array, version_idx) {
            // Stop listening for invalidation events like basic operation redefinition.
            block_t *block = rb_darray_get(version_array, version_idx);
            yjit_unlink_method_lookup_dependency(block);
            yjit_block_assumptions_free(block);
        }
        rb_darray_free(version_array);
    }
    rb_darray_free(body->yjit_blocks);
    body->yjit_blocks = NULL;

#if USE_MJIT
    // Reset output code entry point
    body->jit_func = NULL;
#endif
}

#endif // #if 0

static void
yjit_reg_op(int opcode, codegen_fn gen_fn)
{
    RUBY_ASSERT(opcode >= 0 && opcode < VM_INSTRUCTION_SIZE);
    // Check that the op wasn't previously registered
    RUBY_ASSERT(gen_fns[opcode] == NULL);

    gen_fns[opcode] = gen_fn;
}

void
yjit_init_codegen(void)
{
    // Initialize the code blocks
    uint32_t mem_size = rb_yjit_opts.exec_mem_size * 1024 * 1024;
    uint8_t *mem_block = alloc_exec_mem(mem_size);

    cb = &block;
    cb_init(cb, mem_block, mem_size/2);

    ocb = &outline_block;
    cb_init(ocb, mem_block + mem_size/2, mem_size/2);

    // Generate the interpreter exit code for leave
    leave_exit_code = yjit_gen_leave_exit(cb);

    // Generate full exit code for C func
    gen_full_cfunc_return();
    cb_mark_all_executable(cb);
}
