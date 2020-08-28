use super::code_generator::*;
use super::function_index::FunctionIndex;
use super::opcode;
use super::simple_counter::{AddrSize, SimpleCounter};
use super::var_cache::{VarCache, VariableType, KindCounter};

use simpla_parser::syntax_tree;

const ADDR_SIZE_ZERO: AddrSize = 0;
const LOCAL_MASK: AddrSize = 1 << (ADDR_SIZE_ZERO.count_zeros() - 1);

pub struct ByteCodeGenerator<'a> {
    buff: Vec<u8>,
    var_cache: VarCache<'a>,
    function_index: FunctionIndex<'a>,
    label_counter: SimpleCounter,
    param_counter: KindCounter,
    loop_exit_label: Vec<AddrSize>,
}

impl<'a> ByteCodeGenerator<'a> {
    pub fn new(function_index: FunctionIndex<'a>) -> Self {
        Self {
            buff: Vec::new(),
            var_cache: VarCache::new(),
            function_index,
            label_counter: SimpleCounter::new(),
            loop_exit_label: Vec::new(),
            param_counter: KindCounter::new()
        }
    }
    fn insert_multi_byte_command(&mut self, cmd: u8, data: &[u8]) {
        self.buff.push(cmd);
        self.insert_bytes(data);
    }

    fn insert_address_command(&mut self, cmd: u8, index: AddrSize) {
        self.insert_multi_byte_command(cmd, &index.to_be_bytes());
    }

    fn insert_uncond_jump(&mut self, index: AddrSize) {
        self.insert_address_command(opcode::JUMP, index);
    }

    fn insert_false_cond_jump(&mut self, index: AddrSize) {
        self.insert_address_command(opcode::JNE, index);
    }

    fn insert_label(&mut self, index: AddrSize) {
        self.insert_address_command(opcode::LBL, index);
    }

    fn memory_command<F>(&mut self, name: &str, convert: F)
    where
        F: Fn(&syntax_tree::Kind) -> u8,
    {
        let ((kind, id), ref scope) = self.var_cache.lookup(name);
        let id = match scope {
            VariableType::Global => *id,
            VariableType::Local => *id + LOCAL_MASK
        };
        let cmd = convert(kind); 
        self.insert_multi_byte_command(cmd, &id.to_be_bytes());
    }

    fn load_variable(&mut self, name: &str) {
        self.memory_command(name, load_by_kind);
    }

    fn convert_expression(&mut self, expr: &'a syntax_tree::Expr) {
        match &expr.expr {
            syntax_tree::ExprTree::Node(lhs, op, rhs) => {
                self.convert_expression(lhs);
                self.convert_expression(rhs);          
                self.buff
                    .push(operator_by_kind(op, lhs.kind.borrow().as_ref().unwrap()));
            }
            syntax_tree::ExprTree::Factor(fact) => self.convert_factor(fact),
        }
    }
    fn convert_factor(&mut self, fact: &'a syntax_tree::Factor) {
        match fact {
            syntax_tree::Factor::Id(id) => self.load_variable(id),
            syntax_tree::Factor::CastExpr(cast) => self.convert_cast_expr(cast),
            syntax_tree::Factor::CondExpr(cond_expr) => self.convert_cond_expr(cond_expr),
            syntax_tree::Factor::Const(cons) => self.convert_constant(cons),
            syntax_tree::Factor::FuncCall(f_call) => self.convert_func_call(f_call),
            syntax_tree::Factor::HighPrecedence(expr) => self.convert_expression(expr),
            syntax_tree::Factor::UnaryOp(unary) => self.convert_unary_op(unary),
        }
    }

    fn convert_cond_expr(&mut self, cond_expr: &'a syntax_tree::CondExpr) {
        let false_lbl = self.label_counter.count_one();
        let end_lbl = self.label_counter.count_one();
        self.convert_expression(&cond_expr.cond);
        self.insert_false_cond_jump(false_lbl);
        self.convert_expression(&cond_expr.true_stat);
        self.insert_uncond_jump(end_lbl);
        self.insert_label(false_lbl);
        self.convert_expression(&cond_expr.false_stat);
        self.insert_label(end_lbl);
    }

