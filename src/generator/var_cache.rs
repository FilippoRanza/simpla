use std::collections::HashMap;

use super::simple_counter::{AddrSize, SimpleCounter};
use simpla_parser::syntax_tree::{FuncDecl, Kind, ParamList, Program, VarDecl, VarDeclList};

pub fn build_global_var_cache<'a>(prog: &'a Program) -> (GlobalVarCache<'a>, ParameterAddress<'a>) {
    let mut factory = GlobalVarCacheFactory::new();
    factory.cache_global_vars(&prog.global_vars);

    let mut factory = factory.switch_to_function_factory();
    for func in &prog.functions {
        build_local_var_cache(func, &mut factory);
    }

    factory.build_var_cache()
}

fn build_local_var_cache<'a>(func: &'a FuncDecl, factory: &mut FunctionVarCacheFactory<'a>) {
    factory.insert_function(&func.id);
    factory.cache_params(&func.id, &func.params);
    factory.cache_local_vars(&func.id, &func.vars);
}

struct GlobalVarCacheFactory<'a> {
    global_vars: NameTable<'a>,
}

impl<'a> GlobalVarCacheFactory<'a> {
    fn new() -> Self {
        Self {
            global_vars: NameTable::new(),
        }
    }

    fn cache_global_vars(&mut self, var_decl_list: &'a VarDeclList) {
        cache_var_decl_list(var_decl_list, &mut self.global_vars);
    }

    fn switch_to_function_factory(self) -> FunctionVarCacheFactory<'a> {
        FunctionVarCacheFactory::new(self.global_vars)
    }
}

struct FunctionVarCacheFactory<'a> {
    global_vars: NameTable<'a>,
    function_vars: HashMap<&'a str, NameTable<'a>>,
    param_addr: HashMap<&'a str, Vec<AddrSize>>,
}

impl<'a> FunctionVarCacheFactory<'a> {
    fn new(global_vars: NameTable<'a>) -> Self {
        Self {
            global_vars,
            function_vars: HashMap::new(),
            param_addr: HashMap::new(),
        }
    }

    fn insert_function(&mut self, name: &'a str) {
        let new_table = NameTable::new();
        self.function_vars.insert(name, new_table);
    }

    fn cache_local_vars(&mut self, name: &'a str, var_decl_list: &'a VarDeclList) {
        if let Some(curr) = self.function_vars.get_mut(name) {
            cache_var_decl_list(var_decl_list, curr);
        } else {
            panic!();
        }
    }

    fn cache_params(&mut self, name: &'a str, param_decl_list: &'a ParamList) {
        if let Some(curr) = self.function_vars.get_mut(name) {
            let param_addr = cache_param_decl(param_decl_list, curr);
            self.param_addr.insert(name, param_addr);
        } else {
            panic!();
        }
    }

    fn build_var_cache(self) -> (GlobalVarCache<'a>, ParameterAddress<'a>) {
        (
            GlobalVarCache::new(self.global_vars, self.function_vars),
            ParameterAddress::new(self.param_addr),
        )
    }
}

pub struct ParameterAddress<'a> {
    param_addr: HashMap<&'a str, Vec<AddrSize>>,
}

impl<'a> ParameterAddress<'a> {
    fn new(param_addr: HashMap<&'a str, Vec<AddrSize>>) -> Self {
        Self { param_addr }
    }

    pub fn get_parameter_address(&self, name: &str, index: usize) -> AddrSize {
        let vect = self.param_addr.get(name).unwrap();
        vect[index]
    }
}

pub struct GlobalVarCache<'a> {
    global_vars: VarTable<'a>,
    function_vars: HashMap<&'a str, VarTable<'a>>,
}

impl<'a> GlobalVarCache<'a> {
    fn new(global_vars: NameTable<'a>, function_vars: HashMap<&'a str, NameTable<'a>>) -> Self {
        Self {
            global_vars: global_vars.get_table(),
            function_vars: function_vars
                .into_iter()
                .map(|(name, table)| (name, table.get_table()))
                .collect(),
        }
    }

    pub fn get_local_cache(&'a self, name: &str) -> VarLookup<'a> {
        let local = self.function_vars.get(name).unwrap();
        VarLookup::new_local(&self.global_vars, local)
    }

    pub fn get_global_cache(&'a self) -> VarLookup<'a> {
        VarLookup::new_global(&self.global_vars)
    }
}

pub struct VarLookup<'a> {
    global_vars: &'a VarTable<'a>,
    local_vars: Option<&'a VarTable<'a>>,
}

impl<'a> VarLookup<'a> {
    fn new_local(global_vars: &'a VarTable<'a>, local_vars: &'a VarTable<'a>) -> Self {
        let mut output = Self::new_global(global_vars);
        output.local_vars = Some(local_vars);
        output
    }

    fn new_global(global_vars: &'a VarTable<'a>) -> Self {
        Self {
            global_vars,
            local_vars: None,
        }
    }

    pub fn lookup(&self, name: &str) -> (&VarInfo, VariableType) {
        if let Some(output) = self.local_lookup(name) {
            output
        } else {
            (self.global_vars.get(name).unwrap(), VariableType::Global)
        }
    }

    fn local_lookup(&self, name: &str) -> Option<(&VarInfo, VariableType)> {
        match self.local_vars {
            Some(local_vars) => {
                if let Some(output) = local_vars.get(name) {
                    Some((output, VariableType::Local))
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

pub enum VariableType {
    Global,
    Local,
}

fn cache_param_decl<'a>(param_list: &'a ParamList, map: &mut NameTable<'a>) -> Vec<AddrSize> {
    let mut output = Vec::with_capacity(param_list.len());
    for param in param_list {
        let index = map.insert(&param.id, &param.kind);
        output.push(index);
    }
    output
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
type VarTable<'a> = HashMap<&'a str, VarInfo>;

struct NameTable<'a> {
    table: VarTable<'a>,
    counter: KindCounter,
}

impl<'a> NameTable<'a> {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
            counter: KindCounter::new(),
        }
    }

    fn insert(&mut self, name: &'a str, k: &Kind) -> AddrSize {
        let index = self.counter.get_index(k);
        self.table.insert(name, (k.clone(), index));
        index
    }

    fn get_table(self) -> VarTable<'a> {
        self.table
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

    #[allow(dead_code)]
    fn reset(&mut self) {
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
}
