#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(simpla);

pub mod syntax_tree;

#[cfg(test)]
mod tests {

    use super::*;
    use crate::simpla;
    use crate::syntax_tree::*;

    #[test]
    fn parse_test() {
        let code = r#"
        number: integer;
        func factorial(n: integer): integer
          fact: integer;
          body
            if n == 0 then
                fact = 1;
            else    
                fact = n * factorial(n - 1);
            end;
            return fact;
        end;
        
        func print_factorials(tot: integer): void
          i, f: integer;
          body
            for i=0 to tot do
                f = factorial(i);
                writeln(i, "factorial is", f);
            end;
          end;
        # a comment
        body  
            read(number);
            if number < 0 then
                writeln(number, "is not a valid number");
            else    
                print_factorials(number);
            end;
        end."#;
        let parser = simpla::ProgramParser::new();
        match parser.parse(code) {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{:?}", err),
        }
    }

    /*
        code in the following test may not
        semantically correct but it is always
        (unless specified) syntactically correct
    */

    #[test]
    fn test_if_stat() {
        let code = r#"
            body
                if a > 0 then
                    do_stuff(a, b + c);
                else
                    do_other_stuff(a * c);
                end;
            end.
        "#;

        let tree = parse_correct_code(code);
        let correct = Program::new(
            vec![],
            vec![],
            vec![Stat::IfStat(IfStat::new(
                Expr::Node(
                    Box::new(Expr::Factor(Factor::Id("a".to_owned()))),
                    Operator::Greater,
                    Box::new(Expr::Factor(Factor::Const(Const::IntConst(0)))),
                ),
                vec![Stat::FuncCall(FuncCall::new(
                    "do_stuff".to_owned(),
                    vec![
                        Expr::Factor(Factor::Id("a".to_owned())),
                        Expr::Node(
                            Box::new(Expr::Factor(Factor::Id("b".to_owned()))),
                            Operator::Add,
                            Box::new(Expr::Factor(Factor::Id("c".to_owned()))),
                        ),
                    ],
                ))],
                Some(vec![Stat::FuncCall(FuncCall::new(
                    "do_other_stuff".to_owned(),
                    vec![Expr::Node(
                        Box::new(Expr::Factor(Factor::Id("a".to_owned()))),
                        Operator::Mul,
                        Box::new(Expr::Factor(Factor::Id("c".to_owned()))),
                    )],
                ))]),
            ))],
        );
        assert_eq!(tree, correct);
    }


    #[test]
    fn test_operator_precedence() {
        let code = r#"
            body
                a = 5 + 6 * 7;
            end.
        "#;
        
        let tree = parse_correct_code(code);
        let result = Program::new(vec![], vec![], vec![
            Stat::AssignStat(
                AssignStat::new("a".to_owned(), Expr::Node(
                    Box::new(Expr::Factor(Factor::Const(Const::IntConst(5)))),
                    Operator::Add,
                    Box::new(
                        Expr::Node(
                            Box::new(Expr::Factor(Factor::Const(Const::IntConst(6)))),
                            Operator::Mul,
                            Box::new(Expr::Factor(Factor::Const(Const::IntConst(7))))
                        )
                    )
                ))
            )
        ]);

        assert_eq!(tree, result);

    }

    fn parse_correct_code(code: &str) -> syntax_tree::Program {
        let parser = simpla::ProgramParser::new();
        let result = parser.parse(code);
        match result {
            Ok(output) => output,
            Err(err) => panic!("{:?}", err),
        }
    }
}