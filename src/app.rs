use std::cmp::min;
use std::rc::Rc;
use web_sys::{console, window};
use yew::html::Scope;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::agent::{Agent, AgentInput, AgentOutput, AgentStart, PROGRESS_INCREMENT, PROGRESS_MAX};

use crate::date_component::DateComponent;
use crate::item_component::ItemComponent;
use crate::traveling_merchant::{Item, TravelingMerchant, STOCK_QUANTITY};

enum SeedStatus {
    NotRun,
    NotFound,
    Found(i32),
}

pub enum Message {
    ItemUpdate(usize, Option<Item>),
    DateUpdate(Option<i32>),
    Run,
    AgentOutput(u8, AgentOutput),
}

pub struct App {
    stock: [Option<Item>; STOCK_QUANTITY],
    date: Option<i32>,
    workers: Vec<Box<dyn Bridge<Agent>>>,
    running: u8,
    progress: u64,
    seed_status: SeedStatus,
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let worker_count: u8 = match window() {
            Some(w) => w.navigator().hardware_concurrency() as u8,
            None => 4u8,
        };

        let mut workers: Vec<Box<dyn Bridge<Agent>>> = Vec::new();
        for index in 0u8..worker_count {
            let agent_callback = {
                let link: Scope<App> = ctx.link().clone();
                move |output: AgentOutput| {
                    link.send_message(Self::Message::AgentOutput(index, output))
                }
            };
            let worker: Box<dyn Bridge<Agent>> = Agent::bridge(Rc::new(agent_callback));

            workers.push(worker);
        }

        Self {
            stock: [None; 10usize],
            date: None,
            workers,
            running: 0u8,
            progress: 0u64,
            seed_status: SeedStatus::NotRun,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::ItemUpdate(index, item) => {
                let last_run_enabled: bool = self.run_enabled();
                self.stock[index] = item;
                last_run_enabled != self.run_enabled()
            }
            Message::DateUpdate(date) => {
                let last_run_enabled: bool = self.run_enabled();
                self.date = date;
                last_run_enabled != self.run_enabled()
            }
            Message::Run => {
                if self.running != 0u8 {
                    return false;
                }

                let merchant: TravelingMerchant = TravelingMerchant {
                    stock: [
                        self.stock[0].unwrap(),
                        self.stock[1].unwrap(),
                        self.stock[2].unwrap(),
                        self.stock[3].unwrap(),
                        self.stock[4].unwrap(),
                        self.stock[5].unwrap(),
                        self.stock[6].unwrap(),
                        self.stock[7].unwrap(),
                        self.stock[8].unwrap(),
                        self.stock[9].unwrap(),
                    ],
                };

                let add: u32 = self.workers.len() as u32;

                for (index, worker) in self.workers.iter_mut().enumerate() {
                    worker.send(AgentInput::Start(AgentStart {
                        start: index as u32,
                        add,
                        merchant: merchant.clone(),
                        date: self.date.unwrap(),
                    }));
                    self.running += 1u8;
                }

                self.progress = 0u64;
                self.seed_status = SeedStatus::NotRun;

                true
            }
            Message::AgentOutput(index, output) => match output {
                AgentOutput::Error(error) => {
                    console::log_2(&"Error:".into(), &error.into());
                    false
                }
                AgentOutput::SeedFound(seed) => {
                    self.running -= 1u8;
                    self.progress = PROGRESS_MAX;
                    self.seed_status = SeedStatus::Found(seed);
                    true
                }
                AgentOutput::SeedNotFound => {
                    self.running -= 1u8;
                    if self.running == 0u8 {
                        match self.seed_status {
                            SeedStatus::NotRun => {
                                self.progress = PROGRESS_MAX;
                                self.seed_status = SeedStatus::NotFound;
                            }
                            _ => {}
                        }
                        true
                    } else {
                        false
                    }
                }
                AgentOutput::Progress => {
                    self.progress += PROGRESS_INCREMENT;
                    self.progress = min(self.progress, PROGRESS_MAX);
                    match self.seed_status {
                        SeedStatus::NotRun => self
                            .workers
                            .get_mut(index as usize)
                            .unwrap()
                            .send(AgentInput::Continue),
                        _ => {
                            self.running -= 1u8;
                        }
                    }
                    true
                }
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <section class="section">
                <h1 class="title has-text-centered">{ "Stardew Valley Switch Seed Finder" }</h1>
                <h2 class="subtitle has-text-centered">{ "Enter the first 10 items from the travelling merchant (order matters) and the date." }</h2>
                <div class="container">
                    {
                        (0usize..STOCK_QUANTITY).into_iter().map(|index| {
                            html!(
                                <ItemComponent index={ index } callback={ ctx.link().callback(|(index, item)| Self::Message::ItemUpdate(index, item)) }/>
                            )
                        }).collect::<Html>()
                    }
                    <DateComponent callback={ ctx.link().callback(|date| Self::Message::DateUpdate(date)) }/>
                    <button class="button is-primary is-fullwidth mb-3" disabled={ !self.run_enabled() } onclick={ ctx.link().callback(|_| Self::Message::Run) }>{ "Go" }</button>
                    <progress class="progress is-primary" value={ self.progress.to_string() } max={ PROGRESS_MAX.to_string() }>{ format!("{}/{}", self.progress, PROGRESS_MAX) }</progress>
                    {
                        match self.seed_status {
                            SeedStatus::NotRun => {
                                html!()
                            }
                            SeedStatus::NotFound => {
                                html!(<h1 class="title has-text-centered">{ "Seed Not Found" }</h1>)
                            }
                            SeedStatus::Found(seed) => {
                                html!(
                                    <>
                                        <h1 class="title has-text-centered">{ "Seed Found" }</h1>
                                        <h2 class="subtitle has-text-centered">{ format!("{}", seed) }</h2>
                                    </>
                                )
                            }
                        }
                    }
                </div>
            </section>
        }
    }
}

impl App {
    fn run_enabled(&self) -> bool {
        !self.stock.iter().any(|item| item.is_none()) && self.date.is_some() && self.running == 0u8
    }
}
