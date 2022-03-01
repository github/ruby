//! This module deals with making relevant C functions available to Rust YJIT.
//! Some C functions we use we maintain, some are public C extension APIs,
//! some are internal CRuby APIs.
//!
//! ## General notes about linking
//!
//! The YJIT crate compiles to a native static library, which for our purposes
//! we can understand as a collection of object files. On ELF platforms at least,
//! object files can refer to "external symbols" which we could take some
//! liberty and understand as assembly labels that refer to code defined in other
//! object files resolved when linking. When we are linking, say to produce miniruby,
//! the linker resolves and put concrete addresses for each usage of C function in
//! the Rust static library.
//!
//! By declaring external functions and using them, we are asserting the symbols
//! we use have definition in one of the object files we pass to the linker. Declaring
//! a function here that has no definition anywhere causes a linking error.
//!
//! There are more things going on during linking and this section makes a lot of
//! simplifications but hopefully this gives a good enough working mental model.
//!
//! ## Difference from example in the Rustonomicon
//!
//! You might be wondering about why this is different from the [FFI example]
//! in the Nomicon, an official book about Unsafe Rust.
//!
//! There is no #[link] attribute because we are not linking against an external
//! library, but rather implicitly asserting that we'll supply a concrete definition
//! for all C functions we call, similar to how pure C projects put functions
//! across different compilation units and link them together.
//!
//! TODO(alan): is the model different enough on Windows that this setup is unworkable?
//!             Seems prudent to at least learn more about Windows binary tooling before
//!             committing to a design.
//!
//! Alan recommends reading the Nomicon cover to cover as he thinks the book is
//! not very long in general and especially for something that can save hours of
//! debugging Undefined Behavior (UB) down the road.
//!
//! UBs can cause Safe Rust to crash, at which point it's hard to tell which
//! usage of `unsafe` in the codebase invokes UB. Providing safe Rust interface
//! wrapping `unsafe` Rust is a good technique, but requires practice and knowledge
//! about what's well defined and what's undefined.
//!
//! For an extremely advanced example of building safe primitives using Unsafe Rust,
//! see the [GhostCell] paper. Some parts of the paper assume less background knowledge
//! than other parts, so there should be learning opportunities in it for all experience
//! levels.
//!
//! ## Binding generation
//!
//! For the moment declarations on the Rust side are hand written. The code is boilerplate
//! and could be generated automatically with a custom tooling that depend on
//! rust-lang/rust-bindgen. The output Rust code could be checked in to version control
//! and verified on CI like `make update-deps`.
//!
//! Upsides for this design:
//!  - the YJIT static lib that links with miniruby and friends will not need bindgen
//!    as a dependency at all. This is an important property so Ruby end users can
//!    build a YJIT enabled Ruby with no internet connection using a release tarball
//!  - Less hand-typed boilerplate
//!  - Helps reduce risk of C definitions and Rust declaration going out of sync since
//!    CI verifies synchronicity
//!
//! Downsides and known unknowns:
//!  - Using rust-bindgen this way seems unusual. We might be depending on parts
//!    that the project is not committed to maintaining
//!  - This setup assumes rust-bindgen gives deterministic output, which can't be taken
//!    for granted
//!  - YJIT contributors will need to install libclang on their system to get rust-bindgen
//!    to work if they want to run the generation tool locally
//!
//! The elephant in the room is that we'll still need to use Unsafe Rust to call C functions,
//! and the binding generation can't magically save us from learning Unsafe Rust.
//!
//!
//! [FFI example]: https://doc.rust-lang.org/nomicon/ffi.html
//! [GhostCell]: http://plv.mpi-sws.org/rustbelt/ghostcell/

// CRuby types use snake_case. Allow them so we use one name across languages.
#![allow(non_camel_case_types)]
// A lot of imported CRuby globals aren't all-caps
#![allow(non_upper_case_globals)]

use std::convert::From;
use std::os::raw::{c_int, c_uint, c_long};

// We check that we can do this with the configure script and a couple of
// static asserts. u64 and not usize to play nice with lowering to x86.
pub type size_t = u64;

