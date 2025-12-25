use crate::hashi::{BridgeLine, HashiGrid, Position};
use crate::{Route, hashi};
use serde::Deserialize;
use yew::prelude::*;
use yew_hooks::use_interval;
use yew_router::prelude::*;
// use web_sys::console;
// use web_sys::wasm_bindgen::JsValue;
// console::log_1(&JsValue::from_str("game.rs loaded"));

#[derive(Clone)]
struct GameState {
    grid: HashiGrid,
    selected: Option<Position>,
    shuddered_island: Option<Position>,
    time_elapsed: u32,
    challenge_time: Option<u32>,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            grid: HashiGrid::placeholder(),
            selected: None,
            shuddered_island: None,
            time_elapsed: 0,
            challenge_time: None,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct GameProps {
    pub puzzle_id: u64,
    pub width: u8,
    pub height: u8,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct QueryParams {
    challenge_time: Option<u32>,
}

#[function_component(Game)]
pub fn game(props: &GameProps) -> Html {
    let state: UseStateHandle<GameState> = use_state(GameState::default);
    let navigator = use_navigator().unwrap();
    let puzzle_id = props.puzzle_id;
    let width = props.width;
    let height = props.height;
    let query_params = match use_location() {
        Some(loc) => match loc.query::<QueryParams>() {
            Ok(params) => params,
            Err(_) => QueryParams {
                challenge_time: None,
            },
        },
        None => QueryParams {
            challenge_time: None,
        },
    };

    {
        let state = state.clone();

        use_effect_with(puzzle_id, move |_| {
            {
                let hashi_grid = hashi::HashiGrid::generate_with_seed(width, height, puzzle_id)
                    .unwrap()
                    .wipe_bridges();

                state.set(GameState {
                    grid: hashi_grid,
                    selected: None,
                    shuddered_island: None,
                    time_elapsed: 0,
                    challenge_time: query_params.challenge_time,
                });
            }
            || ()
        });
    }

    // Timer using yew_hooks
    {
        let state = state.clone();
        use_interval(
            move || {
                let mut s = (*state).clone();
                if !s.grid.is_complete() {
                    s.time_elapsed += 1;
                    state.set(s);
                }
            },
            1000,
        );
    }

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Home);
        })
    };

    let on_new_puzzle = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Game {
                width,
                height,
                id: rand::random::<u64>(),
            });
        })
    };

    html! {
        <div class="game-wrapper">
            <div class="game-controls">
                <button onclick={on_back} class="btn btn-game-large">
                    {"← Back"}
                </button>
                <button onclick={on_new_puzzle} class="btn btn-game-large success">
                    {"🎲 Next"}
                </button>
                <div class="game-timer-container">
                    {
                        if let Some(ct) = state.challenge_time {
                            let is_beating = state.time_elapsed < ct;
                            let color_class = if is_beating { "beating" } else { "not-beating" };
                            html! {
                                <>
                                    <div class="challenge-time">
                                        {format!("Time to beat: {}", format_time(ct))}
                                    </div>
                                    <div class={format!("game-timer {}", color_class)}>
                                        {format!("Time: {}", format_time(state.time_elapsed))}
                                    </div>
                                </>
                            }
                        } else {
                            html! {
                                <div class="game-timer">
                                    {format!("Time: {}", format_time(state.time_elapsed))}
                                </div>
                            }
                        }
                    }
                </div>
            </div>
            { render_game(&state) }
        </div>
    }
}

#[function_component(RandomGameRedirect)]
fn random_game_redirect() -> Html {
    html! {
        <Redirect<Route> to={Route::Game { id: rand::random::<u64>(), width: 5, height: 10 }} />
    }
}

