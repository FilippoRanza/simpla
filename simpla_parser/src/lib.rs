#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(simpla);

pub mod syntax_tree;

#[cfg(test)]
mod tests {

    use super::*;
    use crate::simpla;

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
}
