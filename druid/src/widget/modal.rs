use crate::{widget::prelude::*, WidgetPod, Data, Point, MouseEvent};

#[derive(Clone, Copy)]
pub enum ModalPosition {
    Relative,
    Window,
    Cursor,
}

pub struct Modal<T> {
    content: WidgetPod<T, Box<dyn Widget<T>>>,
    visibility: Box<dyn Fn(&T) -> bool>,
    position: ModalPosition,
    window_size: Size,
    cursor_position: Point,
}

impl<T> Modal<T> {
    pub fn new(
        position: ModalPosition,
        visibility: impl Fn(&T) -> bool + 'static,
        content: impl Widget<T> + 'static,
    ) -> Self {
        Self {
            content: WidgetPod::new(Box::new(content)),
            visibility: Box::new(visibility),
            position,
            window_size: Size::ZERO,
            cursor_position: Point::ZERO,
        }
    }
}

impl<T: Data> Widget<T> for Modal<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::WindowSize(size) => self.window_size = *size,
            Event::MouseMove(MouseEvent {pos, ..}) => self.cursor_position = *pos,
            _ => (),
        }

        if (self.visibility)(data) {
            self.content.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.content.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        if (self.visibility)(data) {
            self.content.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        match self.position {
            ModalPosition::Relative => {
                let bc = BoxConstraints::new(Size::ZERO, Size::new( f64::INFINITY, f64::INFINITY));
                let size = self.content.layout(ctx, &bc, data, env);
                self.content.set_layout_rect(ctx, data, env, size.to_rect());
            }
            ModalPosition::Cursor => panic!("Needs cursor events."),
            ModalPosition::Window => {
                self.content.set_layout_absolute(true);
                let bc = BoxConstraints::new(Size::ZERO, self.window_size);
                let size = self.content.layout(ctx, &bc, data, env);
                self.content.set_layout_rect(ctx, data, env, size.to_rect());
            }
            _ => (),
        }
        Size::ZERO
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        todo!()
    }
}
