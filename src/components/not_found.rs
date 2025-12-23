use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(NotFound)]
pub fn not_found() -> Html {
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
