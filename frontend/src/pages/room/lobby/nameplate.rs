use models::room::api::ClientId;
use models::room::Room;
use smol_str::SmolStr;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub(crate) client_id: ClientId,
}

#[function_component(NamePlate)]
pub fn name_plate(props: &Props) -> Html {
    let name = use_context::<Rc<Room>>()
        .and_then(|room| room.members.get(&props.client_id).cloned())
        .map(|m| m.nickname);

    html! {
        <NamePlateInner {name} />
    }
}

#[derive(Properties, PartialEq)]
struct PropsInner {
    name: Option<SmolStr>,
}

#[function_component(NamePlateInner)]
fn name_plate_impl(props: &PropsInner) -> Html {
    match props.name {
        None => html! {
            <span>{"Unknown"}</span>
        },
        Some(ref name) => html! {
            <span>{name.as_str()}</span>
        },
    }
}
