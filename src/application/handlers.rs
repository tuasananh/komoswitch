use std::sync::Arc;

use crate::{
    application::{Application, settings::Settings},
    msgs::UpdateState,
};
use komorebi_client::{SocketMessage, State};
use winsafe::{HMENU, HWND, PostQuitMessage, co, msg, prelude::*};

const WM_SETTINGCHANGED: co::WM = unsafe { co::WM::from_raw(0x001A) }; // WM_SETTINGCHANGE

impl Application {
    pub(super) fn handle_message(&mut self, hwnd: &HWND, p: msg::WndMsg) -> anyhow::Result<isize> {
        match p.msg_id {
            co::WM::CREATE => self.handle_create(),
            co::WM::PAINT => self.handle_paint(hwnd),
            co::WM::LBUTTONDOWN => {
                self.handle_lbuttondown(hwnd, unsafe { msg::wm::LButtonDown::from_generic_wm(p) })
            }
            co::WM::RBUTTONDOWN => {
                self.handle_rbuttondown(hwnd, unsafe { msg::wm::RButtonDown::from_generic_wm(p) })
            }
            co::WM::COMMAND => {
                self.handle_command(hwnd, unsafe { msg::wm::Command::from_generic_wm(p) })
            }
            UpdateState::ID => self.handle_update_state(UpdateState::from_wndmsg(p)),
            WM_SETTINGCHANGED => self.handle_setting_changed(hwnd),
            co::WM::DESTROY => {
                PostQuitMessage(0);
                Ok(0)
            }
            _ => Ok(unsafe { hwnd.DefWindowProc(p) }),
        }
    }

    fn handle_command(&mut self, hwnd: &HWND, mut p: msg::wm::Command) -> anyhow::Result<isize> {
        match p.event.ctrl_id() {
            super::ID_EXIT => {
                log::info!("Exiting application...");
                for window in &self.windows {
                    window.close();
                }
                Ok(0)
            }
            _ => Ok(unsafe { hwnd.DefWindowProc(p.as_generic_wm()) }),
        }
    }

    fn handle_rbuttondown(
        &mut self,
        hwnd: &HWND,
        p: msg::wm::RButtonDown,
    ) -> anyhow::Result<isize> {
        log::info!("Handling WM_RBUTTONDOWN message");
        log::info!("Cursor at: ({}, {})", p.coords.x, p.coords.y);
        let mut menu = HMENU::CreatePopupMenu()?;
        menu.append_item(&[winsafe::MenuItem::Entry {
            cmd_id: super::ID_EXIT,
            text: "Quit",
        }])?;

        menu.track_popup_menu_at_point(p.coords, hwnd, hwnd)?;
        log::debug!("Menu displayed");
        menu.DestroyMenu()?;
        log::debug!("Menu destroyed");
        Ok(0)
    }

    fn handle_lbuttondown(
        &mut self,
        hwnd: &HWND,
        p: msg::wm::LButtonDown,
    ) -> anyhow::Result<isize> {
        log::info!("Handling WM_LBUTTONDOWN message");
        if let Some(idx) = self.workspace_at_x(hwnd, p.coords.x)? {
            log::info!("Switching to workspace {}", idx);
            komorebi_client::send_query(&SocketMessage::FocusWorkspaceNumber(idx))?;
        }
        Ok(0)
    }

    fn handle_setting_changed(&mut self, hwnd: &HWND) -> anyhow::Result<isize> {
        log::info!("Handling WM_SETTINGCHANGE message");
        self.settings = Settings::new()?;
        hwnd.SetLayeredWindowAttributes(
            self.settings.colors.get_color_key(),
            0,
            co::LWA::COLORKEY,
        )?;
        self.resize_to_fit(hwnd)?;
        hwnd.InvalidateRect(None, true)?;
        Ok(0)
    }

    pub fn handle_update_state(&mut self, state: Arc<State>) -> anyhow::Result<isize> {
        self.state = state;
        for window in &self.windows {
            log::info!("Updated state for window: {:?}", window.hwnd);
            self.resize_to_fit(&window.hwnd)?;
            window.hwnd.InvalidateRect(None, true)?;
        }
        Ok(0)
    }

    fn handle_create(&self) -> anyhow::Result<isize> {
        log::info!("Handling WM_CREATE message");
        Ok(0)
    }

    fn handle_paint(&self, hwnd: &HWND) -> anyhow::Result<isize> {
        self.paint_and_get_width(hwnd, true)?;
        Ok(0)
    }
}
