// Copyright 2019 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use druid::widget::prelude::*;
use druid::widget::{
    Container, Controller, CrossAxisAlignment, Flex, Immediate, Label, List, MainAxisAlignment,
    Scroll, WidgetExt,
};
use druid::{
    theme, AppLauncher, Command, Data, Event, KeyEvent, Lens, LocalizedString, MouseEvent,
    Selector, WindowDesc,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

const SIZE_EVENT: Selector = Selector::new("event_log.size_event");

#[derive(Clone, Data, Lens)]
struct EventLogger {
    id_counter: usize,
    events: Arc<Vec<EventSeries>>,
}

type EventSeries = Arc<Vec<EventEntry>>;

#[derive(Clone, Data, Lens)]
struct EventEntry {
    id: usize,
    #[druid(ignore)]
    time: Instant,
    #[druid(ignore)]
    event: Event,
}

struct SizeEventForwarder(WidgetId);

impl<T, W: Widget<T>> Controller<T, W> for SizeEventForwarder {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::Size(size) => ctx.submit_command(Command::new(SIZE_EVENT, size.clone()), self.0),
            _ => (),
        }
        child.event(ctx, event, data, env);
    }
}

struct CaptureArea<T> {
    content: Container<T>,
    on_capture: Box<dyn Fn(&Event, &mut T)>,
}

fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_logger)
        .title(LocalizedString::new("event-logger-title").with_placeholder("Event Logger"))
        .window_size((400.0, 600.0));

    // create the initial app state
    let initial_state = EventLogger {
        id_counter: 0,
        events: Arc::new(vec![Arc::new(Vec::new())]),
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_logger() -> impl Widget<EventLogger> {
    let capture_area_id = WidgetId::next();
    let capture_area = CaptureArea::new(|event: &Event, data: &mut EventLogger| {
        const DELAY: Duration = Duration::from_secs(1);
        if let Some(previous) = data.events.last().unwrap().last() {
            if Instant::now().saturating_duration_since(previous.time) >= DELAY {
                // Add new `Series`
                Arc::make_mut(&mut data.events).push(Arc::new(Vec::new()));
            }
        }
        let entry = EventEntry {
            id: data.id_counter,
            time: Instant::now(),
            event: event.clone(),
        };
        // TODO: This is madness
        Arc::make_mut(Arc::make_mut(&mut data.events).last_mut().unwrap()).push(entry);
        data.id_counter += 1;
    })
    .with_id(capture_area_id);

    let list = Scroll::new(List::new(|| {
        Flex::column()
            .with_child(List::new(|| Immediate::new(event_view)))
            .with_spacer(30.0)
    }))
    .vertical()
    .expand_width()
    .lens(EventLogger::events);

    // arrange the two widgets vertically, with some padding
    let layout = Flex::column()
        .with_child(capture_area.padding(10.0).expand_width().fix_height(200.0))
        .with_spacer(10.0)
        .with_flex_child(list, 1.0)
        .controller(SizeEventForwarder(capture_area_id));

    layout
}

fn event_view(entry: &EventEntry) -> Option<impl Widget<()>> {
    let mut flex = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Label::new(event_name(&entry.event).to_string()));

    let content: Option<Box<dyn Widget<()>>> = match &entry.event {
        Event::WindowConnected => None,
        Event::Size(size) => {
            let details = Flex::column()
                .with_child(Label::new(format!("Width: {}", size.width)))
                .with_child(Label::new(format!("Height: {}", size.height)));
            Some(details.boxed())
        }
        Event::MouseDown(event) => Some(mouse_event_view(&event).boxed()),
        Event::MouseUp(event) => Some(mouse_event_view(&event).boxed()),
        Event::MouseMoved(event) => Some(mouse_event_view(&event).boxed()),
        Event::KeyDown(event) => Some(key_event_view(&event).boxed()),
        Event::KeyUp(event) => Some(key_event_view(&event).boxed()),
        Event::Zoom(value) => Some(Label::new(format!("Value: {}", value)).boxed()),
        _ => Some(Label::new("Details not yet implemented.").boxed()),
    };

    if let Some(content) = content {
        flex.add_child(content.padding((5.0, 0.0)));
    }

    flex.add_spacer(5.0);
    Some(flex)
}

fn mouse_event_view(event: &MouseEvent) -> impl Widget<()> {
    use druid::MouseButton;
    let button_name = match event.button {
        MouseButton::Left => "Left",
        MouseButton::Middle => "Middle",
        MouseButton::Right => "Right",
        MouseButton::X1 => "X1",
        MouseButton::X2 => "X2",
    };
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Label::new(format!("Button: {}", button_name)))
        .with_child(Label::new(format!(
            "Position: {:.1}, {:.1}",
            event.pos.x, event.pos.y
        )))
        .with_child(Label::new(format!("Count: {}", event.count)))
}

fn key_event_view(event: &KeyEvent) -> impl Widget<()> {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Label::new(format!("Key Code: {:?}", event.key_code)))
        .with_child(Label::new(format!("Is Repeat: {}", event.is_repeat)))
}

fn event_name(event: &Event) -> &str {
    use Event::*;
    match event {
        WindowConnected => "Window Connected",
        Size(_) => "Size",
        MouseDown(_) => "Mouse Down",
        MouseUp(_) => "Mouse Up",
        MouseMoved(_) => "Mouse Moved",
        KeyDown(_) => "Key Down",
        KeyUp(_) => "Key Up",
        Paste(_) => "Paste",
        Wheel(_) => "Wheel",
        Zoom(_) => "Zoom",
        Timer(_) => "Timer",
        Command(_) => "Command",
        TargetedCommand(_, _) => "Targeted Command",
    }
}

impl<T: Data> CaptureArea<T> {
    pub fn new(on_capture: impl Fn(&Event, &mut T) + 'static) -> Self {
        CaptureArea {
            content: Flex::column()
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(Label::new("Move cursor here to capture events"))
                .background(theme::BACKGROUND_DARK)
                .border(theme::BORDER_LIGHT, 2.0),
            on_capture: Box::new(on_capture),
        }
    }
}

impl<T: Data> Widget<T> for CaptureArea<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::MouseMoved(_) => {
                ctx.request_focus();
                (self.on_capture)(event, data);
            }
            Event::Command(cmd) => {
                if cmd.selector == SIZE_EVENT {
                    if let Ok(size) = cmd.get_object::<Size>() {
                        (self.on_capture)(&Event::Size(*size), data);
                    } else {
                        eprintln!(
                            "Command {} expects Size as object. ({}:{})",
                            SIZE_EVENT,
                            file!(),
                            line!()
                        );
                    }
                }
            }
            _ => (self.on_capture)(event, data),
        }
        ctx.set_handled();
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.content.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.content.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        self.content.layout(layout_ctx, &bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.content.paint(ctx, data, env);
    }
}
