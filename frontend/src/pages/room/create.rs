use yew::prelude::*;
use patternfly_yew::prelude::*;
use models::GameKind;

turf::style_sheet!("src/pages/room/create.scss");

#[function_component(RoomCreatePage)]
pub fn page() -> Html {
    let selected = use_state_eq(|| None::<GameKind>);
    let onselect = use_callback(selected.clone(), |item, selected| {
        selected.set(Some(item))
    });
    let onclick = Callback::from(move |_| {
        // TODO call create room
    });
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