fn render_game(state: &UseStateHandle<GameState>) -> Html {
    let is_complete = state.grid.is_complete();

    let on_island_click = {
        let state = state.clone();
        Callback::from(move |currently_selected: hashi::Position| {
            let mut s = (*state).clone();

            match s.selected {
                None => s.selected = Some(currently_selected),
                Some(previously_selected) => {
                    if previously_selected != currently_selected {
                        // Is there a valid bridgeline between the two?

                        let proposed_bridge =
                            match hashi::BridgeLine::new(previously_selected, currently_selected) {
                                Err(_) => {
                                    // Invalid bridge (diagonal)
                                    s.shuddered_island = Some(currently_selected);
                                    s.selected = None;
                                    state.set(s.clone());

                                    // Clear shudder after 300ms
                                    let state_for_timeout = state.clone();
                                    gloo_timers::callback::Timeout::new(300, move || {
                                        let mut s = (*state_for_timeout).clone();
                                        s.shuddered_island = None;
                                        s.selected = None;
                                        state_for_timeout.set(s);
                                    })
                                    .forget();

                                    return;
                                }
                                Ok(b) => b,
                            };

                        match s.grid.add_bridge(proposed_bridge) {
                            Ok(_) => {
                                s.selected = None;
                                s.shuddered_island = None;
                            }
                            Err(_) => {
                                // Invalid bridge placement - shudder the island
                                s.shuddered_island = Some(currently_selected);
                                s.selected = None;

                                state.set(s.clone());

                                // Clear shudder after 300ms
                                let state_for_timeout = state.clone();
                                gloo_timers::callback::Timeout::new(300, move || {
                                    let mut s = (*state_for_timeout).clone();
                                    s.shuddered_island = None;
                                    s.selected = None;
                                    state_for_timeout.set(s);
                                })
                                .forget();
                            }
                        }
                    } else {
                        // Clicking the already selected island toggles it off
                        s.selected = None;
                    }
                }
            }

            state.set(s);
        })
    };

    let width = state.grid.width as i32 * 100;
    let height = state.grid.height as i32 * 100;

    html! {
        <div class="game-container">
            <svg
                viewBox={format!("-100 -100 {} {}", width + 100, height + 100)}
                preserveAspectRatio="xMidYMid meet"
                class="game-svg"
            >
                <defs>
                    <filter id="selectedGlow">
                        <feDropShadow
                            dx="0"
                            dy="0"
                            stdDeviation="5"
                            flood-color="#2196F3"
                            flood-opacity="0.7"
                        />
                    </filter>
                </defs>
                { render_bridges(state) }
                { render_islands(state, on_island_click) }
            </svg>

            { if is_complete {
                html! { <VictoryOverlay next_width={state.grid.width} next_height={state.grid.height} elapsed_seconds={state.time_elapsed} challenge_time={state.challenge_time} /> }
            } else {
                html! {}
            }}
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct VictoryOverlayProps {
    next_width: u8,
    next_height: u8,
    elapsed_seconds: u32,
    challenge_time: Option<u32>,
}

#[function_component(VictoryOverlay)]
fn victory_overlay(props: &VictoryOverlayProps) -> Html {
    let navigator = use_navigator().unwrap();
    let nw = props.next_width;
    let nh = props.next_height;

    let on_new_puzzle = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Game {
                width: nw,
                height: nh,
                id: rand::random::<u64>(),
            });
        })
    };

    let on_home = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Home);
        })
    };

    html! {
        <div class="victory-overlay-background victory-overlay">
            <div class="victory-modal">
                <div class="victory-emoji">
                    {"🎉"}
                </div>
                <h2 class="victory-title">
                    {"Puzzle Complete!"}
                </h2>
                <p class="victory-message">
                    {"Congratulations! All islands are connected."}
                </p>
                <div class="victory-time">
                    {"Time: "}{ format_time(props.elapsed_seconds) }
                </div>
                { if let Some(ct) = props.challenge_time {
                    let is_beating = props.elapsed_seconds < ct;
                    let message = if is_beating {
                        "🏆 You beat the challenge!"
                    } else {
                        "Time to beat was"
                    };
                    html! {
                        <div class={if is_beating { "victory-challenge-beating" } else { "victory-challenge-missed" }}>
                            { message }{ " " }{ format_time(ct) }
                        </div>
                    }
                } else {
                    html! {}
                }}
                <div class="victory-buttons">
                    <button onclick={on_new_puzzle} class="btn btn-victory">
                        {"🎲 Next Puzzle"}
                    </button>
                    <button onclick={on_home} class="btn btn-victory-secondary">
                        {"🏠 Home"}
                    </button>
                </div>
            </div>
        </div>
    }
}

