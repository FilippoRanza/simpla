use simpla_parser::syntax_tree::*;

use super::byte_code_generator::ByteCodeGenerator;
use super::code_generator::*;
use super::function_index::build_function_index;
use super::var_cache::{build_global_var_cache, GlobalVarCache};

pub fn translate_to_byte_code<'a>(prog: &'a Program) -> Vec<u8> {
    let function_index = build_function_index(&prog.functions);
    let (glob_var_cache, param_addr) = build_global_var_cache(prog);
    let mut code_gen = ByteCodeGenerator::new(
        function_index,
        glob_var_cache.get_global_cache(),
        param_addr,
    );
    translate(prog, &mut code_gen, &glob_var_cache);
    code_gen.get_result()
}

fn translate<'a>(
    prog: &'a Program,
    tranlator: &mut ByteCodeGenerator<'a>,
    global_cache: &'a GlobalVarCache<'a>,
) {
    tranlator.gen_variables(&prog.global_vars, Scope::Global);

    tranlator.gen_block(&prog.body, BlockType::Main);

    for func in &prog.functions {
        tranlator.switch_local_cache(global_cache.get_local_cache(&func.id));
        tranlator.gen_function(func);
    }
}
