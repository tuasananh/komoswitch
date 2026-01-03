use winsafe::HWND;

#[derive(Debug)]
pub struct Window {
    pub hwnd: HWND,
    pub monitor_id: isize,
}

impl Window {
    pub fn close(&self) {
        log::info!("Closing window with hwnd: {:#?}", self.hwnd);
        unsafe {
            self.hwnd
                .PostMessage(winsafe::msg::WndMsg::new(winsafe::co::WM::CLOSE, 0, 0))
                .unwrap();
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        log::info!("Dropping Window with hwnd: {:#?}", self.hwnd);
        unsafe {
            self.hwnd.SetWindowLongPtr(winsafe::co::GWLP::USERDATA, 0);
        } // clear passed pointer
    }
}
