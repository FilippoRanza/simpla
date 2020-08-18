use std::collections::HashMap;

use simpla_parser::syntax_tree::{Kind, VarDecl, VarDeclList, ParamList};

type NameMap<'a> = HashMap<&'a str, Kind>;

pub struct VarCache<'a> {
    global_vars: NameMap<'a>,
    local_vars: NameMap<'a>,
}

impl<'a> VarCache<'a> {
    pub fn new() -> Self {
        Self {
            global_vars: HashMap::new(),
            local_vars: HashMap::new(),
        }
    }

    pub fn cache_global_vars(&mut self, var_decl_list: &'a VarDeclList) {
        cache_var_decl_list(var_decl_list, &mut self.global_vars);
    }

    pub fn cache_params(&mut self, param_decl_list: &'a ParamList) {
        for param in param_decl_list {
            self.local_vars.insert(&param.id, param.kind.clone());
        }
    }

    pub fn cache_local_vars(&mut self, var_decl_list: &'a VarDeclList) {
        cache_var_decl_list(var_decl_list, &mut self.local_vars);
    }

    pub fn clear_local_vars(&mut self) {
        self.local_vars.clear();
    }

    pub fn lookup(&self, name: &str) -> &Kind {
        if let Some(kind) = self.local_vars.get(name) {
            kind
        } else {
            self.global_vars.get(name).unwrap()
        }
    }
}

fn cache_var_decl_list<'a>(var_decl_list: &'a VarDeclList, map: &mut NameMap<'a>) {
    for var_decl in var_decl_list {
        cache_var_decl(&var_decl, map);
    }
}

fn cache_var_decl<'a>(var_decl: &'a VarDecl, map: &mut NameMap<'a>) {
    for decl in &var_decl.id_list {
        map.insert(&decl, var_decl.kind.clone());
    }
}
