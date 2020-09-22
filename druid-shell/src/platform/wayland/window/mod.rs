use crate::{
    kurbo::{Rect, Size},
    platform::{menu::Menu, timer::Timer},
    Application, Error, Region, Scale, WinHandler,
};
use std::{
    cell::RefCell,
    collections::BinaryHeap,
    sync::{Arc, Mutex},
};

mod builder;
pub use builder::WindowBuilder;

mod handle;
pub use handle::WindowHandle;

mod idle;
pub use idle::IdleHandle;
pub(crate) use idle::IdleKind;

/// The mutable state of the window.
struct WindowState {
    size: Size,
    /// The region that was invalidated since the last time we rendered.
    invalid: Region,
    /// We've told Wayland to destroy this window, so don't so any more X requests with this window id.
    destroyed: bool,
}

pub(crate) struct Window {
    id: u32,
    app: Application,
    handler: RefCell<Box<dyn WinHandler>>,
    state: RefCell<WindowState>,
    /// Timers, sorted by "earliest deadline first"
    timer_queue: Mutex<BinaryHeap<Timer>>,
    idle_queue: Arc<Mutex<Vec<IdleKind>>>,
}

impl Window {
    pub fn show(&self) {
        unimplemented!()
    }

    pub fn close(&self) {
        unimplemented!()
    }

    pub fn resizable(&self, resizable: bool) {}

    pub fn show_titlebar(&self, show_titlebar: bool) {}

    pub fn bring_to_front_and_focus(&self) {}

    pub fn request_anim_frame(&self) {
        unimplemented!()
    }

    pub fn invalidate(&self) {
        unimplemented!()
    }

    pub fn invalidate_rect(&self, rect: Rect) {
        unimplemented!()
    }

    pub fn set_title(&self, title: &str) {}

    pub fn set_menu(&self, menu: Menu) {}

    pub fn get_scale(&self) -> Result<Scale, Error> {
        Ok(Scale::default())
    }
}
