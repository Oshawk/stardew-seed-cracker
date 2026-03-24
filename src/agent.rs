use serde::{Deserialize, Serialize};
use yew_agent::worker::{HandlerId, Worker, WorkerScope};

use crate::traveling_merchant::TravelingMerchant;
use crate::xxhash::shop_seed;

pub const PROGRESS_INCREMENT: u64 = 2u64.pow(20);

#[derive(Serialize, Deserialize)]
pub struct AgentStart {
    pub start: u64,
    pub stride: u32,
    pub days_played: i32,
    pub merchant: TravelingMerchant,
    pub max_k: u64,
}

#[derive(Serialize, Deserialize)]
pub enum AgentInput {
    Start(AgentStart),
    Continue,
}

#[derive(Serialize, Deserialize)]
pub enum AgentOutput {
    Error(String),
    KFound(u64),
    NotFound,
    Progress,
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
            let mut k = start.start;
            let mut count = 0u64;

            while k <= start.max_k && count < PROGRESS_INCREMENT {
                let seed = shop_seed(start.days_played, k * 2);
                if start.merchant.seed_valid(seed) {
                    scope.respond(id, AgentOutput::KFound(k));
                    clear_start = true;
                    break;
                }
                k += start.stride as u64;
                count += 1;
            }

            if !clear_start {
                if k > start.max_k {
                    scope.respond(id, AgentOutput::NotFound);
                    clear_start = true;
                } else {
                    start.start = k;
                    scope.respond(id, AgentOutput::Progress);
                }
            }
        }

        if clear_start {
            self.start = None;
        }
    }
}
