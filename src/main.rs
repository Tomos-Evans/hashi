use std::collections::HashMap;
use gloo::net::http::Request;
use serde::Deserialize;
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;

fn main() {
    yew::Renderer::<App>::new().render();
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
    bridges: HashMap<(u32, u32), u8>, // (min_id, max_id) -> count (1 or 2)
    selected: Option<u32>,
    shuddered_island: Option<u32>,
    puzzle_id: String,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            grid: Grid { width: 0, height: 0, islands: vec![] },
            bridges: std::collections::HashMap::new(),
            selected: None,
            shuddered_island: None,
            puzzle_id: "Unknown".to_string(),
        }
    }
}

/* =======================
   Load puzzle (JSON)
   ======================= */

// fn load_game() -> GameState {
//     let json = r#"
// "#;

//     GameState {
//         grid: serde_json::from_str(json).unwrap(),
//         bridges: HashMap::new(),
//         selected: None,
//         shuddered_island: None,
//     }
// }

/* =======================
   Rule helpers
   ======================= */

impl GameState {

    fn island(&self, id: u32) -> &Island {
        self.grid.islands.iter().find(|i| i.id == id).unwrap()
    }

    fn bridges_for(&self, id: u32) -> u8 {
        self.bridges.iter()
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
   Yew App
   ======================= */

fn render_game(state: &UseStateHandle<GameState>) -> Html {
        if state.grid.islands.is_empty() {
        html! { <p>{"Loading puzzle..."}</p> }
    } else {

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
                                // TODO : clear shudder after animation. This breaks the state or sets it back to what it was
                            }
                        }
                    }
                }
                
                state.set(s);

            })
        };


        let height = state.grid.width * 100;
        let width = state.grid.height * 100;

        html! {
            <>
                <style>
                    {r#"
                        @keyframes shudder {
                            0%   { transform: translateX(0); }
                            20%  { transform: translateX(-5px); }
                            40%  { transform: translateX(5px); }
                            60%  { transform: translateX(-5px); }
                            80%  { transform: translateX(5px); }
                            100% { transform: translateX(0); }
                        }

                        .shudder {
                            animation: shudder 0.3s ease;
                        }   
                    "#}  
                </style>
                <h1>{format!("Puzzle ID: {}", state.puzzle_id)}</h1>

                <div style="width:100vw; overflow:auto;">
                    <svg
                    
                        viewBox={format!("-100 -100 {} {}", width + 100, height+100)}
                        preserveAspectRatio="xMidYMid meet"
                        style="
                            width: 100%;
                            height: auto;
                            aspect-ratio: {};
                            display: block;
                            background: #f5f5f5;
                            touch-action: manipulation;
                            user-select: none;
                            -webkit-user-select: none;
                            -webkit-tap-highlight-color: transparent;
                            "
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
                        { render_bridges(&state) }
                        { render_islands(&state, on_island_click) }
                    </svg>
                </div>
            </>
            
        }
    }
}


fn puzzle_id_from_url() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let path = location.pathname().ok()?;

    // "/abc123" → "abc123"
    let trimmed = path.trim_start_matches('/');

    if trimmed.is_empty() || trimmed.to_lowercase() == "random" {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[function_component(App)]
fn app() -> Html {
    let state: UseStateHandle<GameState> = use_state(GameState::default);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let puzzle_id = puzzle_id_from_url();

                match Request::get("puzzles/data.json").send().await {
                    Ok(resp) => {
                        web_sys::console::log_1(&"Puzzle loaded".into());
                        if let Ok(grids) = resp.json::<HashMap<String, Grid>>().await {

                            if grids.is_empty() {
                                web_sys::console::error_1(&"No puzzles found in data.json".into());
                                return;
                            }


                            let (puzzle_id, grid) = match puzzle_id {
                                Some(id) => {
                                    if let Some(g) = grids.get(&id) {
                                        web_sys::console::log_1(&format!("Loaded puzzle {}", id).into());
                                        (id.clone(), g.clone())
                                    } else {
                                        web_sys::console::error_1(&format!("Puzzle ID {} not found, loading random puzzle", id).into());
                                        let rand_index = rand::random_range(0..grids.len());
                                        let (id, grid) = grids.iter().nth(rand_index).unwrap();
                                        (id.clone(), grid.clone())
                                    }
                                }
                                None => {
                                    // random puzzle
                                    let rand_index = rand::random_range(0..grids.len());
                                    let (id, grid) = grids.iter().nth(rand_index).unwrap();
                                    (id.clone(), grid.clone())
                                }
                            };



                            web_sys::console::log_1(&format!("Loaded puzzle {} with {} islands", puzzle_id, grid.islands.len()).into());

                            state.set(GameState {
                                grid: grid.clone(),
                                bridges: HashMap::new(),
                                selected: None,
                                shuddered_island: None,
                                puzzle_id,
                            });
                        }
                    }
                    Err(err) => {
                        web_sys::console::error_1(&format!("{err:?}").into());
                    }
                }
            });

            || ()
        });    
    }


    html! {
        <div>
            { render_game(&state) }
        </div>
    }
}

/* =======================
   Rendering
   ======================= */

fn render_islands(state: &UseStateHandle<GameState>, cb: Callback<u32>) -> Html {
    state.grid.islands.iter().map(|i| {
        let complete = state.bridges_for(i.id) == i.required;
        let selected = state.selected == Some(i.id);

        let fill = if complete {
            "#8BC34A"        // green
        } else {
            "#FFFFFF"
        };

        let stroke = if selected {
            "#2196F3"        // blue
            // "#000000"
        } else {
            "#000000"
        };

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
                // Invisible larger circle for easier clicking
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
    }).collect()
}


fn render_bridges(state: &UseStateHandle<GameState>) -> Html {
    state.bridges.iter().flat_map(|((a, b), count)| {
        let ia = state.island(*a);
        let ib = state.island(*b);

        // Determine if horizontal or vertical
        let horizontal = ia.y == ib.y;
        let vertical   = ia.x == ib.x;

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
                println!("Warning: diagonal bridge detected between islands {} and {}", a, b);
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
    }).collect()
}