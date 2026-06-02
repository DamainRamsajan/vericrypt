pub mod contractck;
pub mod effectck;
pub mod nameres;
pub mod taintck;
pub mod typeck;
pub mod types;

use crate::ast::Program;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum TypeError {
    #[error("unresolved name `{name}`")]
    #[diagnostic(help("Did you forget to import or declare this name?"))]
    UnresolvedName {
        name: String,
        #[label("not found in this scope")]
        span: SourceSpan,
    },

    #[error("type mismatch")]
    #[diagnostic(help("Expected `{expected}`, found `{found}`."))]
    Mismatch {
        expected: String,
        found: String,
        #[label("expected {expected}")]
        span: SourceSpan,
        #[label("found {found}")]
        found_span: Option<SourceSpan>,
    },

    #[error("affine type violation")]
    #[diagnostic(help("Capability `{name}` was already consumed or moved."))]
    AffineViolation {
        name: String,
        #[label("used again here")]
        span: SourceSpan,
    },

    #[error("effect not discharged")]
    #[diagnostic(help("Effect `{effect}` is performed outside a discharge block."))]
    UndischargedEffect {
        effect: String,
        #[label("performed here")]
        span: SourceSpan,
    },

    #[error("taint violation")]
    #[diagnostic(help("{message}"))]
    TaintViolation {
        message: String,
        #[label("violation")]
        span: SourceSpan,
    },

    #[error("contract violation")]
    #[diagnostic(help("{message}"))]
    ContractViolation {
        message: String,
        #[label("violates contract")]
        span: SourceSpan,
    },

    #[error(transparent)]
    Other(#[from] Box<dyn Diagnostic + Send + Sync + 'static>),
}

/// Run all semantic analysis passes and return a typed AST.
pub fn check(program: Program) -> Result<Program, TypeError> {
    let program = nameres::resolve(program)?;
    let program = typeck::infer_types(program)?;
    // effectck, taintck, contractck remain identity for now
    let program = effectck::check_effects(program)?;
    let program = taintck::check_taint(program)?;
    let program = contractck::check_contracts(program)?;
    Ok(program)
}
