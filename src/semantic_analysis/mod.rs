mod body_check;
mod error_code;
mod error_message_generator;
mod name_table;
mod semantic_check;
mod semantic_error;
mod stat_check;
mod type_check;
mod variable_check;

pub use semantic_check::semantic_check;
