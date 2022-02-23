// YJIT combined compilation unit. This setup allows spreading functions
// across different files without having to worry about putting things
// in headers and prefixing function names.

#include "internal.h"
#include "internal/string.h"
#include "vm_core.h"
#include "vm_callinfo.h"
#include "builtin.h"
#include "insns.inc"
#include "insns_info.inc"
#include "vm_sync.h"
#include "yjit.h"

#ifndef YJIT_CHECK_MODE
# define YJIT_CHECK_MODE 0
#endif

// >= 1: print when output code invalidation happens
// >= 2: dump list of instructions when regions compile
#ifndef YJIT_DUMP_MODE
# define YJIT_DUMP_MODE 0
#endif

#if defined(__x86_64__) && !defined(_WIN32)
# define PLATFORM_SUPPORTED_P 1
#else
# define PLATFORM_SUPPORTED_P 0
#endif

// USE_MJIT comes from configure options
#define JIT_ENABLED USE_MJIT

// Check if we need to include YJIT in the build
#if JIT_ENABLED && PLATFORM_SUPPORTED_P

#include "yjit_asm.c"

// Code block into which we write machine code
static codeblock_t *cb = NULL;

// Code block into which we write out-of-line machine code
static codeblock_t *ocb = NULL;

// NOTE: We can trust that uint8_t has no "padding bits" since the C spec
// guarantees it. Wording about padding bits is more explicit in C11 compared
// to C99. See C11 7.20.1.1p2. All this is to say we have _some_ standards backing to
// use a Rust `* u8` to represent a C `* uint8_t`.
//
// If we don't want to trust that we can interpreter the C standard correctly, we
// could outsource that work to the Rust standard library by sticking to fundamental
// types in C such as int, long, etc. and use `std::os::raw::c_long` and friends on
// the Rust side.
//
// What's up with the long prefix? The "rb_" part is to apease `make leaked-globals`
// which runs on upstream CI. The rationale for the check is unclear to Alan as
// we build with `-fvisibility=hidden` so only explicitly marked functions end
// up as public symbols in libruby.so. Perhaps the check is for the static
// libruby and or general namspacing hygiene? Alan admits his bias towards ELF
// platforms and newer compilers.
//
// The "_yjit_" part is for trying to be informative. We might want different
// suffixes for symbols meant for Rust and symbols meant for broader CRuby.

void
rb_yjit_mark_writable(void *mem_block, uint32_t mem_size)
{
    if (mprotect(mem_block, mem_size, PROT_READ | PROT_WRITE)) {
        fprintf(stderr, "Couldn't make JIT page region (%p, %lu bytes) writeable, errno: %s\n",
            mem_block, (unsigned long)mem_size, strerror(errno));
        abort();
    }
}

void
rb_yjit_mark_executable(void *mem_block, uint32_t mem_size) {
    if (mprotect(mem_block, mem_size, PROT_READ | PROT_EXEC)) {
        fprintf(stderr, "Couldn't make JIT page (%p, %lu bytes) executable, errno: %s\n",
            mem_block, (unsigned long)mem_size, strerror(errno));
        abort();
    }
}

uint32_t
rb_yjit_get_page_size(void)
{
#if defined(_SC_PAGESIZE)
    long page_size = sysconf(_SC_PAGESIZE);
    if (page_size <= 0) rb_bug("yjit: failed to get page size");

    // 1 GiB limit. x86 CPUs with PDPE1GB can do this and anything larger is unexpected.
    // Though our design sort of assume we have fine grained control over memory protection
    // which require small page sizes.
    if (page_size > 0x40000000l) rb_bug("yjit page size too large");

    return (uint32_t)page_size;
#else
#error "YJIT supports POSIX only for now"
#endif
}

