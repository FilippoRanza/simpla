use simpla_parser::syntax_tree::*;

use super::c_generator::CSourceGenerator;
use super::code_generator::*;

pub fn translate_to_c<'a>(prog: &'a Program) -> Vec<u8> {
    let mut code_gen = CSourceGenerator::new();
    translate(prog, &mut code_gen);
    code_gen.get_result()
}

fn translate<'a>(prog: &'a Program, tranlator: &mut dyn CodeGenerator<'a>) {
    tranlator.gen_variables(&prog.global_vars, Scope::Global);

    for func in &prog.functions {
        tranlator.gen_function(func);
    }
    tranlator.gen_block(&prog.body, BlockType::Main);
}
