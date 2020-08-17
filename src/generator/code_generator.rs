use simpla_parser::syntax_tree::{FuncDecl, StatList, VarDeclList};

pub enum BlockType {
    Main,
    General,
}

pub enum Scope {
    Global,
    Local
}

pub trait CodeGenerator<'a> {
    fn gen_function(&mut self, func: &'a FuncDecl);
    fn gen_block(&mut self, bloc: &'a StatList, block: BlockType);
    fn gen_variables(&mut self, vars: &'a VarDeclList, scope: Scope);
    fn get_result(self) -> Vec<u8>;
}
