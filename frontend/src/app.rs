use yew::prelude::*;
use yew_router::prelude::*;
use models::room::Room;
use crate::api_client::{ApiClientContext, ApiClientImpl};
use crate::pages::GeckoPage;
use crate::pages::index::IndexPage;
use crate::pages::room::{RoomCreatePage, RoomJoinPage, RoomLobbyPage};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/gecko")]
    Gecko,

    #[at("/create")]
    Create,

    #[at("/join")]
    Join,

    // #[at("/lobby")]
    // Lobby,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <IndexPage /> },
        Route::Gecko => html! { <GeckoPage /> },
        Route::Create => html! { <RoomCreatePage /> },
        Route::Join => html! { <RoomJoinPage /> },
        // Route::Lobby => html! { <RoomLobbyPage /> },
    }
}

#[function_component(App)]
pub(crate) fn app() -> Html {
    let client = use_memo((), |_| {
         ApiClientImpl::new_from_window()
            .expect("Unable to determine host")
    });
    
    html! {
        <ContextProvider<ApiClientContext<ApiClientImpl>> context={ApiClientContext(client)}>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContextProvider<ApiClientContext<ApiClientImpl>>>
    }
}
