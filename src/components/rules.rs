use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Rules)]
pub fn rules() -> Html {
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
