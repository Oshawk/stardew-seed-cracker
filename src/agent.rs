use serde::{Deserialize, Serialize};
use yew_agent::{HandlerId, Private, Worker, WorkerLink};

use crate::traveling_merchant::TravelingMerchant;

pub const PROGRESS_INCREMENT: u64 = 2u64.pow(24u32);
pub const PROGRESS_MAX: u64 = u32::MAX as u64 + 1u64;

#[derive(Serialize, Deserialize)]
pub struct AgentStart {
    pub start: u32,
    pub add: u32,
    pub date: i32,
    pub merchant: TravelingMerchant,
}

#[derive(Serialize, Deserialize)]
pub enum AgentInput {
    Start(AgentStart),
    Continue,
}

#[derive(Serialize, Deserialize)]
pub enum AgentOutput {
    Error(String),
    SeedFound(i32),
    SeedNotFound,
    Progress,
}

pub struct Agent {
    link: WorkerLink<Self>,
    start: Option<AgentStart>,
}

impl Worker for Agent {
    type Reach = Private<Self>; // I think this is right?
    type Message = ();
    type Input = AgentInput;
    type Output = AgentOutput;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link, start: None }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Self::Input::Start(start) => {
                self.start = Some(start);
            }
            Self::Input::Continue => {}
        }

        let mut clear_start: bool = false;
        match &mut self.start {
            Some(start) => {
                for seed in (start.start..=u32::MAX)
                    .step_by(start.add as usize)
                    .take(PROGRESS_INCREMENT as usize)
                {
                    match start.merchant.seed_valid(seed as i32) {
                        Ok(seed_valid) => {
                            if seed_valid {
                                self.link
                                    .respond(id, AgentOutput::SeedFound(seed as i32 - start.date));
                                clear_start = true;
                                break;
                            }
                        }
                        Err(error) => {
                            self.link.respond(id, AgentOutput::Error(error.to_string()));
                            clear_start = true;
                            break;
                        }
                    }
                }

                if !clear_start {
                    match start
                        .start
                        .checked_add(start.add * PROGRESS_INCREMENT as u32)
                    {
                        Some(result) => {
                            start.start = result;
                            self.link.respond(id, AgentOutput::Progress);
                        }
                        None => {
                            self.link.respond(id, AgentOutput::SeedNotFound);
                            clear_start = true;
                        }
                    }
                }
            }
            None => {}
        }

        if clear_start {
            self.start = None;
        }
    }

    fn name_of_resource() -> &'static str {
        "agent.js"
    }

    fn resource_path_is_relative() -> bool {
        true
    }
}
