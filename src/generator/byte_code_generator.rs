use super::code_generator::*;
use super::function_index::FunctionIndex;
use super::opcode;
use super::simple_counter::{AddrSize, SimpleCounter};
use super::var_cache::{ParameterAddress, VarLookup, VariableType};

use simpla_parser::syntax_tree;

const ADDR_SIZE_ZERO: AddrSize = 0;
const LOCAL_MASK: AddrSize = 1 << (ADDR_SIZE_ZERO.count_zeros() - 1);
const MAX_STR_LEN: usize = (1 << (ADDR_SIZE_ZERO.count_zeros())) - 1;

pub struct ByteCodeGenerator<'a> {
    buff: Vec<u8>,
    function_index: FunctionIndex<'a>,
    label_counter: SimpleCounter,
    loop_exit_label: Vec<AddrSize>,
    local_cache: VarLookup<'a>,
    param_addr: ParameterAddress<'a>,
}

impl<'a> ByteCodeGenerator<'a> {
    pub fn new(
        function_index: FunctionIndex<'a>,
        local_cache: VarLookup<'a>,
        param_addr: ParameterAddress<'a>,
    ) -> Self {
        Self {
            buff: Vec::new(),
            local_cache,
            function_index,
            label_counter: SimpleCounter::new(),
            loop_exit_label: Vec::new(),
            param_addr,
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

    fn insert_true_cond_jump(&mut self, index: AddrSize) {
        self.insert_address_command(opcode::JEQ, index);
    }

    fn insert_label(&mut self, index: AddrSize) {
        self.insert_address_command(opcode::LBL, index);
    }

    fn memory_command<F>(&mut self, name: &str, convert: F)
    where
        F: Fn(&syntax_tree::Kind) -> u8,
    {
        let ((kind, id), ref scope) = self.local_cache.lookup(name);
        let id = match scope {
            VariableType::Global => *id,
            VariableType::Local => *id + LOCAL_MASK,
        };
        let cmd = convert(kind);
        self.insert_multi_byte_command(cmd, &id.to_be_bytes());
    }

    fn load_variable(&mut self, name: &str) {
        self.memory_command(name, load_by_kind);
    }

    fn convert_expression(&mut self, expr: &'a syntax_tree::Expr) {
        match &expr.expr {
            syntax_tree::ExprTree::Node(lhs, op, rhs) => self.convert_node(lhs, op, rhs),
            syntax_tree::ExprTree::Factor(fact) => self.convert_factor(fact),
        }
    }

    fn convert_node(
        &mut self,
        lhs: &'a syntax_tree::Expr,
        op: &'a syntax_tree::Operator,
        rhs: &'a syntax_tree::Expr,
    ) {
        if is_short_circuit_operator(op) {
            self.convert_short_circuit_operator(lhs, op, rhs);
        } else {
            self.convert_standard_node(lhs, op, rhs);
        }
    }

    fn convert_short_circuit_operator(
        &mut self,
        lhs: &'a syntax_tree::Expr,
        op: &'a syntax_tree::Operator,
        rhs: &'a syntax_tree::Expr,
    ) {
        self.convert_expression(lhs);
        let sc_label = self.label_counter.count_one();
        let end_label = self.label_counter.count_one();
        match op {
            syntax_tree::Operator::And => {
                self.insert_false_cond_jump(sc_label);
            }
            syntax_tree::Operator::Or => {
                self.insert_true_cond_jump(sc_label);
            }
            _ => panic!(),
        }
        self.convert_expression(rhs);
        self.insert_uncond_jump(end_label);
        self.insert_label(sc_label);
        match op {
            syntax_tree::Operator::And => {
                self.convert_constant(&syntax_tree::Const::BoolConst(false));
            }
            syntax_tree::Operator::Or => {
                self.convert_constant(&syntax_tree::Const::BoolConst(true));
            }
            _ => panic!(),
        }

        self.insert_label(end_label);
    }

    fn convert_standard_node(
        &mut self,
        lhs: &'a syntax_tree::Expr,
        op: &'a syntax_tree::Operator,
        rhs: &'a syntax_tree::Expr,
    ) {
        self.convert_expression(lhs);
        self.convert_expression(rhs);
        self.buff
            .push(operator_by_kind(op, lhs.kind.borrow().as_ref().unwrap()));
    }

    fn convert_factor(&mut self, fact: &'a syntax_tree::Factor) {
        match &fact.fact {
            syntax_tree::FactorValue::Id(id) => self.load_variable(id),
            syntax_tree::FactorValue::CastExpr(cast) => self.convert_cast_expr(cast),
            syntax_tree::FactorValue::CondExpr(cond_expr) => self.convert_cond_expr(cond_expr),
            syntax_tree::FactorValue::Const(cons) => self.convert_constant(cons),
            syntax_tree::FactorValue::FuncCall(f_call) => self.convert_func_call(f_call),
            syntax_tree::FactorValue::HighPrecedence(expr) => self.convert_expression(expr),
            syntax_tree::FactorValue::UnaryOp(unary) => self.convert_unary_op(unary),
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
            syntax_tree::UnaryOp::Minus(fact) => (
                fact,
                UnaryOp::from_kind(fact.kind.borrow().as_ref().unwrap()),
            ),
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
            syntax_tree::Const::StrConst(s) => self.insert_string(s),
        }
    }

    fn insert_string(&mut self, s: &str) {
        let str_bytes = truncate_str_to_byte_len(s, MAX_STR_LEN);
        let len = str_bytes.len() as AddrSize;
        self.insert_multi_byte_command(opcode::LDSC, &len.to_be_bytes());
        self.insert_bytes(str_bytes);
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
        self.convert_expression(&for_stat.end_expr);
        self.buff.push(opcode::BFOR);
        self.insert_label(for_lbl);
        self.load_variable(&for_stat.id);
        self.buff.push(opcode::CFOR);
        self.buff.push(opcode::LEQI);
        self.insert_false_cond_jump(end_lbl);
        self.gen_block(&for_stat.body, BlockType::General);
        self.convert_constant(&syntax_tree::Const::IntConst(1));
        self.load_variable(&for_stat.id);
        self.buff.push(opcode::ADDI);
        self.assign_value(&for_stat.id);
        self.insert_uncond_jump(for_lbl);
        self.insert_label(end_lbl);
        self.buff.push(opcode::EFOR);
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
        let ((kind, _), _) = self.local_cache.lookup(id);
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
        let f_id = self.function_index.get_function_index(&func_call.id);
        self.insert_address_command(opcode::PARAM, f_id);

        for (index, expr) in func_call.args.iter().enumerate() {
            let addr = self.param_addr.get_parameter_address(&func_call.id, index);
            self.convert_parameter(expr, addr);
        }

        self.insert_address_command(opcode::CALL, f_id);
    }

    fn convert_parameter(&mut self, expr: &'a syntax_tree::Expr, addr: AddrSize) {
        self.convert_expression(expr);
        let expr_kind = expr.kind.borrow();
        let kind = expr_kind.as_ref().unwrap();
        let store = store_param_by_kind(&kind);
        self.buff.push(store);
        let param_id = addr + LOCAL_MASK;
        self.insert_bytes(&param_id.to_be_bytes());
    }

    pub fn switch_local_cache(&mut self, local: VarLookup<'a>) {
        self.local_cache = local;
    }

    fn allocate_variables(&mut self, var_decl_list: &syntax_tree::VarDeclList, cmd: u8) {
        let var_count = VariableCounter::count_variables(var_decl_list);
        self.buff.push(cmd);
        self.insert_bytes(&var_count.vectorize());
    }
}

impl<'a> CodeGenerator<'a> for ByteCodeGenerator<'a> {
    fn gen_function(&mut self, func: &'a syntax_tree::FuncDecl) {
        self.buff.push(opcode::FUNC);
        let var_count = VariableCounter::count_variables(&func.vars);
        let var_count = var_count.count_parameters(&func.params);
        self.buff.push(opcode::INIT);
        self.insert_bytes(&var_count.vectorize());
        self.gen_block(&func.body, BlockType::General);
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

    fn gen_variables(&mut self, var_decl_list: &'a syntax_tree::VarDeclList) {
        self.allocate_variables(var_decl_list, opcode::INIT);
    }

    fn get_result(self) -> Vec<u8> {
        self.buff
    }
}

#[derive(std::default::Default, Debug)]
struct VariableCounter {
    integer_count: u16,
    real_count: u16,
    boolean_count: u16,
    string_count: u16,
}

impl VariableCounter {
    fn count_variables(var_decl_list: &syntax_tree::VarDeclList) -> Self {
        var_decl_list
            .iter()
            .map(|decl| (&decl.kind, decl.id_list.len() as u16))
            .fold(Self::default(), |mut acc, (kind, count)| {
                match kind {
                    syntax_tree::Kind::Int => acc.integer_count += count,
                    syntax_tree::Kind::Real => acc.real_count += count,
                    syntax_tree::Kind::Bool => acc.boolean_count += count,
                    syntax_tree::Kind::Str => acc.string_count += count,
                    _ => unreachable!(),
                }
                acc
            })
    }

    fn count_parameters(mut self, param_list: &syntax_tree::ParamList) -> Self {
        for par_decl in param_list {
            match par_decl.kind {
                syntax_tree::Kind::Int => self.integer_count += 1,
                syntax_tree::Kind::Real => self.real_count += 1,
                syntax_tree::Kind::Str => self.string_count += 1,
                syntax_tree::Kind::Bool => self.boolean_count += 1,
                _ => unreachable!()
            }
        }
        self
    }

    fn vectorize(self) -> [u8; 4 * 2] {
        let [i1, i2] = self.integer_count.to_be_bytes();
        let [r1, r2] = self.real_count.to_be_bytes();
        let [b1, b2] = self.boolean_count.to_be_bytes();
        let [s1, s2] = self.string_count.to_be_bytes();
        [i1, i2, r1, r2, b1, b2, s1, s2]
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
            Self::SameLine => opcode::FLU,
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
            Self::ToInt => opcode::CSTI,
            Self::ToReal => opcode::CSTR,
        }
    }
}

enum UnaryOp {
    IntNeg,
    RealNeg,
    BoolNeg,
}

impl UnaryOp {
    fn from_kind(kind: &syntax_tree::Kind) -> Self {
        match kind {
            syntax_tree::Kind::Int => Self::IntNeg,
            syntax_tree::Kind::Real => Self::RealNeg,
            _ => unreachable!(),
        }
    }

