// use gloo::net::http::Request;
use serde::Deserialize;
use std::{collections::HashMap, str};
// use wasm_bindgen_futures::spawn_local;
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
Data model
======================= */

#[derive(Clone, Deserialize)]
struct Grid {
    width: u32,
    height: u32,
    islands: Vec<Island>,
}

#[derive(Clone, Deserialize, Copy)]
struct Island {
    id: u32,
    x: i32,
    y: i32,
    required: u8,
}

/* =======================
Game state
======================= */

#[derive(Clone)]
struct GameState {
    grid: Grid,
    bridges: HashMap<(u32, u32), u8>,
    selected: Option<u32>,
    shuddered_island: Option<u32>,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            grid: Grid {
                width: 0,
                height: 0,
                islands: vec![],
            },
            bridges: HashMap::new(),
            selected: None,
            shuddered_island: None,
        }
    }
}

/* =======================
Rule helpers
======================= */

impl GameState {
    fn island(&self, id: u32) -> &Island {
        self.grid.islands.iter().find(|i| i.id == id).unwrap()
    }

    fn bridges_for(&self, id: u32) -> u8 {
        self.bridges
            .iter()
            .filter(|((a, b), _)| *a == id || *b == id)
            .map(|(_, c)| *c)
            .sum()
    }

    fn can_add_bridge(&self, a: u32, b: u32) -> bool {
        let ia = self.island(a);
        let ib = self.island(b);

        // must align
        if ia.x != ib.x && ia.y != ib.y {
            return false;
        }

        // no island in between
        if blocked(ia, ib, &self.grid.islands) {
            return false;
        }

        let key = (a.min(b), a.max(b));
        let existing = *self.bridges.get(&key).unwrap_or(&0);
        if existing >= 2 {
            return false;
        }

        if self.bridges_for(a) + 1 > ia.required {
            return false;
        }
        if self.bridges_for(b) + 1 > ib.required {
            return false;
        }

        // crossing check
        for (x, y) in self.bridges.keys() {
            let i1 = self.island(*x);
            let i2 = self.island(*y);
            if crosses(ia, ib, i1, i2) {
                return false;
            }
        }

        true
    }

    fn is_complete(&self) -> bool {
        if self.grid.islands.is_empty() {
            return false;
        }

        // Check if all islands have the required number of bridges
        for island in &self.grid.islands {
            if self.bridges_for(island.id) != island.required {
                return false;
            }
        }

        // Check if all islands are connected (using DFS/BFS)
        if !self.is_connected() {
            return false;
        }

        true
    }

    fn is_connected(&self) -> bool {
        if self.grid.islands.is_empty() {
            return true;
        }

        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![self.grid.islands[0].id];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            // Find all connected islands
            for (a, b) in self.bridges.keys() {
                if *a == current && !visited.contains(b) {
                    stack.push(*b);
                } else if *b == current && !visited.contains(a) {
                    stack.push(*a);
                }
            }
        }

        visited.len() == self.grid.islands.len()
    }
}

fn blocked(a: &Island, b: &Island, islands: &[Island]) -> bool {
    islands.iter().any(|i| {
        if i.id == a.id || i.id == b.id {
            return false;
        }

        if a.x == b.x && i.x == a.x {
            i.y > a.y.min(b.y) && i.y < a.y.max(b.y)
        } else if a.y == b.y && i.y == a.y {
            i.x > a.x.min(b.x) && i.x < a.x.max(b.x)
        } else {
            false
        }
    })
}

