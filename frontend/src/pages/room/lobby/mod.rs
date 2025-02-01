mod nameplate;

use crate::pages::room::lobby::nameplate::NamePlate;
use models::room::Room;
use std::rc::Rc;
use yew::prelude::*;

turf::style_sheet!("src/pages/room/lobby/lobby.scss");

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub room: Rc<Room>,
}

#[function_component(RoomLobbyPage)]
pub fn page(props: &Props) -> Html {
    html! {
        <div>
            <p>{ format!("Room ID: {}", props.room.id) }</p>
            <p>{ format!("Game Kind: {:?}", props.room.game) }</p>
            <p>{"Members:"}</p>
            <ul>
                {props.room.members.keys().map(|client_id| html!{
                    <li key={client_id.to_string()}>
                        {"Name:"}
                        <NamePlate {client_id}/>
                    </li>
                }).collect::<Html>()}
            </ul>
        </div>
    }
}