    fn convert_unary_op(&mut self, unary: &'a syntax_tree::UnaryOp) {
        let (fact, mode) = match unary {
            syntax_tree::UnaryOp::Minus(fact) => (fact, UnaryOp::IntNeg),
            syntax_tree::UnaryOp::Negate(fact) => (fact, UnaryOp::BoolNeg),
        };
        self.convert_factor(fact);
        self.buff.push(mode.get_command())
    }

    fn convert_cast_expr(&mut self, cast: &'a syntax_tree::CastExpr) {
        let (expr, mode) = match cast {
            syntax_tree::CastExpr::Integer(expr) => (expr, CastOp::ToInt),
            syntax_tree::CastExpr::Real(expr) => (expr, CastOp::ToReal),
        };
        self.convert_expression(expr);
        self.buff.push(mode.get_command());
    }

    fn convert_constant(&mut self, const_val: &syntax_tree::Const) {
        match const_val {
            syntax_tree::Const::IntConst(i) => {
                self.insert_multi_byte_command(opcode::LDIC, &i.to_be_bytes())
            }
            syntax_tree::Const::RealConst(r) => {
                self.insert_multi_byte_command(opcode::LDRC, &r.to_be_bytes())
            }
            syntax_tree::Const::BoolConst(b) => {
                self.insert_multi_byte_command(opcode::LDBC, &[if *b { 255 } else { 0 }])
            }
            syntax_tree::Const::StrConst(s) => {
                self.insert_string(s)
            }
        }
    }

    fn insert_string(&mut self, s: &str) {
        let len = s.len() as AddrSize;
        self.insert_multi_byte_command(opcode::LDSC, &len.to_be_bytes());
        self.insert_bytes(s.as_bytes());
    }

    fn assign_value(&mut self, name: &str) {
        self.memory_command(name, store_by_kind);
    }

    fn eval_and_assign(&mut self, name: &str, expr: &'a syntax_tree::Expr) {
        self.convert_expression(expr);
        self.assign_value(name);
    }

    fn convert_statement(&mut self, stat: &'a syntax_tree::Stat) {
        match &stat.stat {
            syntax_tree::StatType::AssignStat(assign_stat) => self.convert_assign_stat(assign_stat),
            syntax_tree::StatType::IfStat(if_stat) => self.convert_if_stat(if_stat),
            syntax_tree::StatType::WhileStat(while_stat) => self.convert_while_stat(while_stat),
            syntax_tree::StatType::ForStat(for_stat) => self.convert_for_stat(for_stat),
            syntax_tree::StatType::ReturnStat(return_stat) => self.convert_return_stat(return_stat),
            syntax_tree::StatType::ReadStat(read_stat) => self.convert_read_stat(read_stat),
            syntax_tree::StatType::WriteStat(write_stat) => self.convert_write_stat(write_stat),
            syntax_tree::StatType::Break => self.convert_break_stat(),
            syntax_tree::StatType::FuncCall(func_call) => self.convert_func_call(func_call),
        }
    }

