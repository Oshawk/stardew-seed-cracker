use std::cmp::min;
use web_sys::{console, window};
use yew::html::Scope;
use yew::prelude::*;
use yew_agent::worker::WorkerBridge;
use yew_agent::Spawnable;

use crate::agent::{Agent, AgentInput, AgentOutput, AgentStart, PROGRESS_INCREMENT};
use crate::date_component::DateComponent;
use crate::disambiguation::{disambiguate, DisambiguationResult};
use crate::item_component::ItemComponent;
use crate::platform_component::PlatformComponent;
use crate::traveling_merchant::{Item, Platform, TravelingMerchant, STOCK_QUANTITY};

/// Seconds between Unix epoch (1970-01-01) and 2012-06-22.
const EPOCH_TO_STARDEW: u64 = 1340323200;

/// 48-hour buffer in seconds for timezone differences.
const TIME_BUFFER: u64 = 172800;

enum SeedStatus {
    NotRun,
    NotFound,
    Found {
        k: u64,
        disambiguation: Option<DisambiguationResult>,
    },
}

pub enum Message {
    PlatformUpdate(Option<Platform>),
    DateUpdate(Option<i32>),
    ItemUpdate(usize, Option<Item>),
    Run,
    AgentOutput(u8, AgentOutput),
}

pub struct App {
    platform: Option<Platform>,
    date: Option<i32>,
    stock: [Option<Item>; STOCK_QUANTITY],
    workers: Vec<WorkerBridge<Agent>>,
    running: u8,
    progress: u64,
    progress_max: u64,
    seed_status: SeedStatus,
}

fn compute_max_k() -> u64 {
    let now_secs = (js_sys::Date::now() / 1000.0) as u64;
    let max_uid = now_secs - EPOCH_TO_STARDEW + TIME_BUFFER;
    max_uid / 2
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let worker_count: u8 = match window() {
            Some(w) => w.navigator().hardware_concurrency() as u8,
            None => 4u8,
        };

        let workers: Vec<WorkerBridge<Agent>> = (0u8..worker_count)
            .map(|index| {
                let link: Scope<App> = ctx.link().clone();
                Agent::spawner()
                    .callback(move |output: AgentOutput| {
                        link.send_message(Message::AgentOutput(index, output))
                    })
                    .spawn("agent.js")
            })
            .collect();

        Self {
            platform: None,
            date: None,
            stock: [None; STOCK_QUANTITY],
            workers,
            running: 0u8,
            progress: 0u64,
            progress_max: 0u64,
            seed_status: SeedStatus::NotRun,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::PlatformUpdate(platform) => {
                let last_run_enabled = self.run_enabled();
                self.platform = platform;
                last_run_enabled != self.run_enabled()
            }
            Message::DateUpdate(date) => {
                let last_run_enabled = self.run_enabled();
                self.date = date;
                last_run_enabled != self.run_enabled()
            }
            Message::ItemUpdate(index, item) => {
                let last_run_enabled = self.run_enabled();
                self.stock[index] = item;
                last_run_enabled != self.run_enabled()
            }
            Message::Run => {
                if self.running != 0u8 {
                    return false;
                }

                let stock = [
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
                ];

                let merchant = TravelingMerchant::new(self.platform.unwrap(), stock);
                let days_played = self.date.unwrap();
                let max_k = compute_max_k();
                let stride = self.workers.len() as u32;

                self.progress_max = max_k + 1;

                for (index, worker) in self.workers.iter_mut().enumerate() {
                    worker.send(AgentInput::Start(AgentStart {
                        start: index as u64,
                        stride,
                        days_played,
                        merchant: merchant.clone(),
                        max_k,
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
                AgentOutput::KFound(k) => {
                    self.running -= 1u8;
                    self.progress = self.progress_max;

                    let disambiguation =
                        disambiguate(k, self.date.unwrap(), self.platform.unwrap());

                    self.seed_status = SeedStatus::Found { k, disambiguation };
                    true
                }
                AgentOutput::NotFound => {
                    self.running -= 1u8;
                    if self.running == 0u8 {
                        if matches!(self.seed_status, SeedStatus::NotRun) {
                            self.progress = self.progress_max;
                            self.seed_status = SeedStatus::NotFound;
                        }
                        true
                    } else {
                        false
                    }
                }
                AgentOutput::Progress => {
                    self.progress = min(self.progress + PROGRESS_INCREMENT, self.progress_max);
                    match self.seed_status {
                        SeedStatus::NotRun => {
                            self.workers
                                .get_mut(index as usize)
                                .unwrap()
                                .send(AgentInput::Continue);
                        }
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
                <h1 class="title has-text-centered">{ "Stardew Seed Cracker" }</h1>
                <h2 class="subtitle has-text-centered">{ "Enter the platform, date and first 10 items from the travelling cart (order matters)." }</h2>
                <div class="container">
                    <div class="columns">
                        <div class="column">
                            <PlatformComponent callback={ ctx.link().callback(Message::PlatformUpdate) }/>
                        </div>
                        <div class="column">
                            <DateComponent callback={ ctx.link().callback(Message::DateUpdate) }/>
                        </div>
                    </div>
                    { for (0..STOCK_QUANTITY).map(|index| html! {
                        <ItemComponent index={ index } callback={ ctx.link().callback(|(index, item)| Message::ItemUpdate(index, item)) }/>
                    }) }
                    <button class="button is-primary is-fullwidth mb-3" disabled={ !self.run_enabled() } onclick={ ctx.link().callback(|_| Message::Run) }>{ "Go" }</button>
                    <progress class="progress is-primary" value={ self.progress.to_string() } max={ self.progress_max.to_string() }>{ format!("{}/{}", self.progress, self.progress_max) }</progress>
                    {
                        match &self.seed_status {
                            SeedStatus::NotRun => html!(),
                            SeedStatus::NotFound => html! {
                                <h1 class="title has-text-centered">{ "Seed Not Found" }</h1>
                            },
                            SeedStatus::Found { k, disambiguation } => {
                                let uid_even = 2 * k;
                                let uid_odd = 2 * k + 1;
                                html! {
                                    <>
                                        <h1 class="title has-text-centered">{ "Success" }</h1>
                                        <h2 class="subtitle has-text-centered">
                                            {
                                                match disambiguation {
                                                    Some(d) => {
                                                        html! {
                                                            <>
                                                                { format!(
                                                                    "If there is a {} in stock on the {} of {} Year {}, the seed is {}.",
                                                                    d.item_name,
                                                                    ordinal(d.day_of_month),
                                                                    d.season_name(),
                                                                    d.year,
                                                                    d.present_uid
                                                                ) }
                                                                <br />
                                                                { format!(
                                                                    "Othewise it is {}.",
                                                                    d.absent_uid
                                                                ) }
                                                            </>
                                                        }
                                                    },
                                                    None => {
                                                        html!(
                                                            <>
                                                                { format!(
                                                                    "Unable to disambiguate. The seed is either {} or {}.",
                                                                    uid_even,
                                                                    uid_odd
                                                                ) }
                                                            </>
                                                        )
                                                    }
                                                }
                                            }
                                        </h2>
                                    </>
                                }
                            },
                        }
                    }
                </div>
            </section>
        }
    }
}

impl App {
    fn run_enabled(&self) -> bool {
        self.platform.is_some()
            && self.date.is_some()
            && self.stock.iter().all(|item| item.is_some())
            && self.running == 0u8
    }
}

fn ordinal(n: i32) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, 11) => "th",
        (2, 12) => "th",
        (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{}{}", n, suffix)
}
