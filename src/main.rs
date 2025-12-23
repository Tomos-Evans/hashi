use crate::hashi::{BridgeLine, HashiGrid, Position};
use std::str;
use web_sys::console;
use web_sys::wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::prelude::*;

mod hashi;

fn main() {
    yew::Renderer::<App>::new().render();
}

/* =======================
Routes
======================= */

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/game/:width/:height/:id")]
    Game { width: u8, height: u8, id: u64 },
    #[at("/rules")]
    Rules,
    #[not_found]
    #[at("/404")]
    NotFound,
}

/* =======================
Game state
======================= */

#[derive(Clone)]
struct GameState {
    grid: HashiGrid,
    selected: Option<Position>,
    shuddered_island: Option<Position>,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            grid: HashiGrid::placeholder(),
            selected: None,
            shuddered_island: None,
        }
    }
}

/* =======================
Page Components
======================= */

#[function_component(Home)]
fn home() -> Html {
    let navigator = use_navigator().unwrap();

    let on_new_game_5x10 = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Game {
                width: 5,
                height: 10,
                id: rand::random::<u64>(),
            });
        })
    };
    let on_new_game_8x16 = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Game {
                width: 8,
                height: 16,
                id: rand::random::<u64>(),
            });
        })
    };
    let on_rules = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Rules);
        })
    };

    html! {
        <div class="home-container">
            <h1 class="home-title">{"Hashi Puzzle Game"}</h1>
            <p class="home-subtitle">
                {"Connect the islands with bridges following the puzzle rules"}
            </p>
            <div class="home-buttons">
                <button onclick={on_new_game_5x10} class="btn btn-primary">
                    {"5x10"}
                </button>
                <button onclick={on_new_game_8x16} class="btn btn-primary">
                    {"8x16"}
                </button>
                <button onclick={on_rules} class="btn btn-success">
                    {"View Rules"}
                </button>
            </div>
        </div>
    }
}

#[function_component(Rules)]
fn rules() -> Html {
    let navigator = use_navigator().unwrap();

    let on_back = {
        Callback::from(move |_| {
            navigator.push(&Route::Home);
        })
    };

    html! {
        <div class="rules-container">
            <h1>{"Hashi Rules"}</h1>
            <div class="rules-content">
                <h2>{"Objective"}</h2>
                <p>{"Connect all islands with bridges according to the numbers on each island."}</p>

                <h2>{"Rules"}</h2>
                <ul>
                    <li>{"The number on each island indicates how many bridges must connect to it"}</li>
                    <li>{"Bridges can only be horizontal or vertical"}</li>
                    <li>{"Bridges cannot cross each other"}</li>
                    <li>{"Bridges cannot cross islands"}</li>
                    <li>{"You can place 1 or 2 bridges between two islands"}</li>
                    <li>{"All islands must be connected in a single network"}</li>
                </ul>

                <h2>{"How to Play"}</h2>
                <ul>
                    <li>{"Click on an island to select it (it will glow blue)"}</li>
                    <li>{"Click on another island to build a bridge between them"}</li>
                    <li>{"Click the same pair again to add a second bridge"}</li>
                    <li>{"Click on a bridge to remove it (reduces double to single, or removes single)"}</li>
                    <li>{"When an island has the correct number of bridges, it turns green"}</li>
                </ul>
            </div>

            <button onclick={on_back} class="btn btn-back">
                {"Back to Home"}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct GameProps {
    pub puzzle_id: u64,
    pub width: u8,
    pub height: u8,
}

#[function_component(Game)]
fn game(props: &GameProps) -> Html {
    let state: UseStateHandle<GameState> = use_state(GameState::default);
    let navigator = use_navigator().unwrap();
    let puzzle_id = props.puzzle_id;
    let width = props.width;
    let height = props.height;
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
                });
            }
            || ()
        });
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
                    {"🎲 New"}
                </button>
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

#[function_component(NotFound)]
fn not_found() -> Html {
    let navigator = use_navigator().unwrap();

    let on_home = {
        Callback::from(move |_| {
            navigator.push(&Route::Home);
        })
    };

    html! {
        <div style="text-align: center; padding: 50px; font-family: sans-serif;">
            <h1>{"404 - Page Not Found"}</h1>
            <p>{"The page you're looking for doesn't exist."}</p>
            <button onclick={on_home} class="btn btn-back">
                {"Go Home"}
            </button>
        </div>
    }
}

/* =======================
Main App with Router
======================= */

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Game { width, height, id } => {
            html! { <Game width={width} height={height} puzzle_id={id} /> }
        }
        Route::Rules => html! { <Rules /> },
        Route::NotFound => html! { <NotFound /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

/* =======================
Game Rendering
======================= */

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

    console::log_1(&JsValue::from_str("1!"));

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
                html! { <VictoryOverlay next_width={state.grid.width} next_height={state.grid.height} /> }
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
                        r={50}
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
    console::log_1(&JsValue::from_str("render_bridges!"));
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

/* =======================
Helper Functions
======================= */

// fn get_base_path() -> String {
//     if let Some(window) = web_sys::window()
//         && let Some(document) = window.document()
//         && let Some(base) = document.query_selector("base").ok().flatten()
//         && let Some(href) = base.get_attribute("href")
//     {
//         return href;
//     }

//     // Fallback to root
//     "/".to_string()
// }

// fn get_data_json_url() -> String {
//     format!("{}puzzles/data.json", get_base_path())
// }
