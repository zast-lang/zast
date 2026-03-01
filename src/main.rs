use zast::{lexer::ZastLexer, parser::ZastParser, sema::ZastSemanticAnalyzer};

fn main() {
    let src = r#"
    fn main(a: i32, b: i32): void {
        
    }
"#;
    let mut lexer = ZastLexer::new(src);
    match lexer.tokenize() {
        Ok(toks) => {
            let mut parser = ZastParser::new(toks);
            match parser.parse_program() {
                Ok(ast) => {
                    let mut sema = ZastSemanticAnalyzer::new();
                    match sema.analyze(ast) {
                        Ok(()) => println!("{:#?}", sema),
                        Err(e) => e.report_all_errors(),
                    };
                }
                Err(err) => err.report_all_errors(),
            };
        }
        Err(err) => {
            err.report_all_errors();
        }
    };
}
