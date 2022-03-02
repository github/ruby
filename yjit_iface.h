//
// These are definitions YJIT uses to interface with the CRuby codebase,
// but which are only used internally by YJIT.
//

#ifndef YJIT_IFACE_H
#define YJIT_IFACE_H 1

#include "ruby/internal/config.h"
#include "ruby_assert.h" // for RUBY_DEBUG
#include "yjit.h" // for YJIT_STATS
#include "vm_core.h"
#include "yjit_core.h"

static void yjit_unlink_method_lookup_dependency(block_t *block);
static void yjit_block_assumptions_free(block_t *block);

#endif // #ifndef YJIT_IFACE_H
