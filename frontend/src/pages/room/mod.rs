mod lobby;
mod create;
mod join;
mod reducer;
mod socket;

use yew::prelude::*;

pub use lobby::*;
pub use create::*;
pub use join::*;
use crate::api_client::{use_api, ApiClient, ApiClientImpl};
use crate::pages::room::reducer::RoomState;
use crate::pages::room::socket::{Dispatcher, SocketHandler, SocketTearDown};

#[function_component(RoomPage)]
pub fn room_page() -> Html {
    let client = use_api::<ApiClientImpl>();
    let join_kind = use_state(|| None::<JoinKind>);
    let room_state = use_reducer_eq(RoomState::default);
    {
        let dispatcher: Dispatcher = room_state.dispatcher();
        // connect socket only after we've loaded an initial room state
        // (since that's means we have a room token)
        use_effect_with(room_state.room.is_some(), move |&has_token| {
            if !has_token {
                return SocketTearDown::Nop;
            }
            let ws = client.connect_room().expect("Unable to connect to room");
            let handler = SocketHandler {
                dispatcher
            };

            handler.spawn(ws)
        });
    }

    if let Some(room) = room_state.room.clone() {
        html! {
            <RoomLobbyPage {room} />
        }
    } else {
        match *join_kind {
            None => {
                let create_room = {
                    let join_kind = join_kind.clone();
                    Callback::from(move |_| {
                        join_kind.set(Some(JoinKind::Create));
                    })
                };
                let join_room = {
                    let join_kind = join_kind.clone();
                    Callback::from(move |_| {
                        join_kind.set(Some(JoinKind::Join));
                    })
                };
                html! {
                    <div>
                        <div style="width: fit-content; margin: auto; display: flex; flex-direction: column; align-items: center;">
                            <button onclick={create_room}>{"Create Room"}</button>
                            <button onclick={join_room}>{"Join Room"}</button>
                        </div>
                    </div>
                }

            },
            Some(JoinKind::Create) => {
                let dispatcher = room_state.dispatcher();
                html! {
                    <RoomCreatePage {dispatcher} />
                }
            },
            Some(JoinKind::Join) => {
                let dispatcher = room_state.dispatcher();
                html! {
                    <RoomJoinPage {dispatcher} />
                }
            },
        }
    }

}

enum JoinKind {
    /// Create a new room
    Create,
    ///
    Join,
}