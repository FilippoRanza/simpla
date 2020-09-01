use std::collections::HashMap;

use super::semantic_error::{NameRidefinition, Ridefinition, SemanticError};
use simpla_parser::syntax_tree;

pub fn name_table_factory<'a>() -> GlobalVariableTable<'a> {
    GlobalVariableTable::new()
}

type VarNameTable<'a> = NameTable<'a, (&'a syntax_tree::Kind, &'a syntax_tree::Location)>;

pub trait VariableTable<'a> {
    fn insert_variable(
        &mut self,
        name: &'a str,
        kind: &'a syntax_tree::Kind,
        loc: &'a syntax_tree::Location,
    ) -> Result<(), SemanticError<'a>>;
}

pub struct GlobalVariableTable<'a> {
    global_table: VarNameTable<'a>,
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
        loc: &'a syntax_tree::Location,
    ) -> Result<(), SemanticError<'a>> {
        self.global_table
            .check_collision(name, loc, Entry::Variable)?;
        self.global_table.insert(name, (kind, loc));
        Ok(())
    }
}

pub struct FunctionTable<'a> {
    global_table: VarNameTable<'a>,
    function_table: NameTable<'a, &'a syntax_tree::FuncDecl>,
}

impl<'a> FunctionTable<'a> {
    fn new(global_table: VarNameTable<'a>) -> Self {
        Self {
            global_table,
            function_table: NameTable::new(Entry::Function),
        }
    }

    pub fn switch_to_local_table(self) -> FactoryLocalVariableTable<'a> {
        FactoryLocalVariableTable::new(self.global_table, self.function_table)
    }

    pub fn insert_function(
        &mut self,
        name: &'a str,
        func_decl: &'a syntax_tree::FuncDecl,
    ) -> Result<(), SemanticError<'a>> {
        self.global_table
            .check_collision(name, &func_decl.loc, Entry::Function)?;
        self.function_table
            .check_collision(name, &func_decl.loc, Entry::Function)?;
        self.function_table.insert(name, func_decl);
        Ok(())
    }
}

pub struct FactoryLocalVariableTable<'a> {
    global_table: VarNameTable<'a>,
    function_table: NameTable<'a, &'a syntax_tree::FuncDecl>,
}

impl<'b, 'a: 'b> FactoryLocalVariableTable<'a> {
    fn new(
        global_table: VarNameTable<'a>,
        function_table: NameTable<'a, &'a syntax_tree::FuncDecl>,
    ) -> Self {
        Self {
            global_table,
            function_table,
        }
    }

    pub fn factory_local_table(&'a self) -> LocalVariableTable<'b> {
        LocalVariableTable::new(&self.global_table, &self.function_table)
    }
}

pub struct LocalVariableTable<'a> {
    global_table: &'a VarNameTable<'a>,
    function_table: &'a NameTable<'a, &'a syntax_tree::FuncDecl>,
    local_table: VarNameTable<'a>,
}

impl<'a> LocalVariableTable<'a> {
    fn new(
        global_table: &'a VarNameTable<'a>,
        function_table: &'a NameTable<'a, &'a syntax_tree::FuncDecl>,
    ) -> Self {
        Self {
            global_table,
            function_table,
            local_table: NameTable::new(Entry::Variable),
        }
    }

    pub fn get_variable(&self, name: &'a str) -> Result<&syntax_tree::Kind, SemanticError<'a>> {
        if let Some((output, _)) = self.local_table.get(name) {
            Ok(output)
        } else if let Some((output, _)) = self.global_table.get(name) {
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
}

impl<'a> VariableTable<'a> for LocalVariableTable<'a> {
    fn insert_variable(
        &mut self,
        name: &'a str,
        kind: &'a syntax_tree::Kind,
        loc: &'a syntax_tree::Location,
    ) -> Result<(), SemanticError<'a>> {
        self.function_table
            .check_collision(name, loc, Entry::Variable)?;
        self.local_table
            .check_collision(name, loc, Entry::Variable)?;
        self.local_table.insert(name, (kind, loc));
        Ok(())
    }
}

enum Entry {
    Variable,
    Function,
}

struct NameTable<'a, T>
where
    T: 'a,
    T: Localizable,
{
    table: HashMap<&'a str, T>,
    entry_kind: Entry,
}

impl<'a, T: Localizable> NameTable<'a, T> {
    fn new(entry_kind: Entry) -> Self {
        Self {
            table: HashMap::new(),
            entry_kind,
        }
    }

    fn get(&self, name: &str) -> Option<&T> {
        self.table.get(name)
    }

    fn insert(&mut self, name: &'a str, entry: T) {
        self.table.insert(name, entry);
    }

