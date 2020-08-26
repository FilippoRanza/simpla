mod byte_code_generator;
mod c_generator;
mod code_generator;
mod function_index;
mod opcode;
mod simple_counter;
mod translate;
mod var_cache;

pub use translate::translate_to_byte_code;
pub use translate::translate_to_c;