    fn insert_bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.buff.push(*b);
        }
    }

    fn convert_assign_stat(&mut self, assign: &'a syntax_tree::AssignStat) {
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
        self.loop_exit_label.push(end_lbl);
        self.insert_label(while_lbl);
        self.convert_expression(&while_stat.cond);
        self.insert_false_cond_jump(end_lbl);
        self.gen_block(&while_stat.body, BlockType::General);
        self.insert_uncond_jump(while_lbl);
        self.insert_label(end_lbl);
        self.loop_exit_label.pop();
    }

    fn convert_for_stat(&mut self, for_stat: &'a syntax_tree::ForStat) {
        let for_lbl = self.label_counter.count_one();
        let end_lbl = self.label_counter.count_one();
        self.loop_exit_label.push(end_lbl);
        self.eval_and_assign(&for_stat.id, &for_stat.begin_expr);
        self.insert_label(for_lbl);
        self.load_variable(&for_stat.id);
        self.convert_expression(&for_stat.end_expr);
        self.buff.push(opcode::LEQI);
        self.insert_false_cond_jump(end_lbl);
        self.gen_block(&for_stat.body, BlockType::General);
        self.convert_constant(&syntax_tree::Const::IntConst(1));
        self.load_variable(&for_stat.id);
        self.buff.push(opcode::ADDI);
        self.assign_value(&for_stat.id);
        self.insert_uncond_jump(for_lbl);
        self.insert_label(end_lbl);
        self.loop_exit_label.pop();
    }

    fn convert_return_stat(&mut self, return_stat: &'a Option<syntax_tree::Expr>) {
        if let Some(expr) = return_stat {
            self.convert_expression(expr);
        }
        self.buff.push(opcode::RET);
    }

    fn convert_read_stat(&mut self, read_stat: &'a Vec<String>) {
        for id in read_stat {
            self.read_value(id);
        }
    }

    fn read_value(&mut self, id: &str) {
        let ((kind, _), _) = self.var_cache.lookup(id);
        self.buff.push(read_by_kind(kind));
        self.assign_value(id);
    }

    fn convert_write_stat(&mut self, write_stat: &'a syntax_tree::WriteStat) {
        let (expr_list, mode) = match write_stat {
            syntax_tree::WriteStat::Write(ref expr_list) => (expr_list, WriteMode::SameLine),
            syntax_tree::WriteStat::WriteLine(ref expr_list) => (expr_list, WriteMode::NewLine),
        };

        for expr in expr_list {
            self.write_value(expr);
        }
        self.buff.push(mode.flush_action())
    }

    fn write_value(&mut self, expr: &'a syntax_tree::Expr) {
        self.convert_expression(expr);
        self.buff
            .push(write_by_kind(expr.kind.borrow().as_ref().unwrap()));
    }

    fn convert_break_stat(&mut self) {
        let index = self.loop_exit_label.last();
        if let Some(index) = index {
            let index = *index;
            self.insert_uncond_jump(index);
        } else {
            panic!("break outside loop");
        }
    }

    fn convert_func_call(&mut self, func_call: &'a syntax_tree::FuncCall) {
        self.buff.push(opcode::PARAM);
        for expr in &func_call.args {
            self.convert_parameter(expr);
        }
        self.param_counter.reset();

        self.insert_address_command(
            opcode::CALL,
            self.function_index.get_function_index(&func_call.id),
        );
    }

    fn convert_parameter(&mut self, expr: &'a syntax_tree::Expr) {
        self.convert_expression(expr);
        let expr_kind = expr.kind.borrow();
        let kind = expr_kind.as_ref().unwrap();
        let store = store_param_by_kind(&kind);
        self.buff.push(store);
        let param_id = self.param_counter.get_index(&kind) + LOCAL_MASK;
        self.insert_bytes(&param_id.to_be_bytes());
    }

}

