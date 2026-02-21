use zast::lexer::tokenizer::ZastLexer;

fn main() {
    let src = r#"
    10*12
"#;
    let mut lexer = ZastLexer::new(src);
    match lexer.tokenize() {
        Ok(toks) => {
            lexer.debug_tokens(toks);
        }
        Err(err) => {
            for e in err {
                println!("Error: {e}");
            }
        }
    };
}
