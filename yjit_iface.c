// This file is a fragment of the yjit.o compilation unit. See yjit.c.
#include "internal.h"
#include "vm_sync.h"
#include "vm_callinfo.h"
#include "builtin.h"
#include "gc.h"
#include "iseq.h"
#include "internal/compile.h"
#include "internal/class.h"
#include "yjit.h"
#include "darray.h"

// Pointer to a YJIT entry point (machine code generated by YJIT)
typedef VALUE (*yjit_func_t)(rb_execution_context_t *, rb_control_frame_t *);

bool
rb_yjit_compile_iseq(const rb_iseq_t *iseq, rb_execution_context_t *ec)
{
    bool success = true;
    RB_VM_LOCK_ENTER();
    rb_vm_barrier();

    // Compile a block version starting at the first instruction
    uint8_t *rb_yjit_iseq_gen_entry_point(const rb_iseq_t *iseq, rb_execution_context_t *ec); // defined in Rust
    uint8_t *code_ptr = rb_yjit_iseq_gen_entry_point(iseq, ec);

    if (code_ptr) {
        iseq->body->jit_func = (yjit_func_t)code_ptr;
    }
    else {
        iseq->body->jit_func = 0;
        success = false;
    }

    RB_VM_LOCK_LEAVE();
    return success;
}

/*
// Verify that calling with cd on receiver goes to callee
static void
check_cfunc_dispatch(VALUE receiver, struct rb_callinfo *ci, void *callee, rb_callable_method_entry_t *compile_time_cme)
{
    if (METHOD_ENTRY_INVALIDATED(compile_time_cme)) {
        rb_bug("yjit: output code uses invalidated cme %p", (void *)compile_time_cme);
    }

    bool callee_correct = false;
    const rb_callable_method_entry_t *cme = rb_callable_method_entry(CLASS_OF(receiver), vm_ci_mid(ci));
    if (cme->def->type == VM_METHOD_TYPE_CFUNC) {
        const rb_method_cfunc_t *cfunc = UNALIGNED_MEMBER_PTR(cme->def, body.cfunc);
        if ((void *)cfunc->func == callee) {
            callee_correct = true;
        }
    }
    if (!callee_correct) {
        rb_bug("yjit: output code calls wrong method");
    }
}
*/

// GC root for interacting with the GC
struct yjit_root_struct {
    int unused; // empty structs are not legal in C99
};

// Map klass => id_table[mid, set of blocks]
// While a block `b` is in the table, b->callee_cme == rb_callable_method_entry(klass, mid).
// See assume_method_lookup_stable()
static st_table *method_lookup_dependency;

// For adding to method_lookup_dependency data with st_update
struct lookup_dependency_insertion {
    //block_t *block;
    ID mid;
};

// Map cme => set of blocks
// See assume_method_lookup_stable()
static st_table *cme_validity_dependency;

static int
mark_and_pin_keys_i(st_data_t k, st_data_t v, st_data_t ignore)
{
    rb_gc_mark((VALUE)k);

    return ST_CONTINUE;
}

// GC callback during mark phase
static void
yjit_root_mark(void *ptr)
{
    if (method_lookup_dependency) {
        // TODO: This is a leak. Unused blocks linger in the table forever, preventing the
        // callee class they speculate on from being collected.
        // We could do a bespoke weak reference scheme on classes similar to
        // the interpreter's call cache. See finalizer for T_CLASS and cc_table_free().
        st_foreach(method_lookup_dependency, mark_and_pin_keys_i, 0);
    }

    if (cme_validity_dependency) {
        // Why not let the GC move the cme keys in this table?
        // Because this is basically a compare_by_identity Hash.
        // If a key moves, we would need to reinsert it into the table so it is rehashed.
        // That is tricky to do, espcially as it could trigger allocation which could
        // trigger GC. Not sure if it is okay to trigger GC while the GC is updating
        // references.
        st_foreach(cme_validity_dependency, mark_and_pin_keys_i, 0);
    }
}

