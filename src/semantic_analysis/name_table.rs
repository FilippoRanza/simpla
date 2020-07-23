use std::collections::HashMap;

use super::semantic_error::{Ridefinition, SemanticError};
use simpla_parser::syntax_tree;

pub fn name_table_factory<'a>() -> GlobalVariableTable<'a> {
    GlobalVariableTable::new()
}

pub trait VariableTable<'a> {
    fn insert_variable(
        &mut self,
        name: &'a str,
        kind: &'a syntax_tree::Kind,
    ) -> Result<(), SemanticError<'a>>;
}

pub struct GlobalVariableTable<'a> {
    global_table: NameTable<'a, &'a syntax_tree::Kind>,
}

impl<'a> GlobalVariableTable<'a> {
    fn new() -> Self {
        Self {
            global_table: NameTable::new(Entry::Variable),
        }
    }

    pub fn switch_to_function_table(self) -> FunctionTable<'a> {
        FunctionTable::new(self.global_table)
    }
}

impl<'a> VariableTable<'a> for GlobalVariableTable<'a> {
    fn insert_variable(
        &mut self,
        name: &'a str,
        kind: &'a syntax_tree::Kind,
    ) -> Result<(), SemanticError<'a>> {
        self.global_table.check_collision(name, Entry::Variable)?;
        self.global_table.insert(name, kind);
        Ok(())
    }
}

pub struct FunctionTable<'a> {
    global_table: NameTable<'a, &'a syntax_tree::Kind>,
    function_table: NameTable<'a, &'a syntax_tree::FuncDecl>,
}

impl<'a> FunctionTable<'a> {
    fn new(global_table: NameTable<'a, &'a syntax_tree::Kind>) -> Self {
        Self {
            global_table,
            function_table: NameTable::new(Entry::Function),
        }
    }

    pub fn switch_to_local_table(self) -> LocalVariableTable<'a> {
        LocalVariableTable::new(self.global_table, self.function_table)
    }

    pub fn insert_function(
        &mut self,
        name: &'a str,
        func_decl: &'a syntax_tree::FuncDecl,
    ) -> Result<(), SemanticError<'a>> {
        self.global_table.check_collision(name, Entry::Function)?;
        self.function_table.check_collision(name, Entry::Function)?;
        self.function_table.insert(name, func_decl);
        Ok(())
    }
}

pub struct LocalVariableTable<'a> {
    global_table: NameTable<'a, &'a syntax_tree::Kind>,
    function_table: NameTable<'a, &'a syntax_tree::FuncDecl>,
    local_table: NameTable<'a, &'a syntax_tree::Kind>,
}

impl<'a> LocalVariableTable<'a> {
    fn new(
        global_table: NameTable<'a, &'a syntax_tree::Kind>,
        function_table: NameTable<'a, &'a syntax_tree::FuncDecl>,
    ) -> Self {
        Self {
            global_table,
            function_table,
            local_table: NameTable::new(Entry::Variable),
        }
    }

    fn new_from_lookup(
        global_table: NameTable<'a, &'a syntax_tree::Kind>,
        function_table: NameTable<'a, &'a syntax_tree::FuncDecl>,
        local_table: NameTable<'a, &'a syntax_tree::Kind>,
    ) -> Self {
        let mut local_table = local_table;
        local_table.clear();
        Self {
            global_table,
            function_table,
            local_table,
        }
    }

    pub fn new_scope(mut self) -> Self {
        self.local_table.clear();
        self
    }

    pub fn switch_to_lookup_table(self) -> LookupTable<'a> {
        LookupTable::new(self.global_table, self.function_table, self.local_table)
    }
}

impl<'a> VariableTable<'a> for LocalVariableTable<'a> {
    fn insert_variable(
        &mut self,
        name: &'a str,
        kind: &'a syntax_tree::Kind,
    ) -> Result<(), SemanticError<'a>> {
        self.global_table.check_collision(name, Entry::Variable)?;
        self.function_table.check_collision(name, Entry::Variable)?;
        self.local_table.check_collision(name, Entry::Variable)?;
        self.local_table.insert(name, kind);
        Ok(())
    }
}

pub struct LookupTable<'a> {
    global_table: NameTable<'a, &'a syntax_tree::Kind>,
    function_table: NameTable<'a, &'a syntax_tree::FuncDecl>,
    local_table: NameTable<'a, &'a syntax_tree::Kind>,
}

impl<'a> LookupTable<'a> {
    fn new(
        global_table: NameTable<'a, &'a syntax_tree::Kind>,
        function_table: NameTable<'a, &'a syntax_tree::FuncDecl>,
        local_table: NameTable<'a, &'a syntax_tree::Kind>,
    ) -> Self {
        Self {
            global_table,
            function_table,
            local_table,
        }
    }

