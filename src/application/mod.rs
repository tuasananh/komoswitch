use std::sync::Arc;

use komorebi_client::State;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use winsafe::{
    ATOM, AtomStr, DispatchMessage, EnumWindows, GetMessage, HINSTANCE, HWND, IdIdcStr, IdMenu,
    IsWindowsVistaOrGreater, MSG, POINT, RECT, RegisterClassEx, SIZE, SetLastError,
    TranslateMessage, WNDCLASSEX, WString, co, msg, prelude::*,
};

use crate::application::{settings::Settings, window::Window};
use crate::utils::{FromRect, RectContains};

mod handlers;
mod layout;
mod paint;
mod settings;
mod window;

const ID_EXIT: u16 = 1001;

const TEXT_PADDING: i32 = 20; // Padding around text in pixels
const BORDER_RADIUS: SIZE = SIZE { cx: 10, cy: 10 };

const TASKBAR_CLASS_NAME: &str = "Shell_TrayWnd";
const TASKBAR_SECONDARY_CLASS_NAME: &str = "Shell_SecondaryTrayWnd";

pub struct Application {
    pub windows: Vec<Window>,
    state: Arc<State>,
    settings: Settings,
}

impl Application {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            windows: Vec::new(),
            state: loop {
                let Ok(new_state) = crate::komo::read_state() else {
                    log::error!("Failed to read state, retrying...");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    continue;
                };
                break Arc::new(new_state);
            },
            settings: Settings::new()?,
        })
    }

    fn destroy(&mut self, hwnd: HWND) {
        self.windows.retain(|w| w.hwnd != hwnd);
    }

    fn register_class(&self, hinst: &HINSTANCE, class_name: &str) -> anyhow::Result<ATOM> {
        let mut wcx = WNDCLASSEX::default();
        wcx.lpfnWndProc = Some(Self::wnd_proc);
        wcx.hInstance = unsafe { hinst.raw_copy() };
        wcx.hCursor = HINSTANCE::NULL
            .LoadCursor(IdIdcStr::Idc(co::IDC::ARROW))?
            .leak();

        let mut wclass_name = if class_name.trim().is_empty() {
            WString::from_str(&format!(
                "WNDCLASS.{:#x}.{:#x}.{:#x}.{:#x}.{:#x}.{:#x}.{:#x}.{:#x}.{:#x}.{:#x}",
                wcx.style,
                wcx.lpfnWndProc.map_or(0, |p| p as usize),
                wcx.cbClsExtra,
                wcx.cbWndExtra,
                wcx.hInstance,
                wcx.hIcon,
                wcx.hCursor,
                wcx.hbrBackground,
                wcx.lpszMenuName(),
                wcx.hIconSm,
            ))
        } else {
            WString::from_str(class_name)
        };
        wcx.set_lpszClassName(Some(&mut wclass_name));

        SetLastError(co::ERROR::SUCCESS);
        match unsafe { RegisterClassEx(&wcx) } {
            Ok(atom) => Ok(atom),
            Err(err) => match err {
                co::ERROR::CLASS_ALREADY_EXISTS => {
                    let hinst = unsafe { wcx.hInstance.raw_copy() };
                    let (atom, _) = hinst.GetClassInfoEx(&wcx.lpszClassName().unwrap())?;
                    Ok(atom)
                }
                err => panic!("ERROR: Window::register_class: {}", err.to_string()),
            },
        }
    }

    fn create_window(
        &mut self,
        class_name: ATOM,
        pos: POINT,
        size: SIZE,
        hinst: &HINSTANCE,
    ) -> anyhow::Result<()> {
        log::info!("Creating window...");
        unsafe {
            HWND::CreateWindowEx(
                co::WS_EX::NOACTIVATE
                    | co::WS_EX::LAYERED
                    | co::WS_EX::TOOLWINDOW
                    | co::WS_EX::TOPMOST,
                AtomStr::Atom(class_name),
                None,
                co::WS::VISIBLE | co::WS::CLIPSIBLINGS | co::WS::POPUP,
                pos,
                size,
                None,
                IdMenu::None,
                hinst,
                Some(self as *const _ as _), // pass pointer to object itself
            )?
        };

        Ok(())
    }

    extern "system" fn wnd_proc(hwnd: HWND, msg: co::WM, wparam: usize, lparam: isize) -> isize {
        let wm_any = msg::WndMsg::new(msg, wparam, lparam);

        let ptr_self = match msg {
            co::WM::NCCREATE => {
                let msg = unsafe { msg::wm::NcCreate::from_generic_wm(wm_any) };
                let ptr_self = msg.createstruct.lpCreateParams as *mut Self;
                unsafe {
                    hwnd.SetWindowLongPtr(co::GWLP::USERDATA, ptr_self as _); // store
                }
                log::info!("HWND NCCREATE: {:#?}", hwnd);
                let ref_self = unsafe { &mut *ptr_self };
                if let Some(window) = ref_self.windows.last_mut() {
                    window.hwnd = hwnd;
                    return unsafe { window.hwnd.DefWindowProc(wm_any) }; // continue processing
                } else {
                    return unsafe { hwnd.DefWindowProc(wm_any) }; // continue processing
                }
            }
            _ => hwnd.GetWindowLongPtr(co::GWLP::USERDATA) as *mut Self, // retrieve
        };

        if ptr_self.is_null() {
            log::error!("Received message for uninitialized window: {:#?}", wm_any);
            return unsafe { hwnd.DefWindowProc(wm_any) };
        }

        let ref_self = unsafe { &mut *ptr_self };

        if msg == co::WM::NCDESTROY {
            log::info!("HWND NCDESTROY: {:#?}", hwnd);
            ref_self.destroy(hwnd);
            return 0;
        }

        ref_self
            .handle_message(&hwnd, wm_any)
            .unwrap_or_else(|err| {
                log::error!("Application error: {err}");
                0
            })
    }

    pub fn run_loop(&self) -> anyhow::Result<()> {
        let mut msg = MSG::default();
        while GetMessage(&mut msg, None, 0, 0)? {
            TranslateMessage(&msg);
            unsafe {
                DispatchMessage(&msg);
            }
        }
        Ok(())
    }

    pub fn prepare(&mut self) -> anyhow::Result<()> {
        if IsWindowsVistaOrGreater()? {
            // SetProcessDPIAware()?;
            unsafe {
                SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
            };
        }

        let hinstance = HINSTANCE::GetModuleHandle(None)?;

        let atom = self.register_class(&hinstance, "komoswitch")?;

        let taskbars = Self::find_all_taskbars()?;

        let monitors = self.state.monitors.clone();
        for monitor in monitors.elements().iter() {
            log::info!("Monitor found: {:#?}", monitor.id);
            log::info!("Monitor size: {:#?}", monitor.size);
            let Some(taskbar) = taskbars.iter().find(|tb| {
                RECT::from_rect(monitor.size).contains(&tb.GetWindowRect().unwrap_or_default())
            }) else {
                log::warn!(
                    "Failed to find taskbar for monitor: {}-{} {:?}",
                    monitor.name,
                    monitor.id,
                    monitor.size
                );
                continue;
            };
            let rect = taskbar.GetClientRect()?;
            self.windows.push(Window {
                hwnd: HWND::NULL,
                monitor_id: monitor.id,
            });
            self.create_window(
                atom,
                POINT { x: 15, y: 0 },
                SIZE {
                    cx: 0, // initially zero, will be resized later
                    cy: rect.bottom - rect.top,
                },
                &hinstance,
            )?;
            if let Some(last_window) = self.windows.last() {
                log::info!("Created window: {:?}", last_window);
                last_window.hwnd.SetParent(taskbar)?;
                last_window.hwnd.SetLayeredWindowAttributes(
                    self.settings.colors.get_color_key(),
                    0,
                    co::LWA::COLORKEY,
                )?;
            }
        }

        Ok(())
    }

    fn find_all_taskbars() -> anyhow::Result<Vec<HWND>> {
        let mut taskbars = Vec::new();
        EnumWindows(|hwnd: HWND| -> bool {
            let class_name = hwnd.GetClassName().unwrap_or_default();
            if class_name == TASKBAR_CLASS_NAME || class_name == TASKBAR_SECONDARY_CLASS_NAME {
                log::info!("Found taskbar window: {:?}", hwnd);
                let monitor = hwnd.MonitorFromWindow(co::MONITOR::DEFAULTTONEAREST);
                let monitor_info = monitor.GetMonitorInfo().unwrap();
                log::info!(
                    "RCWORK {} {} {} {}",
                    monitor_info.rcWork.left,
                    monitor_info.rcWork.top,
                    monitor_info.rcWork.right,
                    monitor_info.rcWork.bottom,
                );
                taskbars.push(hwnd);
            }
            true // continue enumeration
        })?;
        Ok(taskbars)
    }

    pub fn get_primary_hwnd(&self) -> anyhow::Result<&HWND> {
        if let Some(window) = self.windows.first() {
            Ok(&window.hwnd)
        } else {
            Err(anyhow::anyhow!("No windows available"))
        }
    }
}