    fn check_collision(
        &self,
        name: &str,
        loc: &syntax_tree::Location,
        new_entry: Entry,
    ) -> Result<(), SemanticError<'a>> {
        if let Some(data) = self.table.get(name) {
            let (original, new) = match (&self.entry_kind, new_entry) {
                (Entry::Function, Entry::Function) => (
                    Ridefinition::Function(data.get_location().clone()),
                    Ridefinition::Function(loc.clone()),
                ),
                (Entry::Function, Entry::Variable) => (
                    Ridefinition::Function(data.get_location().clone()),
                    Ridefinition::Variable(loc.clone()),
                ),
                (Entry::Variable, Entry::Function) => (
                    Ridefinition::Variable(data.get_location().clone()),
                    Ridefinition::Function(loc.clone()),
                ),
                (Entry::Variable, Entry::Variable) => (
                    Ridefinition::Variable(data.get_location().clone()),
                    Ridefinition::Variable(loc.clone()),
                ),
            };
            let err = NameRidefinition::new(name.to_owned(), original, new);
            let err = SemanticError::NameRidefinition(err);
            Err(err)
        } else {
            Ok(())
        }
    }
}

trait Localizable {
    fn get_location(&self) -> &syntax_tree::Location;
}

impl Localizable for &syntax_tree::FuncDecl {
    fn get_location(&self) -> &syntax_tree::Location {
        &self.loc
    }
}

impl Localizable for (&syntax_tree::Kind, &syntax_tree::Location) {
    fn get_location(&self) -> &syntax_tree::Location {
        &self.1
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
            111,
            222,
        );

        // some random location
        let loc_a = syntax_tree::Location::new(14, 25);
        let loc_b = syntax_tree::Location::new(34, 60);
        let loc_c = syntax_tree::Location::new(67, 91);

        let global_variable = "test_variable";

        let other_global_variable = "global_variable";

        let local_variable = "local_test";
        let other_local_variable = "other_local_variable";

        // state 1: global variables
        let mut table = name_table_factory();
        table
            .insert_variable(global_variable, &syntax_tree::Kind::Int, &loc_a)
            .unwrap();
        table
            .insert_variable(other_global_variable, &syntax_tree::Kind::Real, &loc_b)
            .unwrap();

        let err = table.insert_variable(global_variable, &syntax_tree::Kind::Bool, &loc_c);
        check_status(
            err,
            global_variable,
            Ridefinition::Variable(loc_a.clone()),
            Ridefinition::Variable(loc_c.clone()),
        );

        // state 2: functions
        let mut table = table.switch_to_function_table();
        table.insert_function(function_name, &func_decl).unwrap();

        let err = table.insert_function(global_variable, &func_decl);
        check_status(
            err,
            global_variable,
            Ridefinition::Variable(loc_a.clone()),
            Ridefinition::Function(syntax_tree::Location::new(111, 222)),
        );

        table
            .insert_function(other_function_name, &func_decl)
            .unwrap();

        let err = table.insert_function(function_name, &func_decl);
        check_status(
            err,
            function_name,
            Ridefinition::Function(syntax_tree::Location::new(111, 222)),
            Ridefinition::Function(syntax_tree::Location::new(111, 222)),
        );

        // state 3: local functions (formal parameters and function local variables)
        let loc_d = syntax_tree::Location::new(123, 160);
        let loc_e = syntax_tree::Location::new(178, 190);
        let loc_f = syntax_tree::Location::new(210, 260);

        let table_factory = table.switch_to_local_table();
        let mut table = table_factory.factory_local_table();
        table
            .insert_variable(local_variable, &syntax_tree::Kind::Real, &loc_d)
            .unwrap();

        let err = table.insert_variable(function_name, &syntax_tree::Kind::Bool, &loc_f);
        check_status(
            err,
            function_name,
            Ridefinition::Function(syntax_tree::Location::new(111, 222)),
            Ridefinition::Variable(loc_f.clone()),
        );

        table
            .insert_variable(other_local_variable, &syntax_tree::Kind::Int, &loc_e)
            .unwrap();

        let err = table.insert_variable(other_local_variable, &syntax_tree::Kind::Int, &loc_f);
        check_status(
            err,
            other_local_variable,
            Ridefinition::Variable(loc_e.clone()),
            Ridefinition::Variable(loc_f.clone()),
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
                SemanticError::NameRidefinition(ridef) => {
                    assert_eq!(ridef.name, original_name);
                    assert_eq!(ridef.original, original_kind);
                    assert_eq!(ridef.new, new_kind)
                }
                _ => panic!("Wrong Error kind: {:?}", err),
            },
        }
    }
}
