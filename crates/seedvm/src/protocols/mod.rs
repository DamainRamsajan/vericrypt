//! AGENT‑SEED v15.2 protocol stacks — A2A, MCP, Cognitive Mesh, and Transport.
//!
//! A2A (Agent-to-Agent): task-oriented agent coordination, Agent Cards for
//!   capability discovery, nine-state task lifecycle, JSON‑RPC transport.
//!   Governed by the Linux Foundation, backed by Google and IBM.
//!
//! MCP (Model Context Protocol): standardises how LLM applications access
//!   external tools, data sources, and resources. Server/client architecture
//!   with typed tool schemas and lifecycle management.
//!
//! Cognitive Mesh (MMP): semantic infrastructure for multi-agent LLM systems.
//!   Four primitives — CAT7 schema, SVAF evaluation gate, inter-agent lineage,
//!   and remix storage — solve field-level acceptance, source traceability,
//!   and relevance preservation.
//!
//! Transport: underlying network layer supporting stdio, HTTP/SSE, gRPC,
//!   and WebSocket for intra‑process, inter‑process, and cross‑network
//!   agent communication.

pub mod a2a;
pub mod mcp;
pub mod mesh;
pub mod transport;

pub use a2a::A2AService;
pub use mcp::MCPServer;
pub use mesh::CognitiveMesh;
pub use transport::Transport;
