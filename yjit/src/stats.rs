//! Everything related to the collection of runtime stats in YJIT
//! See the stats feature and the --yjit-stats command-line option

use crate::cruby::*;
use crate::options::*;
use crate::yjit::{yjit_enabled_p};

// YJIT exit counts for each instruction type
static mut EXIT_OP_COUNT: [u64; VM_INSTRUCTION_SIZE] = [0; VM_INSTRUCTION_SIZE];

// Macro to declare the stat counters
macro_rules! make_counters {
    ($($counter_name:ident),+) => {
        // Struct containing the counter values
        #[derive(Default, Debug)]
        pub struct Counters { $(pub $counter_name: u64),+ }

        // Counter names constant
        const COUNTER_NAMES: &'static [&'static str] = &[ $(stringify!($counter_name)),+ ];

        // Global counters instance, initialized to zero
        pub static mut COUNTERS: Counters = Counters { $($counter_name: 0),+ };
    }
}

/// Macro to increment a counter by name
macro_rules! incr_counter {
    // Unsafe is ok here because options are initialized
    // once before any Ruby code executes
    ($counter_name:ident) => {
        unsafe {
            COUNTERS.$counter_name += 1
        }
    };
}
pub(crate) use incr_counter;

/// Macro to get a raw pointer to a given counter
macro_rules! ptr_to_counter {
    ($counter_name:ident) => {
        unsafe {
            let ptr: *mut u64 = &COUNTERS.$counter_name;
            ptr;
        }
    };
}
pub(crate) use ptr_to_counter;

// Declare all the counters we track
make_counters!(
    exec_instruction,

    send_keywords,
    send_kw_splat,
    send_args_splat,
    send_block_arg,
    send_ivar_set_method,
    send_zsuper_method,
    send_undef_method,
    send_optimized_method,
    send_optimized_method_send,
    send_optimized_method_call,
    send_optimized_method_block_call,
    send_missing_method,
    send_bmethod,
    send_refined_method,
    send_cfunc_ruby_array_varg,
    send_cfunc_argc_mismatch,
    send_cfunc_toomany_args,
    send_cfunc_tracing,
    send_cfunc_kwargs,
    send_attrset_kwargs,
    send_iseq_tailcall,
    send_iseq_arity_error,
    send_iseq_only_keywords,
    send_iseq_kwargs_req_and_opt_missing,
    send_iseq_kwargs_mismatch,
    send_iseq_complex_callee,
    send_not_implemented_method,
    send_getter_arity,
    send_se_cf_overflow,
    send_se_protected_check_failed,

    traced_cfunc_return,

    invokesuper_me_changed,
    invokesuper_block,

    leave_se_interrupt,
    leave_interp_return,
    leave_start_pc_non_zero,

    getivar_se_self_not_heap,
    getivar_idx_out_of_range,
    getivar_megamorphic,

    setivar_se_self_not_heap,
    setivar_idx_out_of_range,
    setivar_val_heapobject,
    setivar_name_not_mapped,
    setivar_not_object,
    setivar_frozen,

    oaref_argc_not_one,
    oaref_arg_not_fixnum,

    opt_getinlinecache_miss,

    binding_allocations,
    binding_set,

    vm_insns_count,
    compiled_iseq_count,
    compiled_block_count,
    compilation_failure,

    exit_from_branch_stub,

    invalidation_count,
    invalidate_method_lookup,
    invalidate_bop_redefined,
    invalidate_ractor_spawn,
    invalidate_constant_state_bump,
    invalidate_constant_ic_fill,

    constant_state_bumps,

    expandarray_splat,
    expandarray_postarg,
    expandarray_not_array,
    expandarray_rhs_too_small,

    gbpp_block_param_modified,
    gbpp_block_handler_not_iseq
);

//===========================================================================

/// Primitive called in yjit.rb
/// Check if stats generation is enabled
#[no_mangle]
pub extern "C" fn rb_yjit_stats_enabled_p(ec: EcPtr, ruby_self: VALUE) -> VALUE {
    #[cfg(feature = "stats")]
    if get_option!(gen_stats) {
        return Qtrue
    }

    return Qfalse;
}

