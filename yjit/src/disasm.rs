use crate::cruby::*;

/// Primitive called in yjit.rb
/// Produce a string representing the disassembly for an ISEQ
#[no_mangle]
pub extern "C" fn rb_yjit_disasm_iseq(ec: EcPtr, ruby_self: VALUE, iseq: VALUE) -> VALUE {

    //#[cfg(feature = "disassembly")]
    //disasm_iseq(IseqPtr: iseq);
    // TODO: convert to Ruby string




    //#[cfg(not(feature = "disassembly"))]
    return Qnil;
}

// TODO
fn disasm_iseq(iseq: IseqPtr) -> String {
    return "foo".to_string();
}


















/*
#[test]
#[cfg(feature = "disassembly")]
fn basic_capstone_usage() -> std::result::Result<(), capstone::Error> {
    // Test drive Capstone with simple input
    extern crate capstone;
    use capstone::prelude::*;
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Intel)
        .build()?;

    let insns = cs.disasm_all(&[0xCC], 0x1000)?;

    match insns.as_ref() {
        [insn] => {
            assert_eq!(Some("int3"), insn.mnemonic());
            Ok(())
        }
        _ => Err(capstone::Error::CustomError(
            "expected to disassemble to int3",
        )),
    }
}
*/






/*
static VALUE
yjit_disasm_init(VALUE klass)
{
    csh * handle;
    VALUE disasm = TypedData_Make_Struct(klass, csh, &yjit_disasm_type, handle);
    if (cs_open(CS_ARCH_X86, CS_MODE_64, handle) != CS_ERR_OK) {
        rb_raise(rb_eRuntimeError, "failed to make Capstone handle");
    }
    return disasm;
}
*/



/*
static VALUE
yjit_disasm(VALUE self, VALUE code, VALUE from)
{
    size_t count;
    csh * handle;
    cs_insn *insns;

    TypedData_Get_Struct(self, csh, &yjit_disasm_type, handle);
    count = cs_disasm(*handle, (uint8_t*)StringValuePtr(code), RSTRING_LEN(code), NUM2ULL(from), 0, &insns);
    VALUE insn_list = rb_ary_new_capa(count);

    for (size_t i = 0; i < count; i++) {
        VALUE vals = rb_ary_new_from_args(3, LONG2NUM(insns[i].address),
                rb_str_new2(insns[i].mnemonic),
                rb_str_new2(insns[i].op_str));
        rb_ary_push(insn_list, rb_struct_alloc(cYjitDisasmInsn, vals));
    }
    cs_free(insns, count);
    return insn_list;
}
*/




/*
  if defined?(Disasm)
    def self.disasm(iseq, tty: $stdout && $stdout.tty?)
      iseq = RubyVM::InstructionSequence.of(iseq)

      blocks = blocks_for(iseq)
      return if blocks.empty?

      str = String.new
      str << iseq.disasm
      str << "\n"

      # Sort the blocks by increasing addresses
      sorted_blocks = blocks.sort_by(&:address)

      highlight = ->(str) {
        if tty
          "\x1b[1m#{str}\x1b[0m"
        else
          str
        end
      }

      cs = Disasm.new
      sorted_blocks.each_with_index do |block, i|
        str << "== BLOCK #{i+1}/#{blocks.length}: #{block.code.length} BYTES, ISEQ RANGE [#{block.iseq_start_index},#{block.iseq_end_index}) ".ljust(80, "=")
        str << "\n"

        comments = comments_for(block.address, block.address + block.code.length)
        comment_idx = 0
        cs.disasm(block.code, block.address).each do |i|
          while (comment = comments[comment_idx]) && comment.address <= i.address
            str << "  ; #{highlight.call(comment.comment)}\n"
            comment_idx += 1
          end

          str << sprintf(
            "  %<address>08x:  %<instruction>s\t%<details>s\n",
            address: i.address,
            instruction: i.mnemonic,
            details: i.op_str
          )
        end
      end

      block_sizes = blocks.map { |block| block.code.length }
      total_bytes = block_sizes.sum
      str << "\n"
      str << "Total code size: #{total_bytes} bytes"
      str << "\n"

      str
    end

    def self.comments_for(start_address, end_address)
      Primitive.comments_for(start_address, end_address)
    end

    def self.disasm_block(cs, block, highlight)
      comments = comments_for(block.address, block.address + block.code.length)
      comment_idx = 0
      str = +''
      cs.disasm(block.code, block.address).each do |i|
        while (comment = comments[comment_idx]) && comment.address <= i.address
          str << "  ; #{highlight.call(comment.comment)}\n"
          comment_idx += 1
        end

        str << sprintf(
          "  %<address>08x:  %<instruction>s\t%<details>s\n",
          address: i.address,
          instruction: i.mnemonic,
          details: i.op_str
        )
      end
      str
    end
  end
  */