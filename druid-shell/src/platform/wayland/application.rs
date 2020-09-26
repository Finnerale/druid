//! Implementation of features at the application scope.

use crate::{
    application::AppHandler, error::Error, platform::clipboard::Clipboard, platform::window::Window,
};
use anyhow::Context;
use std::{cell::RefCell, rc::Rc, sync::Mutex};
use wayland_client::{
    protocol::{wl_compositor, wl_display, wl_seat, wl_shm},
    GlobalManager, Main,
};
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_wm_base};

#[derive(Clone)]
pub struct Application {
    pub(super) display: wayland_client::Display,
    pub(super) event_queue: Rc<Mutex<wayland_client::EventQueue>>,
    pub(super) globals: Rc<Globals>,
    pub(super) state: Rc<RefCell<State>>,
}

/// The mutable `Application` state.
#[derive(Default)]
pub(super) struct State {
    /// Whether `Application::quit` has already been called.
    pub(super) quitting: bool,
    /// A collection of all the `Application` windows.
    pub(super) windows: Vec<Rc<Window>>,
    /// Used to identify windows for debugging.
    pub(super) window_counter: u32,
}

pub(crate) struct Globals {
    pub shm: Main<wl_shm::WlShm>,
    pub compositor: Main<wl_compositor::WlCompositor>,
    pub seat: Main<wl_seat::WlSeat>,
    pub wm_base: Main<xdg_wm_base::XdgWmBase>,
}

impl Globals {
    pub fn new(manager: GlobalManager) -> anyhow::Result<Self> {
        let shm = manager
            .instantiate_exact::<wl_shm::WlShm>(1)
            .context("shm")?;
        let compositor = manager
            .instantiate_exact::<wl_compositor::WlCompositor>(4)
            .context("compositor")?;
        let seat = manager
            .instantiate_exact::<wl_seat::WlSeat>(5)
            .context("seat")?;
        let wm_base = manager
            .instantiate_exact::<xdg_wm_base::XdgWmBase>(1)
            .context("wm_base")?;
        Ok(Globals {
            shm,
            compositor,
            wm_base,
            seat,
        })
    }
}

impl Application {
    pub fn new() -> Result<Self, Error> {
        let display = wayland_client::Display::connect_to_env()
            .context("Could not connect to Wayland server")?;
        let mut event_queue = display.create_event_queue();
        let manager = GlobalManager::new(&display.attach(event_queue.token()));
        event_queue
            .sync_roundtrip(&mut (), |_, _, _| {})
            .context("First sync roundtrip failed")?;
        let globals = Globals::new(manager).context("Failed to acquire all globals")?;
        Ok(Application {
            display,
            globals: Rc::new(globals),
            event_queue: Rc::new(Mutex::new(event_queue)),
            state: Rc::new(RefCell::new(State::default())),
        })
    }

    pub fn run(self, handler: Option<Box<dyn AppHandler>>) {
        self.event_queue
            .lock()
            .unwrap()
            .sync_roundtrip(&mut (), |_, _, _| { /* we ignore unfiltered messages */ });
        loop {
            self.event_queue
                .lock()
                .unwrap()
                .dispatch(&mut (), |_, _, _| { /* we ignore unfiltered messages */ });
        }
    }

    pub fn quit(&self) {}

    pub fn clipboard(&self) -> Clipboard {
        Clipboard {}
    }

    pub fn get_locale() -> String {
        "en-US".into()
    }
}
