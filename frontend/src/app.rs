use futures::StreamExt;
use gloo_net::websocket::futures::WebSocket;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

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

    #[at("/lobby")]
    Lobby,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <IndexPage /> },
        Route::Gecko => html! { <GeckoPage /> },
        Route::Create => html! { <RoomCreatePage /> },
        Route::Join => html! { <RoomJoinPage /> },
        Route::Lobby => html! { <RoomLobbyPage /> },
    }
}

#[function_component(App)]
pub(crate) fn app() -> Html {
    use_effect_with((), move |_| {
        let mut ws = WebSocket::open("/api/room/create/gecko").unwrap();
        // let (mut write, mut read) = ws.split();
        spawn_local(async move {
            while let Some(msg) = ws.next().await {
                log::info!("Received message: {:?}", msg);
            }
        });
    });

    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}
