#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(grammar);

pub mod syntax_tree;

#[cfg(test)]
mod tests {

    use super::*;
    use crate::grammar;

    #[test]
    fn parse_test() {
        let code = r#"
        numero: integer;
        func fattoriale(n: integer): integer
          fact: integer;
          body
            if n == 0 then
                fact = 1;
            else    
                fact = n * fattoriale(n - 1);
            end;
            return fact;
        end;
        
        func stampaFattoriali(tot: integer): void
          i, f: integer;
          body
            for i=0 to tot do
                f = fattoriale(i);
                writeln("Il fattoriale di ", i, "è ", f);
            end;
          end;
        # a comment
        body  
            read(numero);
            if numero < 0 then
                writeln("Il numero ", numero, "non è valido");
            else    
                stampaFattoriali(numero);
            end;
        end."#;
        let parser = grammar::ProgramParser::new();
        match parser.parse(code) {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{:?}", err)
        }  
    }

}
