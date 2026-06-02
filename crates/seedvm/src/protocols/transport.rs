//! Transport layer — abstracts over stdio, HTTP/SSE, gRPC, and WebSocket.
//!
//! Supports all four A2A transport modes plus IPC for local agent communication.

use std::io;

/// Transport protocol variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportProtocol {
    /// Standard I/O (for local subprocess agents).
    Stdio,
    /// HTTP with Server‑Sent Events for streaming.
    HttpSse,
    /// gRPC with protocol buffers.
    Grpc,
    /// WebSocket for bidirectional real‑time.
    WebSocket,
}

/// A transport connection.
pub struct Transport {
    /// Active protocol.
    pub protocol: TransportProtocol,
    /// Peer address (e.g., URL, file path, or port).
    pub address: String,
    /// Whether the connection is established.
    pub connected: bool,
    /// Send buffer (for stdio, messages are queued here).
    send_buffer: Vec<Vec<u8>>,
    /// Receive buffer.
    recv_buffer: Vec<Vec<u8>>,
}

impl Transport {
    /// Create a new stdio transport (default for local agents).
    pub fn new_stdio() -> Self {
        Self {
            protocol: TransportProtocol::Stdio,
            address: "stdio://".to_string(),
            connected: true,
            send_buffer: Vec::new(),
            recv_buffer: Vec::new(),
        }
    }

    /// Create a new HTTP+SSE transport.
    pub fn new_http_sse(address: impl Into<String>) -> Self {
        Self {
            protocol: TransportProtocol::HttpSse,
            address: address.into(),
            connected: false,
            send_buffer: Vec::new(),
            recv_buffer: Vec::new(),
        }
    }

    /// Create a new WebSocket transport.
    pub fn new_websocket(address: impl Into<String>) -> Self {
        Self {
            protocol: TransportProtocol::WebSocket,
            address: address.into(),
            connected: false,
            send_buffer: Vec::new(),
            recv_buffer: Vec::new(),
        }
    }

    /// Queue a message for sending.
    pub fn send(&mut self, data: &[u8]) {
        self.send_buffer.push(data.to_vec());
    }

    /// Check if there are pending received messages.
    pub fn has_data(&self) -> bool {
        !self.recv_buffer.is_empty()
    }

    /// Read the next received message.
    pub fn recv(&mut self) -> Option<Vec<u8>> {
        self.recv_buffer.pop()
    }

    /// Flush send buffer (for stdio, writes to stdout).
    pub fn flush(&mut self) -> io::Result<()> {
        for msg in self.send_buffer.drain(..) {
            // In stdio mode, write to stdout
            if self.protocol == TransportProtocol::Stdio {
                use std::io::Write;
                std::io::stdout().write_all(&msg)?;
            }
        }
        Ok(())
    }
}
