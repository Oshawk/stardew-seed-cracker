use std::cmp::min;
use web_sys::{console, window};
use yew::html::Scope;
use yew::prelude::*;
use yew_agent::worker::WorkerBridge;
use yew_agent::Spawnable;

use crate::agent::{Agent, AgentInput, AgentOutput, AgentStart, PROGRESS_INCREMENT, STARDEW_EPOCH_UNIX};
use crate::observation::{Observation, Platform, QuestContent, Season};
use crate::observation_row::{build_observation, ObservationRow, RowDisplayState};
use crate::platform_component::PlatformComponent;

enum CrackStatus {
    NotRun,
    Running,
    Done(Vec<u64>),
}

pub enum Message {
    PlatformUpdate(Option<Platform>),
    AddObservation,
    UpdateRowState(usize, RowDisplayState),
    RemoveObservation(usize),
    Crack,
    WorkerOutput(u8, AgentOutput),
}

pub struct App {
    platform: Option<Platform>,
    /// One entry per row; the row component reports its full display state here.
    row_states: Vec<RowDisplayState>,
    workers: Vec<WorkerBridge<Agent>>,
    running: u8,
    progress: u64,
    t_max: u64,
    crack_status: CrackStatus,
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let worker_count: u8 = match window() {
            Some(w) => w.navigator().hardware_concurrency().max(1.0) as u8,
            None => 4u8,
        };

        let workers: Vec<WorkerBridge<Agent>> = (0u8..worker_count)
            .map(|index| {
                let link: Scope<App> = ctx.link().clone();
                Agent::spawner()
                    .callback(move |output: AgentOutput| {
                        link.send_message(Message::WorkerOutput(index, output))
                    })
                    .spawn("agent.js")
            })
            .collect();

