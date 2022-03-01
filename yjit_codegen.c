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
