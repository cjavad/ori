use crate::{BuildCx, Canvas, DrawCx, Event, EventCx, LayoutCx, Pod, PodState, Space, View};

/// Create a new [`Flex`].
pub fn flex<T, V: View<T>>(flex: f32, content: V) -> Flex<T, V> {
    Flex::new(flex, content)
}

/// A flexible view.
///
/// When used in a stack, will shrink or grow to fill the remaining space.
pub struct Flex<T, V> {
    pub content: Pod<T, V>,
    pub flex: f32,
}

impl<T, V> Flex<T, V> {
    pub fn new(flex: f32, content: V) -> Self {
        Self {
            content: Pod::new(content),
            flex,
        }
    }
}

impl<T, V: View<T>> View<T> for Flex<T, V> {
    type State = PodState<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(
        &mut self,
        state: &mut Self::State,
        cx: &mut crate::RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        if self.flex != old.flex {
            cx.request_layout();
        }

        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        self.content.event(state, cx, data, event);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> crate::Size {
        cx.set_flex(self.flex);

        self.content.layout(state, cx, data, space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        self.content.draw(state, cx, data, canvas);
    }
}
