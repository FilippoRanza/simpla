use simpla_parser::syntax_tree::*;

use super::c_generator::CSourceGenerator;
use super::code_generator::*;

pub fn translate_to_c(prog: &Program) -> Vec<u8> {
    let mut code_gen = CSourceGenerator::new();
    translate(prog, &mut code_gen);
    code_gen.get_result()
}

fn translate(prog: &Program, tranlator: &mut dyn CodeGenerator) {
    tranlator.gen_variables(&prog.global_vars);

    for func in &prog.functions {
        tranlator.gen_function(func);
    }
    tranlator.gen_block(&prog.body, BlockType::Main);
}