static void
yjit_root_free(void *ptr)
{
    // Do nothing. The root lives as long as the process.
}

static size_t
yjit_root_memsize(const void *ptr)
{
    // Count off-gc-heap allocation size of the dependency table
    return st_memsize(method_lookup_dependency); // TODO: more accurate accounting
}

// GC callback during compaction
static void
yjit_root_update_references(void *ptr)
{
}

// Custom type for interacting with the GC
// TODO: make this write barrier protected
static const rb_data_type_t yjit_root_type = {
    "yjit_root",
    {yjit_root_mark, yjit_root_free, yjit_root_memsize, yjit_root_update_references},
    0, 0, RUBY_TYPED_FREE_IMMEDIATELY
};

// st_table iterator for invalidating blocks that are keys to the table.
static int
block_set_invalidate_i(st_data_t key, st_data_t v, st_data_t ignore)
{
    //block_t *version = (block_t *)key;

    // Thankfully, st_table supports deleting while iterating.
    //invalidate_block_version(version);

    return ST_CONTINUE;
}

// Callback for when a cme becomes invalid.
// Invalidate all blocks that depend on cme being valid.
void
rb_yjit_cme_invalidate(VALUE cme)
{
    if (!cme_validity_dependency) return;

    RUBY_ASSERT(IMEMO_TYPE_P(cme, imemo_ment));

    RB_VM_LOCK_ENTER();

    // Delete the block set from the table
    st_data_t cme_as_st_data = (st_data_t)cme;
    st_data_t blocks;
    if (st_delete(cme_validity_dependency, &cme_as_st_data, &blocks)) {
        st_table *block_set = (st_table *)blocks;

#if YJIT_STATS
        yjit_runtime_counters.invalidate_method_lookup += block_set->num_entries;
#endif

        // Invalidate each block
        st_foreach(block_set, block_set_invalidate_i, 0);

        st_free_table(block_set);
    }

    RB_VM_LOCK_LEAVE();
}

// For dealing with refinements
void
rb_yjit_invalidate_all_method_lookup_assumptions(void)
{
    // It looks like Module#using actually doesn't need to invalidate all the
    // method caches, so we do nothing here for now.
}

/* Called when the constant state changes */
void
rb_yjit_constant_state_changed(void)
{
    /*
    if (blocks_assuming_stable_global_constant_state) {
#if YJIT_STATS
        yjit_runtime_counters.constant_state_bumps++;
        yjit_runtime_counters.invalidate_constant_state_bump += blocks_assuming_stable_global_constant_state->num_entries;
#endif

        st_foreach(blocks_assuming_stable_global_constant_state, block_set_invalidate_i, 0);
    }
    */
}

