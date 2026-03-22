use serde::{Deserialize, Serialize};
use yew_agent::worker::{HandlerId, Worker, WorkerScope};

use crate::observation::{Observation, Platform};
use crate::quest_checker::check_all;

/// 2012-06-22T00:00:00 UTC as a Unix timestamp (seconds since 1970-01-01).
pub const STARDEW_EPOCH_UNIX: u64 = 1340323200;

/// How many candidates to test per batch before yielding a progress update.
pub const PROGRESS_INCREMENT: u64 = 1 << 20; // ~1M

#[derive(Serialize, Deserialize)]
pub struct AgentStart {
    /// Starting candidate for this worker (= worker_index).
    pub start: u64,
    /// Stride (= total number of workers).
    pub add: u64,
    /// Exclusive upper bound: seconds since Stardew epoch, computed at crack time.
    pub t_max: u64,
    pub platform: Platform,
    /// Observations sorted by pass_rate() ascending (most discriminating first).
    pub observations: Vec<Observation>,
}

#[derive(Serialize, Deserialize)]
pub enum AgentInput {
    Start(AgentStart),
    Continue,
}

#[derive(Serialize, Deserialize)]
pub enum AgentOutput {
    /// Candidates found in this batch (may be empty).
    Candidates(Vec<u64>),
    /// Batch done; more work remaining. App should send Continue.
    Progress,
    /// Worker has exhausted its assigned range.
    Done,
    Error(String),
}

pub struct Agent {
    start: Option<AgentStart>,
}

impl Worker for Agent {
    type Message = ();
    type Input = AgentInput;
    type Output = AgentOutput;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        Self { start: None }
    }

    fn update(&mut self, _scope: &WorkerScope<Self>, _msg: Self::Message) {}

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
        if let AgentInput::Start(start) = msg {
            self.start = Some(start);
        }

        let mut clear_start = false;

        if let Some(start) = &mut self.start {
            let mut found: Vec<u64> = Vec::new();
            let end = start.t_max;
            let mut overflow = false;

            // Test up to PROGRESS_INCREMENT candidates starting at `start.start`,
            // stepping by `start.add`.
            let mut tested = 0u64;
            let mut current = start.start;

            while current <= end && tested < PROGRESS_INCREMENT {
                if check_all(start.platform, current, &start.observations) {
                    found.push(current);
                }
                // Advance by stride, check for overflow
                match current.checked_add(start.add) {
                    Some(next) => current = next,
                    None => {
                        // Overflow means we've wrapped past u64::MAX — range exhausted
                        overflow = true;
                        break;
                    }
                }
                tested += 1;
            }

            if overflow || current > end {
                // Exhausted our range
                scope.respond(id, AgentOutput::Candidates(found));
                scope.respond(id, AgentOutput::Done);
                clear_start = true;
            } else {
                // Batch done, more work remains
                start.start = current;
                scope.respond(id, AgentOutput::Candidates(found));
                scope.respond(id, AgentOutput::Progress);
            }
        }

        if clear_start {
            self.start = None;
        }
    }
}
