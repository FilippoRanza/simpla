#[derive(Debug)]
pub enum SemanticError<'a> {
    NameRidefinition {
        name: String,
        original: Ridefinition,
        new: Ridefinition,
    },
    VoidVariableDeclaration {
        names: &'a [String],
    },

    InnerError,
}

#[derive(Debug, PartialEq)]
pub enum Ridefinition {
    Function,
    Variable,
}
