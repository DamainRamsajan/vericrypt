#[test]
fn test_parse_dora() {
    let source = include_str!("../../vericrypt/src/compliance/axioms/dora.asl");
    println!("Source: {} bytes", source.len());
    let tokens = seedc::lexer::tokenize(source).unwrap();
    println!("Tokenized: {} tokens", tokens.len());
    let cst = seedc::parser::parse(&tokens).unwrap();
    println!("Parsed successfully");
}
