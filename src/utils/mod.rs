use komorebi_client::Rect;
use winsafe::RECT;
pub trait RectContains {
    fn contains(&self, other: &RECT) -> bool;
}

impl RectContains for RECT {
    fn contains(&self, other: &RECT) -> bool {
        log::info!(
            "Checking if RECT {} {} {} {} contains RECT {} {} {} {}",
            self.left,
            self.top,
            self.right,
            self.bottom,
            other.left,
            other.top,
            other.right,
            other.bottom
        );
        self.left <= other.left
            && self.top <= other.top
            && self.right >= other.right
            && self.bottom >= other.bottom
    }
}

pub trait FromRect {
    fn from_rect(rect: Rect) -> RECT;
}

impl FromRect for RECT {
    fn from_rect(rect: Rect) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            right: rect.left + rect.right,
            bottom: rect.top + rect.bottom,
        }
    }
}
