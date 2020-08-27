use simpla_parser::syntax_tree;
use std::collections::HashMap;

use super::simple_counter::AddrSize;

pub struct FunctionIndex<'a> {
    index: HashMap<&'a str, AddrSize>,
}

impl<'a> FunctionIndex<'a> {
    fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    fn add_function(&mut self, name: &'a str) {
        let curr = self.index.len() as AddrSize;
        self.index.insert(name, curr);
    }

    pub fn get_function_index(&self, name: &str) -> AddrSize {
        *self.index.get(name).unwrap()
    }
}

pub fn build_function_index<'a>(func_decl_list: &'a syntax_tree::FuncDeclList) -> FunctionIndex {
    let mut function_index = FunctionIndex::new();
    for decl in func_decl_list {
        function_index.add_function(&decl.id)
    }
    function_index
}
