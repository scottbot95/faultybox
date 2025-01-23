use yew::prelude::*;
use patternfly_yew::prelude::*;
use yew::platform::spawn_local;
use models::GameKind;
use crate::api_client::{use_api, ApiClient, ApiClientImpl};
use crate::pages::room::RoomLobbyPage;

turf::style_sheet!("src/pages/room/create.scss");

#[function_component(RoomCreatePage)]
pub fn page() -> Html {
    let api_client = use_api::<ApiClientImpl>();
    let room_state = use_state_eq(|| None);
    let selected = use_state_eq(|| None::<GameKind>);
    let onselect = use_callback(selected.clone(), |item, selected| {
        selected.set(Some(item))
    });
    let onclick = {
        let selected = selected.clone();
        let room_state = room_state.clone();
        Callback::from(move |_| {
            let api_client = api_client.clone();
            let room_state = room_state.clone();
            if let Some(game) = *selected {
                spawn_local(async move {
                    let res = api_client.create_room(game).await;
                    match res {
                        Ok(output) => {
                            room_state.set(Some((output.room_id, output.room)));
                        },
                        Err(e) => log::error!("Failed to join room {:?}?", e),
                    }
                })
            }
        })
    };

    match (*room_state).clone() {
        None => html! {
            <div class={ClassName::PAGE}>
                <style>{STYLE_SHEET}</style>
                <span>
                    {"Game:"}
                    <SimpleSelect<GameKind>
                        placeholder = "Choose a game"
                        selected={*selected}
                        entries={GameKind::values().to_vec()}
                        {onselect}
                    />
                </span>
                <Button {onclick}>{"Create"}</Button>
            </div>
        },
        Some((room_id, initial_room)) => html! {
            <RoomLobbyPage {room_id} {initial_room} />
        }
    }
}