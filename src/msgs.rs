use std::sync::Arc;

use komorebi_client::State;
use winsafe::{co::WM, msg::WndMsg};

pub struct UpdateState;

impl UpdateState {
    pub const ID: WM = unsafe { WM::from_raw(WM::APP.raw() + 1) };

    pub fn to_wmdmsg(state: Arc<State>) -> WndMsg {
        let ptr = Arc::into_raw(state) as isize;

        WndMsg {
            msg_id: Self::ID,
            wparam: 0,
            lparam: ptr,
        }
    }

    pub fn from_wndmsg(p: WndMsg) -> Arc<State> {
        let state = unsafe { Arc::from_raw(p.lparam as *mut State) };
        state
    }
}