// Callback from the opt_setinlinecache instruction in the interpreter.
// Invalidate the block for the matching opt_getinlinecache so it could regenerate code
// using the new value in the constant cache.
void
rb_yjit_constant_ic_update(const rb_iseq_t *const iseq, IC ic)
{
    /*
    if (!rb_yjit_enabled_p()) return;

    // We can't generate code in these situations, so no need to invalidate.
    // See gen_opt_getinlinecache.
    if (ic->entry->ic_cref || rb_multi_ractor_p()) {
        return;
    }

    RB_VM_LOCK_ENTER();
    rb_vm_barrier(); // Stop other ractors since we are going to patch machine code.
    {
        const struct rb_iseq_constant_body *const body = iseq->body;
        VALUE *code = body->iseq_encoded;
        const unsigned get_insn_idx = ic->get_insn_idx;

        // This should come from a running iseq, so direct threading translation
        // should have been done
        RUBY_ASSERT(FL_TEST((VALUE)iseq, ISEQ_TRANSLATED));
        RUBY_ASSERT(get_insn_idx < body->iseq_size);
        RUBY_ASSERT(rb_vm_insn_addr2insn((const void *)code[get_insn_idx]) == BIN(opt_getinlinecache));

        // Find the matching opt_getinlinecache and invalidate all the blocks there
        RUBY_ASSERT(insn_op_type(BIN(opt_getinlinecache), 1) == TS_IC);
        if (ic == (IC)code[get_insn_idx + 1 + 1]) {
            rb_yjit_block_array_t getinlinecache_blocks = yjit_get_version_array(iseq, get_insn_idx);

            // Put a bound for loop below to be defensive
            const int32_t initial_version_count = rb_darray_size(getinlinecache_blocks);
            for (int32_t iteration=0; iteration<initial_version_count; ++iteration) {
                getinlinecache_blocks = yjit_get_version_array(iseq, get_insn_idx);

                if (rb_darray_size(getinlinecache_blocks) > 0) {
                    block_t *block = rb_darray_get(getinlinecache_blocks, 0);
                    invalidate_block_version(block);
#if YJIT_STATS
                    yjit_runtime_counters.invalidate_constant_ic_fill++;
#endif
                }
                else {
                    break;
                }
            }

            // All versions at get_insn_idx should now be gone
            RUBY_ASSERT(0 == rb_darray_size(yjit_get_version_array(iseq, get_insn_idx)));
        }
        else {
            RUBY_ASSERT(false && "ic->get_insn_diex not set properly");
        }
    }
    RB_VM_LOCK_LEAVE();
    */
}

void
rb_yjit_before_ractor_spawn(void)
{
    /*
    if (blocks_assuming_single_ractor_mode) {
#if YJIT_STATS
        yjit_runtime_counters.invalidate_ractor_spawn += blocks_assuming_single_ractor_mode->num_entries;
#endif

        st_foreach(blocks_assuming_single_ractor_mode, block_set_invalidate_i, 0);
    }
    */
}

// Primitive called in yjit.rb.
// Export all YJIT statistics as a Ruby hash.
VALUE
rb_yjit_get_stats(rb_execution_context_t *ec, VALUE self)
{
    VALUE stats_hash;
    RB_VM_LOCK_ENTER();
    VALUE rb_yjit_gen_stats_dict(rb_execution_context_t *ec, VALUE self);
    stats_hash = rb_yjit_gen_stats_dict(ec, self);
    RB_VM_LOCK_LEAVE();

    return stats_hash;
}

// Primitives used by yjit.rb
VALUE rb_yjit_stats_enabled_p(rb_execution_context_t *ec, VALUE self);
VALUE rb_yjit_get_stats(rb_execution_context_t *ec, VALUE self);
VALUE rb_yjit_reset_stats_bang(rb_execution_context_t *ec, VALUE self);
VALUE rb_yjit_disasm_iseq(rb_execution_context_t *ec, VALUE self, VALUE iseq);

// Preprocessed yjit.rb generated during build
#include "yjit.rbinc"

void
rb_yjit_iseq_mark(const struct rb_iseq_constant_body *body)
{
    /*
    rb_darray_for(body->yjit_blocks, version_array_idx) {
        rb_yjit_block_array_t version_array = rb_darray_get(body->yjit_blocks, version_array_idx);

        rb_darray_for(version_array, block_idx) {
            block_t *block = rb_darray_get(version_array, block_idx);

            rb_gc_mark_movable((VALUE)block->blockid.iseq);

            cme_dependency_t *cme_dep;
            rb_darray_foreach(block->cme_dependencies, cme_dependency_idx, cme_dep) {
                rb_gc_mark_movable(cme_dep->receiver_klass);
                rb_gc_mark_movable(cme_dep->callee_cme);
            }

            // Mark outgoing branch entries
            rb_darray_for(block->outgoing, branch_idx) {
                branch_t *branch = rb_darray_get(block->outgoing, branch_idx);
                for (int i = 0; i < 2; ++i) {
                    rb_gc_mark_movable((VALUE)branch->targets[i].iseq);
                }
            }

            // Walk over references to objects in generated code.
            uint32_t *offset_element;
            rb_darray_foreach(block->gc_object_offsets, offset_idx, offset_element) {
                uint32_t offset_to_value = *offset_element;
                uint8_t *value_address = cb_get_ptr(cb, offset_to_value);

                VALUE object;
                memcpy(&object, value_address, SIZEOF_VALUE);
                rb_gc_mark_movable(object);
            }
        }
    }
    */
}

