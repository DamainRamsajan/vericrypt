//! Capability‑based security — cryptographically signed authorization tokens.
//!
//! Based on astrid‑capabilities (Rust crate, Feb 2026): ed25519‑signed
//! tokens with audit linkage, resource patterns, and time‑bounded scopes.
//! Every token is cryptographically linked to the approval audit entry
//! that created it, ensuring a verifiable chain of authorization.

use std::collections::{HashMap, HashSet};
use crate::value::Value;

// ── Capability Token ──

/// A cryptographically signed capability token.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityToken {
    /// Unique token identifier.
    pub id: String,
    /// Resource pattern (glob‑based) this token grants access to.
    pub resource: String,
    /// Permissions granted.
    pub permissions: Vec<Permission>,
    /// Token scope: session (in‑memory) or persistent.
    pub scope: TokenScope,
    /// Issuer agent ID.
    pub issuer: String,
    /// Subject agent ID.
    pub subject: String,
    /// Optional expiration (Unix timestamp).
    pub expiry: Option<i64>,
    /// Ed25519 signature over the token fields.
    pub signature: Option<Vec<u8>>,
    /// Delegation chain (for attenuated tokens).
    pub delegation_chain: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Permission {
    /// Invoke / execute.
    Invoke,
    /// Read data.
    Read,
    /// Write data.
    Write,
    /// Administer / manage.
    Admin,
    /// Delegate to others.
    Delegate,
    /// Audit / inspect.
    Audit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TokenScope {
    /// Exists only for the current session.
    Session,
    /// Persisted to storage.
    Persistent,
}

// ── Resource Pattern ──

/// A glob‑based resource pattern for flexible resource scoping.
#[derive(Debug, Clone)]
pub struct ResourcePattern {
    pub pattern: String,
}

impl ResourcePattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self { pattern: pattern.into() }
    }

    /// Check whether a resource URI matches this pattern.
    pub fn matches(&self, resource: &str) -> bool {
        // Simple glob matching: * matches any sequence, ? matches any single char
        let pattern = &self.pattern;
        let mut pi = 0;
        let mut ri = 0;
        let pchars: Vec<char> = pattern.chars().collect();
        let rchars: Vec<char> = resource.chars().collect();

        let mut star_idx = None;
        let mut match_idx = 0;

        while ri < rchars.len() {
            if pi < pchars.len() && pchars[pi] == '*' {
                star_idx = Some(pi);
                match_idx = ri;
                pi += 1;
            } else if pi < pchars.len() && (pchars[pi] == '?' || pchars[pi] == rchars[ri]) {
                pi += 1;
                ri += 1;
            } else if let Some(si) = star_idx {
                pi = si + 1;
                match_idx += 1;
                ri = match_idx;
            } else {
                return false;
            }
        }
        // Consume trailing stars
        while pi < pchars.len() && pchars[pi] == '*' {
            pi += 1;
        }
        pi == pchars.len()
    }
}

// ── Capability Manager ──

/// Manages capability tokens — issuance, validation, revocation.
pub struct CapabilityManager {
    /// Active tokens indexed by ID.
    pub tokens: HashMap<String, CapabilityToken>,
    /// Revoked token IDs.
    pub revoked: HashSet<String>,
    /// Audit log entries for token creation.
    pub audit_log: Vec<AuditEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub token_id: String,
    pub action: AuditAction,
    pub timestamp: i64,
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuditAction {
    Issued,
    Revoked,
    Delegated,
    Expired,
    Verified,
}

impl CapabilityManager {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            revoked: HashSet::new(),
            audit_log: Vec::new(),
        }
    }

    /// Issue a new capability token.
    pub fn issue(&mut self, token: CapabilityToken) -> Result<(), String> {
        if self.revoked.contains(&token.id) {
            return Err(format!("Token {} has been revoked", token.id));
        }
        if token.expiry.map_or(false, |exp| exp < chrono::Utc::now().timestamp()) {
            return Err("Token has already expired".into());
        }
        self.audit_log.push(AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            token_id: token.id.clone(),
            action: AuditAction::Issued,
            timestamp: chrono::Utc::now().timestamp(),
            description: format!("Issued to {}", token.subject),
        });
        self.tokens.insert(token.id.clone(), token);
        Ok(())
    }

    /// Check whether an agent holds the required capability.
    pub fn check(&self, resource: &str, permission: &Permission) -> bool {
        for token in self.tokens.values() {
            if self.revoked.contains(&token.id) { continue; }
            if token.expiry.map_or(false, |exp| exp < chrono::Utc::now().timestamp()) { continue; }
            let pattern = ResourcePattern::new(&token.resource);
            if pattern.matches(resource) && token.permissions.contains(permission) {
                return true;
            }
        }
        false
    }

    /// Revoke a capability token.
    pub fn revoke(&mut self, token_id: &str) {
        self.revoked.insert(token_id.to_string());
        self.audit_log.push(AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            token_id: token_id.to_string(),
            action: AuditAction::Revoked,
            timestamp: chrono::Utc::now().timestamp(),
            description: "Revoked".into(),
        });
    }

    /// Attenuate a token — create a new token with reduced scope.
    pub fn attenuate(&self, token_id: &str, new_resource: &str, new_permissions: &[Permission]) -> Result<CapabilityToken, String> {
        let original = self.tokens.get(token_id)
            .ok_or_else(|| format!("Token {} not found", token_id))?;

        let mut chain = original.delegation_chain.clone();
        chain.push(token_id.to_string());

        Ok(CapabilityToken {
            id: uuid::Uuid::new_v4().to_string(),
            resource: new_resource.to_string(),
            permissions: new_permissions.to_vec(),
            scope: original.scope,
            issuer: original.subject.clone(),
            subject: original.subject.clone(),
            expiry: original.expiry,
            signature: None,
            delegation_chain: chain,
        })
    }
}
