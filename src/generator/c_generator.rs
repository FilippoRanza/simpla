use super::code_generator::*;
use super::var_cache::VarCache;
use simpla_parser::syntax_tree;

const HEADER: &'static str = r#"
#include <stdio.h>
#include <stdlib.h>

#define TRUE 1
#define FALSE 0
#define BUFF_SIZE 1024

char* _INPUT_BUFFER = NULL;


char* _alloc_buffer() {

    char* output = calloc(BUFF_SIZE, sizeof(char));
    if(output == NULL) {
        fprintf(stderr, "cannot allocate buffer of size: %d", BUFF_SIZE);
        abort();
    }

    return output;
}


void _read_buffer() {
    char* tmp = _INPUT_BUFFER;
    int c;
    int count = BUFF_SIZE - 1;
    while((c = getchar()) && c != EOF && c != '\n' && count--) 
        *tmp++ = c;
    *tmp = '\0';
}


char _read_bool() {
    _read_buffer(_INPUT_BUFFER);
    int tmp = atoi(_INPUT_BUFFER);
    return tmp ? TRUE : FALSE;
}

int _read_int() {
    _read_buffer(_INPUT_BUFFER);
    return atoi(_INPUT_BUFFER);
}

double _read_double() {
    _read_buffer(_INPUT_BUFFER);
    return atof(_INPUT_BUFFER);
}

void _read_str(char* str) {
    _read_buffer(str);
}

void _initialize() {
    _INPUT_BUFFER = _alloc_buffer();
}

void _finalize() {
    free(_INPUT_BUFFER);
}

"#;

const ID_HEADER: &'static str = "__";

pub struct CSourceGenerator<'a> {
    buff: Vec<String>,
    var_cache: VarCache<'a>,
}

impl<'a> CSourceGenerator<'a> {
    pub fn new() -> Self {
        Self {
            buff: Vec::new(),
            var_cache: VarCache::new(),
        }
    }

    fn open_block(&mut self) {
        self.add_code("{");
    }

    fn close_block(&mut self) {
        self.add_code("}");
    }

    fn add_code(&mut self, code: &str) {
        let tmp = String::from(code);
        self.buff.push(tmp)
    }

    fn convert_block(&self, block: &syntax_tree::StatList) -> String {
        let mut tmp = Vec::new();
        for stat in block {
            let code = self.convert_statement(&stat.stat);
            tmp.push(code);
        }
        tmp.join("\n")
    }

    fn convert_statement(&self, stat: &syntax_tree::StatType) -> String {
        match stat {
            syntax_tree::StatType::AssignStat(assign) => self.convert_assign_stat(assign),
            syntax_tree::StatType::IfStat(if_stat) => self.convert_if_stat(if_stat),
            syntax_tree::StatType::WhileStat(while_stat) => self.convert_while_stat(while_stat),
            syntax_tree::StatType::ForStat(for_stat) => self.convert_for_stat(for_stat),
            syntax_tree::StatType::ReturnStat(return_stat) => self.convert_return_stat(return_stat),
            syntax_tree::StatType::ReadStat(read) => self.convert_read_stat(read),
            syntax_tree::StatType::WriteStat(write) => self.convert_write_stat(write),
            syntax_tree::StatType::FuncCall(call) => {
                let tmp = self.convert_func_call(call);
                format!("{};", tmp)
            }
            syntax_tree::StatType::Break => self.convert_break_stat(),
        }
    }

    fn convert_assign_stat(&self, assign: &syntax_tree::AssignStat) -> String {
        format!(
            "{} = {};",
            convert_id(ID_HEADER, &assign.id),
            self.convert_expression(&assign.expr)
        )
    }

    fn convert_if_stat(&self, if_stat: &syntax_tree::IfStat) -> String {
        let if_part = format!(
            "if ({}) {{ {} }}",
            self.convert_expression(&if_stat.cond),
            self.convert_block(&if_stat.if_body)
        );
        if let Some(ref else_part) = if_stat.else_body {
            let else_code = self.convert_block(else_part);
            format!("{} else {{ {} }}", if_part, else_code)
        } else {
            if_part
        }
    }

    fn convert_while_stat(&self, while_stat: &syntax_tree::WhileStat) -> String {
        let body = self.convert_block(&while_stat.body);
        let cond = self.convert_expression(&while_stat.cond);
        format!("while ({}) {{ {} }}", cond, body)
    }

    fn convert_for_stat(&self, for_stat: &syntax_tree::ForStat) -> String {
        let begin = self.convert_expression(&for_stat.begin_expr);
        let end = self.convert_expression(&for_stat.end_expr);
        let body = self.convert_block(&for_stat.body);

        format!(
            "for({0} = {1}; {0} < {2}; {0}++) {{ {3} }}",
            convert_id(ID_HEADER, &for_stat.id),
            begin,
            end,
            body
        )
    }

