use std::str;
use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod hashi;

fn main() {
    yew::Renderer::<App>::new().render();
}

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
Main App with Router
======================= */

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <components::home::Home /> },
        Route::Game { width, height, id } => {
            html! { <components::game::Game width={width} height={height} puzzle_id={id} /> }
        }
        Route::Rules => html! { <components::rules::Rules /> },
        Route::NotFound => html! { <components::not_found::NotFound /> },
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
