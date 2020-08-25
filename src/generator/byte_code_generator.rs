use super::code_generator::*;
use super::function_index::FunctionIndex;
use super::opcode;
use super::var_cache::VarCache;

use simpla_parser::syntax_tree;

pub struct ByteCodeGenerator<'a> {
    buff: Vec<u8>,
    var_cache: VarCache<'a>,
    function_index: FunctionIndex<'a>,
}

impl<'a> ByteCodeGenerator<'a> {
    pub fn new(function_index: FunctionIndex<'a>) -> Self {
        Self {
            buff: Vec::new(),
            var_cache: VarCache::new(),
            function_index,
        }
    }

    fn convert_expression(&mut self, expr: &syntax_tree::Expr) {}
}

impl<'a> CodeGenerator<'a> for ByteCodeGenerator<'a> {
    fn gen_function(&mut self, func: &'a syntax_tree::FuncDecl) {
        self.buff.push(opcode::FUNC);
    }
    fn gen_block(&mut self, block: &syntax_tree::StatList, block_type: BlockType) {}
    fn gen_variables(&mut self, var_decl_list: &'a syntax_tree::VarDeclList, scope: Scope) {}
    fn get_result(self) -> Vec<u8> {
        self.buff
    }
}
