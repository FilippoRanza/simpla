pub fn extract_error_code(code: &str, begin: usize, end: usize) -> String {
    let (lines, token) = find_wrong_code(code, begin, end);

    let block = &code[token.begin..token.end];
    let descr = match lines {
        //humans count from 1
        WrongLines::Single(line) => format!("Error on line: {}", line + 1),
        WrongLines::Multiple(from, to) => {
            format!("Error from line: {} to line: {}", from + 1, to + 1)
        }
    };

    format!("{}\n{}", descr, block)
}

struct Token {
    begin: usize,
    end: usize,
}

impl Token {
    fn new() -> Self {
        Self { begin: 0, end: 0 }
    }
    fn set_begin(&mut self, b: usize) {
        self.begin = b;
    }
    fn set_end(&mut self, e: usize) {
        self.end = e;
    }
}

enum WrongLines {
    Single(usize),
    Multiple(usize, usize),
}

fn find_wrong_code(code: &str, begin: usize, end: usize) -> (WrongLines, Token) {
    let mut curr_begin = 0;
    let mut begin_line = 0;
    let mut end_line = 0;
    let mut stat = false;
    let mut token = Token::new();
    for (n, line) in code.lines().enumerate() {
        let curr_end = curr_begin + line.len();
        if (!stat) && curr_end >= begin {
            begin_line = n;
            token.set_begin(curr_begin);
            stat = true;
        }
        if stat && curr_end >= end {
            end_line = n;
            token.set_end(curr_end);
            break;
        } else {
            curr_begin = curr_end + 1;
        }
    }

    if begin_line == end_line {
        (WrongLines::Single(begin_line), token)
    } else {
        (WrongLines::Multiple(begin_line, end_line), token)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use lazy_static::lazy_static;

    static CODE: &'static str = r#"
        Lorem ipsum dolor sit amet,
        consectetur adipisci elit, 
        sed do eiusmod tempor incidunt ut labore 
        et dolore magna aliqua. Ut enim ad minim veniam,
        quis nostrum exercitationem ullamco laboriosam,
        nisi ut aliquid ex ea commodi consequatur. 
        Duis aute irure reprehenderit in voluptate 
        velit esse cillum dolore eu fugiat nulla pariatur.
        Excepteur sint obcaecat cupiditat non proident, 
        sunt in culpa qui officia deserunt mollit anim id est laborum.
        "#;

    lazy_static! {
        // word 'nostrum' on 6th line
        static ref ONE_LINE_ERROR: (usize, usize) = (193, 200);

        //start with 'Ut' on 5th line end with 'ea' on 7th line
        static ref MULTIPLE_LINE_ERROR: (usize, usize) = (155, 265);
    }

    #[test]
    fn test_wrong_code_constructor() {
        let (lines, token) = find_wrong_code(CODE, ONE_LINE_ERROR.0, ONE_LINE_ERROR.1);
        assert!(matches!(lines, WrongLines::Single(n)
             if n == 5));

        assert_eq!(token.begin, 180);
        assert_eq!(token.end, 235);

        let (lines, token) = find_wrong_code(CODE, MULTIPLE_LINE_ERROR.0, MULTIPLE_LINE_ERROR.1);
        assert!(matches!(lines, WrongLines::Multiple(begin, end)
             if begin == 4 && end == 6));

        assert_eq!(token.begin, 123);
        assert_eq!(token.end, 287);
    }

    #[test]
    fn test_format_wrong_code() {
        let wrong_line = "        quis nostrum exercitationem ullamco laboriosam,";
        let correct_format = format!("Error on line: 6\n{}", wrong_line);
        let ans = extract_error_code(&CODE, ONE_LINE_ERROR.0, ONE_LINE_ERROR.1);
        assert_eq!(correct_format, ans);

        let wrong_lines = r#"        et dolore magna aliqua. Ut enim ad minim veniam,
        quis nostrum exercitationem ullamco laboriosam,
        nisi ut aliquid ex ea commodi consequatur. "#;
        let correct_format = format!("Error from line: 5 to line: 7\n{}", wrong_lines);
        let ans = extract_error_code(&CODE, MULTIPLE_LINE_ERROR.0, MULTIPLE_LINE_ERROR.1);
        assert_eq!(ans, correct_format);
    }
}