/*

#if defined(MAP_FIXED_NOREPLACE) && defined(_SC_PAGESIZE)
    // Align the current write position to a multiple of bytes
    static uint8_t *align_ptr(uint8_t *ptr, uint32_t multiple)
    {
        // Compute the pointer modulo the given alignment boundary
        uint32_t rem = ((uint32_t)(uintptr_t)ptr) % multiple;

        // If the pointer is already aligned, stop
        if (rem == 0)
            return ptr;

        // Pad the pointer by the necessary amount to align it
        uint32_t pad = multiple - rem;

        return ptr + pad;
    }
#endif

// Allocate a block of executable memory
uint8_t *rb_yjit_alloc_exec_mem(uint32_t mem_size) {
#ifndef _WIN32
    uint8_t *mem_block;

    // On Linux
    #if defined(MAP_FIXED_NOREPLACE) && defined(_SC_PAGESIZE)
        // Align the requested address to page size
        uint32_t page_size = (uint32_t)sysconf(_SC_PAGESIZE);
        uint8_t *req_addr = align_ptr((uint8_t*)&rb_yjit_alloc_exec_mem, page_size);

        do {
            // Try to map a chunk of memory as executable
            mem_block = (uint8_t*)mmap(
                (void*)req_addr,
                mem_size,
                PROT_READ | PROT_EXEC,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED_NOREPLACE,
                -1,
                0
            );

            // If we succeeded, stop
            if (mem_block != MAP_FAILED) {
                break;
            }

            // +4MB
            req_addr += 4 * 1024 * 1024;
        } while (req_addr < (uint8_t*)&rb_yjit_alloc_exec_mem + INT32_MAX);

    // On MacOS and other platforms
    #else
        // Try to map a chunk of memory as executable
        mem_block = (uint8_t*)mmap(
            (void*)yjit_alloc_exec_mem,
            mem_size,
            PROT_READ | PROT_EXEC,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0
        );
    #endif

    // Fallback
    if (mem_block == MAP_FAILED) {
        // Try again without the address hint (e.g., valgrind)
        mem_block = (uint8_t*)mmap(
            NULL,
            mem_size,
            PROT_READ | PROT_EXEC,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0
        );
    }

    // Check that the memory mapping was successful
    if (mem_block == MAP_FAILED) {
        perror("mmap call failed");
        exit(-1);
    }

    // Fill the executable memory with PUSH DS (0x1E) so that
    // executing uninitialized memory will fault with #UD in
    // 64-bit mode.
    yjit_mark_all_writable(mem_block, mem_size);
    memset(mem_block, 0x1E, mem_size);
    yjit_mark_all_executable(mem_block, mem_size);

    return mem_block;
#else
    // Windows not supported for now
    return NULL;
#endif
}
*/

uint8_t *
rb_yjit_alloc_exec_mem(uint32_t mem_size)
{
    // It's a diff minimization move to wrap instead of rename to export.
    return alloc_exec_mem(mem_size);
}

unsigned int
rb_iseq_encoded_size(const rb_iseq_t *iseq)
{
    return iseq->body->iseq_size;
}

// TODO(alan): consider using an opaque pointer for the payload rather than a void pointer
void *
rb_iseq_get_yjit_payload(const rb_iseq_t *iseq)
{
    RUBY_ASSERT_ALWAYS(IMEMO_TYPE_P(iseq, imemo_iseq));
    return iseq->body->yjit_payload;
}

void
rb_iseq_set_yjit_payload(const rb_iseq_t *iseq, void *payload)
{
    RUBY_ASSERT_ALWAYS(IMEMO_TYPE_P(iseq, imemo_iseq));
    RUBY_ASSERT_ALWAYS(NULL == iseq->body->yjit_payload);
    iseq->body->yjit_payload = payload;
}

// Get the PC for a given index in an iseq
VALUE *
rb_iseq_pc_at_idx(const rb_iseq_t *iseq, uint32_t insn_idx)
{
    RUBY_ASSERT_ALWAYS(IMEMO_TYPE_P(iseq, imemo_iseq));
    RUBY_ASSERT_ALWAYS(insn_idx < iseq->body->iseq_size);
    VALUE *encoded = iseq->body->iseq_encoded;
    VALUE *pc = &encoded[insn_idx];
    return pc;
}

int
rb_iseq_opcode_at_pc(const rb_iseq_t *iseq, const VALUE *pc)
{
    // YJIT should only use iseqs after AST to bytecode compilation
    RUBY_ASSERT_ALWAYS(FL_TEST_RAW((VALUE)iseq, ISEQ_TRANSLATED));

    const VALUE at_pc = *pc;
    return rb_vm_insn_addr2opcode((const void *)at_pc);
}