// Textually include output from rust-bindgen as suggested by its user guide.
include!("cruby_bindings.inc.rs");

// TODO: For #defines that affect memory layout, we need to check for them
// on build and fail if they're wrong. e.g. USE_FLONUM *must* be true.

// TODO:
// Temporary, these external bindings will likely be auto-generated
// and textually included in this file
extern "C" {
    #[link_name = "rb_yjit_alloc_exec_mem"] // we can rename functions with this attribute
    pub fn alloc_exec_mem(mem_size: u32) -> *mut u8;

    // Alan suggests calling these from the C side, not exporting them to Rust
    //pub fn RB_VM_LOCK_ENTER();
    //pub fn RB_VM_LOCK_LEAVE();
    //pub fn rb_vm_barrier();

    //int insn = rb_vm_insn_addr2opcode((const void *)*exit_pc);

    //pub fn rb_intern(???) -> ???
    //pub fn ID2SYM(id: VALUE) -> VALUE;
    //pub fn LL2NUM((long long)ocb->write_pos) -> VALUE;

    #[link_name = "rb_insn_len"]
    pub fn raw_insn_len(v: VALUE) -> c_int;

    #[link_name = "rb_yarv_class_of"]
    pub fn CLASS_OF(v:VALUE) -> VALUE;

    #[link_name = "rb_get_ec_cfp"]
    pub fn get_ec_cfp(ec: EcPtr) -> CfpPtr;

    #[link_name = "rb_get_cfp_pc"]
    pub fn get_cfp_pc(cfp: CfpPtr) -> *mut VALUE;

    #[link_name = "rb_get_cfp_sp"]
    pub fn get_cfp_sp(cfp: CfpPtr) -> *mut VALUE;

    #[link_name = "rb_get_cfp_self"]
    pub fn get_cfp_self(cfp: CfpPtr) -> VALUE;

    #[link_name = "rb_get_cfp_ep"]
    pub fn get_cfp_ep(cfp: CfpPtr) -> *mut VALUE;

    #[link_name = "rb_get_cme_def_type"]
    pub fn get_cme_def_type(cme: * const rb_callable_method_entry_t) -> rb_method_type_t;

    #[link_name = "rb_get_cme_def_method_serial"]
    pub fn get_cme_def_method_serial(cme: * const rb_callable_method_entry_t) -> u64;

    #[link_name = "rb_get_cme_def_body_attr_id"]
    pub fn get_cme_def_body_attr_id(cme: * const rb_callable_method_entry_t) -> ID;

    #[link_name = "rb_get_cme_def_body_optimized_type"]
    pub fn get_cme_def_body_optimized_type(cme: * const rb_callable_method_entry_t) -> method_optimized_type;

    #[link_name = "rb_get_cme_def_body_cfunc"]
    pub fn get_cme_def_body_cfunc(cme: * const rb_callable_method_entry_t) -> *mut rb_method_cfunc_t;

    #[link_name = "rb_get_def_method_serial"]
    /// While this returns a uintptr_t in C, we always use it as a Rust u64
    pub fn get_def_method_serial(def: * const rb_method_definition_t) -> u64;

    #[link_name = "rb_get_mct_argc"]
    pub fn get_mct_argc(mct: * const rb_method_cfunc_t) -> c_int;

    #[link_name = "rb_get_mct_func"]
    pub fn get_mct_func(mct: * const rb_method_cfunc_t) -> *const u8;

    #[link_name = "rb_get_def_iseq_ptr"]
    pub fn get_def_iseq_ptr(def: *const rb_method_definition_t) -> IseqPtr;

    #[link_name = "rb_iseq_encoded_size"]
    pub fn get_iseq_encoded_size(iseq: IseqPtr) -> c_uint;

    #[link_name = "rb_get_iseq_body_iseq_encoded"]
    pub fn get_iseq_body_iseq_encoded(iseq: IseqPtr) -> *mut VALUE;

    #[link_name = "rb_get_iseq_body_builtin_inline_p"]
    pub fn get_iseq_body_builtin_inline_p(iseq: IseqPtr) -> bool;

    #[link_name = "rb_get_iseq_body_stack_max"]
    pub fn get_iseq_body_stack_max(iseq:IseqPtr) -> c_uint;

    #[link_name = "rb_get_iseq_flags_has_opt"]
    pub fn get_iseq_flags_has_opt(iseq: IseqPtr) -> c_int;

    #[link_name = "rb_get_iseq_body_local_table_size"]
    pub fn get_iseq_body_local_table_size(iseq: IseqPtr) -> c_uint;

    #[link_name = "rb_get_iseq_body_param_keyword_num"]
    pub fn get_iseq_body_param_keyword_num(iseq: IseqPtr) -> c_int;

    #[link_name = "rb_get_iseq_body_param_size"]
    pub fn get_iseq_body_param_size(iseq: IseqPtr) -> c_uint;

    #[link_name = "rb_get_iseq_body_param_lead_num"]
    pub fn get_iseq_body_param_lead_num(iseq: IseqPtr) -> c_int;

    #[link_name = "rb_get_iseq_body_param_opt_num"]
    pub fn get_iseq_body_param_opt_num(iseq: IseqPtr) -> c_int;

    #[link_name = "rb_get_iseq_body_param_opt_table"]
    pub fn get_iseq_body_param_opt_table(iseq: IseqPtr) -> *const VALUE;

    #[link_name = "rb_iseq_needs_lead_args_only"]
    pub fn iseq_needs_lead_args_only(iseq: *const rb_iseq_t) -> bool;

    #[link_name = "rb_get_call_data_ci"]
    pub fn get_call_data_ci(cd: * const rb_call_data) -> *const rb_callinfo;

    #[link_name = "rb_yarv_str_eql_internal"]
    pub fn rb_str_eql_internal(str1: VALUE, str2: VALUE) -> VALUE;

    #[link_name = "rb_FL_TEST"]
    pub fn FL_TEST(obj: VALUE, flags: VALUE) -> VALUE;

    #[link_name = "rb_FL_TEST_RAW"]
    pub fn FL_TEST_RAW(obj: VALUE, flags: VALUE) -> VALUE;

    #[link_name = "rb_RB_TYPE_P"]
    pub fn RB_TYPE_P(obj: VALUE, t: ruby_value_type) -> bool;

    // Ruby only defines these in vm_insnhelper.c, not in any header.
    // Parsing it would result in a lot of duplicate definitions.
    pub fn rb_vm_opt_mod(recv: VALUE, obj: VALUE) -> VALUE;
    pub fn rb_vm_splat_array(flag: VALUE, ary: VALUE) -> VALUE;
    pub fn rb_vm_defined(ec: EcPtr, reg_cfp: CfpPtr, op_type: rb_num_t, obj: VALUE, v: VALUE) -> bool;
    pub fn rb_vm_set_ivar_idx(obj: VALUE, idx: u32, val: VALUE) -> VALUE;
    pub fn rb_vm_setinstancevariable(iseq: IseqPtr, obj: VALUE, id: ID, val: VALUE, ic: IVC);
    pub fn rb_aliased_callable_method_entry(me: *const rb_callable_method_entry_t) -> *const rb_callable_method_entry_t;
    pub fn rb_iseq_only_optparam_p(iseq: IseqPtr) -> bool;
    pub fn rb_iseq_only_kwparam_p(iseq: IseqPtr) -> bool;
    pub fn rb_vm_getclassvariable(iseq: IseqPtr, cfp: CfpPtr, id: ID, ic: ICVARC) -> VALUE;
    pub fn rb_vm_setclassvariable(iseq: IseqPtr, cfp: CfpPtr, id: ID, val: VALUE, ic: ICVARC) -> VALUE;

    #[link_name = "rb_vm_ci_argc"]
    pub fn vm_ci_argc(ci: * const rb_callinfo) -> c_int;

    #[link_name = "rb_vm_ci_mid"]
    pub fn vm_ci_mid(ci: * const rb_callinfo) -> ID;

    #[link_name = "rb_vm_ci_flag"]
    pub fn vm_ci_flag(ci: * const rb_callinfo) -> c_uint;

    #[link_name = "rb_METHOD_ENTRY_VISI"]
    pub fn METHOD_ENTRY_VISI(me: * const rb_callable_method_entry_t) -> rb_method_visibility_t;
}