impl<'a> CodeGenerator<'a> for ByteCodeGenerator<'a> {
    fn gen_function(&mut self, func: &'a syntax_tree::FuncDecl) {
        self.buff.push(opcode::FUNC);
        self.var_cache.cache_params(&func.params);
        self.gen_variables(&func.vars, Scope::Local);
        self.gen_block(&func.body, BlockType::General);
        self.var_cache.clear_local_vars();
        if *self.buff.last().unwrap() != opcode::RET {
            self.buff.push(opcode::RET);
        }
    }

    fn gen_block(&mut self, block: &'a syntax_tree::StatList, block_type: BlockType) {
        
        for stat in block {
            self.convert_statement(stat)
        }
        match block_type {
            BlockType::General => {}
            BlockType::Main => self.buff.push(opcode::EXT),
        }
    }

    fn gen_variables(&mut self, var_decl_list: &'a syntax_tree::VarDeclList, scope: Scope) {
        match scope {
            Scope::Local => self.var_cache.cache_local_vars(var_decl_list),
            Scope::Global => self.var_cache.cache_global_vars(var_decl_list),
        };

        for var_decl in var_decl_list {
            for var in &var_decl.id_list {
                let ((_, id), _) = self.var_cache.lookup(var);
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

enum WriteMode {
    NewLine,
    SameLine,
}

impl WriteMode {
    fn flush_action(&self) -> u8 {
        match self {
            Self::NewLine => opcode::FLN,
            Self::SameLine => opcode::FLU
        }
    }
}


fn write_by_kind(k: &syntax_tree::Kind) -> u8 {
    match k {
        syntax_tree::Kind::Bool => opcode::WRB,
        syntax_tree::Kind::Int => opcode::WRI,
        syntax_tree::Kind::Real => opcode::WRR,
        syntax_tree::Kind::Str => opcode::WRS,
        syntax_tree::Kind::Void => unreachable!(),
    }
}

enum CastOp {
    ToInt,
    ToReal,
}

impl CastOp {
    fn get_command(&self) -> u8 {
        match self {
            Self::ToInt => opcode::CSTR,
            Self::ToReal => opcode::CSTI,
        }
    }
}

enum UnaryOp {
    IntNeg,
    BoolNeg,
}

impl UnaryOp {
    fn get_command(&self) -> u8 {
        match self {
            Self::IntNeg => opcode::NEG,
            Self::BoolNeg => opcode::NOT,
        }
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
        syntax_tree::Kind::Str => Vec::from((0 as AddrSize).to_be_bytes()),
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

fn read_by_kind(k: &syntax_tree::Kind) -> u8 {
    match k {
        syntax_tree::Kind::Bool => opcode::RDB,
        syntax_tree::Kind::Int => opcode::RDI,
        syntax_tree::Kind::Real => opcode::RDR,
        syntax_tree::Kind::Str => opcode::RDS,
        syntax_tree::Kind::Void => unreachable!(),
    }
}

fn store_param_by_kind(k: &syntax_tree::Kind) -> u8 {
    match k {
        syntax_tree::Kind::Bool => opcode::STRBP,
        syntax_tree::Kind::Int => opcode::STRIP,
        syntax_tree::Kind::Real => opcode::STRRP,
        syntax_tree::Kind::Str => opcode::STRSP,
        syntax_tree::Kind::Void => unreachable!(),
    }
}

fn operator_by_kind(op: &syntax_tree::Operator, k: &syntax_tree::Kind) -> u8 {
    match k {
        syntax_tree::Kind::Int => integer_operator(op),
        syntax_tree::Kind::Real => real_operator(op),
        syntax_tree::Kind::Bool => bool_operator(op),
        syntax_tree::Kind::Str => unreachable!(),
        syntax_tree::Kind::Void => unreachable!(),
    }
}

fn integer_operator(op: &syntax_tree::Operator) -> u8 {
    match op {
        syntax_tree::Operator::Equal => opcode::EQI,
        syntax_tree::Operator::NotEqual => opcode::NEI,
        syntax_tree::Operator::Greater => opcode::GRI,
        syntax_tree::Operator::GreaterEqual => opcode::GEQI,
        syntax_tree::Operator::Less => opcode::LESQI,
        syntax_tree::Operator::LessEqual => opcode::LEQI,
        syntax_tree::Operator::Add => opcode::ADDI,
        syntax_tree::Operator::Sub => opcode::SUBI,
        syntax_tree::Operator::Mul => opcode::MULI,
        syntax_tree::Operator::Div => opcode::DIVI,
        syntax_tree::Operator::And => unreachable!(),
        syntax_tree::Operator::Or => unreachable!(),
    }
}

fn real_operator(op: &syntax_tree::Operator) -> u8 {
    match op {
        syntax_tree::Operator::Equal => opcode::EQR,
        syntax_tree::Operator::NotEqual => opcode::NER,
        syntax_tree::Operator::Greater => opcode::GRR,
        syntax_tree::Operator::GreaterEqual => opcode::GEQR,
        syntax_tree::Operator::Less => opcode::LESQR,
        syntax_tree::Operator::LessEqual => opcode::LEQR,
        syntax_tree::Operator::Add => opcode::ADDR,
        syntax_tree::Operator::Sub => opcode::SUBR,
        syntax_tree::Operator::Mul => opcode::MULR,
        syntax_tree::Operator::Div => opcode::DIVR,
        syntax_tree::Operator::And => unreachable!(),
        syntax_tree::Operator::Or => unreachable!(),
    }
}

fn bool_operator(op: &syntax_tree::Operator) -> u8 {
    match op {
        syntax_tree::Operator::And => opcode::AND,
        syntax_tree::Operator::Or => opcode::OR,
        _ => unreachable!(),
    }
}
