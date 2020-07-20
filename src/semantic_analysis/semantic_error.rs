#[derive(Debug)]
pub enum SemanticError {
    NameRidefinition {
        name: String,
        original: Ridefinition,
        new: Ridefinition,
    },
    InnerError,
}

#[derive(Debug, PartialEq)]
pub enum Ridefinition {
    Function,
    Variable,
}