pub fn insn_len(opcode:usize) -> u32
{
    #[cfg(test)]
    panic!("insn_len is a CRuby function, and we don't link against CRuby for Rust testing!");

    #[cfg(not(test))]
    unsafe {
        raw_insn_len(VALUE(opcode)).try_into().unwrap()
    }
}

#[cfg(not(test))]
pub fn get_ruby_vm_frozen_core() -> VALUE
{
    // The C side reads this as an extern constant from vm.c (see vm_core.h).
    todo!();
}

#[cfg(test)]
pub fn get_ruby_vm_frozen_core() -> VALUE
{
    // Until we can link with CRuby, return a fake constant.
    VALUE(0xACE_DECADE)
}

/// Opaque iseq type for opaque iseq pointers from vm_core.h
/// See: https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
#[repr(C)]
pub struct rb_iseq_t {
    _data: [u8; 0],
    _marker:
        core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct VALUE(pub usize);

/// Pointer to an ISEQ
pub type IseqPtr = *const rb_iseq_t;

/// Opaque execution-context type from vm_core.h
#[repr(C)]
pub struct rb_execution_context_struct {
    _data: [u8; 0],
    _marker:
        core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Pointer to an execution context (rb_execution_context_struct)
pub type EcPtr = *const rb_execution_context_struct;

// From method.h
#[repr(C)]
pub struct rb_method_definition_t {
    _data: [u8; 0],
    _marker:
        core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
type rb_method_definition_struct = rb_method_definition_t;

/// Opaque cfunc type from method.h
#[repr(C)]
pub struct rb_method_cfunc_t {
    _data: [u8; 0],
    _marker:
        core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque FILE type from the C standard library
#[repr(C)]
pub struct FILE {
    _data: [u8; 0],
    _marker:
        core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque call-data type from vm_callinfo.h
#[repr(C)]
pub struct rb_call_data {
    _data: [u8; 0],
    _marker:
    core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque call-info type from vm_callinfo.h
#[repr(C)]
pub struct rb_callinfo {
    _data: [u8; 0],
    _marker:
    core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque control_frame (CFP) struct from vm_core.h
#[repr(C)]
pub struct rb_control_frame_struct {
    _data: [u8; 0],
    _marker:
    core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Pointer to a control frame pointer (CFP)
pub type CfpPtr = *mut rb_control_frame_struct;

impl VALUE {
    // Return whether the value is truthy or falsy in Ruby -- only nil and false are falsy.
    pub fn test(self:VALUE) -> bool
    {
        let VALUE(cval) = self;
        let VALUE(qnilval) = Qnil;
        (cval & !qnilval) != 0
    }

