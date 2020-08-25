use simpla_parser::syntax_tree;
use std::collections::HashMap;

pub struct FunctionIndex<'a> {
    index: HashMap<&'a str, u16>,
}

impl<'a> FunctionIndex<'a> {
    fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    fn add_function(&mut self, name: &'a str) {
        let curr = self.index.len() as u16;
        self.index.insert(name, curr);
    }

    pub fn get_function_index(&self, name: &str) -> u16 {
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
