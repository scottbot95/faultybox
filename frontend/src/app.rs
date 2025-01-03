use futures::StreamExt;
use gloo_net::websocket::futures::WebSocket;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::GeckoPage;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,

    #[at("/gecko")]
    Gecko,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Redirect<Route> to={Route::Gecko}/>},
        Route::Gecko => html! { <GeckoPage /> },
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
