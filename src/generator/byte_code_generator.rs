use super::code_generator::*;
use super::function_index::FunctionIndex;
use super::opcode;
use super::var_cache::VarCache;
use super::simple_counter::SimpleCounter;

use simpla_parser::syntax_tree;

type addr_size = u16;

pub struct ByteCodeGenerator<'a> {
    buff: Vec<u8>,
    var_cache: VarCache<'a>,
    function_index: FunctionIndex<'a>,
    label_counter: SimpleCounter
}

impl<'a> ByteCodeGenerator<'a> {
    pub fn new(function_index: FunctionIndex<'a>) -> Self {
        Self {
            buff: Vec::new(),
            var_cache: VarCache::new(),
            function_index,
            label_counter: SimpleCounter::new()
        }
    }

    fn insert_address_command(&mut self, cmd: u8, index: addr_size) {
        self.buff.push(cmd);
        self.insert_bytes(&index.to_be_bytes());
    }

    fn insert_uncond_jump(&mut self, index: addr_size) {
        self.insert_address_command(opcode::JUMP, index);
    }

    fn insert_false_cond_jump(&mut self, index: addr_size) {
        self.insert_address_command(opcode::JNE, index);
    }

    fn insert_label(&mut self, index: addr_size) {
        self.insert_address_command(opcode::LBL, index);
    }

    fn memory_command<F>(&mut self, name: &str, convert: F)
    where F: Fn (&syntax_tree::Kind) -> u8 {
        let (kind, id) = self.var_cache.lookup(name);
        let id = *id;
        self.buff.push(convert(kind));
        self.insert_bytes(&id.to_be_bytes());
    }

    fn load_variable(&mut self, name: &str) {
        self.memory_command(name, load_by_kind);
    }

    fn convert_expression(&mut self, expr: &syntax_tree::Expr) {}


    fn assign_value(&mut self, name: &str) {
        self.memory_command(name, store_by_kind);
    }


    fn eval_and_assign(&mut self, name: &str, expr: &syntax_tree::Expr) {
        self.convert_expression(expr);
        self.assign_value(name);
    }


    fn convert_statement(&mut self, stat: &'a syntax_tree::Stat) {
        match &stat.stat {
            syntax_tree::StatType::AssignStat(assign_stat) => self.convert_assign_stat(assign_stat),
            syntax_tree::StatType::IfStat(if_stat) => self.convert_if_stat(if_stat),
            syntax_tree::StatType::WhileStat(while_stat) => self.convert_while_stat(while_stat),
            syntax_tree::StatType::ForStat(for_stat) => self.convert_for_stat(for_stat),
            syntax_tree::StatType::ReturnStat(return_stat) => {}
            syntax_tree::StatType::ReadStat(read_stat) => {}
            syntax_tree::StatType::WriteStat(write_stat) => {}
            syntax_tree::StatType::Break => {}
            syntax_tree::StatType::FuncCall(func_call) => {}
        }
    }

    fn insert_bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.buff.push(*b);
        }
    }

    fn convert_assign_stat(&mut self, assign: &syntax_tree::AssignStat) {
        self.eval_and_assign(&assign.id, &assign.expr);
    }

    fn convert_if_stat(&mut self, if_stat: &'a syntax_tree::IfStat) {
        self.convert_expression(&if_stat.cond);
        let end_if = self.label_counter.count_one();
        self.insert_false_cond_jump(end_if);
        self.gen_block(&if_stat.if_body, BlockType::General);
  
        if let Some(ref else_part) = &if_stat.else_body {
            let end = self.label_counter.count_one();
            self.insert_uncond_jump(end);
            self.insert_label(end_if);
            self.gen_block(else_part, BlockType::General);
            self.insert_label(end);
        } else {
            self.insert_label(end_if);
        }
    }

    fn convert_while_stat(&mut self, while_stat: &'a syntax_tree::WhileStat) {
        let while_lbl = self.label_counter.count_one();
        let end_lbl = self.label_counter.count_one();
        self.insert_label(while_lbl);
        self.convert_expression(&while_stat.cond);
        self.insert_false_cond_jump(end_lbl);
        self.gen_block(&while_stat.body, BlockType::General);
        self.insert_uncond_jump(while_lbl);
        self.insert_label(end_lbl);
    }


    fn convert_for_stat(&mut self, for_stat: &'a syntax_tree::ForStat) {
        self.eval_and_assign(&for_stat.id, &for_stat.begin_expr);
        let for_lbl = self.label_counter.count_one();
        let end_lbl = self.label_counter.count_one();
        self.insert_label(for_lbl);
        self.convert_expression(&for_stat.end_expr);
        self.load_variable(&for_stat.id);
        self.buff.push(opcode::LEQI);
        self.insert_false_cond_jump(end_lbl);
        self.gen_block(&for_stat.body, BlockType::General);
        
    }

}

