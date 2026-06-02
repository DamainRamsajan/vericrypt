//! AGENT‑SEED v15.2 Intermediate Representation (IR).
//!
//! An SSA‑based representation with explicit control flow and effect tracking.

use miette::{Diagnostic, SourceSpan};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub type FuncId = usize;
pub type BlockId = usize;
pub type VarId = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub functions: Vec<Function>,
    pub globals: Vec<GlobalDecl>,
    pub exports: Vec<(String, FuncId)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<VarId>,
    pub return_ty: IrType,
    pub blocks: Vec<BasicBlock>,
    pub entry: BlockId,
    pub max_locals: usize,
    pub effect_set: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instrs: Vec<Instr>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Opcode {
    Const,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Not,
    Load,
    Store,
    Alloca,
    LoadLocal,
    StoreLocal,
    Call,
    CallIndirect,
    Return,
    MemLoad,
    MemStore,
    MemQuery,
    MemPromote,
    MemDecay,
    AgentSpawn,
    AgentSend,
    AgentRecv,
    Discharge,
    Perform,
    Infer,
    Observe,
    HeartbeatTick,
    HeartbeatSleep,
    DreamConsolidate,
    DreamResolve,
    DreamPrune,
    ConfidenceGate,
    ConfidenceAsk,
    CapCheck,
    CapGrant,
    CapRevoke,
    DecisionLog,
    DecisionQuery,
    PipeConnect,
    PipePush,
    PipePull,
    FederationPublish,
    FederationSubscribe,
    FederationQuery,
    CorrigibilityCheck,
    Phi,
    Nop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instr {
    pub opcode: Opcode,
    pub dest: Option<VarId>,
    pub operands: Vec<Operand>,
    #[serde(skip)]
    pub span: Option<SourceSpan>,
}

// NOTE: No Eq/Hash because of f64
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operand {
    Var(VarId),
    Int(i64),
    Float(f64),
    String(usize),
    Bool(bool),
    Type(IrType),
    Label(BlockId),
    Func(FuncId),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IrType {
    Void,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Char,
    String,
    Ptr(Box<IrType>),
    Array(Box<IrType>, usize),
    Struct(Vec<IrType>),
    Func(Vec<IrType>, Box<IrType>),
    Agent,
    Section,
    Capability,
    Unknown,
}

impl fmt::Display for IrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Terminator {
    Branch {
        cond: Operand,
        then_block: BlockId,
        else_block: BlockId,
    },
    Jump(BlockId),
    Return(Option<Operand>),
    Halt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDecl {
    pub name: String,
    pub ty: IrType,
    pub init: Option<Operand>,
    pub mutable: bool,
}

#[derive(Error, Diagnostic, Debug)]
pub enum IrError {
    #[error("use of undefined SSA variable {0}")]
    #[diagnostic(help("This variable was never defined in the current function."))]
    UndefinedVar(VarId),

    #[error("type mismatch in instruction")]
    #[diagnostic(help("Expected {expected}, got {found}."))]
    TypeMismatch {
        expected: String,
        found: String,
        span: Option<SourceSpan>,
    },

    #[error("effect violation")]
    #[diagnostic(help("{msg}"))]
    EffectViolation {
        msg: String,
        span: Option<SourceSpan>,
    },

    #[error("control flow error")]
    #[diagnostic(help("{msg}"))]
    ControlFlowError {
        msg: String,
        span: Option<SourceSpan>,
    },
}

impl Module {
    pub fn new() -> Self {
        Self {
            functions: vec![],
            globals: vec![],
            exports: vec![],
        }
    }
    pub fn add_function(&mut self, f: Function) -> FuncId {
        let id = self.functions.len();
        self.functions.push(f);
        id
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

impl Function {
    pub fn new(name: String, params: Vec<VarId>, return_ty: IrType) -> Self {
        let max_locals = params.len();
        let entry_block = BasicBlock {
            id: 0,
            instrs: vec![],
            terminator: Terminator::Halt,
        };
        Self {
            name,
            params,
            return_ty,
            blocks: vec![entry_block],
            entry: 0,
            max_locals,
            effect_set: vec![],
        }
    }
    pub fn add_block(&mut self) -> BlockId {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            id,
            instrs: vec![],
            terminator: Terminator::Halt,
        });
        id
    }
    pub fn push_instr(&mut self, block: BlockId, instr: Instr) {
        self.blocks[block].instrs.push(instr);
    }
    pub fn set_terminator(&mut self, block: BlockId, term: Terminator) {
        self.blocks[block].terminator = term;
    }
    pub fn new_var(&mut self) -> VarId {
        let v = self.max_locals;
        self.max_locals += 1;
        v
    }
}

impl Instr {
    pub fn new(opcode: Opcode, dest: Option<VarId>, operands: Vec<Operand>) -> Self {
        Self {
            opcode,
            dest,
            operands,
            span: None,
        }
    }
}

// Re‑export the verifier
pub mod verifier;
pub use verifier::verify;
