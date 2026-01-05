use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

const BUILD_DATE: &str = env!("BUILD_DATE");

#[function_component(Home)]
pub fn home() -> Html {
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
            <h1 class="home-title">{"Hashi!"}</h1>
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
            <footer class="home-footer">
                <a href="https://github.com/tomos-evans/hashi" target="_blank" rel="noopener noreferrer" class="github-link">
                    {"View on GitHub"}
                </a>
                <span class="build-date">{format!("Built: {}", BUILD_DATE)}</span>
            </footer>
        </div>
    }
}
