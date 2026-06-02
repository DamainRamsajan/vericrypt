//! Shared type representations for the AGENT‑SEED v15.2 semantic analyser.
//!
//! Mirrors the architecture's `Computation<T>`, `Effect`, `Interval`,
//! `TaintMeta`, `CostInterval`, `CapabilityToken`, `Decision<T>`, and `Failure`.

use std::collections::{HashMap, HashSet};

// ── Type representation (internal, richer than AST types) ──

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    /// A type variable (during inference).
    Var(usize),
    /// Primitive types.
    Prim(PrimTy),
    /// Function type: args → ret, with optional effect set.
    Fn(Vec<Ty>, Box<Ty>, Option<EffectSet>),
    /// Array type.
    Array(Box<Ty>, usize),
    /// Tuple type.
    Tuple(Vec<Ty>),
    /// Named user-defined type.
    Nominal(String, Vec<Ty>),
    /// Reference type.
    Ref(bool, Box<Ty>),
    /// A polymorphic type scheme (∀α. τ).
    Scheme(Vec<usize>, Box<Ty>),
    /// Agent type.
    Agent(String),
    /// Section type.
    Section(String),
    /// Affine (linear) type marker.
    Affine(Box<Ty>),
    /// Unknown / gradual type.
    Unknown,
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Var(i) => write!(f, "?{}", i),
            Ty::Prim(p) => write!(f, "{}", p),
            Ty::Fn(args, ret, eff) => {
                write!(f, "fn(")?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ") -> {}", ret)?;
                if let Some(e) = eff {
                    write!(f, " !{}", e)?;
                }
                Ok(())
            }
            Ty::Array(t, n) => write!(f, "[{}; {}]", t, n),
            Ty::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Ty::Nominal(n, args) => {
                write!(f, "{}", n)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", a)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Ty::Ref(mutbl, t) => write!(f, "&{}{}", if *mutbl { "mut " } else { "" }, t),
            Ty::Scheme(vars, t) => {
                write!(f, "forall ")?;
                for (i, v) in vars.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "?{}", v)?;
                }
                write!(f, ". {}", t)
            }
            Ty::Agent(n) => write!(f, "agent<{}>", n),
            Ty::Section(n) => write!(f, "section<{}>", n),
            Ty::Affine(t) => write!(f, "affine<{}>", t),
            Ty::Unknown => write!(f, "?"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimTy {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Char,
    String,
}

impl std::fmt::Display for PrimTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PrimTy::Bool => "bool",
                PrimTy::U8 => "u8",
                PrimTy::U16 => "u16",
                PrimTy::U32 => "u32",
                PrimTy::U64 => "u64",
                PrimTy::I8 => "i8",
                PrimTy::I16 => "i16",
                PrimTy::I32 => "i32",
                PrimTy::I64 => "i64",
                PrimTy::F32 => "f32",
                PrimTy::F64 => "f64",
                PrimTy::Char => "char",
                PrimTy::String => "string",
            }
        )
    }
}

// ── Effect system ──

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// No effect.
    Pure,
    /// Read from memory layer `n`.
    MemRead(u8),
    /// Write to memory layer `n`.
    MemWrite(u8),
    /// Network I/O.
    Network,
    /// File system access.
    FileIO,
    /// LLM inference call.
    Inference,
    /// Agent spawning.
    AgentSpawn,
    /// Decision logging.
    DecisionLog,
    /// Capability usage.
    Capability(String),
    /// Named custom effect.
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectSet {
    pub effects: HashSet<Effect>,
}

impl std::fmt::Display for EffectSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (i, e) in self.effects.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", e)?;
        }
        write!(f, "}}")
    }
}

impl EffectSet {
    pub fn pure() -> Self {
        Self {
            effects: HashSet::new(),
        }
    }
    pub fn singleton(e: Effect) -> Self {
        let mut set = HashSet::new();
        set.insert(e);
        Self { effects: set }
    }
    pub fn union(&self, other: &Self) -> Self {
        Self {
            effects: self.effects.union(&other.effects).cloned().collect(),
        }
    }
    pub fn contains_any(&self, other: &Self) -> bool {
        !self.effects.is_disjoint(&other.effects)
    }
}

// ── Taint system ──
// Three-level security lattice: Clean ≤ Agnostic ≤ Tainted
// Based on Tant (Bertolo, 2026) and Tant-programming-language

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaintLevel {
    Clean = 0,
    Agnostic = 1,
    Tainted = 2,
}

impl TaintLevel {
    pub fn join(self, other: Self) -> Self {
        self.max(other)
    }
    pub fn can_flow_into(self, sink: Self) -> bool {
        self <= sink
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaintMeta {
    pub level: TaintLevel,
    pub sources: Vec<String>,
}

impl TaintMeta {
    pub fn clean() -> Self {
        Self {
            level: TaintLevel::Clean,
            sources: vec![],
        }
    }
    pub fn agnostic() -> Self {
        Self {
            level: TaintLevel::Agnostic,
            sources: vec![],
        }
    }
    pub fn tainted(source: impl Into<String>) -> Self {
        Self {
            level: TaintLevel::Tainted,
            sources: vec![source.into()],
        }
    }
    pub fn join(&self, other: &Self) -> Self {
        let level = self.level.join(other.level);
        let mut sources = self.sources.clone();
        sources.extend(other.sources.clone());
        Self { level, sources }
    }
}

// ── Capability tokens ──

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityToken {
    pub id: String,
    pub scope: Vec<String>,
    pub expiry: Option<u64>,
}

// ── Computation monad (from architecture spec) ──

#[derive(Debug, Clone)]
pub struct Computation<T> {
    pub value: Option<T>,
    pub effect: EffectSet,
    pub taint: TaintMeta,
    pub capabilities: HashSet<CapabilityToken>,
}

impl<T> Computation<T> {
    pub fn pure(value: T) -> Self {
        Self {
            value: Some(value),
            effect: EffectSet::pure(),
            taint: TaintMeta::clean(),
            capabilities: HashSet::new(),
        }
    }
}

// ── Type environment ──

#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Maps variable names to their types.
    pub vars: HashMap<String, Ty>,
    /// Maps variable names to whether they are affine (consumed on use).
    pub affine_map: HashMap<String, bool>,
    /// Maps function names to their type schemes.
    pub fns: HashMap<String, Ty>,
    /// Maps type names (struct/enum/agent/section) to their definitions.
    pub types: HashMap<String, Ty>,
    /// Maps effect names to their definitions.
    pub effects: HashMap<String, Effect>,
    /// Current scope level (for variable shadowing).
    pub depth: usize,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push_scope(&mut self) {
        self.depth += 1;
    }
    pub fn pop_scope(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}
