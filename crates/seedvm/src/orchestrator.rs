//! Goal completion orchestrator — plan, execute, verify, repair, escalate.
//!
//! Based on:
//!   - Self‑Healing Router (Bholani, Mar 2026) — 93% reduction in control‑plane
//!     LLM calls via Dijkstra‑based tool routing with automatic recovery
//!   - zeph‑orchestration (lib.rs, Apr 2026) — DAG‑based task scheduling with
//!     failure propagation, LLM planning, and SQLite persistence
//!   - AgenticPlanning (lib.rs, Mar 2026) — living intention graphs
//!   - VMAO (arXiv:2603.11445, Mar 2026) — verified multi‑agent orchestration

use std::collections::{HashMap, VecDeque};
use crate::value::Value;

// ── Goal and Task ──

/// A high‑level goal to be achieved.
#[derive(Debug, Clone)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub completion_criteria: String,
    pub priority: GoalPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalPriority { Low, Medium, High, Critical }

/// A sub‑task decomposed from a goal.
#[derive(Debug, Clone)]
pub struct SubTask {
    pub id: String,
    pub goal_id: String,
    pub description: String,
    pub assigned_agent: Option<String>,
    pub status: TaskStatusEnum,
    pub dependencies: Vec<String>,
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatusEnum {
    Pending,
    Ready,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

// ── Planner ──

/// Goal decomposition planner.
pub struct Planner {
    /// LLM provider name for planning.
    pub provider: Option<String>,
    /// Maximum token budget for decomposition.
    pub max_tokens: u32,
    /// Cached plan templates for repeated goals.
    pub plan_cache: HashMap<String, Vec<SubTask>>,
}

impl Planner {
    pub fn new() -> Self { Self { provider: None, max_tokens: 4096, plan_cache: HashMap::new() } }

    /// Decompose a goal into sub‑tasks.
    pub fn decompose(&mut self, goal: &Goal) -> Vec<SubTask> {
        // Check cache first (keyed by normalized goal description)
        let key = goal.description.to_lowercase();
        if let Some(cached) = self.plan_cache.get(&key) {
            return cached.clone();
        }

        // Simple heuristic decomposition (production uses LLM via InferenceEngine)
        let tasks = vec![
            SubTask {
                id: format!("{}-1", goal.id), goal_id: goal.id.clone(),
                description: format!("Analyze: {}", goal.description),
                assigned_agent: None, status: TaskStatusEnum::Pending,
                dependencies: vec![], result: None,
            },
            SubTask {
                id: format!("{}-2", goal.id), goal_id: goal.id.clone(),
                description: format!("Execute: {}", goal.description),
                assigned_agent: None, status: TaskStatusEnum::Pending,
                dependencies: vec![format!("{}-1", goal.id)], result: None,
            },
            SubTask {
                id: format!("{}-3", goal.id), goal_id: goal.id.clone(),
                description: format!("Verify: {}", goal.completion_criteria),
                assigned_agent: None, status: TaskStatusEnum::Pending,
                dependencies: vec![format!("{}-2", goal.id)], result: None,
            },
        ];

        self.plan_cache.insert(key, tasks.clone());
        tasks
    }
}

// ── Goal verifier ──

pub struct GoalVerifier {
    pub completion_checks: Vec<String>,
}

impl GoalVerifier {
    pub fn new() -> Self { Self { completion_checks: Vec::new() } }

    /// Verify that a goal's completion criteria are satisfied by task results.
    pub fn verify(&self, _goal: &Goal, completed_tasks: &[&SubTask]) -> bool {
        // All non‑skipped tasks must be completed
        completed_tasks.iter().all(|t| t.status == TaskStatusEnum::Completed || t.status == TaskStatusEnum::Skipped)
    }
}

// ── Repair module ──

pub struct RepairModule {
    pub max_retries: u32,
    pub retry_counts: HashMap<String, u32>,
}

impl RepairModule {
    pub fn new() -> Self { Self { max_retries: 3, retry_counts: HashMap::new() } }

    /// Detect whether a failed task can be retried.
    pub fn can_retry(&self, task: &SubTask) -> bool {
        let attempts = self.retry_counts.get(&task.id).copied().unwrap_or(0);
        attempts < self.max_retries
    }

    /// Retry a failed task — reset status and increment retry counter.
    pub fn retry(&mut self, task: &mut SubTask) {
        *self.retry_counts.entry(task.id.clone()).or_insert(0) += 1;
        task.status = TaskStatusEnum::Ready;
        task.result = None;
    }
}

// ── Escalation module ──

pub struct EscalationModule {
    pub parent_agent: Option<String>,
    pub escalation_threshold: u32,
    pub pending_escalations: VecDeque<EscalationRequest>,
}

#[derive(Debug, Clone)]
pub struct EscalationRequest {
    pub goal_id: String,
    pub task_id: String,
    pub reason: String,
    pub attempts: u32,
}

impl EscalationModule {
    pub fn new() -> Self { Self { parent_agent: None, escalation_threshold: 3, pending_escalations: VecDeque::new() } }

    /// Determine whether escalation is needed.
    pub fn should_escalate(&self, task: &SubTask, attempts: u32) -> bool {
        task.status == TaskStatusEnum::Failed && attempts >= self.escalation_threshold
    }

    /// Escalate to the parent agent (or human).
    pub fn escalate(&mut self, goal_id: &str, task_id: &str, reason: &str, attempts: u32) {
        self.pending_escalations.push_back(EscalationRequest {
            goal_id: goal_id.to_string(),
            task_id: task_id.to_string(),
            reason: reason.to_string(),
            attempts,
        });
    }
}

// ── Orchestrator ──

/// Top‑level orchestrator combining planner, verifier, repair, and escalation.
pub struct Orchestrator {
    pub planner: Planner,
    pub verifier: GoalVerifier,
    pub repair: RepairModule,
    pub escalation: EscalationModule,
    pub active_goals: HashMap<String, Goal>,
    pub tasks: HashMap<String, SubTask>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            planner: Planner::new(),
            verifier: GoalVerifier::new(),
            repair: RepairModule::new(),
            escalation: EscalationModule::new(),
            active_goals: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    /// Accept a new goal and decompose it.
    pub fn accept_goal(&mut self, goal: Goal) -> Vec<String> {
        let tasks = self.planner.decompose(&goal);
        let task_ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
        for task in tasks {
            self.tasks.insert(task.id.clone(), task);
        }
        self.active_goals.insert(goal.id.clone(), goal);
        task_ids
    }

    /// Execute one tick of the orchestration loop.
    pub fn tick(&mut self) -> Vec<OrchestrationEvent> {
        let mut events = Vec::new();

        // Find ready tasks (all dependencies completed)
        let ready_ids: Vec<String> = self.tasks.values()
            .filter(|t| t.status == TaskStatusEnum::Ready || t.status == TaskStatusEnum::Pending)
            .filter(|t| t.dependencies.iter().all(|dep| {
                self.tasks.get(dep).map_or(false, |d| d.status == TaskStatusEnum::Completed)
            }))
            .map(|t| t.id.clone())
            .collect();

        for id in ready_ids {
            if let Some(task) = self.tasks.get_mut(&id) {
                task.status = TaskStatusEnum::InProgress;
                events.push(OrchestrationEvent {
                    task_id: id.clone(),
                    kind: OrchestrationEventKind::TaskStarted,
                });
            }
        }

        // Check for failed tasks that need retry or escalation
        let failed_ids: Vec<String> = self.tasks.values()
            .filter(|t| t.status == TaskStatusEnum::Failed)
            .map(|t| t.id.clone())
            .collect();

        for id in failed_ids {
            let attempts = self.repair.retry_counts.get(&id).copied().unwrap_or(0);
            if self.escalation.should_escalate(self.tasks.get(&id).unwrap(), attempts) {
                if let Some(task) = self.tasks.get(&id) {
                    self.escalation.escalate(&task.goal_id, &id, "Max retries exceeded", attempts);
                    events.push(OrchestrationEvent {
                        task_id: id,
                        kind: OrchestrationEventKind::Escalated,
                    });
                }
            } else if let Some(task) = self.tasks.get_mut(&id) {
                self.repair.retry(task);
                events.push(OrchestrationEvent {
                    task_id: id,
                    kind: OrchestrationEventKind::Retried,
                });
            }
        }

        events
    }
}

#[derive(Debug, Clone)]
pub struct OrchestrationEvent {
    pub task_id: String,
    pub kind: OrchestrationEventKind,
}

#[derive(Debug, Clone)]
pub enum OrchestrationEventKind {
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    Retried,
    Escalated,
    GoalCompleted,
}
