// Silence dead code warnings until we are done porting YJIT
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_macros)]
#![allow(clippy::style)] // We are laid back about style

mod asm;
mod cruby;
mod core;
mod codegen;
mod invariants;
mod disasm;
mod utils;
mod options;
mod stats;
mod yjit;
