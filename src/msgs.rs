use komorebi_client::{State};
use winsafe::{co::WM, msg::WndMsg};

pub struct UpdateState;

impl UpdateState {
    pub const ID: WM = unsafe { WM::from_raw(WM::APP.raw() + 1) };

    pub fn to_wmdmsg(state: State) -> WndMsg {
        let data = Box::new(state);
        let ptr = Box::into_raw(data) as isize;

        WndMsg {
            msg_id: Self::ID,
            wparam: 0,
            lparam: ptr,
        }
    }

    pub fn from_wndmsg(p: WndMsg) -> State {
        let state = unsafe { Box::from_raw(p.lparam as *mut State) };
        *state
    }
}
