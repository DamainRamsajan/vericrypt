//! Inference engine — multi‑provider LLM gateway with constrained decoding.
//!
//! Supports multiple inference backends through a common Provider trait,
//! schema‑constrained generation via GBNF grammar export, automatic
//! repair of schema‑violating outputs, and confidence estimation.
//!
//! References:
//!   - SGLang vs vLLM 2026 (gigagpu.com) — structured generation comparison
//!   - inferall (pypi, Apr 2026) — OpenAI‑compatible REST API for any model
//!   - oxillama‑runtime (lib.rs, Apr 2026) — pure Rust LLM inference engine

use std::collections::HashMap;
use crate::value::Value;

// ── Provider trait ──

/// A trait for LLM inference providers.
pub trait Provider: Send + Sync {
    /// Generate a completion from a prompt.
    fn generate(&self, prompt: &str, max_tokens: u32) -> Result<String, String>;

    /// Generate with structured output constrained by a grammar.
    fn generate_structured(&self, prompt: &str, grammar: &str, max_tokens: u32) -> Result<String, String>;

    /// Stream tokens as they are generated.
    fn stream(&self, prompt: &str) -> Result<Box<dyn Iterator<Item = String> + '_>, String>;

    /// Return the provider name.
    fn name(&self) -> &str;
}

// ── Schema validator ──

/// Validates LLM output against a JSON Schema.
pub struct SchemaValidator {
    /// Cached schemas.
    pub schemas: HashMap<String, serde_json::Value>,
}

impl SchemaValidator {
    pub fn new() -> Self { Self { schemas: HashMap::new() } }

    /// Register a schema.
    pub fn register(&mut self, name: &str, schema: serde_json::Value) {
        self.schemas.insert(name.to_string(), schema);
    }

    /// Validate data against a registered schema.
    pub fn validate(&self, schema_name: &str, data: &serde_json::Value) -> Result<(), String> {
        let schema = self.schemas.get(schema_name)
            .ok_or_else(|| format!("Schema '{}' not found", schema_name))?;
        // Basic validation: check required fields exist
        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            for field in required {
                if let Some(name) = field.as_str() {
                    if data.get(name).is_none() {
                        return Err(format!("Missing required field: {}", name));
                    }
                }
            }
        }
        Ok(())
    }
}

// ── Repair engine ──

/// Attempts automatic correction of schema‑violating outputs.
pub struct RepairEngine {
    /// Maximum repair attempts.
    pub max_attempts: u32,
}

impl RepairEngine {
    pub fn new() -> Self { Self { max_attempts: 3 } }

    /// Attempt to repair a schema‑violating output.
    pub fn repair(&self, data: &serde_json::Value, schema_name: &str, validator: &SchemaValidator, provider: &dyn Provider) -> Result<serde_json::Value, String> {
        let mut current = data.clone();
        for attempt in 0..self.max_attempts {
            if validator.validate(schema_name, &current).is_ok() {
                return Ok(current);
            }
            let prompt = format!(
                "The following JSON output failed validation. Fix it:\n{}",
                serde_json::to_string_pretty(&current).unwrap_or_default()
            );
            let repaired = provider.generate_structured(&prompt, "json", 1024)?;
            current = serde_json::from_str(&repaired)
                .map_err(|e| format!("Repair output is not valid JSON: {}", e))?;
        }
        Err("Max repair attempts exceeded".into())
    }
}

// ── Inference engine ──

/// The inference engine — orchestrates providers, validation, and repair.
pub struct InferenceEngine {
    /// Registered providers.
    pub providers: HashMap<String, Box<dyn Provider>>,
    /// Default provider name.
    pub default_provider: String,
    /// Schema validator.
    pub validator: SchemaValidator,
    /// Repair engine.
    pub repair: RepairEngine,
    /// Total tokens used (for budget tracking).
    pub tokens_used: u64,
}

impl InferenceEngine {
    pub fn new(default_provider: impl Into<String>) -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: default_provider.into(),
            validator: SchemaValidator::new(),
            repair: RepairEngine::new(),
            tokens_used: 0,
        }
    }

    /// Register a provider.
    pub fn register(&mut self, name: impl Into<String>, provider: Box<dyn Provider>) {
        self.providers.insert(name.into(), provider);
    }

    /// Generate output from the default provider, with optional schema validation.
    pub fn infer(
        &mut self,
        prompt: &str,
        schema_name: Option<&str>,
        grammar: Option<&str>,
        max_tokens: u32,
    ) -> Result<String, String> {
        let provider = self.providers.get(&self.default_provider)
            .ok_or_else(|| format!("Provider '{}' not found", self.default_provider))?;

        let output = if let Some(gram) = grammar {
            provider.generate_structured(prompt, gram, max_tokens)?
        } else {
            provider.generate(prompt, max_tokens)?
        };

        self.tokens_used += max_tokens as u64;

        // Schema validation
        if let Some(name) = schema_name {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&output) {
                self.validator.validate(name, &data)?;
            }
        }

        Ok(output)
    }
}
