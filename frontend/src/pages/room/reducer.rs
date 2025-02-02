use models::room::Room;
use std::rc::Rc;
use smol_str::SmolStr;
use yew::Reducible;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct RoomState {
    pub room: Option<Rc<Room>>,
    pub error: Option<SmolStr>,
}

#[derive(Debug)]
pub enum RoomAction {
    UpdateRoom(Room),
    RoomClosed(CloseReason),
}

#[derive(Debug)]
pub enum CloseReason {
    RoomEnded
}

impl std::fmt::Display for CloseReason {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CloseReason::RoomEnded => write!(f, "Room ended"),
        }
    }
}

impl Reducible for RoomState {
    type Action = RoomAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        log::debug!("Room action: {:?}", action);
        match action {
            RoomAction::UpdateRoom(room) => Self {
                room: Some(room.into()),
                error: self.error.clone(),
            }.into(),
            RoomAction::RoomClosed(reason) => {
                let msg = format!("Room closed. Reason: {}.", reason).into();

                Self {
                    error: Some(msg),
                    room: self.room.clone(),
                }.into()
            }
        }
    }
}
