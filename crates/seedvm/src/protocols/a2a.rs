//! Agent‑to‑Agent (A2A) v1.0 Protocol — native inter‑agent communication.
//!
//! A2A was released March 12, 2026 under the Linux Foundation, donated by
//! Google with IBM ACP merged in. It defines Agent Cards for self‑describing
//! metadata, a nine‑state task lifecycle, and three transports (JSON‑RPC,
//! HTTP+JSON/REST, gRPC).
//!
//! References:
//!   - A2A v1.0 specification (a2a-protocol.org)
//!   - openspawn/openspawn#734 — production A2A architecture
//!   - Google A2A overview (atlan.com, May 2026)

use std::collections::HashMap;

// ── AgentCard ──

/// Self‑describing metadata published by every A2A‑compliant agent.
/// Served at `GET /.well-known/agent.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentCard {
    /// Unique agent identifier (DID or UUID).
    pub id: String,
    /// Human‑readable name.
    pub name: String,
    /// Description of what this agent does.
    pub description: String,
    /// Supported skills / capabilities.
    pub skills: Vec<Skill>,
    /// Supported input/output modes (text, structured, multimodal).
    pub io_modes: Vec<IoMode>,
    /// Authentication requirements.
    pub auth: Option<AuthInfo>,
    /// Agent's endpoint URL for A2A communication.
    pub endpoint: String,
    /// Protocol version.
    pub version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum IoMode {
    Text,
    Structured,
    Multimodal,
    Streaming,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthInfo {
    pub method: AuthMethod,
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthMethod {
    None,
    Hmac,
    OAuth2,
    ApiKey,
}

// ── Task lifecycle (9 states) ──

/// A2A task states — the complete lifecycle of long‑running agent work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskState {
    /// Task received, not yet accepted.
    Submitted,
    /// Task accepted, waiting for capacity.
    Pending,
    /// Agent is actively working.
    Working,
    /// Agent needs additional input from the task owner.
    InputRequired,
    /// Work complete, output ready.
    Completed,
    /// Work finished with errors.
    Failed,
    /// Task explicitly cancelled by the task owner.
    Canceled,
    /// Task rejected by the agent.
    Rejected,
    /// Task paused (e.g., for human approval).
    Paused,
}

/// A tracked A2A task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: String,
    pub context_id: Option<String>,
    pub state: TaskState,
    pub agent_id: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub artifacts: Vec<Artifact>,
    pub history: Vec<TaskEvent>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskEvent {
    pub timestamp: String,
    pub from_state: TaskState,
    pub to_state: TaskState,
    pub reason: Option<String>,
}

// ── RPC methods (11 defined by A2A v1.0) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum A2ARpcMethod {
    /// Send a message to an agent.
    #[serde(rename = "SendMessage")]
    SendMessage {
        task_id: String,
        message: serde_json::Value,
    },
    /// Retrieve a task by ID.
    #[serde(rename = "GetTask")]
    GetTask { task_id: String },
    /// List tasks matching optional filters.
    #[serde(rename = "ListTasks")]
    ListTasks {
        context_id: Option<String>,
        state: Option<TaskState>,
    },
    /// Cancel a running task.
    #[serde(rename = "CancelTask")]
    CancelTask { task_id: String },
    /// Subscribe to push notifications for a task.
    #[serde(rename = "SubscribeTask")]
    SubscribeTask { task_id: String },
    /// Unsubscribe from task notifications.
    #[serde(rename = "UnsubscribeTask")]
    UnsubscribeTask { task_id: String },
    /// Set a task's artifact.
    #[serde(rename = "SetTaskArtifact")]
    SetTaskArtifact { task_id: String, artifact: Artifact },
    /// Get a task's artifact.
    #[serde(rename = "GetTaskArtifact")]
    GetTaskArtifact {
        task_id: String,
        artifact_id: String,
    },
    /// Propose a task to be listed on the agent's public board.
    #[serde(rename = "ProposeTask")]
    ProposeTask { task: Task },
    /// Accept a proposed task.
    #[serde(rename = "AcceptTask")]
    AcceptTask { task_id: String },
    /// Reject a proposed task.
    #[serde(rename = "RejectTask")]
    RejectTask {
        task_id: String,
        reason: Option<String>,
    },
}

// ── A2A Service ──

/// The A2A service — manages agent registration, task lifecycle, and
/// inter‑agent message routing.
pub struct A2AService {
    /// This agent's card.
    pub card: AgentCard,
    /// Known peer agents, keyed by agent ID.
    pub peers: HashMap<String, AgentCard>,
    /// Active and archived tasks.
    pub tasks: HashMap<String, Task>,
    /// Webhook callbacks for push notifications.
    pub subscriptions: HashMap<String, Vec<String>>,
    /// Monotonic task ID counter.
    next_task_id: u64,
}

impl A2AService {
    pub fn new(card: AgentCard) -> Self {
        Self {
            card,
            peers: HashMap::new(),
            tasks: HashMap::new(),
            subscriptions: HashMap::new(),
            next_task_id: 0,
        }
    }

    /// Generate a new unique task ID.
    pub fn next_task_id(&mut self) -> String {
        let id = format!("task-{:08x}", self.next_task_id);
        self.next_task_id += 1;
        id
    }

    /// Register a peer agent.
    pub fn register_peer(&mut self, card: AgentCard) {
        self.peers.insert(card.id.clone(), card);
    }

    /// Create a new task and transition to Submitted.
    pub fn create_task(&mut self, agent_id: &str, input: serde_json::Value) -> String {
        let id = self.next_task_id();
        let task = Task {
            id: id.clone(),
            context_id: None,
            state: TaskState::Submitted,
            agent_id: agent_id.to_string(),
            input,
            output: None,
            artifacts: vec![],
            history: vec![TaskEvent {
                timestamp: chrono::Utc::now().to_rfc3339(),
                from_state: TaskState::Submitted,
                to_state: TaskState::Submitted,
                reason: Some("task created".into()),
            }],
        };
        self.tasks.insert(id.clone(), task);
        id
    }

    /// Transition a task to a new state.
    pub fn transition_task(&mut self, task_id: &str, new_state: TaskState, reason: Option<String>) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            let old_state = task.state;
            task.state = new_state;
            task.history.push(TaskEvent {
                timestamp: chrono::Utc::now().to_rfc3339(),
                from_state: old_state,
                to_state: new_state,
                reason,
            });
        }
    }

    /// Execute a SendMessage RPC.
    pub fn send_message(
        &mut self,
        task_id: &str,
        _message: &serde_json::Value,
    ) -> Result<(), String> {
        if let Some(task) = self.tasks.get(task_id) {
            if task.state == TaskState::Completed
                || task.state == TaskState::Failed
                || task.state == TaskState::Canceled
                || task.state == TaskState::Rejected
            {
                return Err(format!(
                    "Cannot send message to task in state {:?}",
                    task.state
                ));
            }
            self.transition_task(task_id, TaskState::Working, Some("message received".into()));
            Ok(())
        } else {
            Err(format!("Task {} not found", task_id))
        }
    }
}