/// Primitive called in yjit.rb.
/// Export all YJIT statistics as a Ruby hash.
/// This needs to be wrapped on the C side with RB_VM_LOCK_ENTER()
#[no_mangle]
pub extern "C" fn rb_yjit_gen_stats_dict(ec: EcPtr, ruby_self: VALUE) -> VALUE {

    // Return Qnil if YJIT isn't enabled
    if !yjit_enabled_p() {
        return Qnil;
    }

    unsafe {
        let hash = rb_hash_new();

        /*
        {
            VALUE key = ID2SYM(rb_intern("inline_code_size"));
            VALUE value = LL2NUM((long long)cb->write_pos);
            rb_hash_aset(hash, key, value);

            key = ID2SYM(rb_intern("outlined_code_size"));
            value = LL2NUM((long long)ocb->write_pos);
            rb_hash_aset(hash, key, value);
        }
        */

        // If the stats feature is enabled and we're generating stats
        #[cfg(feature = "stats")]
        if get_option!(gen_stats) {

            /*
            // Indicate that the complete set of stats is available
            rb_hash_aset(hash, ID2SYM(rb_intern("all_stats")), Qtrue);

            int64_t *counter_reader = (int64_t *)&yjit_runtime_counters;
            int64_t *counter_reader_end = &yjit_runtime_counters.last_member;

            // For each counter in yjit_counter_names, add that counter as
            // a key/value pair.

            // Iterate through comma separated counter name list
            char *name_reader = yjit_counter_names;
            char *counter_name_end = yjit_counter_names + sizeof(yjit_counter_names);
            while (name_reader < counter_name_end && counter_reader < counter_reader_end) {
                if (*name_reader == ',' || *name_reader == ' ') {
                    name_reader++;
                    continue;
                }

                // Compute length of counter name
                int name_len;
                char *name_end;
                {
                    name_end = strchr(name_reader, ',');
                    if (name_end == NULL) break;
                    name_len = (int)(name_end - name_reader);
                }

                // Put counter into hash
                VALUE key = ID2SYM(rb_intern2(name_reader, name_len));
                VALUE value = LL2NUM((long long)*counter_reader);
                rb_hash_aset(hash, key, value);

                counter_reader++;
                name_reader = name_end;
            }

            // For each entry in exit_op_count, add a stats entry with key "exit_INSTRUCTION_NAME"
            // and the value is the count of side exits for that instruction.

            char key_string[rb_vm_max_insn_name_size + 6]; // Leave room for "exit_" and a final NUL
            for (int i = 0; i < VM_INSTRUCTION_SIZE; i++) {
                const char *i_name = insn_name(i); // Look up Ruby's NUL-terminated insn name string
                snprintf(key_string, rb_vm_max_insn_name_size + 6, "%s%s", "exit_", i_name);

                VALUE key = ID2SYM(rb_intern(key_string));
                VALUE value = LL2NUM((long long)exit_op_count[i]);
                rb_hash_aset(hash, key, value);
            }
            */
        }

        return hash;
    }
}

/// Primitive called in yjit.rb. Zero out all the counters.
#[no_mangle]
pub extern "C" fn rb_yjit_reset_stats_bang(ec: EcPtr, ruby_self: VALUE) -> VALUE {
    unsafe {
        EXIT_OP_COUNT = [0; VM_INSTRUCTION_SIZE];
        COUNTERS = Counters::default();
    }

    return Qnil;
}

/// Increment the number of instructions executed by the interpreter
#[no_mangle]
pub extern "C" fn rb_yjit_collect_vm_usage_insn() {
    incr_counter!(vm_insns_count);
}

#[no_mangle]
pub extern "C" fn rb_yjit_collect_binding_alloc() {
    incr_counter!(binding_allocations);
}

#[no_mangle]
pub extern "C" fn rb_yjit_collect_binding_set() {
    incr_counter!(binding_set);
}

#[no_mangle]
pub extern "C" fn yjit_count_side_exit_op(exit_pc: *const VALUE) -> *const VALUE
{
    // FIXME
    //int insn = rb_vm_insn_addr2opcode((const void *)*exit_pc);
    //exit_op_count[insn]++;

    return exit_pc; // This function must return exit_pc!
}
