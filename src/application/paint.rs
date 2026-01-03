use crate::application::Application;
use komorebi_client::{DefaultLayout, Layout, Workspace};
use winsafe::{COLORREF, HBRUSH, HDC, HWND, RECT, co};

impl Application {
    pub(super) fn paint_and_get_width(&self, hwnd: &HWND, paint: bool) -> anyhow::Result<i32> {
        let hdc_guard = hwnd.BeginPaint()?;
        let hdc = &*hdc_guard;
        let _old_font = hdc.SelectObject(&self.settings.font)?;
        let _old_pen = hdc.SelectObject(&self.settings.transparent_pen)?;

        let rect = hwnd.GetClientRect()?;

        if paint {
            hdc.FillRect(rect, &self.settings.transparent_brush)?;
            hdc.SetTextColor(self.settings.colors.foreground)?;
            hdc.SetBkMode(co::BKMODE::TRANSPARENT)?;
        }

        let mut left = 0;
        if self.state.is_paused {
            self.draw_paused(hdc, &rect, &mut left, paint)?;
            return Ok(left);
        }

        let workspaces = self.workspaces(hwnd)?;
        let focused_idx = workspaces.focused_idx();
        for (idx, workspace) in workspaces.elements().iter().enumerate() {
            self.draw_workspace(hdc, &rect, idx, workspace, focused_idx, &mut left, paint)?;
        }

        if let Some(cw) = workspaces.focused() {
            let mut current_state = String::new();

            if let Some(hwnd) = komorebi_client::WindowsApi::foreground_window().ok() {
                if let Some(window) = cw.maximized_window() {
                    if hwnd == window.hwnd {
                        current_state = "Maximized".to_string();
                    }
                }
                if let Some(container) = cw.monocle_container() {
                    if container.contains_window(hwnd) {
                        current_state = "Monocle".to_string();
                    }
                }
            }

            if current_state.is_empty() {
                if matches!(cw.layout, Layout::Default(DefaultLayout::Scrolling)) {
                    let focused_idx = cw.containers.focused_idx();
                    let total_containers = cw.containers().len();

                    if total_containers > 1 {
                        left += super::TEXT_PADDING;

                        if total_containers >= 3 {
                            let text = if focused_idx > 1 { "•" } else { "" };
                            self.draw_small_box(
                                hdc,
                                &rect,
                                text,
                                0,
                                self.settings.colors.get_color_key(),
                                &mut left,
                                20,
                                paint,
                            )?;
                        }
                        if total_containers > 2 || (total_containers == 2 && focused_idx == 1) {
                            let text = if focused_idx > 0 {
                                (focused_idx).to_string()
                            } else {
                                "".to_string()
                            };
                            self.draw_small_box(
                                hdc,
                                &rect,
                                &text,
                                12,
                                if focused_idx > 0 {
                                    self.settings.colors.empty
                                } else {
                                    self.settings.colors.get_color_key()
                                },
                                &mut left,
                                16,
                                paint,
                            )?;
                        }
                        self.draw_small_box(
                            hdc,
                            &rect,
                            &(focused_idx + 1).to_string(),
                            16,
                            self.settings.colors.monocle,
                            &mut left,
                            14,
                            paint,
                        )?;
                        if total_containers >= 2 {
                            let text = if focused_idx + 1 < total_containers {
                                (focused_idx + 2).to_string()
                            } else {
                                "".to_string()
                            };
                            self.draw_small_box(
                                hdc,
                                &rect,
                                &text,
                                12,
                                if focused_idx + 1 < total_containers {
                                    self.settings.colors.empty
                                } else {
                                    self.settings.colors.get_color_key()
                                },
                                &mut left,
                                16,
                                paint,
                            )?;
                        }
                        if total_containers >= 3 {
                            let text = if focused_idx + 2 < total_containers {
                                "•"
                            } else {
                                ""
                            };
                            self.draw_small_box(
                                hdc,
                                &rect,
                                text,
                                0,
                                self.settings.colors.get_color_key(),
                                &mut left,
                                20,
                                paint,
                            )?;
                        }
                    }
                }
            } else {
                self.draw_current_state(hdc, &rect, &current_state, &mut left, paint)?;
            }
        }

        Ok(left)
    }

