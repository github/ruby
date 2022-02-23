#ifndef YJIT_CODEGEN_H
#define YJIT_CODEGEN_H 1

typedef enum codegen_status {
    YJIT_END_BLOCK,
    YJIT_KEEP_COMPILING,
    YJIT_CANT_COMPILE
} codegen_status_t;

// Code generation function signature
typedef codegen_status_t (*codegen_fn)(jitstate_t *jit, ctx_t *ctx, codeblock_t *cb);

static void jit_ensure_block_entry_exit(jitstate_t *jit);

#endif // #ifndef YJIT_CODEGEN_H
