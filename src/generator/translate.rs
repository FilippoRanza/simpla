use simpla_parser::syntax_tree::*;

use super::byte_code_generator::ByteCodeGenerator;
use super::code_generator::*;
use super::function_index::build_function_index;


pub fn translate_to_byte_code<'a>(prog: &'a Program) -> Vec<u8> {
    let function_index = build_function_index(&prog.functions);
    let mut code_gen = ByteCodeGenerator::new(function_index);
    translate(prog, &mut code_gen,);
    code_gen.get_result()
}



fn translate<'a>(prog: &'a Program, tranlator: &mut dyn CodeGenerator<'a>) {
    tranlator.gen_variables(&prog.global_vars, Scope::Global);

    tranlator.gen_block(&prog.body, BlockType::Main);
 

    for func in &prog.functions {
        tranlator.gen_function(func);
    }

}
