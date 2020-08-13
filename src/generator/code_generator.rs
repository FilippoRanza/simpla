use simpla_parser::syntax_tree::{FuncDecl, StatList, VarDeclList};

pub enum BlockType {
    Main,
    General
}

pub trait CodeGenerator {
    fn gen_function(&mut self, func: &FuncDecl);
    fn gen_block(&mut self, bloc: &StatList, block: BlockType);
    fn gen_variables(&mut self, vars: &VarDeclList);
    fn get_result(self) -> Vec<u8>;
}
