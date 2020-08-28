use std::collections::HashMap;

use super::simple_counter::{AddrSize, SimpleCounter};
use simpla_parser::syntax_tree::{Kind, ParamList, VarDecl, VarDeclList};

pub enum VariableType {
    Global,
    Local,
}

pub struct VarCache<'a> {
    global_vars: NameTable<'a>,
    local_vars: NameTable<'a>,
    var_count: KindCounter,
}

impl<'a> VarCache<'a> {
    pub fn new() -> Self {
        Self {
            global_vars: NameTable::new(),
            local_vars: NameTable::new(),
            var_count: KindCounter::new(),
        }
    }

    pub fn cache_global_vars(&mut self, var_decl_list: &'a VarDeclList) {
        cache_var_decl_list(var_decl_list, &mut self.global_vars);
    }

    pub fn cache_params(&mut self, param_decl_list: &'a ParamList) {
        for param in param_decl_list {
            self.local_vars.insert(&param.id, &param.kind);
        }
    }

    pub fn cache_local_vars(&mut self, var_decl_list: &'a VarDeclList) {
        cache_var_decl_list(var_decl_list, &mut self.local_vars);
    }

    pub fn clear_local_vars(&mut self) {
        self.var_count.reset();
        self.local_vars.reset();
    }

    pub fn lookup(&self, name: &str) -> (&VarInfo, VariableType) {
        if let Some(output) = self.local_vars.get(name) {
            (output, VariableType::Local)
        } else {
            (self.global_vars.get(name).unwrap(), VariableType::Global)
        }
    }
}

fn cache_var_decl_list<'a>(var_decl_list: &'a VarDeclList, map: &mut NameTable<'a>) {
    for var_decl in var_decl_list {
        cache_var_decl(&var_decl, map);
    }
}

fn cache_var_decl<'a>(var_decl: &'a VarDecl, map: &mut NameTable<'a>) {
    for decl in &var_decl.id_list {
        map.insert(&decl, &var_decl.kind);
    }
}

type VarInfo = (Kind, AddrSize);

struct NameTable<'a> {
    table: HashMap<&'a str, VarInfo>,
    counter: KindCounter,
}

impl<'a> NameTable<'a> {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
            counter: KindCounter::new(),
        }
    }

    fn insert(&mut self, name: &'a str, k: &Kind) {
        let index = self.counter.get_index(k);
        self.table.insert(name, (k.clone(), index));
    }

    fn get(&self, name: &str) -> Option<&(Kind, AddrSize)> {
        self.table.get(name)
    }

    fn reset(&mut self) {
        self.counter.reset();
        self.table.clear();
    }
}

pub struct KindCounter {
    int_count: SimpleCounter,
    real_count: SimpleCounter,
    str_count: SimpleCounter,
    bool_count: SimpleCounter,
}

impl KindCounter {
    pub fn new() -> Self {
        Self {
            int_count: SimpleCounter::new(),
            real_count: SimpleCounter::new(),
            str_count: SimpleCounter::new(),
            bool_count: SimpleCounter::new(),
        }
    }

    pub fn get_index(&mut self, k: &Kind) -> AddrSize {
        match k {
            Kind::Bool => self.bool_count.count_one(),
            Kind::Int => self.int_count.count_one(),
            Kind::Real => self.real_count.count_one(),
            Kind::Str => self.str_count.count_one(),
            Kind::Void => panic!("void variable found!"),
        }
    }

    pub fn reset(&mut self) {
        self.bool_count.reset();
        self.int_count.reset();
        self.real_count.reset();
        self.str_count.reset();
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref KIND_VECTOR: &'static [(Kind, AddrSize)] = &[
            (Kind::Str, 0),
            (Kind::Str, 1),
            (Kind::Int, 0),
            (Kind::Bool, 0),
            (Kind::Real, 0),
            (Kind::Bool, 1),
            (Kind::Bool, 2),
            (Kind::Int, 1),
        ];
    }

    #[test]
    fn test_kind_counter() {
        let mut counter = KindCounter::new();

        for (k, i) in KIND_VECTOR.iter() {
            let tmp = counter.get_index(k);
            assert_eq!(tmp, *i);
        }

        counter.reset();
    }

    #[test]
    #[should_panic(expected = "void variable found!")]
    fn test_error_kind_counter() {
        let mut counter = KindCounter::new();
        counter.get_index(&Kind::Void);
    }

    #[test]
    fn test_name_table() {
        let names = name_list("name", KIND_VECTOR.len());
        let mut table = NameTable::new();
        for (name, (kind, _)) in names.iter().zip(KIND_VECTOR.iter()) {
            table.insert(name, kind);
        }

        assert_eq!(table.table.len(), names.len());

        for (name, (kind, index)) in names.iter().zip(KIND_VECTOR.iter()) {
            let (k, i) = table.get(name).unwrap();
            assert_eq!(k, kind);
            assert_eq!(i, index);
        }

        table.reset();
        assert_eq!(table.table.len(), 0);
    }

    fn name_list(base: &str, count: usize) -> Vec<String> {
        (0..count).map(|i| format!("{}_{}", base, i)).collect()
    }
}
