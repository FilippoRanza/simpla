use simpla_parser::syntax_tree::*;

use super::byte_code_generator::ByteCodeGenerator;
use super::c_generator::CSourceGenerator;
use super::code_generator::*;
use super::function_index::build_function_index;

pub fn translate_to_c<'a>(prog: &'a Program) -> Vec<u8> {
    let mut code_gen = CSourceGenerator::new();
    translate(prog, &mut code_gen, TranslationMode::MainAfter);
    code_gen.get_result()
}

pub fn translate_to_byte_code<'a>(prog: &'a Program) -> Vec<u8> {
    let function_index = build_function_index(&prog.functions);
    let mut code_gen = ByteCodeGenerator::new(function_index);
    translate(prog, &mut code_gen, TranslationMode::MainBefore);
    code_gen.get_result()
}

enum TranslationMode {
    MainBefore,
    MainAfter
}

fn translate<'a>(prog: &'a Program, tranlator: &mut dyn CodeGenerator<'a>, mode: TranslationMode) {
    tranlator.gen_variables(&prog.global_vars, Scope::Global);

    match mode {
        TranslationMode::MainBefore => tranlator.gen_block(&prog.body, BlockType::Main),
        _ => {}
    }

    for func in &prog.functions {
        tranlator.gen_function(func);
    }

    match mode { 
        TranslationMode::MainAfter => tranlator.gen_block(&prog.body, BlockType::Main),
        _ => {}
    }

}
