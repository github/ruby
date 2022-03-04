/// Trait for casting to [usize] that allows you to say `.as_usize()`.
/// Implementation conditional on the the cast preserving the numeric value on
/// all inputs and being inexpensive.
///
/// [usize] is only guaranteed to be more than 16-bit wide, so we can't use
/// `.into()` to cast an `u32` or an `u64` to a `usize` even though in all
/// the platforms YJIT supports these two casts are pretty much no-ops.
/// We could say `as usize` or `.try_convert().unwrap()` everywhere
/// for those casts but they both have undesirable consequences if and when
/// we decide to support 32-bit platforms. Unfortunately we can't implement
/// [::core::convert::From] for [usize] since both the trait and the type are
/// external 😞. Naming the method `into()` also runs into naming conflicts.
pub(crate) trait IntoUsize {
    /// Convert to usize. Implementation conditional on width of [usize].
    fn as_usize(self) -> usize;
}

#[cfg(target_pointer_width = "64")]
impl IntoUsize for u64 {
    fn as_usize(self) -> usize {
        self as usize
    }
}

#[cfg(target_pointer_width = "64")]
impl IntoUsize for u32 {
    fn as_usize(self) -> usize {
        self as usize
    }
}

impl IntoUsize for u16 {
    /// Alias for `.into()`. For convenience so you could use the trait for
    /// all unsgined types.
    fn as_usize(self) -> usize {
        self.into()
    }
}

impl IntoUsize for u8 {
    /// Alias for `.into()`. For convenience so you could use the trait for
    /// all unsgined types.
    fn as_usize(self) -> usize {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn min_max_preserved_after_cast_to_usize() {
        use crate::utils::IntoUsize;

        let min:usize = u64::MIN.as_usize();
        assert_eq!(min, u64::MIN.try_into().unwrap());
        let max:usize = u64::MAX.as_usize();
        assert_eq!(max, u64::MAX.try_into().unwrap());

        let min:usize = u32::MIN.as_usize();
        assert_eq!(min, u32::MIN.try_into().unwrap());
        let max:usize = u32::MAX.as_usize();
        assert_eq!(max, u32::MAX.try_into().unwrap());
    }
}

// TODO: we may want to move this function into yjit.c, maybe add a convenient Rust-side wrapper
/*
// For debugging. Print the disassembly of an iseq.
RBIMPL_ATTR_MAYBE_UNUSED()
static void
yjit_print_iseq(const rb_iseq_t *iseq)
{
    char *ptr;
    long len;
    VALUE disassembly = rb_iseq_disasm(iseq);
    RSTRING_GETMEM(disassembly, ptr, len);
    fprintf(stderr, "%.*s\n", (int)len, ptr);
}
*/

/*
// Save caller-save registers on the stack before a C call
static void
push_regs(codeblock_t *cb)
{
    push(cb, RAX);
    push(cb, RCX);
    push(cb, RDX);
    push(cb, RSI);
    push(cb, RDI);
    push(cb, R8);
    push(cb, R9);
    push(cb, R10);
    push(cb, R11);
    pushfq(cb);
}

// Restore caller-save registers from the after a C call
static void
pop_regs(codeblock_t *cb)
{
    popfq(cb);
    pop(cb, R11);
    pop(cb, R10);
    pop(cb, R9);
    pop(cb, R8);
    pop(cb, RDI);
    pop(cb, RSI);
    pop(cb, RDX);
    pop(cb, RCX);
    pop(cb, RAX);
}

static void
print_int_cfun(int64_t val)
{
    fprintf(stderr, "%lld\n", (long long int)val);
}

RBIMPL_ATTR_MAYBE_UNUSED()
static void
print_int(codeblock_t *cb, x86opnd_t opnd)
{
    push_regs(cb);

    if (opnd.num_bits < 64 && opnd.type != OPND_IMM)
        movsx(cb, RDI, opnd);
    else
        mov(cb, RDI, opnd);

    // Call the print function
    mov(cb, RAX, const_ptr_opnd((void*)&print_int_cfun));
    call(cb, RAX);

    pop_regs(cb);
}

static void
print_ptr_cfun(void *val)
{
    fprintf(stderr, "%p\n", val);
}

RBIMPL_ATTR_MAYBE_UNUSED()
static void
print_ptr(codeblock_t *cb, x86opnd_t opnd)
{
    assert (opnd.num_bits == 64);

    push_regs(cb);

    mov(cb, RDI, opnd);
    mov(cb, RAX, const_ptr_opnd((void*)&print_ptr_cfun));
    call(cb, RAX);

    pop_regs(cb);
}

static void
print_str_cfun(const char *str)
{
    fprintf(stderr, "%s\n", str);
}

// Print a constant string to stdout
static void
print_str(codeblock_t *cb, const char *str)
{
    //as.comment("printStr(\"" ~ str ~ "\")");
    size_t len = strlen(str);

    push_regs(cb);

    // Load the string address and jump over the string data
    lea(cb, RDI, mem_opnd(8, RIP, 5));
    jmp32(cb, (int32_t)len + 1);

    // Write the string chars and a null terminator
    for (size_t i = 0; i < len; ++i)
        cb_write_byte(cb, (uint8_t)str[i]);
    cb_write_byte(cb, 0);

    // Call the print function
    mov(cb, RAX, const_ptr_opnd((void*)&print_str_cfun));
    call(cb, RAX);

    pop_regs(cb);
}
*/