    fn draw_workspace(
        &self,
        hdc: &HDC,
        rect: &RECT,
        idx: usize,
        workspace: &Workspace,
        focused_idx: usize,
        left: &mut i32,
        paint: bool,
    ) -> anyhow::Result<()> {
        let workspace_name = workspace.name.clone().unwrap_or((idx + 1).to_string());
        let sz = hdc.GetTextExtentPoint32(&workspace_name)?;

        if paint {
            let text_rect = RECT {
                left: *left,
                right: *left + sz.cx + super::TEXT_PADDING * 2,
                top: 0,
                bottom: rect.bottom - 10,
            };
            hdc.DrawText(
                &workspace_name,
                text_rect,
                co::DT::CENTER | co::DT::VCENTER | co::DT::SINGLELINE,
            )?;

            let h_padding = if focused_idx == idx { 5 } else { 10 };

            let focused_rect = RECT {
                left: *left + h_padding,
                right: *left + sz.cx + super::TEXT_PADDING * 2 - h_padding,
                top: rect.bottom - 20,
                bottom: rect.bottom - 10,
            };

            let focused_brush = HBRUSH::CreateSolidBrush(if focused_idx == idx {
                self.settings.colors.focused
            } else if workspace.is_empty() {
                self.settings.colors.empty
            } else {
                self.settings.colors.nonempty
            })?;
            let _old_brush = hdc.SelectObject(&*focused_brush);
            hdc.RoundRect(focused_rect, super::BORDER_RADIUS)?;
        }

        *left += sz.cx + super::TEXT_PADDING * 2;
        Ok(())
    }

    fn draw_paused(
        &self,
        hdc: &HDC,
        rect: &RECT,
        left: &mut i32,
        paint: bool,
    ) -> anyhow::Result<()> {
        let pause_text = "Paused";
        let sz = hdc.GetTextExtentPoint32(pause_text)?;

        if paint {
            let text_rect = RECT {
                left: *left,
                right: *left + sz.cx + super::TEXT_PADDING * 2,
                top: rect.top + 12,
                bottom: rect.bottom - 12,
            };

            let focused_brush = HBRUSH::CreateSolidBrush(self.settings.colors.empty)?;
            let _old_brush = hdc.SelectObject(&*focused_brush);
            hdc.RoundRect(text_rect, super::BORDER_RADIUS)?;
            hdc.DrawText(
                pause_text,
                text_rect,
                co::DT::CENTER | co::DT::VCENTER | co::DT::SINGLELINE,
            )?;
        }

        *left += sz.cx + super::TEXT_PADDING * 2;
        Ok(())
    }

    fn draw_current_state(
        &self,
        hdc: &HDC,
        rect: &RECT,
        current_state: &str,
        left: &mut i32,
        paint: bool,
    ) -> anyhow::Result<()> {
        let sz = hdc.GetTextExtentPoint32(current_state)?;
        if paint {
            let text_rect = RECT {
                left: *left,
                right: *left + sz.cx + super::TEXT_PADDING * 2,
                top: rect.top + 12,
                bottom: rect.bottom - 12,
            };

            let focused_brush = HBRUSH::CreateSolidBrush(if current_state == "Maximized" {
                self.settings.colors.maximized
            } else {
                self.settings.colors.monocle
            })?;
            let _old_brush = hdc.SelectObject(&*focused_brush);
            hdc.RoundRect(text_rect, super::BORDER_RADIUS)?;
            hdc.DrawText(
                current_state,
                text_rect,
                co::DT::CENTER | co::DT::VCENTER | co::DT::SINGLELINE,
            )?;
        }

        *left += sz.cx + super::TEXT_PADDING * 2;
        Ok(())
    }

    fn draw_small_box(
        &self,
        hdc: &HDC,
        rect: &RECT,
        text: &str,
        padding: i32,
        bg_color: COLORREF,
        left: &mut i32,
        v_padding: i32,
        paint: bool,
    ) -> anyhow::Result<()> {
        const TEXT_WIDTH: i32 = 20;
        if paint {
            let text_rect = RECT {
                left: *left,
                right: *left + TEXT_WIDTH + padding * 2,
                top: rect.top + v_padding,
                bottom: rect.bottom - v_padding,
            };

            let focused_brush = HBRUSH::CreateSolidBrush(bg_color)?;
            let _old_brush = hdc.SelectObject(&*focused_brush);
            hdc.RoundRect(text_rect, super::BORDER_RADIUS)?;
            if !text.is_empty() {
                hdc.DrawText(
                    text,
                    text_rect,
                    co::DT::CENTER | co::DT::VCENTER | co::DT::SINGLELINE,
                )?;
            }
        }

        *left += TEXT_WIDTH + padding * 2;

        Ok(())
    }
}