void
rb_yjit_iseq_update_references(const struct rb_iseq_constant_body *body)
{
    /*
    rb_vm_barrier();

    rb_darray_for(body->yjit_blocks, version_array_idx) {
        rb_yjit_block_array_t version_array = rb_darray_get(body->yjit_blocks, version_array_idx);

        rb_darray_for(version_array, block_idx) {
            block_t *block = rb_darray_get(version_array, block_idx);

            block->blockid.iseq = (const rb_iseq_t *)rb_gc_location((VALUE)block->blockid.iseq);

            cme_dependency_t *cme_dep;
            rb_darray_foreach(block->cme_dependencies, cme_dependency_idx, cme_dep) {
                cme_dep->receiver_klass = rb_gc_location(cme_dep->receiver_klass);
                cme_dep->callee_cme = rb_gc_location(cme_dep->callee_cme);
            }

            // Update outgoing branch entries
            rb_darray_for(block->outgoing, branch_idx) {
                branch_t *branch = rb_darray_get(block->outgoing, branch_idx);
                for (int i = 0; i < 2; ++i) {
                    branch->targets[i].iseq = (const void *)rb_gc_location((VALUE)branch->targets[i].iseq);
                }
            }

            // Walk over references to objects in generated code.
            uint32_t *offset_element;
            rb_darray_foreach(block->gc_object_offsets, offset_idx, offset_element) {
                uint32_t offset_to_value = *offset_element;
                uint8_t *value_address = cb_get_ptr(cb, offset_to_value);

                VALUE object;
                memcpy(&object, value_address, SIZEOF_VALUE);
                VALUE possibly_moved = rb_gc_location(object);
                // Only write when the VALUE moves, to be CoW friendly.
                if (possibly_moved != object) {
                    // Possibly unlock the page we need to update
                    cb_mark_position_writeable(cb, offset_to_value);

                    // Object could cross a page boundary, so unlock there as well
                    cb_mark_position_writeable(cb, offset_to_value + SIZEOF_VALUE - 1);
                    memcpy(value_address, &possibly_moved, SIZEOF_VALUE);
                }
            }
        }
    }

    // If YJIT isn't initialized, then cb or ocb could be NULL.
    if (cb) {
        cb_mark_all_executable(cb);
    }

    if (ocb) {
        cb_mark_all_executable(ocb);
    }
    */
}

// Free the yjit resources associated with an iseq
void
rb_yjit_iseq_free(const struct rb_iseq_constant_body *body)
{
    /*
    rb_darray_for(body->yjit_blocks, version_array_idx) {
        rb_yjit_block_array_t version_array = rb_darray_get(body->yjit_blocks, version_array_idx);

        rb_darray_for(version_array, block_idx) {
            block_t *block = rb_darray_get(version_array, block_idx);
            yjit_free_block(block);
        }

        rb_darray_free(version_array);
    }

    rb_darray_free(body->yjit_blocks);
    */
}

// Can raise RuntimeError
void
rb_yjit_init(void)
{
    if (!PLATFORM_SUPPORTED_P || !JIT_ENABLED) {
        return;
    }

    // Call the Rust initialization code
    void rb_yjit_init_rust(void);
    return rb_yjit_init_rust();

    /*
    // Initialize the GC hooks
    struct yjit_root_struct *root;
    VALUE yjit_root = TypedData_Make_Struct(0, struct yjit_root_struct, &yjit_root_type, root);
    rb_gc_register_mark_object(yjit_root);
    */
}