fn crosses(a1: &Island, b1: &Island, a2: &Island, b2: &Island) -> bool {
    let h1 = a1.y == b1.y;
    let h2 = a2.y == b2.y;

    if h1 == h2 {
        return false;
    }

    let (h, v) = if h1 {
        ((a1, b1), (a2, b2))
    } else {
        ((a2, b2), (a1, b1))
    };

    let hy = h.0.y;
    let hx1 = h.0.x.min(h.1.x);
    let hx2 = h.0.x.max(h.1.x);

    let vx = v.0.x;
    let vy1 = v.0.y.min(v.1.y);
    let vy2 = v.0.y.max(v.1.y);

    vx > hx1 && vx < hx2 && hy > vy1 && hy < vy2
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
                let hashi_grid =
                    hashi::HashiGrid::generate_with_seed(width, height, puzzle_id).unwrap();

                let mut islands = Vec::new();
                for (island_id, (pos, hashi_island)) in hashi_grid.islands.iter().enumerate() {
                    islands.push(Island {
                        id: island_id as u32,
                        x: pos.x as i32,
                        y: pos.y as i32,
                        required: hashi_island.required_bridges,
                    });
                }

                // generate a puzzle
                let game_grid = Grid {
                    width: hashi_grid.width as u32,
                    height: hashi_grid.height as u32,
                    islands,
                };

                state.set(GameState {
                    grid: game_grid,
                    bridges: HashMap::new(),
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
        <div>
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
    if state.grid.islands.is_empty() {
        html! { <p class="loading">{"Loading puzzle..."}</p> }
    } else {
        let is_complete = state.is_complete();

        let on_island_click = {
            let state = state.clone();
            Callback::from(move |id: u32| {
                let mut s = (*state).clone();

                match s.selected {
                    None => s.selected = Some(id),
                    Some(prev) => {
                        if prev != id {
                            if s.can_add_bridge(prev, id) {
                                let key = (prev.min(id), prev.max(id));
                                *s.bridges.entry(key).or_insert(0) += 1;
                                s.selected = None;
                            } else {
                                s.shuddered_island = Some(id);
                                s.selected = None;
                            }
                        }
                    }
                }

                state.set(s);
            })
        };

        let width = state.grid.width * 100;
        let height = state.grid.height * 100;

        html! {
            <>
                <style>
                    {r#"
                        #selectedGlow { }
                    "#}  
                </style>
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
                        html! { <VictoryOverlay next_width={state.grid.width as u8} next_height={state.grid.height as u8} /> }
                    } else {
                        html! {}
                    }}
                </div>
            </>
        }
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

fn render_islands(state: &UseStateHandle<GameState>, cb: Callback<u32>) -> Html {
    state
        .grid
        .islands
        .iter()
        .map(|i| {
            let complete = state.bridges_for(i.id) == i.required;
            let selected = state.selected == Some(i.id);

            let fill = if complete { "#8BC34A" } else { "#FFFFFF" };
            let stroke = if selected { "#2196F3" } else { "#000000" };
            let stroke_width = if selected { 4 } else { 2 };
            let radius = if selected { 32 } else { 28 };

            let onclick = {
                let cb = cb.clone();
                let id = i.id;
                Callback::from(move |_| cb.emit(id))
            };

            let filter = if selected { "url(#selectedGlow)" } else { "" };
            let shudder_class = if state.shuddered_island == Some(i.id) {
                "shudder"
            } else {
                ""
            };

            html! {
                <g onclick={onclick} style="cursor:pointer;" class={shudder_class}>
                    <circle
                        cx={(i.x * 100).to_string()}
                        cy={(i.y * 100).to_string()}
                        r={50}
                        fill="transparent"
                    />
                    <circle
                        cx={(i.x * 100).to_string()}
                        cy={(i.y * 100).to_string()}
                        r={radius.to_string()}
                        fill={fill}
                        stroke={stroke}
                        stroke-width={stroke_width.to_string()}
                        filter={filter}
                    />
                    <text
                        x={(i.x * 100).to_string()}
                        y={(i.y * 100 + 7).to_string()}
                        text-anchor="middle"
                        font-size="20"
                        font-family="sans-serif"
                        pointer-events="none"
                    >
                        { i.required.to_string() }
                    </text>
                </g>
            }
        })
        .collect()
}

fn render_bridges(state: &UseStateHandle<GameState>) -> Html {
    state
        .bridges
        .iter()
        .flat_map(|((a, b), count)| {
            let ia = state.island(*a);
            let ib = state.island(*b);

            // Determine if horizontal or vertical
            let horizontal = ia.y == ib.y;
            let vertical = ia.x == ib.x;

            // offsets for single vs double
            let offsets: Vec<i32> = if *count == 1 {
                vec![0] // single line, no offset
            } else {
                vec![-5, 5] // double line, 5px apart
            };

            offsets.into_iter().map(move |offset: i32| {
                let (x1, y1, x2, y2) = if horizontal {
                    // horizontal → offset dy
                    (
                        ia.x * 100,
                        (ia.y * 100) + offset,
                        ib.x * 100,
                        (ib.y * 100) + offset,
                    )
                } else if vertical {
                    // vertical → offset dx
                    (
                        (ia.x * 100) + offset,
                        ia.y * 100,
                        (ib.x * 100) + offset,
                        ib.y * 100,
                    )
                } else {
                    // fallback diagonal (shouldn't happen)
                    println!(
                        "Warning: diagonal bridge detected between islands {} and {}",
                        a, b
                    );
                    (ia.x * 100, ia.y * 100, ib.x * 100, ib.y * 100)
                };

                // clone state for click
                let state = state.clone();
                let key = (a.min(b).to_owned(), a.max(b).to_owned());
                let onclick = Callback::from(move |_| {
                    let mut s = (*state).clone();
                    if let Some(c) = s.bridges.get_mut(&key) {
                        if *c > 1 {
                            *c -= 1; // double → single
                        } else {
                            s.bridges.remove(&key); // single → remove
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
