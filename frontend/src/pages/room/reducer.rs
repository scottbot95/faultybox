use models::room::Room;
use std::rc::Rc;
use yew::Reducible;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct RoomState {
    pub room: Option<Room>,
}

pub enum RoomAction {
    UpdateRoom(Room),
}

impl Reducible for RoomState {
    type Action = RoomAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            RoomAction::UpdateRoom(room) => Self { room: Some(room) }.into(),
        }
    }
}
