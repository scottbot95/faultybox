use yew::prelude::*;

turf::style_sheet!("src/pages/room/join.scss");

#[function_component(RoomJoinPage)]
pub fn page() -> Html {
    html! {
        <div /*class={ClassName::PAGE}*/>
            <style>{STYLE_SHEET}</style>
        </div>
    }
}