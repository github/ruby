// This file is a fragment of the yjit.o compilation unit. See yjit.c.
//
// Note that the definition for some of these functions don't specify
// static inline, but their declaration in yjit_asm.h do. The resulting
// linkage is the same as if they both specify. The relevant sections in
// N1256 is 6.2.2p4, 6.2.2p5, and 6.7.4p5.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <stdint.h>
#include <assert.h>
#include <errno.h>

// For mmapp(), sysconf()
#ifndef _WIN32
#include <unistd.h>
#include <sys/mman.h>
#endif

#include "yjit_asm.h"

// Compute the number of bits needed to encode a signed value
uint32_t sig_imm_size(int64_t imm)
{
    // Compute the smallest size this immediate fits in
    if (imm >= INT8_MIN && imm <= INT8_MAX)
        return 8;
    if (imm >= INT16_MIN && imm <= INT16_MAX)
        return 16;
    if (imm >= INT32_MIN && imm <= INT32_MAX)
        return 32;

    return 64;
}

// Allocate a block of executable memory
static uint8_t *alloc_exec_mem(uint32_t mem_size)
{
    uint8_t *mem_block;

    // On Linux
#if defined(MAP_FIXED_NOREPLACE) && defined(_SC_PAGESIZE)
        // Align the requested address to page size
        uint32_t page_size = (uint32_t)sysconf(_SC_PAGESIZE);
        uint8_t *req_addr = align_ptr((uint8_t*)&alloc_exec_mem, page_size);

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
        } while (req_addr < (uint8_t*)&alloc_exec_mem + INT32_MAX);

    // On MacOS and other platforms
#else
        // Try to map a chunk of memory as executable
        mem_block = (uint8_t*)mmap(
            (void*)alloc_exec_mem,
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

    codeblock_t block;
    codeblock_t *cb = &block;

    cb_init(cb, mem_block, mem_size);

    // Fill the executable memory with PUSH DS (0x1E) so that
    // executing uninitialized memory will fault with #UD in
    // 64-bit mode.
    cb_mark_all_writeable(cb);
    memset(mem_block, 0x1E, mem_size);
    cb_mark_all_executable(cb);

    return mem_block;
}

// Initialize a code block object
void cb_init(codeblock_t *cb, uint8_t *mem_block, uint32_t mem_size)
{
    assert (mem_block);
    cb->mem_block_ = mem_block;
    cb->mem_size = mem_size;
    cb->write_pos = 0;
    cb->num_labels = 0;
    cb->num_refs = 0;
    cb->current_aligned_write_pos = ALIGNED_WRITE_POSITION_NONE;
}

// Set the current write position
void cb_set_pos(codeblock_t *cb, uint32_t pos)
{
    // Assert here since while assembler functions do bounds checking, there is
    // nothing stopping users from taking out an out-of-bounds pointer and
    // doing bad accesses with it.
    assert (pos < cb->mem_size);
    cb->write_pos = pos;
}

// Set the current write position from a pointer
void cb_set_write_ptr(codeblock_t *cb, uint8_t *code_ptr)
{
    intptr_t pos = code_ptr - cb->mem_block_;
    assert (pos < cb->mem_size);
    cb_set_pos(cb, (uint32_t)pos);
}

// Get a direct pointer into the executable memory block
uint8_t *cb_get_ptr(const codeblock_t *cb, uint32_t index)
{
    if (index < cb->mem_size) {
        return &cb->mem_block_[index];
    }
    else {
        return NULL;
    }
}




void cb_mark_all_writeable(codeblock_t * cb)
{
    if (mprotect(cb->mem_block_, cb->mem_size, PROT_READ | PROT_WRITE)) {
        fprintf(stderr, "Couldn't make JIT page (%p) writeable, errno: %s", (void *)cb->mem_block_, strerror(errno));
        abort();
    }
}

void cb_mark_position_writeable(codeblock_t * cb, uint32_t write_pos)
{
#ifdef _WIN32
    uint32_t pagesize = 0x1000; // 4KB
#else
    uint32_t pagesize = (uint32_t)sysconf(_SC_PAGESIZE);
#endif
    uint32_t aligned_position = (write_pos / pagesize) * pagesize;

    if (cb->current_aligned_write_pos != aligned_position) {
        cb->current_aligned_write_pos = aligned_position;
        void *const page_addr = cb_get_ptr(cb, aligned_position);
        if (mprotect(page_addr, pagesize, PROT_READ | PROT_WRITE)) {
            fprintf(stderr, "Couldn't make JIT page (%p) writeable, errno: %s", page_addr, strerror(errno));
            abort();
        }
    }
}

void cb_mark_all_executable(codeblock_t * cb)
{
    cb->current_aligned_write_pos = ALIGNED_WRITE_POSITION_NONE;
    if (mprotect(cb->mem_block_, cb->mem_size, PROT_READ | PROT_EXEC)) {
        fprintf(stderr, "Couldn't make JIT page (%p) executable, errno: %s", (void *)cb->mem_block_, strerror(errno));
        abort();
    }
}
