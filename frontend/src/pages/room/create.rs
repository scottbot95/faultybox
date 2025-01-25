use yew::prelude::*;
use patternfly_yew::prelude::*;
use yew::platform::spawn_local;
use models::GameKind;
use crate::api_client::{use_api, ApiClient, ApiClientImpl};
use crate::pages::room::reducer::RoomAction;
use crate::pages::room::socket::Dispatcher;

turf::style_sheet!("src/pages/room/create.scss");


#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) dispatcher: Dispatcher,
}

#[function_component(RoomCreatePage)]
pub fn page(props: &Props) -> Html {
    let api_client = use_api::<ApiClientImpl>();
    let selected = use_state_eq(|| None::<GameKind>);
    let onselect = use_callback(selected.clone(), |item, selected| {
        selected.set(Some(item))
    });
    let onclick = {
        let selected = selected.clone();
        let dispatcher = props.dispatcher.clone();
        Callback::from(move |_| {
            let api_client = api_client.clone();
            let dispatcher = dispatcher.clone();
            if let Some(game) = *selected {
                spawn_local(async move {
                    let res = api_client.create_room(game).await;
                    match res {
                        Ok(output) => {
                            dispatcher.dispatch(RoomAction::UpdateRoom(output.room));
                        },
                        Err(e) => log::error!("Failed to join room {:?}?", e),
                    }
                })
            }
        })
    };

    html! {
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
    }
}