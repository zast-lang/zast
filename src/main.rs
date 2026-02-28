use zast::{lexer::ZastLexer, parser::ZastParser};

fn main() {
    let src = r#"
    const x: u8 = 5;
    let x: u8 = 5;
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
