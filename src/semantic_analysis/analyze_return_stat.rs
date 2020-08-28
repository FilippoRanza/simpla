use super::semantic_error::MissingReturn;
use simpla_parser::syntax_tree;

pub fn check_full_return_cover<'a>(
    body: &'a syntax_tree::StatList,
    func_loc: &'a syntax_tree::Location,
    kind: &'a syntax_tree::Kind,
) -> Result<(), MissingReturn<'a>> {
    if let Some(last) = body.last() {
        check_return(last, func_loc, kind)
    } else {
        Ok(())
    }
}

fn check_return<'a>(
    stat: &'a syntax_tree::Stat,
    func_loc: &'a syntax_tree::Location,
    kind: &'a syntax_tree::Kind,
) -> Result<(), MissingReturn<'a>> {
    match &stat.stat {
        syntax_tree::StatType::ReturnStat(_) => Ok(()),
        syntax_tree::StatType::IfStat(if_stat) => {
            if let Some(else_part) = &if_stat.else_body {
                check_full_return_cover(&if_stat.if_body, func_loc, kind)?;
                check_full_return_cover(else_part, func_loc, kind)
            } else {
                Err(MissingReturn::new(func_loc, &stat.loc, kind))
            }
        }
        _ => Err(MissingReturn::new(func_loc, &stat.loc, kind)),
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use simpla_parser;

    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    const BASE_DIR: &'static str = "test_code";

    #[test]
    fn test_missing_return_inside_if() {
        run_error_test("missing_return_inside_if-error.simpla", 251, 256);
    }

    #[test]
    fn test_missing_return() {
        run_error_test("missing_return-error.simpla", 93, 100);
    }

    fn run_error_test(name: &str, stat_begin: usize, stat_end: usize) {
        let prog = compile_file(name);
        let func = &prog.functions[0];
        let stat = check_full_return_cover(&func.body, &func.loc, &func.kind);

        let err = stat.unwrap_err();
        assert_eq!(err.kind, &func.kind);
        assert_eq!(err.func_loc, &func.loc);
        assert_eq!(err.stat_loc.begin, stat_begin);
        assert_eq!(err.stat_loc.end, stat_end);
    }

    #[test]
    fn test_standard_return() {
        let prog = compile_file("return_correct.simpla");
        let func = &prog.functions[0];
        let stat = check_full_return_cover(&func.body, &func.loc, &func.kind);
        assert!(stat.is_ok(), "{:?}", stat);
    }

    #[test]
    fn test_return_into_if_stat() {
        let prog = compile_file("return_inside_if-correct.simpla");
        let func = &prog.functions[0];
        let stat = check_full_return_cover(&func.body, &func.loc, &func.kind);
        assert!(stat.is_ok(), "{:?}", stat);
    }

    fn compile_file(name: &str) -> syntax_tree::Program {
        let file = PathBuf::from(BASE_DIR).join(name);
        let mut file = File::open(file).unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        let parser = simpla_parser::ProgramParser::new();
        let prog = parser.parse(&code).unwrap();
        prog
    }
}