    // Return true if the number is an immediate integer, flonum or static symbol
    pub fn immediate_p(self:VALUE) -> bool
    {
        let VALUE(cval) = self;
        (cval & 7) != 0
    }

    // Return true if the value is a Ruby immediate integer, flonum, static symbol, nil or false
    pub fn special_const_p(self:VALUE) -> bool
    {
        self.immediate_p() || !self.test()
    }

    // Return true if the value is a Ruby Fixnum (immediate-size integer)
    pub fn fixnum_p(self:VALUE) -> bool
    {
        let VALUE(cval) = self;
        (cval & 1) == 1
    }

    // Return true if the value is an immediate Ruby floating-point number (flonum)
    pub fn flonum_p(self:VALUE) -> bool {
        let VALUE(cval) = self;
        (cval & 3) == 2
    }

    // Return true for a static (non-heap) Ruby symbol
    pub fn static_sym_p(self:VALUE) -> bool {
        let VALUE(cval) = self;
        (cval & 0xff) == RUBY_SYMBOL_FLAG
    }

    // Returns true or false depending on whether the value is nil
    pub fn nil_p(self:VALUE) -> bool {
        self == Qnil
    }

    // Read the flags bits from the RBasic object, then return a Ruby type enum (e.g. RUBY_T_ARRAY)
    pub fn builtin_type(self:VALUE) -> ruby_value_type {
        assert!(self.special_const_p());

        let VALUE(cval) = self;
        let rbasic_ptr = cval as *const RBasic;
        let flags_bits:usize = unsafe { (*rbasic_ptr).flags }.as_usize();
        (flags_bits & (RUBY_T_MASK as usize)) as ruby_value_type
    }