// Query the instruction length in bytes for YARV opcode insn
int
rb_insn_len(VALUE insn)
{
    return insn_len(insn);
}

unsigned int
rb_vm_ci_argc(const struct rb_callinfo *ci) {
    return vm_ci_argc(ci);
}

ID
rb_vm_ci_mid(const struct rb_callinfo *ci) {
    return vm_ci_mid(ci);
}

unsigned int
rb_vm_ci_flag(const struct rb_callinfo *ci) {
    return vm_ci_flag(ci);
}

rb_method_visibility_t
rb_METHOD_ENTRY_VISI(rb_callable_method_entry_t *me) {
    return METHOD_ENTRY_VISI(me);
}

VALUE
rb_get_cme_defined_class(rb_callable_method_entry_t* cme) {
    return cme->defined_class;
}

rb_method_type_t
rb_get_cme_def_type(rb_callable_method_entry_t* cme)
{
    return cme->def->type;
}

ID
rb_get_cme_def_body_attr_id(rb_callable_method_entry_t* cme)
{
    return cme->def->body.attr.id;
}

enum method_optimized_type
rb_get_cme_def_body_optimized_type(rb_callable_method_entry_t* cme)
{
    return cme->def->body.optimized.type;
}

unsigned int
rb_get_iseq_body_local_table_size(rb_iseq_t* iseq) {
    return iseq->body->local_table_size;
}

VALUE*
rb_get_iseq_body_iseq_encoded(rb_iseq_t* iseq) {
    return iseq->body->iseq_encoded;
}

int
rb_get_iseq_flags_has_opt(rb_iseq_t* iseq) {
    return iseq->body->param.flags.has_opt;
}

int
rb_get_iseq_body_param_num(rb_iseq_t* iseq) {
    return iseq->body->param.keyword->num;
}

struct rb_control_frame_struct *
rb_get_ec_cfp(rb_execution_context_t *ec) {
    return ec->cfp;
}

VALUE*
rb_get_cfp_pc(struct rb_control_frame_struct *cfp) {
    return (VALUE*)cfp->pc;
}

VALUE*
rb_get_cfp_sp(struct rb_control_frame_struct *cfp) {
    return cfp->sp;
}

VALUE
rb_get_cfp_self(struct rb_control_frame_struct *cfp) {
    return cfp->self;
}

VALUE*
rb_get_cfp_ep(struct rb_control_frame_struct *cfp) {
    return (VALUE*)cfp->ep;
}

VALUE
rb_yarv_class_of(VALUE obj)
{
    return rb_class_of(obj);
}

// YJIT needs this function to never allocate and never raise
VALUE
rb_yarv_str_eql_internal(VALUE str1, VALUE str2)
{
    // We wrap this since it's static inline
    return rb_str_eql_internal(str1, str2);
}

// The FL_TEST() macro
VALUE
rb_FL_TEST(VALUE obj, VALUE flags)
{
    return RB_FL_TEST(obj, flags);
}

// The FL_TEST_RAW() macro, normally an internal implementation detail
VALUE
rb_FL_TEST_RAW(VALUE obj, VALUE flags)
{
    return FL_TEST_RAW(obj, flags);
}

// The RB_TYPE_P macro
bool
rb_RB_TYPE_P(VALUE obj, enum ruby_value_type t)
{
    return RB_TYPE_P(obj, t);
}

const struct rb_callinfo*
rb_get_call_data_ci(struct rb_call_data* cd) {
    return cd->ci;
}

// The number of bytes counting from the beginning of the inline code block
// that should not be changed. After patching for global invalidation, no one
// should make changes to the invalidated code region anymore. This is used to
// break out of invalidation race when there are multiple ractors.
static uint32_t yjit_codepage_frozen_bytes = 0;

#include "yjit_core.c"
#include "yjit_iface.c"
#include "yjit_codegen.c"

#endif // if JIT_ENABLED && PLATFORM_SUPPORTED_P
