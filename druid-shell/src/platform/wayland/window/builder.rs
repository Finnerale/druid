use crate::{
    kurbo::{Point, Size},
    platform::{application::Application, menu::Menu, window::WindowHandle},
    window, Error, WinHandler,
};

pub struct WindowBuilder {
    app: Application,
    handler: Option<Box<dyn WinHandler>>,
    title: String,
    size: Size,
    min_size: Size,
}

impl WindowBuilder {
    pub fn new(app: Application) -> WindowBuilder {
        WindowBuilder {
            app,
            handler: None,
            title: String::new(),
            size: Size::new(500.0, 400.0),
            min_size: Size::new(0.0, 0.0),
        }
    }

    pub fn set_handler(&mut self, handler: Box<dyn WinHandler>) {
        self.handler = Some(handler);
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    pub fn set_min_size(&mut self, min_size: Size) {
        self.min_size = min_size;
    }

    pub fn resizable(&mut self, _resizable: bool) {}

    pub fn show_titlebar(&mut self, _show_titlebar: bool) {}

    pub fn set_position(&mut self, _position: Point) {}

    pub fn set_level(&mut self, _level: window::WindowLevel) {}

    pub fn set_window_state(&self, _state: window::WindowState) {}

    pub fn set_title<S: Into<String>>(&mut self, title: S) {
        self.title = title.into();
    }

    pub fn set_menu(&mut self, _menu: Menu) {}

    pub fn build(self) -> Result<WindowHandle, Error> {
        unimplemented!()
    }
}
