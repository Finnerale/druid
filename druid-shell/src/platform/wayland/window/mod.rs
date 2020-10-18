#![allow(dead_code)]

use crate::{
    kurbo::{Rect, Size},
    piet::{Piet, RenderContext},
    platform::{menu::Menu, timer::Timer},
    Error, Region, Scale, WinHandler,
};
use anyhow::{anyhow, Result};
use cairo::ImageSurface;
use std::os::unix::io::AsRawFd;
use std::{
    cell::RefCell,
    collections::BinaryHeap,
    rc,
    sync::{atomic, atomic::AtomicBool, Arc, Mutex},
};
use wayland_client::{
    protocol::{wl_buffer, wl_callback, wl_shm, wl_shm_pool, wl_surface},
    Main,
};
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_toplevel};

mod builder;
pub use builder::WindowBuilder;

mod handle;
pub use handle::WindowHandle;

mod idle;
pub use idle::IdleHandle;
pub(crate) use idle::IdleKind;

use super::application::Application;

/// The mutable state of the window.
#[derive(Default)]
pub(super) struct WindowState {
    size: Size,
    /// The region that was invalidated since the last time we rendered.
    invalid: Region,
    /// We've told Wayland to destroy this window, so don't so any more X requests with this window id.
    destroyed: bool,
}

pub(crate) struct Window {
    pub(crate) id: u32,
    pub(crate) this: RefCell<rc::Weak<Window>>,
    pub(super) app: Application,
    pub(super) handler: RefCell<Box<dyn WinHandler>>,
    pub(super) state: RefCell<WindowState>,
    pub(super) configured: AtomicBool,

    pub(super) cairo_surface: RefCell<ImageSurface>,
    pub(super) pool_handle: RefCell<Option<wl_shm_pool::WlShmPool>>,
    pub(super) buffer_handle: RefCell<Option<wl_buffer::WlBuffer>>,
    pub(super) wl_surface: Main<wl_surface::WlSurface>,
    pub(super) xdg_surface: Main<xdg_surface::XdgSurface>,
    pub(super) xdg_toplevel: Main<xdg_toplevel::XdgToplevel>,

    /// Timers, sorted by "earliest deadline first"
    pub(super) timer_queue: Mutex<BinaryHeap<Timer>>,
    pub(super) idle_queue: Arc<Mutex<Vec<IdleKind>>>,
}

#[derive(Clone)]
pub struct CustomCursor();

impl Window {
    pub fn show(&self) {
        if let Err(e) = self.render() {
            eprintln!("{}", e);
            return;
        }
        println!("Show finished 'successfully'");
    }

    pub fn close(&self) {
        eprintln!("Can't close windows yet!");
    }

    pub fn resizable(&self, _resizable: bool) {}

    pub fn show_titlebar(&self, _show_titlebar: bool) {}

    pub fn bring_to_front_and_focus(&self) {}

    pub fn request_anim_frame(&self) {
        let callback = self.wl_surface.frame();
        let this = borrow!(self.this).unwrap().clone();
        callback.quick_assign(move |_, event, _| {
            if let wl_callback::Event::Done { .. } = event {
                if let Some(this) = this.upgrade() {
                    if let Err(err) = this.render() {
                        log::error!("{}", err);
                    }
                }
            }
        });
        self.wl_surface.commit();
    }

    fn invalidate(&self) {
        match borrow!(self.state).map(|state| state.size.to_rect()) {
            Ok(rect) => self.invalidate_rect(rect),
            Err(err) => log::error!("Window::invalidate - failed to get state: {}", err),
        }
    }

    fn invalidate_rect(&self, rect: Rect) {
        if let Err(err) = self.add_invalid_rect(rect) {
            log::error!("Window::invalidate_rect - failed to enlarge rect: {}", err);
        }
        self.request_anim_frame();
    }

    pub fn set_title(&self, _title: &str) {}

    pub fn set_menu(&self, _menu: Menu) {}

    pub fn get_scale(&self) -> Result<Scale, Error> {
        Ok(Scale::default())
    }
}

impl Window {
    fn add_invalid_rect(&self, rect: Rect) -> Result<(), Error> {
        borrow_mut!(self.state)?.invalid.add_rect(rect.expand());
        Ok(())
    }

    pub fn render(&self) -> Result<()> {
        if !self.configured.load(atomic::Ordering::Acquire) {
            return Ok(());
        }
        borrow_mut!(self.handler)?.prepare_paint();
        self.update_surface()?;
        {
            let state = borrow!(self.state)?;
            let surface = borrow!(self.cairo_surface)?;
            let cairo_ctx = cairo::Context::new(&surface);

            for rect in state.invalid.rects() {
                cairo_ctx.rectangle(rect.x0, rect.y0, rect.width(), rect.height());
            }
            cairo_ctx.clip();

            let mut piet_ctx = Piet::new(&cairo_ctx);

            let err;
            match borrow_mut!(self.handler) {
                Ok(mut handler) => {
                    handler.paint(&mut piet_ctx, &state.invalid);
                    err = piet_ctx
                        .finish()
                        .map_err(|e| anyhow!("Window::render - piet finish failed: {}", e));
                }
                Err(e) => {
                    err = Err(e);
                    if let Err(e) = piet_ctx.finish() {
                        // We can't return both errors, so just log this one.
                        log::error!("Window::render - piet finish failed in error branch: {}", e);
                    }
                }
            };
            cairo_ctx.reset_clip();

            err?;
        }
        self.update_buffer()?;
        Ok(())
    }

    fn update_surface(&self) -> Result<()> {
        let size = borrow!(self.state)?.size;
        borrow_mut!(self.state)?.invalid.add_rect(size.to_rect());
        let buf_len = (size.width * size.height) as u64 * 4;
        let file = tempfile::tempfile()?;
        file.set_len(buf_len)?;
        let mem = unsafe {
            memmap::MmapOptions::new()
                .len(buf_len as usize)
                .map_mut(&file)?
        };
        let format = cairo::Format::ARgb32;
        let stride = format
            .stride_for_width(size.width as u32)
            .map_err(|_| anyhow!("Could not get Cairo stride"))?;
        let cairo_surface = ImageSurface::create_for_data(
            mem,
            format,
            size.width as i32,
            size.height as i32,
            stride,
        )?;
        self.cairo_surface.replace(cairo_surface);

        let pool = self
            .app
            .globals
            .shm
            .create_pool(file.as_raw_fd(), buf_len as i32);
        let buffer = pool.create_buffer(
            0,
            size.width as i32,
            size.height as i32,
            size.width as i32 * 4,
            wl_shm::Format::Argb8888,
        );
        self.pool_handle.replace(Some(pool.detach()));
        self.buffer_handle.replace(Some(buffer.detach()));
        Ok(())
    }

    fn update_buffer(&self) -> Result<()> {
        let size = borrow!(self.state)?.size;
        let wl_surface = &self.wl_surface;
        let handle = borrow!(self.buffer_handle)?;
        let buffer: &wl_buffer::WlBuffer = handle.as_ref().unwrap();
        wl_surface.attach(Some(buffer), 0, 0);
        wl_surface.damage_buffer(0, 0, size.width as i32, size.height as i32);
        wl_surface.commit();
        Ok(())
    }
}
