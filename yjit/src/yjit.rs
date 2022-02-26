use crate::cruby::{EcPtr, IseqPtr};
use crate::codegen::*;
use crate::core::*;
use crate::options::*;

use std::sync::atomic::{AtomicBool,Ordering};
use std::os::raw;

/// For tracking whether the user enabled YJIT through command line arguments or environment
/// variables. AtomicBool to avoid `unsafe`. On x86 it compiles to simple movs.
/// See https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html
/// See [rb_yjit_enabled_p]
static YJIT_ENABLED: AtomicBool = AtomicBool::new(false);

/// Parse one command-line option
#[no_mangle]
pub extern "C" fn rb_yjit_parse_option(str_ptr: *const raw::c_char) -> bool
{
    return parse_option(str_ptr).is_some();
}

/// Is YJIT on? The interpreter uses this function to decide whether to increment
/// ISEQ call counters. See mjit_exec().
/// This is used frequently since it's used on every method call in the interpreter.
#[no_mangle]
pub extern "C" fn rb_yjit_enabled_p() -> raw::c_int {
    // Note that we might want to call this function from signal handlers so
    // might need to ensure signal-safety(7).
    YJIT_ENABLED.load(Ordering::Acquire).into()
}

/// Like rb_yjit_enabled_p, but for Rust code.
pub fn yjit_enabled_p() -> bool {
    YJIT_ENABLED.load(Ordering::Acquire)
}

/// On which invocation of the ISEQ to invoke YJIT?
#[no_mangle]
pub extern "C" fn rb_yjit_call_threshold() -> raw::c_uint {
    // TODO: read this from command line arg
    10
}

/// This function is called from C code
#[no_mangle]
pub extern "C" fn rb_yjit_init_rust()
{
    // TODO: need to make sure that command-line options have been
    // initialized by CRuby

    // Catch panics to avoid UB for unwinding into C frames.
    // See https://doc.rust-lang.org/nomicon/exception-safety.html
    // TODO: set a panic handler so the we don't print a message
    //       everytime we panic.
    let result = std::panic::catch_unwind(|| {

        CodegenGlobals::init();


        // TODO:
        //Invariants::init() ?

        YJIT_ENABLED.store(true, Ordering::Release);
    });


    if let Err(_) = result {
        println!("YJIT: rb_yjit_init_rust() panicked. Aborting.");
        std::process::abort();
    }
}

/// Called from C code to begin compiling a function
/// NOTE: this should be wrapped in RB_VM_LOCK_ENTER(), rb_vm_barrier() on the C side
#[no_mangle]
pub extern "C" fn rb_yjit_iseq_gen_entry_point(iseq: IseqPtr, ec: EcPtr) -> *const u8 {
    let maybe_code_ptr = gen_entry_point(iseq, 0, ec);

    dbg!(maybe_code_ptr);

    match maybe_code_ptr {
        Some(ptr) => ptr.raw_ptr(),
        None => std::ptr::null()
    }
}

/// Called from C code when a branch stub is hit
/// NOTE: this should be wrapped in RB_VM_LOCK_ENTER(), rb_vm_barrier() on the C side
#[no_mangle]
pub extern "C" fn rb_yjit_branch_stub_hit(/* branch_t *branch, */ target_idx: u32, ec: EcPtr) -> *const u8
{
    // TODO: figure out what to pass instead of a branch pointer?

    // TODO
    todo!("branch_stub_hit not implemented");
}
