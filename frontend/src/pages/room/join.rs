use crate::api_client::{use_api, ApiClient, ApiClientImpl};
use crate::pages::room::reducer::RoomAction;
use crate::pages::room::socket::Dispatcher;
use models::room::api::JoinRoomInput;
use models::room::{RoomId, ROOM_ID_LEN};
use patternfly_yew::prelude::*;
use smol_str::SmolStr;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

turf::style_sheet!("src/pages/room/join.scss");

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) dispatcher: Dispatcher,
}

#[function_component(RoomJoinPage)]
pub fn page(props: &Props) -> Html {
    let nickname_ref = use_node_ref();
    let api_client = use_api::<ApiClientImpl>();
    let room_code = use_state(|| "".to_string()); // FIXME this should be able to use use_state_eq
    log::debug!("Page rendered. Code: {}", &*room_code);
    let on_room_code = use_callback(
        room_code.clone(),
        |event: InputEvent, room_code: &UseStateHandle<String>| {
            event.prevent_default();
            event.stop_propagation();
            let mut new_value = event.target_unchecked_into::<HtmlInputElement>().value();
            log::debug!("room code changed: {}", new_value);
            new_value.truncate(ROOM_ID_LEN);
            new_value.make_ascii_lowercase();
            new_value.retain(|c| c.is_ascii_lowercase());
            room_code.set(new_value);
        },
    );
    let onclick = {
        let nickname_ref = nickname_ref.clone();
        let room_code = room_code.clone();
        let dispatcher = props.dispatcher.clone();
        Callback::from(move |_| {
            let nickname_ref = nickname_ref.clone();
            let api_client = api_client.clone();
            let room_code = (*room_code).clone();
            let dispatcher = dispatcher.clone();
            if !room_code.is_empty() {
                spawn_local(async move {
                    let nickname = nickname_ref
                        .cast::<HtmlInputElement>()
                        .expect("nickname_ref not bound")
                        .value();
                    let input = JoinRoomInput {
                        room_id: RoomId(room_code.into()),
                        nickname: SmolStr::new(nickname),
                    };
                    let res = api_client.join_room(input).await;
                    match res {
                        Ok(output) => {
                            dispatcher.dispatch(RoomAction::UpdateRoom(output.room));
                        }
                        Err(e) => {
                            log::error!("Failed to join room {:?}?", e);
                        }
                    }
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
                <TextInput oninput={on_room_code} value={(*room_code).clone()} placeholder="Room code" required=true autofocus=true/>
                <TextInput r#ref={nickname_ref} placeholder="Name" required=true/>
                <Button {onclick}>{"Join"}</Button>
            </span>
        </div>
    }
}
