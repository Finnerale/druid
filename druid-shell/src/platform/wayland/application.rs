#![allow(dead_code)]

//! Implementation of features at the application scope.

use crate::{
    application::AppHandler, error::Error, platform::clipboard::Clipboard,
    platform::window::Window, MouseEvent, WinHandler
};
use anyhow::{Context, Result};
use std::{cell::RefCell, rc::Rc, sync::Mutex};
use wayland_client::{
    protocol::{wl_compositor, wl_keyboard, wl_pointer, wl_seat, wl_shm, wl_surface},
    GlobalManager, Main,
};
use wayland_protocols::xdg_shell::client::xdg_wm_base;

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

mod events {
    use wayland_client::event_enum;
    use wayland_client::protocol::{wl_keyboard, wl_pointer};
    event_enum! {
        Events |
        Pointer => wl_pointer::WlPointer,
        Keyboard => wl_keyboard::WlKeyboard
    }
}
use events::Events;

impl Globals {
    pub fn new(manager: GlobalManager) -> anyhow::Result<Self> {
        let shm = manager
            .instantiate_exact::<wl_shm::WlShm>(1)
            .context("shm")?;
        let compositor = manager
            .instantiate_exact::<wl_compositor::WlCompositor>(4)
            .context("compositor")?;
        let seat = manager
            .instantiate_exact::<wl_seat::WlSeat>(1)
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
        let app = Application {
            display,
            globals: Rc::new(globals),
            event_queue: Rc::new(Mutex::new(event_queue)),
            state: Rc::new(RefCell::new(State::default())),
        };
        // Doing this in `run` seems better, but then it's too late.
        app.assign_filter();
        Ok(app)
    }

    pub fn run(self, _handler: Option<Box<dyn AppHandler>>) {
        if let Err(err) = self.inner_run() {
            eprintln!("Application::run failed: {}", err);
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

impl Application {
    fn assign_filter(&self) {
        let input_filter = event_filter(self.clone());
        let mut pointer_created = false;
        let mut keyboard_created = false;
        println!("Quick assign to seat");
        self.globals.seat.quick_assign(move |seat, event, _| {
            if let wl_seat::Event::Capabilities { capabilities } = event {
                println!("Received capabilities");
                if !pointer_created && capabilities.contains(wl_seat::Capability::Pointer) {
                    pointer_created = true;
                    seat.get_pointer().assign(input_filter.clone());
                }
                if !keyboard_created && capabilities.contains(wl_seat::Capability::Keyboard) {
                    keyboard_created = true;
                    seat.get_pointer().assign(input_filter.clone());
                }
            }
        });
    }

    pub(crate) fn roundtrip(&self) -> Result<()> {
        self.event_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock Application::event_queue"))?
            .sync_roundtrip(&mut (), |_, _, _| { /* we ignore unfiltered messages */ })?;
        Ok(())
    }

    fn inner_run(&self) -> Result<()> {
        self.roundtrip()?;
        loop {
            self.event_queue
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock Application::event_queue"))?
                .dispatch(&mut (), |_, _, _| { /* we ignore unfiltered messages */ })?;
        }
    }

    fn with_window_handler(&self, focused: wl_surface::WlSurface, mut function: impl FnMut(&mut dyn WinHandler)) -> Result<()> {
        let state = borrow!(self.state)?;
        for window in &state.windows {
            if window.wl_surface.detach() == focused {
                let mut handler = borrow_mut!(window.handler)?;
                function(handler.as_mut());
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("Could not find window handler for window {:?}", focused))
    }
}

fn event_filter(app: Application) -> wayland_client::Filter<Events> {
    let mut handler = EventHandler::default();
    wayland_client::Filter::new(move |event, _, _| {
        if let Err(err) = handler.handle(event, &app) {
            log::error!("Failed to handle event: {}", err);
        }
    })
}

#[derive(Default)]
struct EventHandler {
    focused: Option<wl_surface::WlSurface>,
    cursor: kurbo::Point,
}

impl EventHandler {
    pub fn handle(&mut self, event: Events, app: &Application) -> Result<()> {
        match event {
            Events::Pointer { event, .. } => match event {
                wl_pointer::Event::Enter {
                    surface,
                    surface_x,
                    surface_y,
                    ..
                } => {
                    self.focused = Some(surface);
                    self.cursor = kurbo::Point::new(surface_x, surface_y);
                }
                wl_pointer::Event::Leave { .. } => {
                    self.focused = None;
                }
                wl_pointer::Event::Motion {
                    surface_x,
                    surface_y,
                    ..
                } => {
                    self.cursor = kurbo::Point::new(surface_x, surface_y);
                    if let Some(focused) = self.focused.clone() {
                        app.with_window_handler(focused, |handler| {
                            handler.mouse_move(&MouseEvent {
                                pos: self.cursor,
                                buttons: crate::mouse::MouseButtons::default(),
                                mods: crate::Modifiers::default(),
                                count: 0,
                                focus: true,
                                button: crate::mouse::MouseButton::None,
                                wheel_delta: kurbo::Vec2::ZERO,
                            });
                        })?;
                    }
                }
                wl_pointer::Event::Button {
                    button: _,
                    state: button_state,
                    ..
                } => {
                    if let Some(focused) = self.focused.clone() {
                        app.with_window_handler(focused, |handler| {
                            let event = MouseEvent {
                                pos: self.cursor,
                                buttons: crate::mouse::MouseButtons::default()
                                    .with(crate::mouse::MouseButton::Left),
                                mods: crate::Modifiers::default(),
                                count: 1,
                                focus: true,
                                button: crate::mouse::MouseButton::Left,
                                wheel_delta: kurbo::Vec2::ZERO,
                            };
                            match button_state {
                                wl_pointer::ButtonState::Pressed => {
                                    handler.mouse_down(&event);
                                }
                                wl_pointer::ButtonState::Released => {
                                    handler.mouse_up(&event);
                                }
                                _ => unimplemented!(),
                            }
                        })?;
                    }
                }
                _ => {}
            },
            Events::Keyboard { event, .. } => match event {
                wl_keyboard::Event::Enter { .. } => {
                    println!("Gained keyboard focus.");
                }
                wl_keyboard::Event::Leave { .. } => {
                    println!("Lost keyboard focus.");
                }
                wl_keyboard::Event::Key { key, state, .. } => {
                    println!("Key with id {} was {:?}.", key, state);
                }
                _ => {}
            },
        }
        Ok(())
    }
}
