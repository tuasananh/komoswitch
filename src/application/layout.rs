use crate::application::Application;
use komorebi_client::{Ring, Workspace};
use winsafe::{HWND, POINT, SIZE, co};

impl Application {
    fn get_monitor_id(&self, hwnd: &HWND) -> anyhow::Result<isize> {
        self.windows
            .iter()
            .find(|w| &w.hwnd == hwnd)
            .map(|w| w.monitor_id)
            .ok_or_else(|| anyhow::anyhow!("No monitor ID found for hwnd: {:#?}", hwnd))
    }

    pub(super) fn workspaces(&self, hwnd: &HWND) -> anyhow::Result<&Ring<Workspace>> {
        let monitor_id = self.get_monitor_id(hwnd)?;
        let workspaces = &self
            .state
            .monitors
            .elements()
            .iter()
            .find(|m| m.id == monitor_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No monitor found with ID {} in state: {:#?}",
                    monitor_id,
                    self.state
                )
            })?
            .workspaces;
        Ok(workspaces)
    }

    pub(super) fn workspace_at_x(&self, hwnd: &HWND, x: i32) -> anyhow::Result<Option<usize>> {
        let hdc = hwnd.GetDC()?;
        let rect = hwnd.GetClientRect()?;
        let _old_font = hdc.SelectObject(&*self.get_font(rect.bottom - rect.top)?)?;
        let workspaces = self.workspaces(hwnd)?;
        let focused_idx = workspaces.focused_idx();
        let text_padding = self.get_text_padding(rect.bottom - rect.top);
        let mut left = 0;

        for (idx, workspace) in workspaces.elements().iter().enumerate() {
            let workspace_name = workspace.name.clone().unwrap_or((idx + 1).to_string());
            let sz = hdc.GetTextExtentPoint32(&workspace_name)?;

            let h_padding = self.get_h_padding(rect.bottom - rect.top, focused_idx == idx);

            let rect_left = left + h_padding;
            let rect_right = left + sz.cx + text_padding * 2 - h_padding;

            if x >= rect_left && x <= rect_right {
                return Ok(Some(idx));
            }

            left += sz.cx + text_padding * 2;
        }
        Ok(None)
    }

    pub(super) fn resize_to_fit(&self, hwnd: &HWND) -> anyhow::Result<bool> {
        let total_width = self.paint_and_get_width(hwnd, false)?;

        let rect = hwnd.GetClientRect()?;

        if rect.right - rect.left == total_width {
            return Ok(false);
        }

        hwnd.SetWindowPos(
            winsafe::HwndPlace::Place(co::HWND_PLACE::default()),
            POINT::default(),
            SIZE {
                cx: total_width,
                cy: rect.bottom - rect.top,
            },
            co::SWP::NOACTIVATE | co::SWP::NOZORDER | co::SWP::NOMOVE | co::SWP::NOREDRAW,
        )?;

        Ok(true)
    }
}
