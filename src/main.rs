use zast::{lexer::ZastLexer, parser::ZastParser};

fn main() {
    let src = r#"
    fn main(a: **i32, b: i32,): void {}
"#;
    let mut lexer = ZastLexer::new(src);
    match lexer.tokenize() {
        Ok(toks) => {
            let mut parser = ZastParser::new(toks);
            match parser.parse_program() {
                Ok(ast) => println!("{:#?}", ast),
                Err(err) => err.report_all_errors(),
            };
        }
        Err(err) => {
            err.report_all_errors();
        }
    };
}
