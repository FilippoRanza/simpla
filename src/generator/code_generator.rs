use simpla_parser::syntax_tree::*;

pub fn translate(prog: &Program) -> Vec<u8> {
    let mut output = Vec::new();

    generate_variables(&prog.global_vars, &mut output);
    generate_block(&prog.body, &mut output);
    for func in &prog.functions {
        generate_functions(func, &mut output)
    }

    output
}

fn generate_functions(func: &FuncDecl, bytes: &mut Vec<u8>) {}

fn generate_block(block: &StatList, bytes: &mut Vec<u8>) {}

fn generate_variables(vars: &VarDeclList, bytes: &mut Vec<u8>) {}

