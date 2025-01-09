use crate::app::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(IndexPage)]
pub fn page() -> Html {
    html! {
        <div>
            <div style="width: fit-content; margin: auto; display: flex; flex-direction: column; align-items: center;">
                <Link<Route> to={Route::Create}>
                    <button>{"Create Room"}</button>
                </Link<Route>>
                <Link<Route> to={Route::Join}>
                    <button>{"Join Room"}</button>
                </Link<Route>>
            </div>
        </div>
    }
}