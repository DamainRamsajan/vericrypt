//! Runtime value representation for the AGENT-SEED VM.
//!
//! Uses Rust's `enum` (tagged union) to represent all possible VM values.
//! This provides memory safety, niche optimisation, and efficient pattern
//! matching compared to C-style `union` approaches.

use crate::computation::Computation;
use std::fmt;
use std::rc::Rc;

// ── Value type ──

/// A runtime value in the VM.
///
/// Supports all primitive types, compound types, and agent-specific
/// references needed by the IR instruction set.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Unit / void value.
    Unit,
    /// Boolean.
    Bool(bool),
    /// 8-bit unsigned integer.
    U8(u8),
    /// 16-bit unsigned integer.
    U16(u16),
    /// 32-bit unsigned integer.
    U32(u32),
    /// 64-bit unsigned integer.
    U64(u64),
    /// 8-bit signed integer.
    I8(i8),
    /// 16-bit signed integer.
    I16(i16),
    /// 32-bit signed integer.
    I32(i32),
    /// 64-bit signed integer.
    I64(i64),
    /// 32-bit IEEE 754 float.
    F32(f32),
    /// 64-bit IEEE 754 float.
    F64(f64),
    /// Unicode character.
    Char(char),
    /// UTF-8 string (reference-counted for cheap cloning).
    String(Rc<String>),
    /// Raw byte array.
    Bytes(Vec<u8>),
    /// Array of values.
    Array(Vec<Value>),
    /// Tuple of values.
    Tuple(Vec<Value>),
    /// An agent reference (opaque handle).
    AgentHandle(u64),
    /// A section reference (opaque handle).
    SectionHandle(u64),
    /// A capability token.
    Capability(String, Vec<String>),
    /// A memory layer reference.
    MemoryRef(u8),
    /// A function reference (index into function table).
    FuncRef(usize),
    /// A block label (for structured control flow tracking).
    Label(usize),
    /// Null / none sentinel.
    Null,
    Computation(Computation),
}

// ── Display ──

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::U8(v) => write!(f, "{}u8", v),
            Value::U16(v) => write!(f, "{}u16", v),
            Value::U32(v) => write!(f, "{}u32", v),
            Value::U64(v) => write!(f, "{}u64", v),
            Value::I8(v) => write!(f, "{}i8", v),
            Value::I16(v) => write!(f, "{}i16", v),
            Value::I32(v) => write!(f, "{}i32", v),
            Value::I64(v) => write!(f, "{}i64", v),
            Value::F32(v) => write!(f, "{}f32", v),
            Value::F64(v) => write!(f, "{}f64", v),
            Value::Char(c) => write!(f, "{}", c),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bytes(b) => write!(f, "<{} bytes>", b.len()),
            Value::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Tuple(t) => {
                write!(f, "(")?;
                for (i, v) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::AgentHandle(h) => write!(f, "<agent#{}>", h),
            Value::SectionHandle(h) => write!(f, "<section#{}>", h),
            Value::Capability(id, scope) => write!(f, "cap<{}:{:?}>", id, scope),
            Value::MemoryRef(l) => write!(f, "<mem:L{}>", l),
            Value::FuncRef(i) => write!(f, "<fn#{}>", i),
            Value::Label(l) => write!(f, "<label:{}>", l),
            Value::Null => write!(f, "null"),
            Value::Computation(c) => write!(f, "{}", c),
        }
    }
}

// ── Conversions ──

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}
impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::U8(v)
    }
}
impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Value::U16(v)
    }
}
impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::U32(v)
    }
}
impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::U64(v)
    }
}
impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Value::I8(v)
    }
}
impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Value::I16(v)
    }
}
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::I32(v)
    }
}
impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::I64(v)
    }
}
impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::F32(v)
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::F64(v)
    }
}
impl From<char> for Value {
    fn from(v: char) -> Self {
        Value::Char(v)
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(Rc::new(v))
    }
}
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(Rc::new(v.to_string()))
    }
}
impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Bytes(v)
    }
}
impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}
impl From<Computation> for Value {
    fn from(c: Computation) -> Self {
        Value::Computation(c)
    }
}

// ── Type checking helpers ──

impl Value {
    /// Returns a human-readable type tag for this value.
    pub fn type_tag(&self) -> &'static str {
        match self {
            Value::Unit => "unit",
            Value::Bool(_) => "bool",
            Value::U8(_) => "u8",
            Value::U16(_) => "u16",
            Value::U32(_) => "u32",
            Value::U64(_) => "u64",
            Value::I8(_) => "i8",
            Value::I16(_) => "i16",
            Value::I32(_) => "i32",
            Value::I64(_) => "i64",
            Value::F32(_) => "f32",
            Value::F64(_) => "f64",
            Value::Char(_) => "char",
            Value::String(_) => "string",
            Value::Bytes(_) => "bytes",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::AgentHandle(_) => "agent",
            Value::SectionHandle(_) => "section",
            Value::Capability(_, _) => "capability",
            Value::MemoryRef(_) => "memory_ref",
            Value::FuncRef(_) => "func_ref",
            Value::Label(_) => "label",
            Value::Null => "null",
            Value::Computation(_) => "computation",
        }
    }

    /// Test whether this value is truthy (for branch conditions).
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null | Value::Unit => false,
            Value::I32(0) | Value::I64(0) => false,
            Value::F32(f) if *f == 0.0 => false,
            Value::F64(f) if *f == 0.0 => false,
            Value::String(s) => !s.is_empty(),
            Value::Computation(c) => c.value.is_truthy(),
            _ => true, // non-null values are truthy
        }
    }
}
