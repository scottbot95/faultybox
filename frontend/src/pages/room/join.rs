use models::room::RoomId;
use patternfly_yew::prelude::*;
use yew::{platform::spawn_local, prelude::*};
use yew_router::hooks::use_navigator;

use crate::{api_client::{use_api, ApiClient, ApiClientImpl}, app::Route};

turf::style_sheet!("src/pages/room/join.scss");

#[function_component(RoomJoinPage)]
pub fn page() -> Html {
    let navigator = use_navigator().unwrap();
    let api_client = use_api::<ApiClientImpl>();
    let room_code = use_state(|| "".to_string());
    let onchange = use_callback(room_code.clone(), |new_value, room_code| room_code.set(new_value));
    let onclick = {
        let room_code = room_code.clone();
        Callback::from(move |_| {
            let api_client = api_client.clone();
            let navigator = navigator.clone();
            let room_code = (*room_code).clone();
            if !room_code.is_empty() {
                spawn_local(async move {
                    let _ = api_client.join_room(&RoomId(room_code)).await;
                    navigator.push(&Route::Lobby);
                })
            }
        })
    };

    html! {
        <div class={ClassName::PAGE}>
            <style>{STYLE_SHEET}</style>
            <h2>{"Join a room"}</h2>
            <span>
                {"Room code:"}
                <TextInput {onchange} value={(*room_code).clone()} placeholder="Room code" required=true autofocus=true/>
                <Button {onclick}>{"Join"}</Button>
            </span>
        </div>
    }
}