    pub fn get_variable(&self, name: &'a str) -> Result<&syntax_tree::Kind, SemanticError<'a>> {
        if let Some(output) = self.local_table.get(name) {
            Ok(output)
        } else if let Some(output) = self.global_table.get(name) {
            Ok(output)
        } else {
            Err(SemanticError::UnknownVariable(name))
        }
    }

    pub fn get_function(&self, name: &'a str) -> Result<&syntax_tree::FuncDecl, SemanticError<'a>> {
        if let Some(output) = self.function_table.get(name) {
            Ok(output)
        } else {
            Err(SemanticError::UnknownFunction(name))
        }
    }

    pub fn switch_to_local_table(self) -> LocalVariableTable<'a> {
        LocalVariableTable::new_from_lookup(
            self.global_table,
            self.function_table,
            self.local_table,
        )
    }
}

enum Entry {
    Variable,
    Function,
}

struct NameTable<'a, T>
where
    T: 'a,
{
    table: HashMap<&'a str, T>,
    entry_kind: Entry,
}

impl<'a, T> NameTable<'a, T> {
    fn new(entry_kind: Entry) -> Self {
        Self {
            table: HashMap::new(),
            entry_kind,
        }
    }

    fn get(&self, name: &str) -> Option<&T> {
        self.table.get(name)
    }

    fn clear(&mut self) {
        self.table.clear();
    }

    fn insert(&mut self, name: &'a str, entry: T) {
        self.table.insert(name, entry);
    }

    fn check_collision(&self, name: &str, new_entry: Entry) -> Result<(), SemanticError<'a>> {
        if let Some(_) = self.table.get(name) {
            let (original, new) = match (&self.entry_kind, new_entry) {
                (Entry::Function, Entry::Function) => {
                    (Ridefinition::Function, Ridefinition::Function)
                }
                (Entry::Function, Entry::Variable) => {
                    (Ridefinition::Function, Ridefinition::Variable)
                }
                (Entry::Variable, Entry::Function) => {
                    (Ridefinition::Variable, Ridefinition::Function)
                }
                (Entry::Variable, Entry::Variable) => {
                    (Ridefinition::Variable, Ridefinition::Variable)
                }
            };
            let err = SemanticError::NameRidefinition {
                name: name.to_owned(),
                original,
                new,
            };
            Err(err)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_name_ridefination() {
        let function_name = "test_function";
        let other_function_name = "other_function";
        let func_decl = syntax_tree::FuncDecl::new(
            function_name.to_owned(),
            vec![],
            syntax_tree::Kind::Int,
            vec![],
            vec![],
        );

        let global_variable = "test_variable";
        let other_global_variable = "global_variable";

        let local_variable = "local_test";
        let other_local_variable = "other_local_variable";

        // state 1: global variables
        let mut table = name_table_factory();
        table
            .insert_variable(global_variable, &syntax_tree::Kind::Int)
            .unwrap();
        table
            .insert_variable(other_global_variable, &syntax_tree::Kind::Real)
            .unwrap();

        let err = table.insert_variable(global_variable, &syntax_tree::Kind::Bool);
        check_status(
            err,
            global_variable,
            Ridefinition::Variable,
            Ridefinition::Variable,
        );

        // state 2: functions
        let mut table = table.switch_to_function_table();
        table.insert_function(function_name, &func_decl).unwrap();

        let err = table.insert_function(global_variable, &func_decl);
        check_status(
            err,
            global_variable,
            Ridefinition::Variable,
            Ridefinition::Function,
        );

        table
            .insert_function(other_function_name, &func_decl)
            .unwrap();

        let err = table.insert_function(function_name, &func_decl);
        check_status(
            err,
            function_name,
            Ridefinition::Function,
            Ridefinition::Function,
        );

        // state 3: local functions (formal parameters and function local variables)
        let mut table = table.switch_to_local_table();
        table
            .insert_variable(local_variable, &syntax_tree::Kind::Real)
            .unwrap();

        let err = table.insert_variable(global_variable, &syntax_tree::Kind::Str);
        check_status(
            err,
            global_variable,
            Ridefinition::Variable,
            Ridefinition::Variable,
        );

        let err = table.insert_variable(function_name, &syntax_tree::Kind::Bool);
        check_status(
            err,
            function_name,
            Ridefinition::Function,
            Ridefinition::Variable,
        );

        table
            .insert_variable(other_local_variable, &syntax_tree::Kind::Int)
            .unwrap();

        let err = table.insert_variable(other_local_variable, &syntax_tree::Kind::Int);
        check_status(
            err,
            other_local_variable,
            Ridefinition::Variable,
            Ridefinition::Variable,
        );
    }

    fn check_status(
        stat: Result<(), SemanticError>,
        original_name: &str,
        original_kind: Ridefinition,
        new_kind: Ridefinition,
    ) {
        match stat {
            Ok(_) => panic!("{} has been ridefined", original_name),
            Err(err) => match err {
                SemanticError::NameRidefinition {
                    name,
                    original,
                    new,
                } => {
                    assert_eq!(name, original_name);
                    assert_eq!(original, original_kind);
                    assert_eq!(new, new_kind)
                }
                _ => panic!("Wrong Error kind: {:?}", err),
            },
        }
    }
}