        Self {
            platform: None,
            row_states: vec![RowDisplayState::default()],
            workers,
            running: 0u8,
            progress: 0u64,
            t_max: 0u64,
            crack_status: CrackStatus::NotRun,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::PlatformUpdate(p) => {
                self.platform = p;
                true
            }

            Message::AddObservation => {
                self.row_states.push(next_row_state(&self.row_states));
                true
            }

            Message::UpdateRowState(index, state) => {
                if let Some(slot) = self.row_states.get_mut(index) {
                    *slot = state;
                }
                true
            }

            Message::RemoveObservation(index) => {
                if index < self.row_states.len() {
                    self.row_states.remove(index);
                }
                true
            }

            Message::Crack => {
                if self.running != 0 || !self.crack_enabled() {
                    return false;
                }

                // Compute t_max from current time
                let now_ms = js_sys::Date::now();
                let now_secs = (now_ms / 1000.0) as u64;
                self.t_max = now_secs.saturating_sub(STARDEW_EPOCH_UNIX);

                // Collect and sort observations by pass_rate ascending
                let mut sorted_obs: Vec<Observation> = self
                    .row_states
                    .iter()
                    .filter_map(|s| build_observation(s))
                    .collect();
                sorted_obs.sort_by(|a, b| {
                    a.pass_rate()
                        .partial_cmp(&b.pass_rate())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let add = self.workers.len() as u64;
                for (i, worker) in self.workers.iter_mut().enumerate() {
                    worker.send(AgentInput::Start(AgentStart {
                        start: i as u64,
                        add,
                        t_max: self.t_max,
                        platform: self.platform.unwrap(),
                        observations: sorted_obs.clone(),
                    }));
                    self.running += 1;
                }

                self.progress = 0;
                self.crack_status = CrackStatus::Running;
                true
            }

            Message::WorkerOutput(index, output) => match output {
                AgentOutput::Error(e) => {
                    console::log_2(&"Worker error:".into(), &e.into());
                    self.running = self.running.saturating_sub(1);
                    true
                }

                AgentOutput::Candidates(ids) => {
                    // Accumulate candidates. Don't touch running — that's Progress/Done's job.
                    // Workers are strided so no two workers cover the same ID.
                    if !ids.is_empty() {
                        match &mut self.crack_status {
                            CrackStatus::Running => {
                                self.crack_status = CrackStatus::Done(ids);
                            }
                            CrackStatus::Done(found) => {
                                found.extend_from_slice(&ids);
                            }
                            CrackStatus::NotRun => {}
                        }
                        return true;
                    }
                    false
                }

                AgentOutput::Progress => {
                    self.progress = min(
                        self.progress + PROGRESS_INCREMENT,
                        self.t_max,
                    );
                    // Only send Continue if we haven't already found candidates.
                    // Once Done, we let workers finish their current batch (no Continue).
                    match self.crack_status {
                        CrackStatus::Running => {
                            if let Some(worker) = self.workers.get_mut(index as usize) {
                                worker.send(AgentInput::Continue);
                            }
                        }
                        _ => {
                            self.running = self.running.saturating_sub(1);
                        }
                    }
                    true
                }

                AgentOutput::Done => {
                    self.running = self.running.saturating_sub(1);
                    if self.running == 0 {
                        if let CrackStatus::Running = self.crack_status {
                            // All workers finished — no candidates found
                            self.crack_status = CrackStatus::Done(vec![]);
                        }
                        self.progress = self.t_max;
                    }
                    true
                }
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let estimated = self.estimated_candidates();
        let has_complete_obs = self.row_states.iter().any(|s| build_observation(s).is_some());
        let all_complete = !self.row_states.is_empty()
            && self.row_states.iter().all(|s| build_observation(s).is_some());

        let confidence_text = if has_complete_obs {
            if estimated < 2.0 {
                "~1 candidate estimated — ready to crack!".to_string()
            } else {
                format!("~{:.0} candidates estimated", estimated)
            }
        } else {
            String::new()
        };

        let enabled = self.crack_enabled();
        let crack_button: Html = html! {
            <button
                class={if enabled { "button is-primary is-fullwidth mb-3" } else { "button is-fullwidth mb-3" }}
                disabled={!enabled}
                onclick={ctx.link().callback(|_| Message::Crack)}
            >
                { "Crack Seed" }
            </button>
        };

        let progress_bar: Html = match &self.crack_status {
            CrackStatus::NotRun => html! {},
            CrackStatus::Running | CrackStatus::Done(_) => {
                let max = self.t_max.max(1);
                html! {
                    <progress
                        class="progress is-primary mb-3"
                        value={self.progress.to_string()}
                        max={max.to_string()}
                    >
                        { format!("{}/{}", self.progress, max) }
                    </progress>
                }
            }
        };

        let result_html: Html = match &self.crack_status {
            CrackStatus::NotRun | CrackStatus::Running => html! {},
            CrackStatus::Done(ids) if ids.is_empty() => html! {
                <p class="has-text-centered has-text-danger">
                    { "No seed found — add more observations." }
                </p>
            },
            CrackStatus::Done(ids) => {
                let header = match ids.len() {
                    1 => "1 candidate found".to_string(),
                    n => format!("{n} candidates found — add more observations to narrow down."),
                };
                html! {
                    <div class="box">
                        <p class="has-text-centered mb-3">{ header }</p>
                        <ul style="list-style:none; padding:0; margin:0">
                            { for ids.iter().map(|id| html! {
                                <li class="has-text-centered">
                                    <code style="font-size:1.1rem">{ id.to_string() }</code>
                                </li>
                            }) }
                        </ul>
                    </div>
                }
            }
        };

        html! {
            <section class="section">
                <h1 class="title has-text-centered">{ "Stardew Seed Cracker" }</h1>
                <h2 class="subtitle has-text-centered">{ "1.6 — Notice Board Method" }</h2>
                <div class="container">

                    // Platform selector
                    <div class="columns">
                        <div class="column">
                            <PlatformComponent
                                callback={ctx.link().callback(Message::PlatformUpdate)}
                            />
                        </div>
                    </div>

                    // Observation rows
                    { for self.row_states.iter().enumerate().map(|(i, row_state)| {
                        let on_change = ctx.link().callback(move |state: RowDisplayState| {
                            Message::UpdateRowState(i, state)
                        });
                        let on_delete = ctx.link().callback(move |_: ()| {
                            Message::RemoveObservation(i)
                        });
                        html! {
                            <ObservationRow
                                key={i}
                                display_state={row_state.clone()}
                                on_change={on_change}
                                on_delete={on_delete}
                            />
                        }
                    }) }

                    // Add observation button — disabled until all existing rows are complete
                    <button
                        class="button is-light is-fullwidth mb-3"
                        disabled={!all_complete}
                        onclick={ctx.link().callback(|_| Message::AddObservation)}
                    >
                        { "+ Add Observation" }
                    </button>

                    // Confidence estimate
                    if has_complete_obs {
                        <p class="has-text-centered mb-3">{ confidence_text }</p>
                    }

                    // Crack button (shown when ready)
                    { crack_button }

                    // Progress bar
                    { progress_bar }

                    // Result
                    { result_html }

                </div>
            </section>
        }
    }
}

impl App {
    fn crack_enabled(&self) -> bool {
        self.platform.is_some()
            && !self.row_states.is_empty()
            && self.row_states.iter().all(|s| build_observation(s).is_some())
            && self.estimated_candidates() < 2.0
            && self.running == 0
    }

    fn estimated_candidates(&self) -> f64 {
        // Approximate t_max based on current time
        let now_ms = js_sys::Date::now();
        let now_secs = (now_ms / 1000.0) as u64;
        let space = now_secs.saturating_sub(STARDEW_EPOCH_UNIX) as f64;

        let obs: Vec<Observation> = self
            .row_states
            .iter()
            .filter_map(|s| build_observation(s))
            .collect();

        let estimate = obs.iter().fold(space, |acc, ob| acc * ob.pass_rate());

        // Due to id/2 truncation: consecutive seeds (2N, 2N+1) always produce identical
        // quest-type outcomes. Without a fishing or resource observation (which use the
        // full id for content matching), pairs are indistinguishable, so always ≥ 2.
        let has_distinguishing = obs.iter().any(|ob| {
            matches!(
                ob.quest_content,
                QuestContent::Fishing(_) | QuestContent::ResourceCollection(_)
            )
        });

        if has_distinguishing { estimate } else { estimate.max(2.0) }
    }
}

/// Build the initial `RowDisplayState` for a newly added row.
/// Pre-fills the date to the day after the last row that has a valid date.
fn next_row_state(existing: &[RowDisplayState]) -> RowDisplayState {
    let last_with_date = existing
        .iter()
        .rev()
        .find(|s| s.day.is_some() && s.season.is_some() && s.year.is_some());

    let Some(prev) = last_with_date else {
        return RowDisplayState::default();
    };

    let (day, season, year) = advance_one_day(
        prev.day.unwrap(),
        prev.season.unwrap(),
        prev.year.unwrap(),
    );

    RowDisplayState {
        day_value: day.to_string(),
        day: Some(day),
        season: Some(season),
        year_value: year.to_string(),
        year: Some(year),
        ..RowDisplayState::default()
    }
}

fn advance_one_day(day: u8, season: Season, year: u32) -> (u8, Season, u32) {
    if day < 28 {
        (day + 1, season, year)
    } else {
        match season {
            Season::Spring => (1, Season::Summer, year),
            Season::Summer => (1, Season::Fall, year),
            Season::Fall  => (1, Season::Winter, year),
            Season::Winter => (1, Season::Spring, year + 1),
        }
    }
}
