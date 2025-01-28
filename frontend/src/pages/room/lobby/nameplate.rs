use yew::prelude::*;
use models::room::api::ClientId;
use models::room::Room;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) client_id: ClientId
}

#[function_component(NamePlate)]
pub fn name_plate(props: &Props) -> Html {
    let name = use_context::<Room>()
        .and_then(|room| room.members.get(&props.client_id).cloned())
        .and_then(|m| m.nickname);

    html! {
        <NamePlateInner {name} />
    }
}

#[derive(Properties, PartialEq)]
struct PropsInner {
    name: Option<AttrValue>,
}

#[function_component(NamePlateInner)]
fn name_plate_impl(props: &PropsInner) -> Html {
    match props.name {
        None => html!{
            <span>{"Unknown"}</span>
        },
        Some(ref name) => html! {
            <span>{name}</span>
        },
    }
}