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
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}
