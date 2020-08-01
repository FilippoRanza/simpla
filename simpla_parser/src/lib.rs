#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(simpla);

pub mod syntax_tree;
pub use simpla::ProgramParser;

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
                Expr::new(
                    ExprTree::Node(
                        Box::new(Expr::new(
                            ExprTree::Factor(Factor::Id("a".to_owned())),
                            37,
                            38,
                        )),
                        Operator::Greater,
                        Box::new(Expr::new(
                            ExprTree::Factor(Factor::Const(Const::IntConst(0))),
                            41,
                            42,
                        )),
                    ),
                    37,
                    42,
                ),
                vec![Stat::FuncCall(FuncCall::new(
                    "do_stuff".to_owned(),
                    vec![
                        Expr::new(ExprTree::Factor(Factor::Id("a".to_owned())), 77, 78),
                        Expr::new(
                            ExprTree::Node(
                                Box::new(Expr::new(
                                    ExprTree::Factor(Factor::Id("b".to_owned())),
                                    80,
                                    81,
                                )),
                                Operator::Add,
                                Box::new(Expr::new(
                                    ExprTree::Factor(Factor::Id("c".to_owned())),
                                    84,
                                    85,
                                )),
                            ),
                            80,
                            85,
                        ),
                    ],
                    68,
                    86,
                ))],
                Some(vec![Stat::FuncCall(FuncCall::new(
                    "do_other_stuff".to_owned(),
                    vec![Expr::new(
                        ExprTree::Node(
                            Box::new(Expr::new(
                                ExprTree::Factor(Factor::Id("a".to_owned())),
                                144,
                                145,
                            )),
                            Operator::Mul,
                            Box::new(Expr::new(
                                ExprTree::Factor(Factor::Id("c".to_owned())),
                                148,
                                149,
                            )),
                        ),
                        144,
                        149,
                    )],
                    129,
                    150,
                ))]),
                34,
                171,
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
                Expr::new(
                    ExprTree::Node(
                        Box::new(Expr::new(
                            ExprTree::Factor(Factor::Const(Const::IntConst(5))),
                            38,
                            39,
                        )),
                        Operator::Add,
                        Box::new(Expr::new(
                            ExprTree::Node(
                                Box::new(Expr::new(
                                    ExprTree::Node(
                                        Box::new(Expr::new(
                                            ExprTree::Factor(Factor::Const(Const::IntConst(6))),
                                            42,
                                            43,
                                        )),
                                        Operator::Mul,
                                        Box::new(Expr::new(
                                            ExprTree::Factor(Factor::Const(Const::IntConst(7))),
                                            46,
                                            47,
                                        )),
                                    ),
                                    42,
                                    47,
                                )),
                                Operator::Mul,
                                Box::new(Expr::new(
                                    ExprTree::Factor(Factor::HighPrecedence(Box::new(Expr::new(
                                        ExprTree::Node(
                                            Box::new(Expr::new(
                                                ExprTree::Factor(Factor::Const(Const::IntConst(8))),
                                                51,
                                                52,
                                            )),
                                            Operator::Add,
                                            Box::new(Expr::new(
                                                ExprTree::Factor(Factor::Const(Const::IntConst(9))),
                                                55,
                                                56,
                                            )),
                                        ),
                                        51,
                                        56,
                                    )))),
                                    50,
                                    57,
                                )),
                            ),
                            42,
                            57,
                        )),
                    ),
                    38,
                    57,
                ),
                34,
                57,
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
        // the displacement is irrelevant to lalrpop, is used just to
        // get non terminals in the same location
        let code_a = r#"
            body 
                a = ((((b * next_number(45)))));
            end.
        "#;
        let code_b = r#"
            body
                a =  (   b * next_number(45)   );
            end.
        "#;

        let tree_a = parse_correct_code(code_a);
        let tree_b = parse_correct_code(code_b);

        assert_eq!(tree_a.global_vars, tree_b.global_vars);
        assert_eq!(tree_a.functions, tree_b.functions);

        assert_eq!(tree_a.body.len(), tree_b.body.len());
        match (&tree_a.body[0], &tree_b.body[0]) {
            (Stat::AssignStat(assign_a), Stat::AssignStat(assign_b)) => {
                assert_eq!(assign_a.id, assign_b.id);
                assert_eq!(assign_a.expr, assign_b.expr);
            }
            _ => panic!(),
        }
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
                    Expr::new(
                        ExprTree::Factor(Factor::Const(Const::RealConst(5.67))),
                        38,
                        42,
                    ),
                    34,
                    42,
                )),
                Stat::AssignStat(AssignStat::new(
                    "b".to_owned(),
                    Expr::new(
                        ExprTree::Factor(Factor::CastExpr(CastExpr::Integer(
                            Box::new(Expr::new(
                                ExprTree::Factor(Factor::Id("a".to_owned())),
                                72,
                                73,
                            )),
                        ))),
                        64,
                        74,
                    ),
                    60,
                    74,
                )),
                Stat::AssignStat(AssignStat::new(
                    "c".to_owned(),
                    Expr::new(
                        ExprTree::Factor(Factor::FuncCall(FuncCall::new(
                            "function".to_owned(),
                            vec![Expr::new(
                                ExprTree::Factor(Factor::Id("b".to_owned())),
                                105,
                                106,
                            )],
                            96,
                            107,
                        ))),
                        96,
                        107,
                    ),
                    92,
                    107,
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
            vec![VarDecl::new(vec!["n".to_owned()], Kind::Int, 13, 25)],
            vec![
                FuncDecl::new(
                    "do_stuff".to_owned(),
                    vec![ParamDecl::new("a".to_owned(), Kind::Int)],
                    Kind::Void,
                    vec![
                        VarDecl::new(vec!["i".to_owned(), "j".to_owned()], Kind::Int, 87, 102),
                        VarDecl::new(vec!["c".to_owned(), "d".to_owned()], Kind::Str, 119, 133),
                    ],
                    vec![
                        Stat::AssignStat(AssignStat::new(
                            "c".to_owned(),
                            Expr::new(
                                ExprTree::Factor(Factor::Const(Const::StrConst(
                                    "while is a keyword".to_owned(),
                                ))),
                                171,
                                191,
                            ),
                            167,
                            191,
                        )),
                        Stat::AssignStat(AssignStat::new(
                            "d".to_owned(),
                            Expr::new(
                                ExprTree::Factor(Factor::Const(Const::StrConst(
                                    "for = 45 is illegal in simpla".to_owned(),
                                ))),
                                213,
                                244,
                            ),
                            209,
                            244,
                        )),
                        Stat::ForStat(ForStat::new(
                            "i".to_owned(),
                            Expr::new(
                                ExprTree::Factor(Factor::Const(Const::IntConst(0))),
                                270,
                                271,
                            ),
                            Expr::new(ExprTree::Factor(Factor::Id("a".to_owned())), 275, 276),
                            vec![
                                Stat::WriteStat(WriteStat::Write(vec![Expr::new(
                                    ExprTree::Factor(Factor::Id("c".to_owned())),
                                    306,
                                    307,
                                )])),
                                Stat::AssignStat(AssignStat::new(
                                    "j".to_owned(),
                                    Expr::new(
                                        ExprTree::Factor(Factor::Id("i".to_owned())),
                                        334,
                                        335,
                                    ),
                                    330,
                                    335,
                                )),
                                Stat::WhileStat(WhileStat::new(
                                    Expr::new(
                                        ExprTree::Node(
                                            Box::new(Expr::new(
                                                ExprTree::Factor(Factor::Id("j".to_owned())),
                                                363,
                                                364,
                                            )),
                                            Operator::Greater,
                                            Box::new(Expr::new(
                                                ExprTree::Factor(Factor::Const(Const::IntConst(0))),
                                                367,
                                                368,
                                            )),
                                        ),
                                        363,
                                        368,
                                    ),
                                    vec![
                                        Stat::IfStat(IfStat::new(
                                            Expr::new(
                                                ExprTree::Node(
                                                    Box::new(Expr::new(
                                                        ExprTree::Node(
                                                            Box::new(Expr::new(
                                                                ExprTree::Factor(Factor::Id(
                                                                    "j".to_owned(),
                                                                )),
                                                                399,
                                                                400,
                                                            )),
                                                            Operator::Div,
                                                            Box::new(Expr::new(
                                                                ExprTree::Factor(Factor::Const(
                                                                    Const::IntConst(2),
                                                                )),
                                                                403,
                                                                404,
                                                            )),
                                                        ),
                                                        399,
                                                        404,
                                                    )),
                                                    Operator::Greater,
                                                    Box::new(Expr::new(
                                                        ExprTree::Factor(Factor::Const(
                                                            Const::IntConst(5),
                                                        )),
                                                        407,
                                                        408,
                                                    )),
                                                ),
                                                399,
                                                408,
                                            ),
                                            vec![Stat::WriteStat(WriteStat::Write(vec![
                                                Expr::new(
                                                    ExprTree::Factor(Factor::Id("d".to_owned())),
                                                    448,
                                                    449,
                                                ),
                                            ]))],
                                            None,
                                            396,
                                            479,
                                        )),
                                        Stat::AssignStat(AssignStat::new(
                                            "j".to_owned(),
                                            Expr::new(
                                                ExprTree::Node(
                                                    Box::new(Expr::new(
                                                        ExprTree::Factor(Factor::Id(
                                                            "j".to_owned(),
                                                        )),
                                                        509,
                                                        510,
                                                    )),
                                                    Operator::Sub,
                                                    Box::new(Expr::new(
                                                        ExprTree::Factor(Factor::Const(
                                                            Const::IntConst(1),
                                                        )),
                                                        513,
                                                        514,
                                                    )),
                                                ),
                                                509,
                                                514,
                                            ),
                                            505,
                                            514,
                                        )),
                                    ],
                                    357,
                                    539,
                                )),
                                Stat::WriteStat(WriteStat::WriteLine(vec![])),
                            ],
                            262,
                            591,
                        )),
                    ],
                    38,
                    609,
                ),
                FuncDecl::new(
                    "return_five".to_owned(),
                    vec![],
                    Kind::Int,
                    vec![],
                    vec![Stat::ReturnStat(Some(Expr::new(
                        ExprTree::Factor(Factor::Const(Const::IntConst(5))),
                        692,
                        693,
                    )))],
                    623,
                    711,
                ),
            ],
            vec![
                Stat::ReadStat(vec!["n".to_owned()]),
                Stat::FuncCall(FuncCall::new(
                    "do_stuff".to_owned(),
                    vec![Expr::new(
                        ExprTree::Factor(Factor::Id("n".to_owned())),
                        780,
                        781,
                    )],
                    771,
                    782,
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

        let _tree = parse_correct_code(code);
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
                Expr::new(
                    ExprTree::Factor(Factor::HighPrecedence(Box::new(Expr::new(
                        ExprTree::Node(
                            Box::new(Expr::new(
                                ExprTree::Factor(Factor::Id("b".to_owned())),
                                39,
                                40,
                            )),
                            Operator::Mul,
                            Box::new(Expr::new(
                                ExprTree::Factor(Factor::HighPrecedence(Box::new(Expr::new(
                                    ExprTree::Node(
                                        Box::new(Expr::new(
                                            ExprTree::Factor(Factor::Id("c".to_owned())),
                                            44,
                                            45,
                                        )),
                                        Operator::Add,
                                        Box::new(Expr::new(
                                            ExprTree::Factor(Factor::Id("d".to_owned())),
                                            48,
                                            49,
                                        )),
                                    ),
                                    44,
                                    49,
                                )))),
                                43,
                                50,
                            )),
                        ),
                        39,
                        50,
                    )))),
                    38,
                    51,
                ),
                34,
                51,
            ))],
        );
        assert_eq!(correct, tree);

        let correct = Program::new(
            vec![],
            vec![],
            vec![Stat::AssignStat(AssignStat::new(
                "a".to_owned(),
                Expr::new(
                    ExprTree::Factor(Factor::HighPrecedence(Box::new(Expr::new(
                        ExprTree::Node(
                            Box::new(Expr::new(
                                ExprTree::Factor(Factor::Id("b".to_owned())),
                                44,
                                45,
                            )),
                            Operator::Mul,
                            Box::new(Expr::new(
                                ExprTree::Factor(Factor::HighPrecedence(Box::new(Expr::new(
                                    ExprTree::Node(
                                        Box::new(Expr::new(
                                            ExprTree::Factor(Factor::Id("c".to_owned())),
                                            53,
                                            54,
                                        )),
                                        Operator::Add,
                                        Box::new(Expr::new(
                                            ExprTree::Factor(Factor::Id("d".to_owned())),
                                            57,
                                            58,
                                        )),
                                    ),
                                    53,
                                    58,
                                )))),
                                48,
                                63,
                            )),
                        ),
                        44,
                        63,
                    )))),
                    39,
                    68,
                ),
                35,
                68,
            ))],
        );

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