    fn convert_return_stat(&self, return_stat: &Option<syntax_tree::Expr>) -> String {
        if let Some(ref expr) = return_stat {
            let expr = self.convert_expression(expr);
            format!("return {};", expr)
        } else {
            String::from("return;")
        }
    }

    fn convert_read_stat(&self, read: &syntax_tree::IdList) -> String {
        let mut read_stats = Vec::new();
        for id in read {
            let c_id = convert_id(ID_HEADER, id);
            let stat = convert_read_stat(&c_id, self.var_cache.lookup(id));
            read_stats.push(stat);
        }
        read_stats.join("\n")
    }

    fn convert_write_stat(&self, write: &syntax_tree::WriteStat) -> String {
        match write {
            syntax_tree::WriteStat::Write(expr) => self.generate_printf(expr),
            syntax_tree::WriteStat::WriteLine(expr) => {
                let printf = self.generate_printf(expr);
                format!(r"{} putchar('\n');", printf)
            }
        }
    }

    fn generate_printf(&self, expr_list: &syntax_tree::ExprList) -> String {
        if expr_list.len() == 0 {
            String::new()
        } else {
            let spec_string = self.generate_printf_specifier(expr_list);
            let expr_code = self.convert_expression_list(expr_list);

            format!(r#"printf("{}", {});"#, spec_string, expr_code)
        }
    }

    fn generate_printf_specifier(&self, expr_list: &syntax_tree::ExprList) -> String {
        //each printf specifier requires 2 characters plus one spece
        let mut output = String::with_capacity(expr_list.len() * 3);
        for expr in expr_list {
            let tmp = format!(
                "{} ",
                printf_type_specifier(expr.kind.borrow().as_ref().unwrap())
            );
            output.push_str(&tmp);
        }

        output
    }

    fn convert_func_call(&self, f_call: &syntax_tree::FuncCall) -> String {
        format!(
            "{}({})",
            convert_id(ID_HEADER, &f_call.id),
            self.convert_expression_list(&f_call.args)
        )
    }

    fn convert_break_stat(&self) -> String {
        String::from("break;")
    }

    fn convert_expression_list(&self, expr_list: &syntax_tree::ExprList) -> String {
        join_list(expr_list, |e| self.convert_expression(e))
    }

    fn convert_expression(&self, expr: &syntax_tree::Expr) -> String {
        match &expr.expr {
            syntax_tree::ExprTree::Node(left, op, right) => format!(
                "{} {} {}",
                self.convert_expression(&left),
                convert_to_c_operator(&op),
                self.convert_expression(&right)
            ),
            syntax_tree::ExprTree::Factor(fact) => self.convert_factor(fact),
        }
    }

    fn convert_factor(&self, fact: &syntax_tree::Factor) -> String {
        match fact {
            syntax_tree::Factor::Id(id) => convert_id(ID_HEADER, id),
            syntax_tree::Factor::UnaryOp(unary) => self.convert_unary_operator(unary),
            syntax_tree::Factor::CondExpr(cond) => self.convert_cond_expr(cond),
            syntax_tree::Factor::CastExpr(cast) => self.convert_cast_expr(cast),
            syntax_tree::Factor::FuncCall(f_call) => self.convert_func_call(f_call),
            syntax_tree::Factor::Const(val) => self.convert_const(val),
            syntax_tree::Factor::HighPrecedence(expr) => {
                format!("({})", self.convert_expression(expr))
            }
        }
    }

    fn convert_const(&self, cons_val: &syntax_tree::Const) -> String {
        match cons_val {
            syntax_tree::Const::BoolConst(b) => {
                if *b {
                    format!("TRUE")
                } else {
                    format!("FALSE")
                }
            }
            syntax_tree::Const::IntConst(i) => format!("{}", i),
            syntax_tree::Const::RealConst(r) => format!("{}", r),
            syntax_tree::Const::StrConst(s) => format!(r#""{}""#, s),
        }
    }

    fn convert_unary_operator(&self, unary: &syntax_tree::UnaryOp) -> String {
        match unary {
            syntax_tree::UnaryOp::Minus(fact) => format!("-{}", self.convert_factor(fact)),
            syntax_tree::UnaryOp::Negate(fact) => format!("!{}", self.convert_factor(fact)),
        }
    }

    fn convert_cast_expr(&self, cast: &syntax_tree::CastExpr) -> String {
        match cast {
            syntax_tree::CastExpr::Integer(expr) => {
                format!("(int)({})", self.convert_expression(expr))
            }
            syntax_tree::CastExpr::Real(expr) => {
                format!("(double)({})", self.convert_expression(expr))
            }
        }
    }

    fn convert_cond_expr(&self, cond: &syntax_tree::CondExpr) -> String {
        let true_stat = self.convert_expression(&cond.true_stat);
        let false_stat = self.convert_expression(&cond.false_stat);
        let cond = self.convert_expression(&cond.cond);
        format!("{} ? {} : {}", cond, true_stat, false_stat)
    }
}

impl<'a> CodeGenerator<'a> for CSourceGenerator<'a> {
    fn gen_function(&mut self, func: &'a syntax_tree::FuncDecl) {
        let signature = make_function_signature(func);
        self.buff.push(signature);
        self.open_block();
        self.gen_variables(&func.vars, Scope::Local);
        self.gen_block(&func.body, BlockType::General);
        self.close_block();
        self.var_cache.clear_local_vars();
    }

    fn gen_block(&mut self, block: &syntax_tree::StatList, block_type: BlockType) {
        let code = self.convert_block(block);

        let code = match block_type {
            BlockType::General => code,
            BlockType::Main => format!(
                "int main(){{\n_initialize();\n{}\n_finalize();\nreturn 0;\n}}",
                code
            ),
        };

        self.buff.push(code);
    }

    fn gen_variables(&mut self, var_decl_list: &'a syntax_tree::VarDeclList, scope: Scope) {
        for var_decl in var_decl_list {
            let type_names = convert_to_c_types(&var_decl.kind);
            let names = join_list(&var_decl.id_list, |id| convert_id(ID_HEADER, id));
            let code = format!("{} {};", type_names, names);
            self.buff.push(code);
        }
        match scope {
            Scope::Global => self.var_cache.cache_global_vars(var_decl_list),
            Scope::Local => self.var_cache.cache_local_vars(var_decl_list),
        }
    }
    fn get_result(self) -> Vec<u8> {
        let code = self.buff.join("\n");
        let program = format!("{}\n\n{}", HEADER, code);
        program.into_bytes()
    }
}

fn make_function_signature(f_decl: &syntax_tree::FuncDecl) -> String {
    let type_name = convert_to_c_types(&f_decl.kind);
    let params = convert_param_list(&f_decl.params);
    let id = convert_id(ID_HEADER, &f_decl.id);
    format!("{} {}({})", type_name, id, params)
}

fn convert_param_list(params: &syntax_tree::ParamList) -> String {
    join_list(params, |p| convert_param(p))
}

fn convert_param(param: &syntax_tree::ParamDecl) -> String {
    let tn = convert_to_c_types(&param.kind);
    let param_id = convert_id(ID_HEADER, &param.id);
    format!("{} {}", tn, param_id)
}

fn convert_to_c_types(k: &syntax_tree::Kind) -> &'static str {
    match k {
        syntax_tree::Kind::Bool => "char",
        syntax_tree::Kind::Int => "int",
        syntax_tree::Kind::Real => "double",
        syntax_tree::Kind::Str => "char*",
        syntax_tree::Kind::Void => "void",
    }
}

