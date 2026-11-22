use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    AppContext as _, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, Styled as _, TestAppContext, Window, div, px, rgb,
};
use gpui_animation::{animation::TransitionExt as _, transition::general::Linear};

struct Inner {
    wide: bool,
    renders: Rc<Cell<usize>>,
}

impl Render for Inner {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        let wide = self.wide;
        div()
            .id("transitioned")
            .size(px(10.))
            .bg(rgb(0x336699))
            .with_transition("transitioned")
            .transition_when(wide, Duration::from_secs(5), Linear, |state| {
                state.w(px(100.))
            })
    }
}

struct Outer {
    inner: Entity<Inner>,
    renders: Rc<Cell<usize>>,
}

impl Render for Outer {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        div()
            .size_full()
            .children((0..20).map(|_| div().size(px(4.))))
            .child(self.inner.clone())
    }
}

/// A running transition draws again only the view its element is in, not
/// every window: the view around it is left as it was drawn.
#[test]
fn a_transition_notifies_only_the_view_it_is_drawn_in() {
    let mut cx = TestAppContext::single();
    let outer_renders = Rc::new(Cell::new(0));
    let inner_renders = Rc::new(Cell::new(0));
    let window = cx.add_window({
        let (outer_renders, inner_renders) = (outer_renders.clone(), inner_renders.clone());
        move |_, cx| Outer {
            inner: cx.new(|_| Inner {
                wide: false,
                renders: inner_renders,
            }),
            renders: outer_renders,
        }
    });
    cx.run_until_parked();
    let inner = window
        .update(&mut cx, |outer, _, _| outer.inner.clone())
        .unwrap();
    inner.update(&mut cx, |inner, cx| {
        inner.wide = true;
        cx.notify();
    });
    cx.run_until_parked();
    outer_renders.set(0);
    inner_renders.set(0);

    for _ in 0..10 {
        cx.executor().advance_clock(Duration::from_millis(9));
        cx.run_until_parked();
    }
    assert!(
        inner_renders.get() >= 5,
        "the transition draws its view on its frames: {} renders",
        inner_renders.get()
    );
    assert_eq!(
        outer_renders.get(),
        0,
        "the view around it is not drawn again"
    );
}
