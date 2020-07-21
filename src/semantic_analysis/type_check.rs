

use simpla_parser::syntax_tree;
use super::semantic_error::SemanticError;
use super::name_table::LocalVariableTable;

fn type_check<'a>(expr: &'a syntax_tree::Expr, table: &'a LocalVariableTable) -> Result<syntax_tree::Kind, SemanticError<'a>> {
    // so it compiles.
    Ok(syntax_tree::Kind::Bool)
}