fn convert_to_c_operator(op: &syntax_tree::Operator) -> &'static str {
    match op {
        syntax_tree::Operator::Equal => "==",
        syntax_tree::Operator::NotEqual => "!=",
        syntax_tree::Operator::Greater => ">",
        syntax_tree::Operator::GreaterEqual => ">=",
        syntax_tree::Operator::Less => "<",
        syntax_tree::Operator::LessEqual => "<=",
        syntax_tree::Operator::Add => "+",
        syntax_tree::Operator::Sub => "-",
        syntax_tree::Operator::Mul => "*",
        syntax_tree::Operator::Div => "/",
        syntax_tree::Operator::And => "&&",
        syntax_tree::Operator::Or => "||",
    }
}

fn printf_type_specifier(kind: &syntax_tree::Kind) -> &'static str {
    match kind {
        syntax_tree::Kind::Bool => "%c",
        syntax_tree::Kind::Int => "%d",
        syntax_tree::Kind::Real => "%f",
        syntax_tree::Kind::Str => "%s",
        syntax_tree::Kind::Void => panic!(),
    }
}

fn convert_read_stat(id: &str, kind: &syntax_tree::Kind) -> String {
    match kind {
        syntax_tree::Kind::Bool => format!("{} = _read_bool();", id),
        syntax_tree::Kind::Int => format!("{} = _read_int();", id),
        syntax_tree::Kind::Real => format!("{} = _read_double();", id),
        syntax_tree::Kind::Str => format!("_read_str({});", id),
        syntax_tree::Kind::Void => panic!(),
    }
}

fn convert_id(head: &str, id: &str) -> String {
    format!("{}{}", head, id)
}

fn join_list<T, F>(list: &[T], convert: F) -> String
where
    F: Fn(&T) -> String,
{
    list.iter()
        .map(|i| convert(i))
        .fold(String::new(), |acc, curr| {
            if acc.len() > 0 {
                format!("{}, {}", acc, curr)
            } else {
                curr.to_owned()
            }
        })
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_join_list() {
        let list = [2, 3, 4, 5, 6];

        let join = join_list(&list, |n| format!("{}", n));
        assert_eq!(join, "2, 3, 4, 5, 6");
    }
}
