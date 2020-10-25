use crate::{
    kurbo::{Point, Size},
    platform::{application::Application, menu::Menu, window::WindowHandle},
    window, Error, WinHandler,
};
use anyhow::Context;
use std::{
    cell::RefCell,
    collections::BinaryHeap,
    rc::{self, Rc},
    sync::{
        atomic::{self, AtomicBool},
        Arc, Mutex,
    },
};
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

use super::{Window, WindowState};

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
        let id = self.app.state.borrow_mut().window_counter;
        self.app.state.borrow_mut().window_counter += 1;

        let wl_surface = self.app.globals.compositor.create_surface();
        let xdg_surface = self.app.globals.wm_base.get_xdg_surface(&*wl_surface);
        let xdg_toplevel = xdg_surface.get_toplevel();
        self.app.globals.wm_base.quick_assign(|wm_base, event, _| {
            use xdg_wm_base::Event;
            if let Event::Ping { serial } = event {
                wm_base.pong(serial);
            }
        });
        xdg_toplevel.set_title("Wayplay".to_string());
        xdg_toplevel.set_app_id("wayplay".to_string());

        let cairo_surface = RefCell::new(
            cairo::ImageSurface::create(cairo::Format::ARgb32, 0, 0)
                .context("Could not create empty Cairo surface")?,
        );
        let pool_handle = RefCell::new(None);
        let buffer_handle = RefCell::new(None);
        let timer_queue = Mutex::new(BinaryHeap::new());
        let idle_queue = Arc::new(Mutex::new(Vec::new()));

        let handler = RefCell::new(
            self.handler
                .ok_or_else(|| anyhow::anyhow!("Handler must be set."))?,
        );

        let app = self.app.clone();

        let mut state = WindowState::default();
        state.size = self.size;
        let state = RefCell::new(state);
        let configured = AtomicBool::new(false);
        let frame_requested = AtomicBool::new(false);

        let this = RefCell::new(rc::Weak::new());

        let window = Window {
            id,
            this,
            app,
            handler,
            state,
            configured,

            frame_requested,
            cairo_surface,
            pool_handle,
            buffer_handle,
            wl_surface,
            xdg_surface,
            xdg_toplevel,

            timer_queue,
            idle_queue,
        };
        let window = Rc::new(window);
        window.this.replace(Rc::downgrade(&window));

        let handle = WindowHandle::new(id, Rc::downgrade(&window));
        window.xdg_surface.quick_assign({
            let window = window.clone();
            let shell_handle = crate::WindowHandle::from(handle.clone());
            let size = self.size;
            move |xdg_surface, event, _| {
                use xdg_surface::Event;
                if let Event::Configure { serial } = event {
                    xdg_surface.ack_configure(serial);
                    if !window.configured.swap(true, atomic::Ordering::Release) {
                        borrow_mut!(window.handler).unwrap().connect(&shell_handle);
                        borrow_mut!(window.handler).unwrap().size(size);
                        if let Err(err) = window.render() {
                            log::error!("Failed to present window {}: {}", window.id, err);
                        }
                    }
                }
            }
        });
        window.xdg_toplevel.quick_assign({
            let window = window.clone();
            let shell_handle = crate::WindowHandle::from(handle.clone());
            move |_, event, _| {
                use xdg_toplevel::Event;
                match event {
                    Event::Configure { width, height, .. } => {
                        if width > 0 && height > 0 {
                            let size = Size::new(width as f64, height as f64);
                            if let Ok(mut state) = window.state.try_borrow_mut() {
                                state.size = dbg!(size);
                            }
                            borrow_mut!(window.handler).unwrap().size(size);
                            window.render();
                        }
                    }
                    Event::Close => {
                        window.close();
                    }
                    _ => {}
                }
            }
        });
        window.wl_surface.commit();

        self.app.state.borrow_mut().windows.push(window);

        Ok(handle)
    }
}
