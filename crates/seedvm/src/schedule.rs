use serde::Serialize;
use std::fmt;

/// A single entry in the schedule trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScheduleStep {
    pub step: usize,
    pub opcode: String,
    pub stack_depth: usize,
    pub description: String,
    pub inside_discharge: bool,
}

/// An append-only schedule trace.
#[derive(Debug, Clone, Serialize)]
pub struct ScheduleTrace {
    steps: Vec<ScheduleStep>,
}

impl ScheduleTrace {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn record(
        &mut self,
        opcode: impl Into<String>,
        stack_depth: usize,
        desc: impl Into<String>,
        inside_discharge: bool,
    ) {
        self.steps.push(ScheduleStep {
            step: self.steps.len(),
            opcode: opcode.into(),
            stack_depth,
            description: desc.into(),
            inside_discharge,
        });
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ScheduleStep> {
        self.steps.iter()
    }

    pub fn get(&self, index: usize) -> Option<&ScheduleStep> {
        self.steps.get(index)
    }

    pub fn compare(&self, other: &ScheduleTrace) -> bool {
        if self.steps.len() != other.steps.len() {
            return false;
        }
        self.steps
            .iter()
            .zip(other.steps.iter())
            .all(|(a, b)| a == b)
    }
}

impl fmt::Display for ScheduleTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for step in &self.steps {
            writeln!(
                f,
                "[{}] {} (stack:{}) {}",
                step.step, step.opcode, step.stack_depth, step.description
            )?;
        }
        Ok(())
    }
}
