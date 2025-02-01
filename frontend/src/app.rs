use crate::api_client::{ApiClientContext, ApiClientImpl};
use crate::pages::room::RoomPage;
use crate::pages::GeckoPage;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/gecko")]
    Gecko,

    #[at("/room")]
    Room,

    // #[at("/lobby")]
    // Lobby,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Redirect<Route> to={Route::Room} /> },
        Route::Gecko => html! { <GeckoPage /> },
        Route::Room => html! { <RoomPage /> },
        Route::NotFound => html! {
            <div>
                <h1>{ "Page not found :(" }</h1>
                <Link<Route> to={Route::Home}>
                    <button>{"Home"}</button>
                </Link<Route>>
            </div>
        },
    }
}

#[function_component(App)]
pub(crate) fn app() -> Html {
    let client = use_memo((), |_| {
        ApiClientImpl::new_from_window().expect("Unable to determine host")
    });

    html! {
        <ContextProvider<ApiClientContext<ApiClientImpl>> context={ApiClientContext(client)}>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContextProvider<ApiClientContext<ApiClientImpl>>>
    }
}
