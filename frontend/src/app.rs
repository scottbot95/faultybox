use yew::prelude::*;
use yew_router::prelude::*;
use crate::api_client::{ApiClientContext, ApiClientImpl};
use crate::pages::GeckoPage;
use crate::pages::room::RoomPage;

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
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Redirect<Route> to={Route::Room} /> },
        Route::Gecko => html! { <GeckoPage /> },
        Route::Room => html! { <RoomPage /> },
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
