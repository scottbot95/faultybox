use models::room::{Room, RoomId};
use patternfly_yew::prelude::*;
use yew::{platform::spawn_local, prelude::*};

use crate::{api_client::{use_api, ApiClient, ApiClientImpl}};
use crate::pages::room::RoomLobbyPage;

turf::style_sheet!("src/pages/room/join.scss");

#[function_component(RoomJoinPage)]
pub fn page() -> Html {
    let api_client = use_api::<ApiClientImpl>();
    let room_code = use_state(|| "".to_string()); // FIXME this should be able to use use_state_eq
    log::debug!("Page rendered. Code: {}", &*room_code);
    let room_state = use_state_eq(|| None);
    let oninput = use_callback(room_code.clone(), |event: InputEvent, room_code: &UseStateHandle<String>| {
        event.prevent_default();
        event.stop_propagation();
        let Some(mut new_value) = event.data() else { return; };
        log::debug!("room code changed: {}", new_value);
        new_value.make_ascii_lowercase();
        new_value.retain(|c| c.is_ascii_lowercase());
        room_code.set((**room_code).clone() + &new_value);
    });
    let onclick = {
        let room_code = room_code.clone();
        let room_state = room_state.clone();
        Callback::from(move |_| {
            let api_client = api_client.clone();
            let room_state = room_state.clone();
            let room_code = (*room_code).clone();
            if !room_code.is_empty() {
                spawn_local(async move {
                    let res = api_client.join_room(&RoomId(room_code)).await;
                    match res {
                        Ok(output) => {
                            room_state.set(Some((output.room_id, output.room)));
                        },
                        Err(e) => {
                            log::error!("Failed to join room {:?}?", e);
                        },
                    }
                })
            }
        })
    };

    match (*room_state).clone() {
        None => html! {
            <div class={ClassName::PAGE}>
                <style>{STYLE_SHEET}</style>
                <h2>{"Join a room"}</h2>
                <span>
                    {"Room code:"}
                    <TextInput {oninput} value={(*room_code).clone()} placeholder="Room code" required=true autofocus=true/>
                    <Button {onclick}>{"Join"}</Button>
                </span>
            </div>
        },
        Some((room_id, initial_room)) => html! {
            <RoomLobbyPage {room_id} {initial_room} />
        }
    }
}