use models::room::{Room, RoomId};
use yew::Reducible;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RoomState {
    pub room_id: RoomId,
    pub room: Room,
}

pub enum RoomAction {
    UpdateRoom(Room),
}

impl Reducible for RoomState {
    type Action = RoomAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            RoomAction::UpdateRoom(room) => Self {
                room,
                room_id: self.room_id.clone(),
            }.into()
        }
    }
}