//! Code to track assumptions made during code generation and invalidate
//! generated code if and when these assumptions are invalidated.

use crate::core::*;
use crate::cruby::*;
use crate::codegen::*;
use crate::stats::*;
use crate::asm::OutlinedCb;
use crate::yjit::yjit_enabled_p;
use std::collections::HashMap;

// Invariants to track:
// assume_bop_not_redefined(jit, INTEGER_REDEFINED_OP_FLAG, BOP_PLUS)
// assume_method_lookup_stable(comptime_recv_klass, cme, jit);
// assume_single_ractor_mode(jit)
// assume_stable_global_constant_state(jit);

struct MethodLookupDependency {
    block: BlockRef,
    mid: ID
}

/// Used to track all of the various block references that contain assumptions
/// about the state of the virtual machine.
pub struct Invariants {
    /// Tracks block assumptions about the stability of a basic operator on a
    /// given class.
    basic_operators: HashMap<(RedefinitionFlag, ruby_basic_operators), Vec<BlockRef>>,

    /// Tracks block assumptions about callable method entry validity.
    cme_validity: HashMap<*const rb_callable_method_entry_t, Vec<BlockRef>>,

    /// Tracks block assumptions about method lookup. Maps a class to a table of
    /// method ID points to a set of blocks. While a block `b` is in the table,
    /// b->callee_cme == rb_callable_method_entry(klass, mid).
    method_lookup: HashMap<VALUE, HashMap<ID, Vec<MethodLookupDependency>>>
}

/// Private singleton instance of the invariants global struct.
static mut INVARIANTS: Option<Invariants> = None;

impl Invariants {
    pub fn init() {
        // Wrapping this in unsafe to assign directly to a global.
        unsafe {
            INVARIANTS = Some(Invariants {
                basic_operators: HashMap::new(),
                cme_validity: HashMap::new(),
                method_lookup: HashMap::new()
            });
        }
    }

    /// Get a mutable reference to the codegen globals instance
    pub fn get_instance() -> &'static mut Invariants {
        unsafe { INVARIANTS.as_mut().unwrap() }
    }

    /// Returns the vector of blocks that are currently assuming the given basic
    /// operator on the given class has not been redefined.
    pub fn get_bop_assumptions(klass: RedefinitionFlag, bop: ruby_basic_operators) -> &'static mut Vec<BlockRef> {
        Invariants::get_instance().basic_operators.entry((klass, bop)).or_insert(Vec::new())
    }
}

/// A public function that can be called from within the code generation
/// functions to ensure that the block being generated is invalidated when the
/// basic operator is redefined.
pub fn assume_bop_not_redefined(jit: &mut JITState, ocb: &mut OutlinedCb, klass: RedefinitionFlag, bop: ruby_basic_operators) -> bool {
    if unsafe { BASIC_OP_UNREDEFINED_P(bop, klass) } {
        jit_ensure_block_entry_exit(jit, ocb);
        Invariants::get_bop_assumptions(klass, bop).push(jit.get_block());
        return true;
    } else {
        return false;
    }
}

// Remember that a block assumes that
// `rb_callable_method_entry(receiver_klass, cme->called_id) == cme` and that
// `cme` is valid.
// When either of these assumptions becomes invalid, rb_yjit_method_lookup_change() or
// rb_yjit_cme_invalidate() invalidates the block.
//
// @raise NoMemoryError
pub fn assume_method_lookup_stable(jit: &mut JITState, ocb: &mut OutlinedCb, receiver_klass: VALUE, callee_cme: *const rb_callable_method_entry_t) {
    // RUBY_ASSERT(rb_callable_method_entry(receiver_klass, cme->called_id) == cme);
    // RUBY_ASSERT_ALWAYS(RB_TYPE_P(receiver_klass, T_CLASS) || RB_TYPE_P(receiver_klass, T_ICLASS));
    // RUBY_ASSERT_ALWAYS(!rb_objspace_garbage_object_p(receiver_klass));

    jit_ensure_block_entry_exit(jit, ocb);

    let block = jit.get_block();
    block.borrow_mut().add_cme_dependency(receiver_klass, callee_cme);

    Invariants::get_instance().cme_validity.entry(callee_cme).or_insert(Vec::new()).push(block.clone());

    let mid = unsafe { (*callee_cme).called_id };
    Invariants::get_instance().method_lookup
        .entry(receiver_klass).or_insert(HashMap::new())
        .entry(mid).or_insert(Vec::new())
        .push(MethodLookupDependency { block: block.clone(), mid });
}

/// Called when a basic operation is redefined.
#[no_mangle]
pub extern "C" fn rb_yjit_bop_redefined(klass: RedefinitionFlag, bop: ruby_basic_operators) {
    for block in Invariants::get_bop_assumptions(klass, bop).iter() {
        invalidate_block_version(block);
        incr_counter!(invalidate_bop_redefined);
    }
}