fn render_islands(state: &UseStateHandle<GameState>, cb: Callback<Position>) -> Html {
    state
        .grid
        .islands
        .iter()
        .map(|(position, island)| {
            let terminating_bridges = state
                .grid
                .bridges
                .iter()
                .filter(|(BridgeLine { start, end, .. }, _)| start == position || end == position)
                .map(|(_, bridge_type)| match bridge_type {
                    hashi::BridgeType::Single => 1,
                    hashi::BridgeType::Double => 2,
                })
                .sum::<u8>();

            let complete = terminating_bridges == island.required_bridges;
            let selected = state.selected == Some(position.to_owned());

            let fill = if complete { "#8BC34A" } else { "#FFFFFF" };
            let stroke = if selected { "#2196F3" } else { "#000000" };
            let stroke_width = if selected { 4 } else { 2 };
            let radius = if selected { 32 } else { 28 };

            let onclick = {
                let cb = cb.clone();
                let pos = position.to_owned();
                Callback::from(move |_| cb.emit(pos))
            };

            let filter = if selected { "url(#selectedGlow)" } else { "" };
            let shudder_class = if state.shuddered_island == Some(position.to_owned()) {
                "shudder"
            } else {
                ""
            };

            html! {
                <g onclick={onclick} style="cursor:pointer;" class={shudder_class}>
                    <circle
                        cx={(position.x as i32 * 100).to_string()}
                        cy={(position.y as i32 * 100).to_string()}
                        r={35}
                        fill="transparent"
                    />
                    <circle
                        cx={(position.x as i32 * 100).to_string()}
                        cy={(position.y as i32 * 100).to_string()}
                        r={radius.to_string()}
                        fill={fill}
                        stroke={stroke}
                        stroke-width={stroke_width.to_string()}
                        filter={filter}
                    />
                    <text
                        x={(position.x as i32 * 100).to_string()}
                        y={(position.y as i32 * 100 + 7).to_string()}
                        text-anchor="middle"
                        font-size="20"
                        font-family="sans-serif"
                        pointer-events="none"
                    >
                        { island.required_bridges.to_string() }
                    </text>
                </g>
            }
        })
        .collect()
}

fn render_bridges(state: &UseStateHandle<GameState>) -> Html {
    state
        .grid
        .bridges
        .iter()
        .flat_map(|(bridge_line, bridge_type)| {
            // offsets for single vs double
            let offsets: Vec<i32> = match bridge_type {
                hashi::BridgeType::Single => vec![0], // single line, no offset
                hashi::BridgeType::Double => vec![-5, 5], // double line, 5px apart
            };

            offsets.into_iter().map(move |offset: i32| {
                let (x1, y1, x2, y2) = match bridge_line.direction {
                    hashi::BridgeDirection::Right => (
                        (bridge_line.start.x as i32 * 100),
                        (bridge_line.start.y as i32 * 100) + offset,
                        (bridge_line.end.x as i32 * 100),
                        (bridge_line.end.y as i32 * 100) + offset,
                    ),
                    hashi::BridgeDirection::Down => (
                        (bridge_line.start.x as i32 * 100) + offset,
                        (bridge_line.start.y as i32 * 100),
                        (bridge_line.end.x as i32 * 100) + offset,
                        (bridge_line.end.y as i32 * 100),
                    ),
                };

                // clone state for click
                let state = state.clone();
                let key = bridge_line.to_owned();
                let onclick = Callback::from(move |_| {
                    let mut s = (*state).clone();

                    if let Some(existing_bridge_type) = s.grid.bridges.get(&key) {
                        match existing_bridge_type {
                            hashi::BridgeType::Double => {
                                // Remove one bridge (double -> single)
                                s.grid.bridges.insert(key, hashi::BridgeType::Single);
                            }
                            hashi::BridgeType::Single => {
                                // Remove the bridge entirely
                                s.grid.bridges.remove(&key);
                            }
                        }
                    }
                    state.set(s);
                });

                html! {
                    <>
                        <line
                            x1={x1.to_string()}
                            y1={y1.to_string()}
                            x2={x2.to_string()}
                            y2={y2.to_string()}
                            stroke="black"
                            stroke-width="4"
                            stroke-linecap="round"
                            style="cursor:pointer;"
                        />
                        <line
                            x1={x1.to_string()}
                            y1={y1.to_string()}
                            x2={x2.to_string()}
                            y2={y2.to_string()}
                            stroke="transparent"
                            stroke-width="35"
                            style="cursor:pointer;"
                            {onclick}
                        />
                    </>
                }
            })
        })
        .collect()
}

fn format_time(seconds: u32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}
