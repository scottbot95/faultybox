mod reducer;

use yew::prelude::*;
use models::room::Room;

turf::style_sheet!("src/pages/room/lobby/lobby.scss");

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub room: Room,
}

#[function_component(RoomLobbyPage)]
pub fn page(props: &Props) -> Html {
    html! {
        <div>
            <p>{ format!("Room ID: {}", props.room.id) }</p>
            <p>{ format!("Game Kind: {:?}", props.room.game) }</p>
            <p>{"Members:"}</p>
            <ul>
                {props.room.members.iter().map(|(client_id, member)| html!{
                    <li key={client_id.to_string()}>
                        {"Name:"}
                        {member.nickname.clone().unwrap_or_else(|| "Unknown".to_string())}
                    </li>
                }).collect::<Html>()}
            </ul>
        </div>
    }
}
