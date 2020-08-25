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

    fn convert_var_decl(&mut self, var_decl: &syntax_tree::VarDecl) {}

    fn convert_expression(&mut self, expr: &syntax_tree::Expr) {}

    fn insert_bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.buff.push(*b);
        }
    }
}

impl<'a> CodeGenerator<'a> for ByteCodeGenerator<'a> {
    fn gen_function(&mut self, func: &'a syntax_tree::FuncDecl) {
        self.buff.push(opcode::FUNC);
        self.gen_variables(&func.vars, Scope::Local);
        self.gen_block(&func.body, BlockType::General);
        self.var_cache.clear_local_vars();
    }
    
    fn gen_block(&mut self, block: &syntax_tree::StatList, block_type: BlockType) {}

    fn gen_variables(&mut self, var_decl_list: &'a syntax_tree::VarDeclList, scope: Scope) {
        match scope {
            Scope::Local => self.var_cache.cache_local_vars(var_decl_list),
            Scope::Global => self.var_cache.cache_global_vars(var_decl_list)
        };
        
        for var_decl in var_decl_list {
            for var in &var_decl.id_list {
                let (_, id) = self.var_cache.lookup(var);
                let id = *id; // so linter is happy 
                let (first, second) = init_command(&var_decl.kind);
                let data = defaut_value(&var_decl.kind);
                self.buff.push(first);
                self.insert_bytes(&data);
                self.buff.push(second);
                self.insert_bytes(&id.to_be_bytes());
            }
        }
    }

    fn get_result(self) -> Vec<u8> {
        self.buff
    }

}

fn init_command(kind: &syntax_tree::Kind) -> (u8, u8) {
    match kind {
        syntax_tree::Kind::Bool => (opcode::LDBC, opcode::STRB),
        syntax_tree::Kind::Int => (opcode::LDIC, opcode::STRI),
        syntax_tree::Kind::Real => (opcode::LDRC, opcode::STRR),
        syntax_tree::Kind::Str => (opcode::LDSC, opcode::STRS),
        syntax_tree::Kind::Void => unreachable!(),

    }
}

fn defaut_value(kind: &syntax_tree::Kind) -> Vec<u8> {
    match kind {
        syntax_tree::Kind::Bool => Vec::from([0]),
        syntax_tree::Kind::Int => Vec::from((0 as i32).to_be_bytes()),
        syntax_tree::Kind::Real => Vec::from((0.0 as f64).to_be_bytes()),
        syntax_tree::Kind::Str => Vec::from((0 as u16).to_be_bytes()),
        syntax_tree::Kind::Void => unreachable!(),
    }
}

