use yew::prelude::*;

turf::style_sheet!("src/pages/room/lobby.scss");

#[function_component(RoomLobbyPage)]
pub fn page() -> Html {
    html! {
        <div class={ClassName::PAGE}>
            <style>{STYLE_SHEET}</style>
        </div>
    }
}