//! AGENT‑SEED v15.2 compiler frontend.
//!
//! Pipeline: source → tokens → CST → typed AST → IR → binary.
//! Diagnostic version: writes the parsed AST to `ast_dump.txt`.

#![allow(unused)]
#![allow(dead_code)]
pub mod ast;
pub mod binary;
pub mod ir;
pub mod lexer;
pub mod lowering;
pub mod parser;
pub mod sema;
pub mod token;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

// ── Unified error type ──

#[derive(Error, Diagnostic, Debug)]
pub enum CompileError {
    #[error("lexical error")]
    #[diagnostic(help("The source text contains an unexpected character or malformed token."))]
    Lex(#[from] LexError),

    #[error("syntax error")]
    #[diagnostic(help("The parser could not understand this part of the program."))]
    Parse(#[from] parser::ParseError),

    #[error("type error")]
    #[diagnostic(help("The type checker found inconsistent types."))]
    Type(#[from] sema::TypeError),

    #[error("IR verification error")]
    #[diagnostic(help("The intermediate representation failed a safety check."))]
    Ir(#[from] ir::IrError),

    #[error(transparent)]
    Other(#[from] Box<dyn Diagnostic + Send + Sync + 'static>),
}

// ── Lex error ──

#[derive(Error, Diagnostic, Debug)]
#[error("unexpected character `{ch}`")]
#[diagnostic(help("Remove or replace this character."))]
pub struct LexError {
    pub ch: char,
    #[label("here")]
    pub span: SourceSpan,
}

// ── Top‑level pipeline (with temporary AST dump) ──

/// Compile a complete source string into an `.aslb` binary.
pub fn compile(source: &str) -> Result<Vec<u8>, CompileError> {
    let tokens = lexer::tokenize(source)?;
    let cst = parser::parse(&tokens)?;

    // ── Temporary AST dump ──
    let dump = format!("{:#?}", &cst);
    std::fs::write("ast_dump.txt", &dump).ok();
    // ── End dump ──

    let typed = sema::check(cst)?;
    let ir_mod = lowering::lower(&typed);
    ir::verify(&ir_mod)?;
    let binary = binary::serialize(&ir_mod)?;
    Ok(binary)
}
