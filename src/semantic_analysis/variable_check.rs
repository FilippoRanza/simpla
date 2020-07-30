use super::name_table::VariableTable;
use super::semantic_error::{SemanticError, VoidVariableDeclaration};
use simpla_parser::syntax_tree::{Kind, VarDeclList};

pub fn check_variables<'a, T>(
    var_decl_list: &'a VarDeclList,
    table: &mut T,
) -> Result<(), SemanticError<'a>>
where
    T: VariableTable<'a>,
{
    for var_decl in var_decl_list {
        if var_decl.kind == Kind::Void {
            return Err(SemanticError::VoidVariableDeclaration(
                VoidVariableDeclaration::new(&var_decl),
            ));
        }
        for id in &var_decl.id_list {
            table.insert_variable(id, &var_decl.kind)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use super::super::name_table::name_table_factory;
    use super::super::semantic_error::Ridefinition;
    use super::*;
    use simpla_parser::syntax_tree::VarDecl;

    #[test]
    fn test_correct_variable_decl() {
        let var_decl_list = vec![
            var_decl_list_factory(&["var", "a", "test"], Kind::Bool),
            var_decl_list_factory(&["name", "uniq", "f"], Kind::Real),
            var_decl_list_factory(&["important", "csv_file"], Kind::Str),
            var_decl_list_factory(&["pi", "e"], Kind::Real),
            var_decl_list_factory(&["i", "j"], Kind::Int),
        ];

        let mut table = name_table_factory();
        check_variables(&var_decl_list, &mut table).unwrap();
    }

    #[test]
    fn test_void_variable_decl() {
        let void_names = ["void_name", "invalid_type", "illegal_decl"];
        let var_decl_list = vec![
            var_decl_list_factory(&["var", "a", "test"], Kind::Bool),
            var_decl_list_factory(&void_names, Kind::Void),
        ];

        let mut table = name_table_factory();
        let stat = check_variables(&var_decl_list, &mut table);
        match stat {
            Ok(_) => panic!("This test should generate a VoidVariableDeclaration"),
            Err(err) => match err {
                SemanticError::VoidVariableDeclaration(decl) => {
                    assert_eq!(decl.names, &var_decl_list[1])
                }
                err => panic!("Wrong error variant: {:?}", err),
            },
        }
    }

    #[test]
    fn test_duplicated_variable_name() {
        let var_decl_list = vec![
            var_decl_list_factory(&["var", "a", "test"], Kind::Bool),
            var_decl_list_factory(&["name", "a", "f"], Kind::Real),
        ];
        let mut table = name_table_factory();
        let stat = check_variables(&var_decl_list, &mut table);
        match stat {
            Ok(_) => panic!("This test should generate a NameRidefinition"),
            Err(err) => match err {
                SemanticError::NameRidefinition(ridef) => {
                    assert_eq!(ridef.name, "a");
                    assert_eq!(ridef.original, Ridefinition::Variable);
                    assert_eq!(ridef.new, Ridefinition::Variable);
                }
                err => panic!("Wrong error variant: {:?}", err),
            },
        }
    }

    fn var_decl_list_factory(names: &[&str], kind: Kind) -> VarDecl {
        let tmp = names.iter().map(|s| s.to_string()).collect();
        VarDecl::new(tmp, kind, 0, 0)
    }
}
