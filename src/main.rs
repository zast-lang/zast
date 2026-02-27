use zast::lexer::ZastLexer;

fn main() {
    let src = r#"
    #
"#;
    let mut lexer = ZastLexer::new(src);
    match lexer.tokenize() {
        Ok(toks) => {
            lexer.debug_tokens(toks);
        }
        Err(err) => {
            err.report_all_errors();
        }
    };
}
