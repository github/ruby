/* automatically generated by rust-bindgen 0.59.2 */

pub const INTEGER_REDEFINED_OP_FLAG: u32 = 1;
pub const FLOAT_REDEFINED_OP_FLAG: u32 = 2;
pub const STRING_REDEFINED_OP_FLAG: u32 = 4;
pub const ARRAY_REDEFINED_OP_FLAG: u32 = 8;
pub const HASH_REDEFINED_OP_FLAG: u32 = 16;
pub const SYMBOL_REDEFINED_OP_FLAG: u32 = 64;
pub const TIME_REDEFINED_OP_FLAG: u32 = 128;
pub const REGEXP_REDEFINED_OP_FLAG: u32 = 256;
pub const NIL_REDEFINED_OP_FLAG: u32 = 512;
pub const TRUE_REDEFINED_OP_FLAG: u32 = 1024;
pub const FALSE_REDEFINED_OP_FLAG: u32 = 2048;
pub const PROC_REDEFINED_OP_FLAG: u32 = 4096;
pub const VM_BLOCK_HANDLER_NONE: u32 = 0;
pub type ID = ::std::os::raw::c_ulong;
extern "C" {
    pub fn rb_singleton_class(obj: VALUE) -> VALUE;
}
pub type rb_alloc_func_t = ::std::option::Option<unsafe extern "C" fn(klass: VALUE) -> VALUE>;
extern "C" {
    pub fn rb_get_alloc_func(klass: VALUE) -> rb_alloc_func_t;
}
#[repr(C)]
pub struct RBasic {
    pub flags: VALUE,
    pub klass: VALUE,
}
pub const RUBY_T_NONE: ruby_value_type = 0;
pub const RUBY_T_OBJECT: ruby_value_type = 1;
pub const RUBY_T_CLASS: ruby_value_type = 2;
pub const RUBY_T_MODULE: ruby_value_type = 3;
pub const RUBY_T_FLOAT: ruby_value_type = 4;
pub const RUBY_T_STRING: ruby_value_type = 5;
pub const RUBY_T_REGEXP: ruby_value_type = 6;
pub const RUBY_T_ARRAY: ruby_value_type = 7;
pub const RUBY_T_HASH: ruby_value_type = 8;
pub const RUBY_T_STRUCT: ruby_value_type = 9;
pub const RUBY_T_BIGNUM: ruby_value_type = 10;
pub const RUBY_T_FILE: ruby_value_type = 11;
pub const RUBY_T_DATA: ruby_value_type = 12;
pub const RUBY_T_MATCH: ruby_value_type = 13;
pub const RUBY_T_COMPLEX: ruby_value_type = 14;
pub const RUBY_T_RATIONAL: ruby_value_type = 15;
pub const RUBY_T_NIL: ruby_value_type = 17;
pub const RUBY_T_TRUE: ruby_value_type = 18;
pub const RUBY_T_FALSE: ruby_value_type = 19;
pub const RUBY_T_SYMBOL: ruby_value_type = 20;
pub const RUBY_T_FIXNUM: ruby_value_type = 21;
pub const RUBY_T_UNDEF: ruby_value_type = 22;
pub const RUBY_T_IMEMO: ruby_value_type = 26;
pub const RUBY_T_NODE: ruby_value_type = 27;
pub const RUBY_T_ICLASS: ruby_value_type = 28;
pub const RUBY_T_ZOMBIE: ruby_value_type = 29;
pub const RUBY_T_MOVED: ruby_value_type = 30;
pub const RUBY_T_MASK: ruby_value_type = 31;
pub type ruby_value_type = u32;
pub type st_data_t = ::std::os::raw::c_ulong;
pub type st_index_t = st_data_t;
extern "C" {
    pub static mut rb_mKernel: VALUE;
}
extern "C" {
    pub static mut rb_cBasicObject: VALUE;
}
extern "C" {
    pub static mut rb_cFalseClass: VALUE;
}
extern "C" {
    pub static mut rb_cFloat: VALUE;
}
extern "C" {
    pub static mut rb_cInteger: VALUE;
}
extern "C" {
    pub static mut rb_cModule: VALUE;
}
extern "C" {
    pub static mut rb_cNilClass: VALUE;
}
extern "C" {
    pub static mut rb_cString: VALUE;
}
extern "C" {
    pub static mut rb_cSymbol: VALUE;
}
extern "C" {
    pub static mut rb_cThread: VALUE;
}
extern "C" {
    pub static mut rb_cTrueClass: VALUE;
}
extern "C" {
    pub fn rb_ary_resurrect(ary: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_ary_clear(ary: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_hash_new() -> VALUE;
}
extern "C" {
    pub fn rb_hash_aset(hash: VALUE, key: VALUE, val: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_hash_bulk_insert(argc: ::std::os::raw::c_long, argv: *const VALUE, hash: VALUE);
}
extern "C" {
    pub fn rb_id2sym(id: ID) -> VALUE;
}
extern "C" {
    pub fn rb_intern(name: *const ::std::os::raw::c_char) -> ID;
}
extern "C" {
    pub fn rb_obj_is_kind_of(obj: VALUE, klass: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_backref_get() -> VALUE;
}
extern "C" {
    pub fn rb_range_new(beg: VALUE, end: VALUE, excl: ::std::os::raw::c_int) -> VALUE;
}
extern "C" {
    pub fn rb_reg_nth_match(n: ::std::os::raw::c_int, md: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_reg_last_match(md: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_reg_match_pre(md: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_reg_match_post(md: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_reg_match_last(md: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_ivar_get(obj: VALUE, name: ID) -> VALUE;
}
extern "C" {
    pub fn rb_attr_get(obj: VALUE, name: ID) -> VALUE;
}
extern "C" {
    pub fn rb_reg_new_ary(ary: VALUE, options: ::std::os::raw::c_int) -> VALUE;
}
pub const idDot2: ruby_method_ids = 128;
pub const idDot3: ruby_method_ids = 129;
pub const idUPlus: ruby_method_ids = 132;
pub const idUMinus: ruby_method_ids = 133;
pub const idPow: ruby_method_ids = 134;
pub const idCmp: ruby_method_ids = 135;
pub const idPLUS: ruby_method_ids = 43;
pub const idMINUS: ruby_method_ids = 45;
pub const idMULT: ruby_method_ids = 42;
pub const idDIV: ruby_method_ids = 47;
pub const idMOD: ruby_method_ids = 37;
pub const idLTLT: ruby_method_ids = 136;
pub const idGTGT: ruby_method_ids = 137;
pub const idLT: ruby_method_ids = 60;
pub const idLE: ruby_method_ids = 138;
pub const idGT: ruby_method_ids = 62;
pub const idGE: ruby_method_ids = 139;
pub const idEq: ruby_method_ids = 140;
pub const idEqq: ruby_method_ids = 141;
pub const idNeq: ruby_method_ids = 142;
pub const idNot: ruby_method_ids = 33;
pub const idAnd: ruby_method_ids = 38;
pub const idOr: ruby_method_ids = 124;
pub const idBackquote: ruby_method_ids = 96;
pub const idEqTilde: ruby_method_ids = 143;
pub const idNeqTilde: ruby_method_ids = 144;
pub const idAREF: ruby_method_ids = 145;
pub const idASET: ruby_method_ids = 146;
pub const idCOLON2: ruby_method_ids = 147;
pub const idANDOP: ruby_method_ids = 148;
pub const idOROP: ruby_method_ids = 149;
pub const idANDDOT: ruby_method_ids = 150;
pub const tPRESERVED_ID_BEGIN: ruby_method_ids = 150;
pub const idNilP: ruby_method_ids = 151;
pub const idNULL: ruby_method_ids = 152;
pub const idEmptyP: ruby_method_ids = 153;
pub const idEqlP: ruby_method_ids = 154;
pub const idRespond_to: ruby_method_ids = 155;
pub const idRespond_to_missing: ruby_method_ids = 156;
pub const idIFUNC: ruby_method_ids = 157;
pub const idCFUNC: ruby_method_ids = 158;
pub const id_core_set_method_alias: ruby_method_ids = 159;
pub const id_core_set_variable_alias: ruby_method_ids = 160;
pub const id_core_undef_method: ruby_method_ids = 161;
pub const id_core_define_method: ruby_method_ids = 162;
pub const id_core_define_singleton_method: ruby_method_ids = 163;
pub const id_core_set_postexe: ruby_method_ids = 164;
pub const id_core_hash_merge_ptr: ruby_method_ids = 165;
pub const id_core_hash_merge_kwd: ruby_method_ids = 166;
pub const id_core_raise: ruby_method_ids = 167;
pub const id_core_sprintf: ruby_method_ids = 168;
pub const id_debug_created_info: ruby_method_ids = 169;
pub const tPRESERVED_ID_END: ruby_method_ids = 170;
pub const tTOKEN_LOCAL_BEGIN: ruby_method_ids = 169;
pub const tMax: ruby_method_ids = 170;
pub const tMin: ruby_method_ids = 171;
pub const tFreeze: ruby_method_ids = 172;
pub const tInspect: ruby_method_ids = 173;
pub const tIntern: ruby_method_ids = 174;
pub const tObject_id: ruby_method_ids = 175;
pub const tConst_added: ruby_method_ids = 176;
pub const tConst_missing: ruby_method_ids = 177;
pub const tMethodMissing: ruby_method_ids = 178;
pub const tMethod_added: ruby_method_ids = 179;
pub const tSingleton_method_added: ruby_method_ids = 180;
pub const tMethod_removed: ruby_method_ids = 181;
pub const tSingleton_method_removed: ruby_method_ids = 182;
pub const tMethod_undefined: ruby_method_ids = 183;
pub const tSingleton_method_undefined: ruby_method_ids = 184;
pub const tLength: ruby_method_ids = 185;
pub const tSize: ruby_method_ids = 186;
pub const tGets: ruby_method_ids = 187;
pub const tSucc: ruby_method_ids = 188;
pub const tEach: ruby_method_ids = 189;
pub const tProc: ruby_method_ids = 190;
pub const tLambda: ruby_method_ids = 191;
pub const tSend: ruby_method_ids = 192;
pub const t__send__: ruby_method_ids = 193;
pub const t__attached__: ruby_method_ids = 194;
pub const t__recursive_key__: ruby_method_ids = 195;
pub const tInitialize: ruby_method_ids = 196;
pub const tInitialize_copy: ruby_method_ids = 197;
pub const tInitialize_clone: ruby_method_ids = 198;
pub const tInitialize_dup: ruby_method_ids = 199;
pub const tTo_int: ruby_method_ids = 200;
pub const tTo_ary: ruby_method_ids = 201;
pub const tTo_str: ruby_method_ids = 202;
pub const tTo_sym: ruby_method_ids = 203;
pub const tTo_hash: ruby_method_ids = 204;
pub const tTo_proc: ruby_method_ids = 205;
pub const tTo_io: ruby_method_ids = 206;
pub const tTo_a: ruby_method_ids = 207;
pub const tTo_s: ruby_method_ids = 208;
pub const tTo_i: ruby_method_ids = 209;
pub const tTo_f: ruby_method_ids = 210;
pub const tTo_r: ruby_method_ids = 211;
pub const tBt: ruby_method_ids = 212;
pub const tBt_locations: ruby_method_ids = 213;
pub const tCall: ruby_method_ids = 214;
pub const tMesg: ruby_method_ids = 215;
pub const tException: ruby_method_ids = 216;
pub const tLocals: ruby_method_ids = 217;
pub const tNOT: ruby_method_ids = 218;
pub const tAND: ruby_method_ids = 219;
pub const tOR: ruby_method_ids = 220;
pub const tDiv: ruby_method_ids = 221;
pub const tDivmod: ruby_method_ids = 222;
pub const tFdiv: ruby_method_ids = 223;
pub const tQuo: ruby_method_ids = 224;
pub const tName: ruby_method_ids = 225;
pub const tNil: ruby_method_ids = 226;
pub const tUScore: ruby_method_ids = 227;
pub const tNUMPARAM_1: ruby_method_ids = 228;
pub const tNUMPARAM_2: ruby_method_ids = 229;
pub const tNUMPARAM_3: ruby_method_ids = 230;
pub const tNUMPARAM_4: ruby_method_ids = 231;
pub const tNUMPARAM_5: ruby_method_ids = 232;
pub const tNUMPARAM_6: ruby_method_ids = 233;
pub const tNUMPARAM_7: ruby_method_ids = 234;
pub const tNUMPARAM_8: ruby_method_ids = 235;
pub const tNUMPARAM_9: ruby_method_ids = 236;
pub const tTOKEN_LOCAL_END: ruby_method_ids = 237;
pub const tTOKEN_INSTANCE_BEGIN: ruby_method_ids = 236;
pub const tTOKEN_INSTANCE_END: ruby_method_ids = 237;
pub const tTOKEN_GLOBAL_BEGIN: ruby_method_ids = 236;
pub const tLASTLINE: ruby_method_ids = 237;
pub const tBACKREF: ruby_method_ids = 238;
pub const tERROR_INFO: ruby_method_ids = 239;
pub const tTOKEN_GLOBAL_END: ruby_method_ids = 240;
pub const tTOKEN_CONST_BEGIN: ruby_method_ids = 239;
pub const tTOKEN_CONST_END: ruby_method_ids = 240;
pub const tTOKEN_CLASS_BEGIN: ruby_method_ids = 239;
pub const tTOKEN_CLASS_END: ruby_method_ids = 240;
pub const tTOKEN_ATTRSET_BEGIN: ruby_method_ids = 239;
pub const tTOKEN_ATTRSET_END: ruby_method_ids = 240;
pub const tNEXT_ID: ruby_method_ids = 240;
pub const idMax: ruby_method_ids = 2721;
pub const idMin: ruby_method_ids = 2737;
pub const idFreeze: ruby_method_ids = 2753;
pub const idInspect: ruby_method_ids = 2769;
pub const idIntern: ruby_method_ids = 2785;
pub const idObject_id: ruby_method_ids = 2801;
pub const idConst_added: ruby_method_ids = 2817;
pub const idConst_missing: ruby_method_ids = 2833;
pub const idMethodMissing: ruby_method_ids = 2849;
pub const idMethod_added: ruby_method_ids = 2865;
pub const idSingleton_method_added: ruby_method_ids = 2881;
pub const idMethod_removed: ruby_method_ids = 2897;
pub const idSingleton_method_removed: ruby_method_ids = 2913;
pub const idMethod_undefined: ruby_method_ids = 2929;
pub const idSingleton_method_undefined: ruby_method_ids = 2945;
pub const idLength: ruby_method_ids = 2961;
pub const idSize: ruby_method_ids = 2977;
pub const idGets: ruby_method_ids = 2993;
pub const idSucc: ruby_method_ids = 3009;
pub const idEach: ruby_method_ids = 3025;
pub const idProc: ruby_method_ids = 3041;
pub const idLambda: ruby_method_ids = 3057;
pub const idSend: ruby_method_ids = 3073;
pub const id__send__: ruby_method_ids = 3089;
pub const id__attached__: ruby_method_ids = 3105;
pub const id__recursive_key__: ruby_method_ids = 3121;
pub const idInitialize: ruby_method_ids = 3137;
pub const idInitialize_copy: ruby_method_ids = 3153;
pub const idInitialize_clone: ruby_method_ids = 3169;
pub const idInitialize_dup: ruby_method_ids = 3185;
pub const idTo_int: ruby_method_ids = 3201;
pub const idTo_ary: ruby_method_ids = 3217;
pub const idTo_str: ruby_method_ids = 3233;
pub const idTo_sym: ruby_method_ids = 3249;
pub const idTo_hash: ruby_method_ids = 3265;
pub const idTo_proc: ruby_method_ids = 3281;
pub const idTo_io: ruby_method_ids = 3297;
pub const idTo_a: ruby_method_ids = 3313;
pub const idTo_s: ruby_method_ids = 3329;
pub const idTo_i: ruby_method_ids = 3345;
pub const idTo_f: ruby_method_ids = 3361;
pub const idTo_r: ruby_method_ids = 3377;
pub const idBt: ruby_method_ids = 3393;
pub const idBt_locations: ruby_method_ids = 3409;
pub const idCall: ruby_method_ids = 3425;
pub const idMesg: ruby_method_ids = 3441;
pub const idException: ruby_method_ids = 3457;
pub const idLocals: ruby_method_ids = 3473;
pub const idNOT: ruby_method_ids = 3489;
pub const idAND: ruby_method_ids = 3505;
pub const idOR: ruby_method_ids = 3521;
pub const idDiv: ruby_method_ids = 3537;
pub const idDivmod: ruby_method_ids = 3553;
pub const idFdiv: ruby_method_ids = 3569;
pub const idQuo: ruby_method_ids = 3585;
pub const idName: ruby_method_ids = 3601;
pub const idNil: ruby_method_ids = 3617;
pub const idUScore: ruby_method_ids = 3633;
pub const idNUMPARAM_1: ruby_method_ids = 3649;
pub const idNUMPARAM_2: ruby_method_ids = 3665;
pub const idNUMPARAM_3: ruby_method_ids = 3681;
pub const idNUMPARAM_4: ruby_method_ids = 3697;
pub const idNUMPARAM_5: ruby_method_ids = 3713;
pub const idNUMPARAM_6: ruby_method_ids = 3729;
pub const idNUMPARAM_7: ruby_method_ids = 3745;
pub const idNUMPARAM_8: ruby_method_ids = 3761;
pub const idNUMPARAM_9: ruby_method_ids = 3777;
pub const idLASTLINE: ruby_method_ids = 3799;
pub const idBACKREF: ruby_method_ids = 3815;
pub const idERROR_INFO: ruby_method_ids = 3831;
pub const tLAST_OP_ID: ruby_method_ids = 169;
pub const idLAST_OP_ID: ruby_method_ids = 10;
pub type ruby_method_ids = u32;
extern "C" {
    pub fn rb_ary_tmp_new_from_values(
        arg1: VALUE,
        arg2: ::std::os::raw::c_long,
        arg3: *const VALUE,
    ) -> VALUE;
}
extern "C" {
    pub fn rb_ec_ary_new_from_values(
        ec: *mut rb_execution_context_struct,
        n: ::std::os::raw::c_long,
        elts: *const VALUE,
    ) -> VALUE;
}
pub type rb_serial_t = ::std::os::raw::c_ulonglong;
extern "C" {
    pub fn rb_class_allocate_instance(klass: VALUE) -> VALUE;
}
pub const METHOD_VISI_UNDEF: rb_method_visibility_t = 0;
pub const METHOD_VISI_PUBLIC: rb_method_visibility_t = 1;
pub const METHOD_VISI_PRIVATE: rb_method_visibility_t = 2;
pub const METHOD_VISI_PROTECTED: rb_method_visibility_t = 3;
pub const METHOD_VISI_MASK: rb_method_visibility_t = 3;
pub type rb_method_visibility_t = u32;
#[repr(C)]
pub struct rb_method_entry_struct {
    pub flags: VALUE,
    pub defined_class: VALUE,
    pub def: *mut rb_method_definition_struct,
    pub called_id: ID,
    pub owner: VALUE,
}
pub type rb_method_entry_t = rb_method_entry_struct;
#[repr(C)]
pub struct rb_callable_method_entry_struct {
    pub flags: VALUE,
    pub defined_class: VALUE,
    pub def: *mut rb_method_definition_struct,
    pub called_id: ID,
    pub owner: VALUE,
}
pub type rb_callable_method_entry_t = rb_callable_method_entry_struct;
pub const VM_METHOD_TYPE_ISEQ: rb_method_type_t = 0;
pub const VM_METHOD_TYPE_CFUNC: rb_method_type_t = 1;
pub const VM_METHOD_TYPE_ATTRSET: rb_method_type_t = 2;
pub const VM_METHOD_TYPE_IVAR: rb_method_type_t = 3;
pub const VM_METHOD_TYPE_BMETHOD: rb_method_type_t = 4;
pub const VM_METHOD_TYPE_ZSUPER: rb_method_type_t = 5;
pub const VM_METHOD_TYPE_ALIAS: rb_method_type_t = 6;
pub const VM_METHOD_TYPE_UNDEF: rb_method_type_t = 7;
pub const VM_METHOD_TYPE_NOTIMPLEMENTED: rb_method_type_t = 8;
pub const VM_METHOD_TYPE_OPTIMIZED: rb_method_type_t = 9;
pub const VM_METHOD_TYPE_MISSING: rb_method_type_t = 10;
pub const VM_METHOD_TYPE_REFINED: rb_method_type_t = 11;
pub type rb_method_type_t = u32;
pub const OPTIMIZED_METHOD_TYPE_SEND: method_optimized_type = 0;
pub const OPTIMIZED_METHOD_TYPE_CALL: method_optimized_type = 1;
pub const OPTIMIZED_METHOD_TYPE_BLOCK_CALL: method_optimized_type = 2;
pub const OPTIMIZED_METHOD_TYPE_STRUCT_AREF: method_optimized_type = 3;
pub const OPTIMIZED_METHOD_TYPE_STRUCT_ASET: method_optimized_type = 4;
pub const OPTIMIZED_METHOD_TYPE__MAX: method_optimized_type = 5;
pub type method_optimized_type = u32;
extern "C" {
    pub fn rb_method_entry_at(obj: VALUE, id: ID) -> *const rb_method_entry_t;
}
extern "C" {
    pub fn rb_callable_method_entry(klass: VALUE, id: ID) -> *const rb_callable_method_entry_t;
}
pub type rb_num_t = ::std::os::raw::c_ulong;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct iseq_inline_iv_cache_entry {
    pub entry: *mut rb_iv_index_tbl_entry,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct iseq_inline_cvar_cache_entry {
    pub entry: *mut rb_cvar_class_tbl_entry,
}
pub const BOP_PLUS: ruby_basic_operators = 0;
pub const BOP_MINUS: ruby_basic_operators = 1;
pub const BOP_MULT: ruby_basic_operators = 2;
pub const BOP_DIV: ruby_basic_operators = 3;
pub const BOP_MOD: ruby_basic_operators = 4;
pub const BOP_EQ: ruby_basic_operators = 5;
pub const BOP_EQQ: ruby_basic_operators = 6;
pub const BOP_LT: ruby_basic_operators = 7;
pub const BOP_LE: ruby_basic_operators = 8;
pub const BOP_LTLT: ruby_basic_operators = 9;
pub const BOP_AREF: ruby_basic_operators = 10;
pub const BOP_ASET: ruby_basic_operators = 11;
pub const BOP_LENGTH: ruby_basic_operators = 12;
pub const BOP_SIZE: ruby_basic_operators = 13;
pub const BOP_EMPTY_P: ruby_basic_operators = 14;
pub const BOP_NIL_P: ruby_basic_operators = 15;
pub const BOP_SUCC: ruby_basic_operators = 16;
pub const BOP_GT: ruby_basic_operators = 17;
pub const BOP_GE: ruby_basic_operators = 18;
pub const BOP_NOT: ruby_basic_operators = 19;
pub const BOP_NEQ: ruby_basic_operators = 20;
pub const BOP_MATCH: ruby_basic_operators = 21;
pub const BOP_FREEZE: ruby_basic_operators = 22;
pub const BOP_UMINUS: ruby_basic_operators = 23;
pub const BOP_MAX: ruby_basic_operators = 24;
pub const BOP_MIN: ruby_basic_operators = 25;
pub const BOP_CALL: ruby_basic_operators = 26;
pub const BOP_AND: ruby_basic_operators = 27;
pub const BOP_OR: ruby_basic_operators = 28;
pub const BOP_LAST_: ruby_basic_operators = 29;
pub type ruby_basic_operators = u32;
pub type IVC = *mut iseq_inline_iv_cache_entry;
pub type ICVARC = *mut iseq_inline_cvar_cache_entry;
pub const VM_FRAME_MAGIC_METHOD: u32 = 286326785;
pub const VM_FRAME_MAGIC_BLOCK: u32 = 572653569;
pub const VM_FRAME_MAGIC_CLASS: u32 = 858980353;
pub const VM_FRAME_MAGIC_TOP: u32 = 1145307137;
pub const VM_FRAME_MAGIC_CFUNC: u32 = 1431633921;
pub const VM_FRAME_MAGIC_IFUNC: u32 = 1717960705;
pub const VM_FRAME_MAGIC_EVAL: u32 = 2004287489;
pub const VM_FRAME_MAGIC_RESCUE: u32 = 2022178817;
pub const VM_FRAME_MAGIC_DUMMY: u32 = 2040070145;
pub const VM_FRAME_MAGIC_MASK: u32 = 2147418113;
pub const VM_FRAME_FLAG_FINISH: u32 = 32;
pub const VM_FRAME_FLAG_BMETHOD: u32 = 64;
pub const VM_FRAME_FLAG_CFRAME: u32 = 128;
pub const VM_FRAME_FLAG_LAMBDA: u32 = 256;
pub const VM_FRAME_FLAG_MODIFIED_BLOCK_PARAM: u32 = 512;
pub const VM_FRAME_FLAG_CFRAME_KW: u32 = 1024;
pub const VM_FRAME_FLAG_PASSED: u32 = 2048;
pub const VM_ENV_FLAG_LOCAL: u32 = 2;
pub const VM_ENV_FLAG_ESCAPED: u32 = 4;
pub const VM_ENV_FLAG_WB_REQUIRED: u32 = 8;
pub const VM_ENV_FLAG_ISOLATED: u32 = 16;
pub type _bindgen_ty_12 = u32;
pub const VM_CALL_ARGS_SPLAT_bit: vm_call_flag_bits = 0;
pub const VM_CALL_ARGS_BLOCKARG_bit: vm_call_flag_bits = 1;
pub const VM_CALL_FCALL_bit: vm_call_flag_bits = 2;
pub const VM_CALL_VCALL_bit: vm_call_flag_bits = 3;
pub const VM_CALL_ARGS_SIMPLE_bit: vm_call_flag_bits = 4;
pub const VM_CALL_BLOCKISEQ_bit: vm_call_flag_bits = 5;
pub const VM_CALL_KWARG_bit: vm_call_flag_bits = 6;
pub const VM_CALL_KW_SPLAT_bit: vm_call_flag_bits = 7;
pub const VM_CALL_TAILCALL_bit: vm_call_flag_bits = 8;
pub const VM_CALL_SUPER_bit: vm_call_flag_bits = 9;
pub const VM_CALL_ZSUPER_bit: vm_call_flag_bits = 10;
pub const VM_CALL_OPT_SEND_bit: vm_call_flag_bits = 11;
pub const VM_CALL_KW_SPLAT_MUT_bit: vm_call_flag_bits = 12;
pub const VM_CALL__END: vm_call_flag_bits = 13;
pub type vm_call_flag_bits = u32;
extern "C" {
    pub fn rb_str_concat_literals(num: size_t, strary: *const VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_ec_str_resurrect(ec: *mut rb_execution_context_struct, str_: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_hash_new_with_size(size: st_index_t) -> VALUE;
}
extern "C" {
    pub fn rb_hash_resurrect(hash: VALUE) -> VALUE;
}
extern "C" {
    pub fn rb_obj_ensure_iv_index_mapping(obj: VALUE, id: ID) -> u32;
}
extern "C" {
    pub fn rb_gvar_get(arg1: ID) -> VALUE;
}
extern "C" {
    pub fn rb_gvar_set(arg1: ID, arg2: VALUE) -> VALUE;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rb_builtin_function {
    pub func_ptr: *const ::std::os::raw::c_void,
    pub argc: ::std::os::raw::c_int,
    pub index: ::std::os::raw::c_int,
    pub name: *const ::std::os::raw::c_char,
    pub compiler: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut FILE,
            arg2: ::std::os::raw::c_long,
            arg3: ::std::os::raw::c_uint,
            arg4: bool,
        ),
    >,
}
extern "C" {
    pub fn rb_vm_insn_addr2opcode(addr: *const ::std::os::raw::c_void) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn rb_yjit_mark_writable(mem_block: *mut ::std::os::raw::c_void, mem_size: u32);
}
extern "C" {
    pub fn rb_yjit_mark_executable(mem_block: *mut ::std::os::raw::c_void, mem_size: u32);
}
extern "C" {
    pub fn rb_yjit_get_page_size() -> u32;
}
extern "C" {
    pub fn rb_iseq_get_yjit_payload(iseq: *const rb_iseq_t) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn rb_iseq_set_yjit_payload(iseq: *const rb_iseq_t, payload: *mut ::std::os::raw::c_void);
}
extern "C" {
    pub fn rb_iseq_pc_at_idx(iseq: *const rb_iseq_t, insn_idx: u32) -> *mut VALUE;
}
extern "C" {
    pub fn rb_iseq_opcode_at_pc(iseq: *const rb_iseq_t, pc: *const VALUE) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn rb_leaf_invokebuiltin_iseq_p(iseq: *const rb_iseq_t) -> bool;
}
extern "C" {
    pub fn rb_leaf_builtin_function(iseq: *const rb_iseq_t) -> *const rb_builtin_function;
}
extern "C" {
    pub fn rb_set_cfp_pc(cfp: *mut rb_control_frame_struct, pc: *const VALUE);
}
extern "C" {
    pub fn rb_set_cfp_sp(cfp: *mut rb_control_frame_struct, sp: *mut VALUE);
}
extern "C" {
    pub fn rb_cfp_get_iseq(cfp: *mut rb_control_frame_struct) -> *mut rb_iseq_t;
}
#[repr(C)]
pub struct rb_iv_index_tbl_entry {
    pub index: u32,
    pub class_serial: rb_serial_t,
    pub class_value: VALUE,
}
#[repr(C)]
pub struct rb_cvar_class_tbl_entry {
    pub index: u32,
    pub global_cvar_state: rb_serial_t,
    pub class_value: VALUE,
}
