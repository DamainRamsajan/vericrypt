//! Binary serialization for AGENT‑SEED v15.2 IR modules.
//!
//! Produces the `.aslb` binary format using `bincode`.
//! The format is versioned and checksummed for integrity.

use crate::ir::Module;
use serde::{Deserialize, Serialize};

/// Magic bytes for `.aslb` files: "\0aslb"
pub const MAGIC: [u8; 4] = [0x00, b'a', b's', b'l'];
/// Current format version (major.minor).
pub const VERSION: (u32, u32) = (15, 2);

/// The header written at the beginning of every `.aslb` file.
#[derive(Serialize, Deserialize)]
struct Header {
    magic: [u8; 4],
    version_major: u32,
    version_minor: u32,
    checksum: u32, // CRC32 of the module payload
    module_size: u64,
}

/// Serialize a module to a vector of bytes.
pub fn serialize(module: &Module) -> Result<Vec<u8>, IrError> {
    let payload = bincode::serialize(module).map_err(|e| IrError::ControlFlowError {
        msg: format!("Serialization error: {}", e),
        span: None,
    })?;
    let checksum = crc32fast::hash(&payload);
    let header = Header {
        magic: MAGIC,
        version_major: VERSION.0,
        version_minor: VERSION.1,
        checksum,
        module_size: payload.len() as u64,
    };
    let mut out = Vec::new();
    out.extend(
        bincode::serialize(&header).map_err(|e| IrError::ControlFlowError {
            msg: format!("Header serialization error: {}", e),
            span: None,
        })?,
    );
    out.extend(payload);
    Ok(out)
}

/// Deserialize a module from bytes.
pub fn deserialize(data: &[u8]) -> Result<Module, IrError> {
    if data.len() < std::mem::size_of::<Header>() {
        return Err(IrError::ControlFlowError {
            msg: "File too small".into(),
            span: None,
        });
    }
    let header: Header =
        bincode::deserialize(&data[..std::mem::size_of::<Header>()]).map_err(|e| {
            IrError::ControlFlowError {
                msg: format!("Header deserialization error: {}", e),
                span: None,
            }
        })?;
    if header.magic != MAGIC {
        return Err(IrError::ControlFlowError {
            msg: "Invalid magic bytes".into(),
            span: None,
        });
    }
    if header.version_major != VERSION.0 {
        return Err(IrError::ControlFlowError {
            msg: format!("Unsupported major version: {}", header.version_major),
            span: None,
        });
    }
    let payload = &data[std::mem::size_of::<Header>()..];
    let checksum = crc32fast::hash(payload);
    if checksum != header.checksum {
        return Err(IrError::ControlFlowError {
            msg: "Checksum mismatch".into(),
            span: None,
        });
    }
    bincode::deserialize(payload).map_err(|e| IrError::ControlFlowError {
        msg: format!("Deserialization error: {}", e),
        span: None,
    })
}

// Re‑export IrError from ir.rs for convenience
use crate::ir::IrError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Function, IrType};

    #[test]
    fn test_roundtrip() {
        let mut module = Module::new();
        let f = Function::new("test".into(), vec![0], IrType::I32);
        module.add_function(f);
        let data = serialize(&module).unwrap();
        let module2 = deserialize(&data).unwrap();
        assert_eq!(module2.functions.len(), 1);
        assert_eq!(module2.functions[0].name, "test");
    }
}
