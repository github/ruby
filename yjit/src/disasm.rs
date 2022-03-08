use std::fmt::Write;
use crate::cruby::*;
use crate::core::*;

/// Primitive called in yjit.rb
/// Produce a string representing the disassembly for an ISEQ
#[no_mangle]
pub extern "C" fn rb_yjit_disasm_iseq(ec: EcPtr, ruby_self: VALUE, iseqw: VALUE) -> VALUE {

    #[cfg(not(feature = "disasm"))]
    {
        return Qnil;
    }

    #[cfg(feature = "disasm")]
    {
        // TODO:
        //if unsafe { CLASS_OF(iseqw) != rb_cISeq } {
        //    return Qnil;
        //}

        // Get the iseq pointer from the wrapper
        let iseq = unsafe { rb_iseqw_to_iseq(iseqw) };

        let out_string = disasm_iseq(iseq);

        return rust_str_to_ruby(&out_string);
    }
}

#[cfg(feature = "disasm")]
fn disasm_iseq(iseq: IseqPtr) -> String {
    let mut out = String::from("");

    // Get a list of block versions generated for this iseq
    let mut block_list = get_iseq_block_list(iseq);

    // Sort the blocks by increasing start addresses
    block_list.sort_by(|a, b| {
        use std::cmp::Ordering;

        // Get the start addresses for each block
        let addr_a = a.borrow().get_start_addr().unwrap().raw_ptr();
        let addr_b = b.borrow().get_start_addr().unwrap().raw_ptr();

        if addr_a < addr_b {
            Ordering::Less
        }
        else if addr_a == addr_b {
            Ordering::Equal
        }
        else {
            Ordering::Greater
        }
    });

    // Compute total code size in bytes for all blocks in the function
    let mut total_code_size = 0;
    for blockref in &block_list {
        total_code_size = blockref.borrow().code_size();
    }

    // Initialize capstone
    extern crate capstone;
    use capstone::prelude::*;
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Intel)
        .build()
        .unwrap();

    out.push_str(&format!("NUM BLOCK VERSIONS: {}\n", block_list.len()));
    out.push_str(&format!("TOTAL INLINE CODE SIZE: {} bytes\n", total_code_size));

    // TODO: may want to list block i/n here

    // For each block, sorted by increasing start address
    for block_idx in 0..block_list.len() {
        let block = block_list[block_idx].borrow();
        let blockid = block.get_blockid();
        let end_idx = block.get_end_idx();
        let start_addr = block.get_start_addr().unwrap().raw_ptr();
        let code_size = block.code_size();

        // Write some info about the block
        let block_ident = format!(
            "BLOCK {}/{}, ISEQ RANGE [{},{}), {} bytes ",
            block_idx + 1,
            block_list.len(),
            blockid.idx,
            end_idx,
            code_size
        );
        out.push_str(&format!("== {:=<60}\n", block_ident));

        // Disassemble the instructions
        let code_slice = unsafe { std::slice::from_raw_parts(start_addr, code_size) };
        let insns = cs.disasm_all(code_slice, start_addr as u64).unwrap();

        // For each instruction in this block
        for insn in insns.as_ref() {
            out.push_str(&format!("  {}\n", insn));
        }
    }

    return out;
}
