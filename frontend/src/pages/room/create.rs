use yew::prelude::*;
use patternfly_yew::{components::nav, prelude::*};
use yew::platform::spawn_local;
use models::GameKind;
use yew_router::{hooks::use_navigator, AnyRoute};
use crate::{api_client::{use_api, ApiClient, ApiClientImpl}, app::Route};

turf::style_sheet!("src/pages/room/create.scss");

#[function_component(RoomCreatePage)]
pub fn page() -> Html {
    let navigator = use_navigator().unwrap();
    let api_client = use_api::<ApiClientImpl>();
    let selected = use_state_eq(|| None::<GameKind>);
    let onselect = use_callback(selected.clone(), |item, selected| {
        selected.set(Some(item))
    });
    let onclick = {
        let selected = selected.clone();
        Callback::from(move |_| {
            let api_client = api_client.clone();
            let navigator = navigator.clone();
            if let Some(game) = *selected {
                spawn_local(async move {
                    let _ = api_client.create_room(game).await;
                    navigator.push(&Route::Lobby);
                })
            }
        })
    };
    html! {
        <div /*class={ClassName::PAGE}*/>
            <style>{STYLE_SHEET}</style>
            <span>
                {"Game:"}
                <SimpleSelect<GameKind>
                    placeholder = "Choose a game"
                    selected={*selected}
                    entries={vec![GameKind::Gecko]}
                    {onselect}
                />
            </span>
            <Button {onclick}>{"Create"}</Button>
        </div>
    }
}