    fn get_command(&self) -> u8 {
        match self {
            Self::RealNeg => opcode::NEGR,
            Self::IntNeg => opcode::NEGI,
            Self::BoolNeg => opcode::NOT,
        }
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
        syntax_tree::Kind::Str => str_operator(op),
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
        syntax_tree::Operator::Equal => opcode::EQB,
        syntax_tree::Operator::NotEqual => opcode::NEB,
        syntax_tree::Operator::Greater => opcode::GRB,
        syntax_tree::Operator::GreaterEqual => opcode::GEQB,
        syntax_tree::Operator::Less => opcode::LESQB,
        syntax_tree::Operator::LessEqual => opcode::LEQB,
        _ => unreachable!(),
    }
}

fn str_operator(op: &syntax_tree::Operator) -> u8 {
    match op {
        syntax_tree::Operator::And => opcode::AND,
        syntax_tree::Operator::Or => opcode::OR,
        syntax_tree::Operator::Equal => opcode::EQS,
        syntax_tree::Operator::NotEqual => opcode::NES,
        syntax_tree::Operator::Greater => opcode::GRS,
        syntax_tree::Operator::GreaterEqual => opcode::GEQS,
        syntax_tree::Operator::Less => opcode::LESQS,
        syntax_tree::Operator::LessEqual => opcode::LEQS,
        _ => unreachable!(),
    }
}

fn truncate_str_to_byte_len(string: &str, byte_count: usize) -> &[u8] {
    let mut output_len = if string.len() > byte_count {
        byte_count
    } else {
        string.len()
    };
    while !string.is_char_boundary(output_len) {
        output_len -= 1;
    }
    string[..output_len].as_bytes()
}

fn is_short_circuit_operator(op: &syntax_tree::Operator) -> bool {
    match op {
        syntax_tree::Operator::And | syntax_tree::Operator::Or => true,
        _ => false,
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_truncate_string_with_long_string() {
        // each emoji requires 4 bytes
        let string = "😁😁😁😁😁😁😁😁😁😁😁";
        let byte_count = 5;
        let bytes = truncate_str_to_byte_len(string, byte_count);
        assert!(bytes.len() < byte_count);
        let byte_vec = Vec::from(bytes);
        let new_string = String::from_utf8(byte_vec).unwrap();
        // so a string long at most 5 bytes can contain just one emoji
        assert_eq!(new_string, "😁");
    }

    #[test]
    fn test_truncate_string_with_short_string() {
        let string = "😁😁😁😁😁😁😁😁😁😁😁";
        let byte_count = 100;
        let bytes = truncate_str_to_byte_len(string, byte_count);
        assert_eq!(bytes.len(), string.len());
        let byte_vec = Vec::from(bytes);
        let new_string = String::from_utf8(byte_vec).unwrap();
        assert_eq!(new_string, string);
    }

    use simpla_parser;

    #[test]
    fn test_variable_counter() {
        let simpla_code = r#"
            body
                writeln("Base Case");
            end.
        "#;
        run_variable_count_test(&simpla_code, 0, 0, 0, 0);

        let simpla_code = r#"
            a, b, c: integer;
            d: integer;

            s: string;
            q, w, e, r, t, y: boolean;
            h, j, k: real;
            p: string;

            body
                writeln("With some variables");
            end.
        "#;

        run_variable_count_test(&simpla_code, 4, 3, 6, 2);
    }

    fn run_variable_count_test(
        code: &str,
        int_count: u16,
        real_count: u16,
        bool_count: u16,
        str_count: u16,
    ) {
        let parser = simpla_parser::ProgramParser::new();
        let tree = parser.parse(code).unwrap();

        let var_count = VariableCounter::count_variables(&tree.global_vars);

        assert_eq!(var_count.integer_count, int_count);
        assert_eq!(var_count.real_count, real_count);
        assert_eq!(var_count.boolean_count, bool_count);
        assert_eq!(var_count.string_count, str_count);
    }
}
