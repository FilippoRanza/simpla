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
                a = 5 + 6 * 7 * (8 + 9);
            end.
        "#;

        let tree = parse_correct_code(code);
        let result = Program::new(
            vec![],
            vec![],
            vec![Stat::AssignStat(AssignStat::new(
                "a".to_owned(),
                Expr::Node(
                    Box::new(Expr::Factor(Factor::Const(Const::IntConst(5)))),
                    Operator::Add,
                    Box::new(Expr::Node(
                        Box::new(Expr::Node(
                            Box::new(Expr::Factor(Factor::Const(Const::IntConst(6)))),
                            Operator::Mul,
                            Box::new(Expr::Factor(Factor::Const(Const::IntConst(7)))),
                        )),
                        Operator::Mul,
                        Box::new(Expr::Factor(Factor::HighPrecedence(Box::new(Expr::Node(
                            Box::new(Expr::Factor(Factor::Const(Const::IntConst(8)))),
                            Operator::Add,
                            Box::new(Expr::Factor(Factor::Const(Const::IntConst(9)))),
                        ))))),
                    )),
                ),
            ))],
        );

        assert_eq!(tree, result);
    }

    #[test]
    fn test_keywords() {
        // ensure that keywords get the correct precedence over ids
        let keywords = [
            "if", "func", "body", "end", "break", "then", "else", "while", "for", "do", "to",
            "return", "read", "write", "writeln", "and", "or", "not", "integer", "real", "string",
            "boolean", "void", "true", "false",
        ];
        for kw in &keywords {
            assign_keyword(kw);
        }
    }

    #[test]
    #[should_panic(expected = "Success with: a")]
    fn test_assign_keyword() {
        /*
        this test ensures that assign_keyword produces a
        correct code unless a simpla's keyword is given as
        argument
        */
        assign_keyword("a");
    }

    #[test]
    fn test_useless_brackets() {
        let code_a = r#"
            body 
                a = ((((b * next_number(45)))));
            end.
        "#;
        let code_b = r#"
            body
                a = (b * next_number(45));
            end.
        "#;

        let tree_a = parse_correct_code(code_a);
        let tree_b = parse_correct_code(code_b);

        assert_eq!(tree_a, tree_b);
    }

    #[test]
    fn test_ignore_comment() {
        // this comment contains some illegal code

        let code = r#"
            body
                #while = if ** end;
                writeln("Hello, World!");
            end.
        "#;
        parse_correct_code(code);
    }

    #[test]
    fn test_type_cast() {
        let code = r#"
            body
                a = 5.67;
                b = integer(a);
                c = function(b);
            end.
        "#;
        let tree = parse_correct_code(code);
        let correct = Program::new(
            vec![],
            vec![],
            vec![
                Stat::AssignStat(AssignStat::new(
                    "a".to_owned(),
                    Expr::Factor(Factor::Const(Const::RealConst(5.67))),
                )),
                Stat::AssignStat(AssignStat::new(
                    "b".to_owned(),
                    Expr::Factor(Factor::CastExpr(CastExpr::Integer(Box::new(Expr::Factor(
                        Factor::Id("a".to_owned()),
                    ))))),
                )),
                Stat::AssignStat(AssignStat::new(
                    "c".to_owned(),
                    Expr::Factor(Factor::FuncCall(FuncCall::new(
                        "function".to_owned(),
                        vec![Expr::Factor(Factor::Id("b".to_owned()))],
                    ))),
                )),
            ],
        );

        assert_eq!(tree, correct);
    }

    #[test]
    fn test_function_declaration() {
        let code = r#"
            n : integer;
            func do_stuff(a: integer) : void
                i, j : integer;
                c, d : string;
            body
                c = "while is a keyword";
                d = "for = 45 is illegal in simpla";
                for i = 0 to a do
                    write(c);
                    j = i;
                    while j > 0 do
                        if j / 2 > 5 then
                            write(d);
                        end;
                        j = j - 1;
                    end;
                    writeln();
                end;
            end;

            func return_five() : integer
            body
                return 5;
            end;

            body
                read(n);
                do_stuff(n);
            end.
        "#;
        let tree = parse_correct_code(code);
        let correct = Program::new(
            vec![VarDecl::new(vec!["n".to_owned()], Kind::Int)],
            vec![
                FuncDecl::new(
                    "do_stuff".to_owned(),
                    vec![ParamDecl::new("a".to_owned(), Kind::Int)],
                    Kind::Void,
                    vec![
                        VarDecl::new(vec!["i".to_owned(), "j".to_owned()], Kind::Int),
                        VarDecl::new(vec!["c".to_owned(), "d".to_owned()], Kind::Str),
                    ],
                    vec![
                        Stat::AssignStat(AssignStat::new(
                            "c".to_owned(),
                            Expr::Factor(Factor::Const(Const::StrConst(
                                "while is a keyword".to_owned(),
                            ))),
                        )),
                        Stat::AssignStat(AssignStat::new(
                            "d".to_owned(),
                            Expr::Factor(Factor::Const(Const::StrConst(
                                "for = 45 is illegal in simpla".to_owned(),
                            ))),
                        )),
                        Stat::ForStat(ForStat::new(
                            "i".to_owned(),
                            Expr::Factor(Factor::Const(Const::IntConst(0))),
                            Expr::Factor(Factor::Id("a".to_owned())),
                            vec![
                                Stat::WriteStat(WriteStat::Write(vec![Expr::Factor(Factor::Id(
                                    "c".to_owned(),
                                ))])),
                                Stat::AssignStat(AssignStat::new(
                                    "j".to_owned(),
                                    Expr::Factor(Factor::Id("i".to_owned())),
                                )),
                                Stat::WhileStat(WhileStat::new(
                                    Expr::Node(
                                        Box::new(Expr::Factor(Factor::Id("j".to_owned()))),
                                        Operator::Greater,
                                        Box::new(Expr::Factor(Factor::Const(Const::IntConst(0)))),
                                    ),
                                    vec![
                                        Stat::IfStat(IfStat::new(
                                            Expr::Node(
                                                Box::new(Expr::Node(
                                                    Box::new(Expr::Factor(Factor::Id(
                                                        "j".to_owned(),
                                                    ))),
                                                    Operator::Div,
                                                    Box::new(Expr::Factor(Factor::Const(
                                                        Const::IntConst(2),
                                                    ))),
                                                )),
                                                Operator::Greater,
                                                Box::new(Expr::Factor(Factor::Const(
                                                    Const::IntConst(5),
                                                ))),
                                            ),
                                            vec![Stat::WriteStat(WriteStat::Write(vec![
                                                Expr::Factor(Factor::Id("d".to_owned())),
                                            ]))],
                                            None,
                                        )),
                                        Stat::AssignStat(AssignStat::new(
                                            "j".to_owned(),
                                            Expr::Node(
                                                Box::new(Expr::Factor(Factor::Id("j".to_owned()))),
                                                Operator::Sub,
                                                Box::new(Expr::Factor(Factor::Const(
                                                    Const::IntConst(1),
                                                ))),
                                            ),
                                        )),
                                    ],
                                )),
                                Stat::WriteStat(WriteStat::WriteLine(vec![])),
                            ],
                        )),
                    ],
                ),
                FuncDecl::new(
                    "return_five".to_owned(),
                    vec![],
                    Kind::Int,
                    vec![],
                    vec![Stat::ReturnStat(Some(Expr::Factor(Factor::Const(
                        Const::IntConst(5),
                    ))))],
                ),
            ],
            vec![
                Stat::ReadStat(vec!["n".to_owned()]),
                Stat::FuncCall(FuncCall::new(
                    "do_stuff".to_owned(),
                    vec![Expr::Factor(Factor::Id("n".to_owned()))],
                )),
            ],
        );

        assert_eq!(tree, correct);
    }

    #[test]
    fn test_unary_operators() {
        let code = r#"
            body
                a = not b;
                c = -(5 * 6);
            end.
        "#;

        let tree = parse_correct_code(code);
    }

    #[test]
    fn test_brackets_removal() {
        /*
         like test_useless_brackets but checks
         the result of the parsing
        */
        let code = r#"
            body
                a = (b * (c + d));
            end.
        "#;

        let tree = parse_correct_code(code);
        let correct = Program::new(
            vec![],
            vec![],
            vec![Stat::AssignStat(AssignStat::new(
                "a".to_owned(),
                Expr::Factor(Factor::HighPrecedence(Box::new(Expr::Node(
                    Box::new(Expr::Factor(Factor::Id("b".to_owned()))),
                    Operator::Mul,
                    Box::new(Expr::Factor(Factor::HighPrecedence(Box::new(Expr::Node(
                        Box::new(Expr::Factor(Factor::Id("c".to_owned()))),
                        Operator::Add,
                        Box::new(Expr::Factor(Factor::Id("d".to_owned()))),
                    ))))),
                )))),
            ))],
        );
        assert_eq!(correct, tree);

        let code = r"#
            body
                a = (((((b * (((((c + d))))))))));
            end.
        #";

        let tree = parse_correct_code(code);
        assert_eq!(correct, tree);
    }

    fn assign_keyword(word: &str) {
        let code = format!(
            r#"
            body
                {} = 45;
            end.
        "#,
            word
        );

        let parser = simpla::ProgramParser::new();
        let result = parser.parse(&code);
        match result {
            Ok(_) => panic!("Success with: {}\nCode: {}", word, code),
            Err(_) => {}
        }
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
