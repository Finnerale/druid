use crate::{
    error::Error as ShellError,
    kurbo::{Point, Rect, Size},
    piet::PietText,
    platform::{menu::Menu, timer::Timer, window::IdleHandle, window::Window},
    window::FileDialogToken,
    window, Cursor, CursorDesc, FileDialogOptions, FileInfo, Scale, TimerToken, WindowLevel,
};
use std::{rc::Weak, sync::Arc, time::Instant};

#[derive(Clone, Default)]
pub struct WindowHandle {
    id: u32,
    window: Weak<Window>,
}

impl WindowHandle {
    pub(super) fn new(id: u32, window: Weak<Window>) -> WindowHandle {
        WindowHandle { id, window }
    }

    pub fn show(&self) {
        if let Some(w) = self.window.upgrade() {
            w.show();
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn close(&self) {
        if let Some(w) = self.window.upgrade() {
            w.close();
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn resizable(&self, resizable: bool) {
        if let Some(w) = self.window.upgrade() {
            w.resizable(resizable);
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn show_titlebar(&self, show_titlebar: bool) {
        if let Some(w) = self.window.upgrade() {
            w.show_titlebar(show_titlebar);
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn set_position(&self, _position: Point) {}

    pub fn get_position(&self) -> Point {
        Point::new(0.0, 0.0)
    }

    pub fn set_level(&self, _level: WindowLevel) {}

    pub fn set_size(&self, _size: Size) {}

    pub fn get_size(&self) -> Size {
        Size::new(0.0, 0.0)
    }

    pub fn set_window_state(&self, _state: window::WindowState) {}

    pub fn get_window_state(&self) -> window::WindowState {
        window::WindowState::RESTORED
    }

    pub fn handle_titlebar(&self, _val: bool) {}

    pub fn bring_to_front_and_focus(&self) {
        if let Some(w) = self.window.upgrade() {
            w.bring_to_front_and_focus();
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn request_anim_frame(&self) {
        if let Some(w) = self.window.upgrade() {
            w.request_anim_frame();
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn invalidate(&self) {
        if let Some(w) = self.window.upgrade() {
            w.invalidate();
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn invalidate_rect(&self, rect: Rect) {
        if let Some(w) = self.window.upgrade() {
            w.invalidate_rect(rect);
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn set_title(&self, title: &str) {
        if let Some(w) = self.window.upgrade() {
            w.set_title(title);
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn set_menu(&self, menu: Menu) {
        if let Some(w) = self.window.upgrade() {
            w.set_menu(menu);
        } else {
            log::error!("Window {} has already been dropped", self.id);
        }
    }

    pub fn text(&self) -> PietText {
        PietText::new()
    }

    pub fn request_timer(&self, deadline: Instant) -> TimerToken {
        if let Some(w) = self.window.upgrade() {
            let timer = Timer::new(deadline);
            w.timer_queue.lock().unwrap().push(timer);
            timer.token()
        } else {
            TimerToken::INVALID
        }
    }

    pub fn set_cursor(&mut self, _cursor: &Cursor) {}

    pub fn make_cursor(&self, _cursor_desc: &CursorDesc) -> Option<Cursor> {
        None
    }

    pub fn open_file(&mut self, _options: FileDialogOptions) -> Option<FileDialogToken> {
        None
    }

    pub fn open_file_sync(&mut self, _options: FileDialogOptions) -> Option<FileInfo> {
        None
    }

    pub fn save_as_sync(&mut self, _options: FileDialogOptions) -> Option<FileInfo> {
        None
    }

    pub fn save_as(&mut self, _options: FileDialogOptions) -> Option<FileDialogToken> {
        None
    }

    pub fn show_context_menu(&self, _menu: Menu, _pos: Point) {}

    pub fn get_idle_handle(&self) -> Option<IdleHandle> {
        if let Some(w) = self.window.upgrade() {
            Some(IdleHandle {
                queue: Arc::clone(&w.idle_queue),
            })
        } else {
            None
        }
    }

    pub fn get_scale(&self) -> Result<Scale, ShellError> {
        if let Some(w) = self.window.upgrade() {
            Ok(w.get_scale()?)
        } else {
            log::error!("Window {} has already been dropped", self.id);
            Ok(Scale::new(1.0, 1.0))
        }
    }
}