/// Callback for when rb_callable_method_entry(klass, mid) is going to change.
/// Invalidate blocks that assume stable method lookup of `mid` in `klass` when this happens.
/// This needs to be wrapped on the C side with RB_VM_LOCK_ENTER().
#[no_mangle]
pub extern "C" fn rb_yjit_method_lookup_change(klass: VALUE, mid: ID) {
    Invariants::get_instance().method_lookup.entry(klass).and_modify(|deps| {
        deps.remove(&mid).map(|deps| {
            for dep in deps.iter() {
                invalidate_block_version(&dep.block);
                incr_counter!(invalidate_method_lookup);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bop_assumptions() {
        Invariants::init();

        let block = Block::new(BLOCKID_NULL, &Context::default());
        let bops = &mut Invariants::get_instance().basic_operators;

        // Configure the set of assumptions such that one block is assuming
        // Integer#+ is not redefined and one block is assuming String#+ is not
        // redefined.
        bops.insert((INTEGER_REDEFINED_OP_FLAG, BOP_PLUS), vec![block.clone()]);
        bops.insert((STRING_REDEFINED_OP_FLAG, BOP_PLUS), vec![block.clone()]);

        assert_eq!(Invariants::get_bop_assumptions(INTEGER_REDEFINED_OP_FLAG, BOP_PLUS).len(), 1);
    }
}






//static st_table *blocks_assuming_single_ractor_mode;

// Can raise NoMemoryError.
//RBIMPL_ATTR_NODISCARD()
pub fn assume_single_ractor_mode(jit: &JITState) -> bool
{
    todo!()
    /*
    if (rb_multi_ractor_p()) return false;

    jit_ensure_block_entry_exit(jit);

    //st_insert(blocks_assuming_single_ractor_mode, (st_data_t)jit->block, 1);
    true
    */
}





//static st_table *blocks_assuming_stable_global_constant_state;

// Assume that the global constant state has not changed since call to this function.
// Can raise NoMemoryError.
pub fn assume_stable_global_constant_state(jit: &JITState)
{
    todo!()
    /*
    jit_ensure_block_entry_exit(jit);
    st_insert(blocks_assuming_stable_global_constant_state, (st_data_t)jit->block, 1);
    */
}



/*
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
*/


/*
static void
yjit_block_assumptions_free(block_t *block)
{
    st_data_t as_st_data = (st_data_t)block;
    if (blocks_assuming_stable_global_constant_state) {
        st_delete(blocks_assuming_stable_global_constant_state, &as_st_data, NULL);
    }

    if (blocks_assuming_single_ractor_mode) {
        st_delete(blocks_assuming_single_ractor_mode, &as_st_data, NULL);
    }

    if (blocks_assuming_bops) {
        st_delete(blocks_assuming_bops, &as_st_data, NULL);
    }
}
*/




/*
// When a block is deleted, remove the assumptions associated with it
static void
yjit_block_assumptions_free(block_t *block)
{
    /*
    st_data_t as_st_data = (st_data_t)block;
    if (blocks_assuming_stable_global_constant_state) {
        st_delete(blocks_assuming_stable_global_constant_state, &as_st_data, NULL);
    }

    if (blocks_assuming_single_ractor_mode) {
        st_delete(blocks_assuming_single_ractor_mode, &as_st_data, NULL);
    }

    if (blocks_assuming_bops) {
        st_delete(blocks_assuming_bops, &as_st_data, NULL);
    }
    */
}
*/




/*
// Free the yjit resources associated with an iseq
void
rb_yjit_iseq_free(const struct rb_iseq_constant_body *body)
{
    rb_darray_for(body->yjit_blocks, version_array_idx) {
        rb_yjit_block_array_t version_array = rb_darray_get(body->yjit_blocks, version_array_idx);

        rb_darray_for(version_array, block_idx) {
            block_t *block = rb_darray_get(version_array, block_idx);
            yjit_free_block(block);
        }

        rb_darray_free(version_array);
    }

    rb_darray_free(body->yjit_blocks);
}
*/





/*

/* Called when the constant state changes */
void
rb_yjit_constant_state_changed(void)
{
    if (blocks_assuming_stable_global_constant_state) {
#if YJIT_STATS
        yjit_runtime_counters.constant_state_bumps++;
        yjit_runtime_counters.invalidate_constant_state_bump += blocks_assuming_stable_global_constant_state->num_entries;
#endif

        st_foreach(blocks_assuming_stable_global_constant_state, block_set_invalidate_i, 0);
    }
}

// Callback from the opt_setinlinecache instruction in the interpreter.
// Invalidate the block for the matching opt_getinlinecache so it could regenerate code
// using the new value in the constant cache.
void
rb_yjit_constant_ic_update(const rb_iseq_t *const iseq, IC ic)
{
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
}

void
rb_yjit_before_ractor_spawn(void)
{
    if (blocks_assuming_single_ractor_mode) {
#if YJIT_STATS
        yjit_runtime_counters.invalidate_ractor_spawn += blocks_assuming_single_ractor_mode->num_entries;
#endif

        st_foreach(blocks_assuming_single_ractor_mode, block_set_invalidate_i, 0);
    }
}
*/

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
#[no_mangle]
pub extern "C" fn rb_yjit_tracing_invalidate_all()
{
    if !yjit_enabled_p() {
        return;
    }

    todo!();
/*
    // Stop other ractors since we are going to patch machine code.
    RB_VM_LOCK_ENTER();
    rb_vm_barrier();

    // Make it so all live block versions are no longer valid branch targets
    rb_objspace_each_objects(tracing_invalidate_all_i, NULL);

    // Apply patches
    const uint32_t old_pos = cb->write_pos;
    rb_darray_for(global_inval_patches, patch_idx) {
        struct codepage_patch patch = rb_darray_get(global_inval_patches, patch_idx);
        cb.set_pos(patch.inline_patch_pos);
        uint8_t *jump_target = cb_get_ptr(ocb, patch.outlined_target_pos);
        jmp_ptr(cb, jump_target);
    }
    cb.set_pos(old_pos);

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
*/
}

/*
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
*/