impl<'a> CodeGenerator<'a> for ByteCodeGenerator<'a> {
    fn gen_function(&mut self, func: &'a syntax_tree::FuncDecl) {
        self.buff.push(opcode::FUNC);
        self.gen_variables(&func.vars, Scope::Local);
        self.gen_block(&func.body, BlockType::General);
        self.var_cache.clear_local_vars();
    }

    fn gen_block(&mut self, block: &'a syntax_tree::StatList, _block_type: BlockType) {
        for stat in block {
            self.convert_statement(stat)
        }
    }

    fn gen_variables(&mut self, var_decl_list: &'a syntax_tree::VarDeclList, scope: Scope) {
        match scope {
            Scope::Local => self.var_cache.cache_local_vars(var_decl_list),
            Scope::Global => self.var_cache.cache_global_vars(var_decl_list),
        };

        for var_decl in var_decl_list {
            for var in &var_decl.id_list {
                let (_, id) = self.var_cache.lookup(var);
                let id = *id; // so linter is happy
                let (first, second) = init_command(&var_decl.kind);
                let data = defaut_value(&var_decl.kind);
                self.buff.push(first);
                self.insert_bytes(&data);
                self.buff.push(second);
                self.insert_bytes(&id.to_be_bytes());
            }
        }
    }

    fn get_result(self) -> Vec<u8> {
        self.buff
    }
}

fn init_command(kind: &syntax_tree::Kind) -> (u8, u8) {
    match kind {
        syntax_tree::Kind::Bool => (opcode::LDBC, opcode::STRB),
        syntax_tree::Kind::Int => (opcode::LDIC, opcode::STRI),
        syntax_tree::Kind::Real => (opcode::LDRC, opcode::STRR),
        syntax_tree::Kind::Str => (opcode::LDSC, opcode::STRS),
        syntax_tree::Kind::Void => unreachable!(),
    }
}

fn defaut_value(kind: &syntax_tree::Kind) -> Vec<u8> {
    match kind {
        syntax_tree::Kind::Bool => Vec::from([0]),
        syntax_tree::Kind::Int => Vec::from((0 as i32).to_be_bytes()),
        syntax_tree::Kind::Real => Vec::from((0.0 as f64).to_be_bytes()),
        syntax_tree::Kind::Str => Vec::from((0 as u16).to_be_bytes()),
        syntax_tree::Kind::Void => unreachable!(),
    }
}

fn store_by_kind(k: &syntax_tree::Kind) -> u8 {
    match k {
        syntax_tree::Kind::Bool => opcode::STRB,
        syntax_tree::Kind::Int => opcode::STRI,
        syntax_tree::Kind::Real => opcode::STRR,
        syntax_tree::Kind::Str => opcode::STRS,
        syntax_tree::Kind::Void => unreachable!(),
    }
}

fn load_by_kind(k: &syntax_tree::Kind) -> u8 {
    match k {
        syntax_tree::Kind::Bool => opcode::LDB,
        syntax_tree::Kind::Int => opcode::LDI,
        syntax_tree::Kind::Real => opcode::LDR,
        syntax_tree::Kind::Str => opcode::LDS,
        syntax_tree::Kind::Void => unreachable!(),
    }

}