    pub fn class_of(self:VALUE) -> VALUE {
        unsafe { CLASS_OF(self) }
    }

    pub fn as_isize(self:VALUE) -> isize {
        let VALUE(is) = self;
        is as isize
    }

    pub fn as_i32(self:VALUE) -> i32 {
        let VALUE(i) = self;
        i.try_into().unwrap()
    }

    pub fn as_u32(self:VALUE) -> u32 {
        let VALUE(i) = self;
        i.try_into().unwrap()
    }

    pub fn as_i64(self:VALUE) -> i64 {
        let VALUE(i) = self;
        i.try_into().unwrap()
    }

    pub fn as_u64(self:VALUE) -> u64 {
        let VALUE(i) = self;
        i.try_into().unwrap()
    }

    pub fn as_usize(self:VALUE) -> usize {
        let VALUE(us) = self;
        us as usize
    }
}

impl VALUE {
    pub fn fixnum_from_usize(item: usize) -> Self {
        assert!(item <= (RUBY_FIXNUM_MAX as usize)); // An unsigned will always be greater than RUBY_FIXNUM_MIN
        let k : usize = item.wrapping_add(item.wrapping_add(1));
        VALUE(k)
    }
}

impl From<VALUE> for u64 {
    fn from(value: VALUE) -> Self {
        let VALUE(uimm) = value;
        uimm as u64
    }
}

impl From<VALUE> for i64 {
    fn from(value: VALUE) -> Self {
        let VALUE(uimm) = value;
        assert!(uimm <= (i64::MAX as usize));
        uimm as i64
    }
}

impl From<VALUE> for i32 {
    fn from(value: VALUE) -> Self {
        let VALUE(uimm) = value;
        assert!(uimm <= (i32::MAX as usize));
        uimm as i32
    }
}

// Non-idiomatic capitalization for consistency with CRuby code
#[allow(non_upper_case_globals)]
pub const Qfalse: VALUE = VALUE(0);
#[allow(non_upper_case_globals)]
pub const Qnil: VALUE = VALUE(8);
#[allow(non_upper_case_globals)]
pub const Qtrue: VALUE = VALUE(20);
#[allow(non_upper_case_globals)]
pub const Qundef: VALUE = VALUE(52);

pub const RUBY_SYMBOL_FLAG: usize = 0x0c;

pub const RUBY_LONG_MIN:isize = std::os::raw::c_long::MIN as isize;
pub const RUBY_LONG_MAX:isize = std::os::raw::c_long::MAX as isize;

pub const RUBY_FIXNUM_MIN:isize = RUBY_LONG_MIN / 2;
pub const RUBY_FIXNUM_MAX:isize = RUBY_LONG_MAX / 2;
pub const RUBY_FIXNUM_FLAG:usize = 0x1;

pub const RUBY_FLONUM_FLAG:usize = 0x2;
pub const RUBY_FLONUM_MASK:usize = 0x3;

pub const RUBY_IMMEDIATE_MASK:usize = 0x7;

pub const RUBY_SPECIAL_SHIFT:usize = 8;

// Constants from vm_core.h
pub const VM_SPECIAL_OBJECT_VMCORE:usize = 0x1;
pub const VM_ENV_DATA_INDEX_SPECVAL:isize = -1;
pub const VM_ENV_DATA_INDEX_FLAGS:isize = 0;
pub const VM_ENV_DATA_SIZE:usize = 3;

// From vm_callinfo.h
pub const VM_CALL_ARGS_SPLAT:u32    = 1 << VM_CALL_ARGS_SPLAT_bit;
pub const VM_CALL_ARGS_BLOCKARG:u32 = 1 << VM_CALL_ARGS_BLOCKARG_bit;
pub const VM_CALL_FCALL:u32         = 1 << VM_CALL_FCALL_bit;
pub const VM_CALL_KWARG:u32         = 1 << VM_CALL_KWARG_bit;
pub const VM_CALL_KW_SPLAT:u32      = 1 << VM_CALL_KW_SPLAT_bit;
pub const VM_CALL_TAILCALL:u32      = 1 << VM_CALL_TAILCALL_bit;

pub const SIZEOF_VALUE: usize = 8;
pub const SIZEOF_VALUE_I32: i32 = SIZEOF_VALUE as i32;

pub const RUBY_FL_SINGLETON:usize = RUBY_FL_USER_0;

pub const ROBJECT_EMBED:usize = RUBY_FL_USER_1;
pub const ROBJECT_EMBED_LEN_MAX:usize = 3; // This is a complex calculation in ruby/internal/core/robject.h

// Constants from include/ruby/internal/fl_type.h
pub const RUBY_FL_USHIFT:usize = 12;
pub const RUBY_FL_USER_0:usize = 1 << (RUBY_FL_USHIFT + 0);
pub const RUBY_FL_USER_1:usize = 1 << (RUBY_FL_USHIFT + 1);
pub const RUBY_FL_USER_2:usize = 1 << (RUBY_FL_USHIFT + 2);
pub const RUBY_FL_USER_3:usize = 1 << (RUBY_FL_USHIFT + 3);
pub const RUBY_FL_USER_4:usize = 1 << (RUBY_FL_USHIFT + 4);
pub const RUBY_FL_USER_5:usize = 1 << (RUBY_FL_USHIFT + 5);
pub const RUBY_FL_USER_6:usize = 1 << (RUBY_FL_USHIFT + 6);
pub const RUBY_FL_USER_7:usize = 1 << (RUBY_FL_USHIFT + 7);
pub const RUBY_FL_USER_8:usize = 1 << (RUBY_FL_USHIFT + 8);
pub const RUBY_FL_USER_9:usize = 1 << (RUBY_FL_USHIFT + 9);
pub const RUBY_FL_USER_10:usize = 1 << (RUBY_FL_USHIFT + 10);
pub const RUBY_FL_USER_11:usize = 1 << (RUBY_FL_USHIFT + 11);
pub const RUBY_FL_USER_12:usize = 1 << (RUBY_FL_USHIFT + 12);
pub const RUBY_FL_USER_13:usize = 1 << (RUBY_FL_USHIFT + 13);
pub const RUBY_FL_USER_14:usize = 1 << (RUBY_FL_USHIFT + 14);
pub const RUBY_FL_USER_15:usize = 1 << (RUBY_FL_USHIFT + 15);
pub const RUBY_FL_USER_16:usize = 1 << (RUBY_FL_USHIFT + 16);
pub const RUBY_FL_USER_17:usize = 1 << (RUBY_FL_USHIFT + 17);
pub const RUBY_FL_USER_18:usize = 1 << (RUBY_FL_USHIFT + 18);
pub const RUBY_FL_USER_19:usize = 1 << (RUBY_FL_USHIFT + 19);

// Constants from include/ruby/internal/core/rarray.h
pub const RARRAY_EMBED_FLAG:usize = RUBY_FL_USER_1;
pub const RARRAY_EMBED_LEN_SHIFT:usize = RUBY_FL_USHIFT + 3;
pub const RARRAY_EMBED_LEN_MASK:usize = RUBY_FL_USER_3 | RUBY_FL_USER_4;

// We'll need to encode a lot of Ruby struct/field offsets as constants unless we want to
// redeclare all the Ruby C structs and write our own offsetof macro. For now, we use constants.
pub const RUBY_OFFSET_RBASIC_FLAGS:i32 = 0;  // struct RBasic, field "flags"
pub const RUBY_OFFSET_RBASIC_KLASS:i32 = 8;  // struct RBasic, field "klass"
pub const RUBY_OFFSET_RARRAY_AS_HEAP_LEN:i32 = 16;  // struct RArray, subfield "as.heap.len"
pub const RUBY_OFFSET_RARRAY_AS_ARY:i32 = 16;  // struct RArray, subfield "as.ary"
pub const RUBY_OFFSET_RARRAY_AS_HEAP_PTR:i32 = 16;  // struct RArray, subfield "as.heap.ptr"

pub const RUBY_OFFSET_ROBJECT_AS_ARY:i32 = 16; // struct RObject, subfield "as.ary"
pub const RUBY_OFFSET_ROBJECT_AS_HEAP_NUMIV:i32 = 16; // struct RObject, subfield "as.heap.numiv"
pub const RUBY_OFFSET_ROBJECT_AS_HEAP_IVPTR:i32 = 20; // struct RObject, subfield "as.heap.ivptr"

// Constants from rb_control_frame_t vm_core.h
pub const RUBY_OFFSET_CFP_PC: i32 = 0;
pub const RUBY_OFFSET_CFP_SP: i32 = 8;
pub const RUBY_OFFSET_CFP_ISEQ: i32 = 16;
pub const RUBY_OFFSET_CFP_SELF: i32 = 24;
pub const RUBY_OFFSET_CFP_EP: i32 = 32;
pub const RUBY_OFFSET_CFP_BLOCK_CODE: i32 = 40;
pub const RUBY_OFFSET_CFP_BP: i32 = 48; // field __bp__
pub const RUBY_OFFSET_CFP_JIT_RETURN: i32 = 56;
pub const RUBY_SIZEOF_CONTROL_FRAME: usize = 64;

// Constants from rb_execution_context_t vm_core.h
pub const RUBY_OFFSET_EC_CFP: i32 = 16;
pub const RUBY_OFFSET_EC_INTERRUPT_FLAG: i32 = 32; // rb_atomic_t (u32)
pub const RUBY_OFFSET_EC_INTERRUPT_MASK: i32 = 36; // rb_atomic_t (u32)
pub const RUBY_OFFSET_EC_THREAD_PTR: i32 = 48;

// TODO: need to dynamically autogenerate constants for all the YARV opcodes from insns.def
pub const OP_NOP:usize = 0;
pub const OP_GETLOCAL:usize = 1;
pub const OP_SETLOCAL:usize = 2;
pub const OP_GETBLOCKPARAM:usize = 3;
pub const OP_SETBLOCKPARAM:usize = 4;
pub const OP_GETBLOCKPARAMPROXY:usize = 5;
pub const OP_GETSPECIAL:usize = 6;
pub const OP_SETSPECIAL:usize = 7;
pub const OP_GETINSTANCEVARIABLE:usize = 8;
pub const OP_SETINSTANCEVARIABLE:usize = 9;
pub const OP_GETCLASSVARIABLE:usize = 10;
pub const OP_SETCLASSVARIABLE:usize = 11;
pub const OP_GETCONSTANT:usize = 12;
pub const OP_SETCONSTANT:usize = 13;
pub const OP_GETGLOBAL:usize = 14;
pub const OP_SETGLOBAL:usize = 15;
pub const OP_PUTNIL:usize = 16;
pub const OP_PUTSELF:usize = 17;
pub const OP_PUTOBJECT:usize = 18;
pub const OP_PUTSPECIALOBJECT:usize = 19;
pub const OP_PUTSTRING:usize = 20;
pub const OP_CONCATSTRINGS:usize = 21;
pub const OP_ANYTOSTRING:usize = 22;
pub const OP_TOREGEXP:usize = 23;
pub const OP_INTERN:usize = 24;
pub const OP_NEWARRAY:usize = 25;
pub const OP_NEWARRAYKWSPLAT:usize = 26;
pub const OP_DUPARRAY:usize = 27;
pub const OP_DUPHASH:usize = 28;
pub const OP_EXPANDARRAY:usize = 29;
pub const OP_CONCATARRAY:usize = 30;
pub const OP_SPLATARRAY:usize = 31;
pub const OP_NEWHASH:usize = 32;
pub const OP_NEWRANGE:usize = 33;
pub const OP_POP:usize = 34;
pub const OP_DUP:usize = 35;
pub const OP_DUPN:usize = 36;
pub const OP_SWAP:usize = 37;
pub const OP_TOPN:usize = 38;
pub const OP_SETN:usize = 39;
pub const OP_ADJUSTSTACK:usize = 40;
pub const OP_DEFINED:usize = 41;
pub const OP_CHECKMATCH:usize = 42;
pub const OP_CHECKKEYWORD:usize = 43;
pub const OP_CHECKTYPE:usize = 44;
pub const OP_DEFINECLASS:usize = 45;
pub const OP_DEFINEMETHOD:usize = 46;
pub const OP_DEFINESMETHOD:usize = 47;
pub const OP_SEND:usize = 48;
pub const OP_OPT_SEND_WITHOUT_BLOCK:usize = 49;
pub const OP_OBJTOSTRING:usize = 50;
pub const OP_OPT_STR_FREEZE:usize = 51;
pub const OP_OPT_NIL_P:usize = 52;
pub const OP_OPT_STR_UMINUS:usize = 53;
pub const OP_OPT_NEWARRAY_MAX:usize = 54;
pub const OP_OPT_NEWARRAY_MIN:usize = 55;
pub const OP_INVOKESUPER:usize = 56;
pub const OP_INVOKEBLOCK:usize = 57;
pub const OP_LEAVE:usize = 58;
pub const OP_THROW:usize = 59;
pub const OP_JUMP:usize = 60;
pub const OP_BRANCHIF:usize = 61;
pub const OP_BRANCHUNLESS:usize = 62;
pub const OP_BRANCHNIL:usize = 63;
pub const OP_OPT_GETINLINECACHE:usize = 64;
pub const OP_OPT_SETINLINECACHE:usize = 65;
pub const OP_ONCE:usize = 66;
pub const OP_OPT_CASE_DISPATCH:usize = 67;
pub const OP_OPT_PLUS:usize = 68;
pub const OP_OPT_MINUS:usize = 69;
pub const OP_OPT_MULT:usize = 70;
pub const OP_OPT_DIV:usize = 71;
pub const OP_OPT_MOD:usize = 72;
pub const OP_OPT_EQ:usize = 73;
pub const OP_OPT_NEQ:usize = 74;
pub const OP_OPT_LT:usize = 75;
pub const OP_OPT_LE:usize = 76;
pub const OP_OPT_GT:usize = 77;
pub const OP_OPT_GE:usize = 78;
pub const OP_OPT_LTLT:usize = 79;
pub const OP_OPT_AND:usize = 80;
pub const OP_OPT_OR:usize = 81;
pub const OP_OPT_AREF:usize = 82;
pub const OP_OPT_ASET:usize = 83;
pub const OP_OPT_ASET_WITH:usize = 84;
pub const OP_OPT_AREF_WITH:usize = 85;
pub const OP_OPT_LENGTH:usize = 86;
pub const OP_OPT_SIZE:usize = 87;
pub const OP_OPT_EMPTY_P:usize = 88;
pub const OP_OPT_SUCC:usize = 89;
pub const OP_OPT_NOT:usize = 90;
pub const OP_OPT_REGEXPMATCH2:usize = 91;
pub const OP_INVOKEBUILTIN:usize = 92;
pub const OP_OPT_INVOKEBUILTIN_DELEGATE:usize = 93;
pub const OP_OPT_INVOKEBUILTIN_DELEGATE_LEAVE:usize = 94;
pub const OP_GETLOCAL_WC_0:usize = 95;
pub const OP_GETLOCAL_WC_1:usize = 96;
pub const OP_SETLOCAL_WC_0:usize = 97;
pub const OP_SETLOCAL_WC_1:usize = 98;
pub const OP_PUTOBJECT_INT2FIX_0_:usize = 99;
pub const OP_PUTOBJECT_INT2FIX_1_:usize = 100;

pub const VM_INSTRUCTION_SIZE:usize = 202;
