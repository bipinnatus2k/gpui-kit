use gpui::{
    AnyElement, App, Axis, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, KeyDownEvent, ParentElement, RenderOnce, Role, Stateful,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, div,
};
use smallvec::SmallVec;

use crate::StyledExt as _;

/// Upper bound on tab-stop hops when wrapping focus back into the toolbar, so
/// a toolbar whose items all vanished from the tab order can never hang the
/// key handler. Mirrors `Root`'s focus-trap loop bound.
const MAX_FOCUS_ATTEMPTS: usize = 100;

/// An unstyled container that groups a set of controls and owns roving
/// keyboard focus among them.
///
/// This is the behavior primitive behind a styled toolbar. The container
/// exposes `Toolbar` semantics to assistive technology (`Role::Toolbar` with
/// an orientation) and moves focus between its focusable descendants with the
/// arrow keys, so applications do not have to reimplement the roving-focus
/// contract per toolbar. It works with any focusable children — buttons,
/// menu triggers, inputs — because traversal walks the rendered tab stops
/// and constrains them to this subtree, mirroring how `Root` constrains
/// focus-trap cycling.
///
/// Keyboard contract:
///
/// - Arrow keys move focus along the toolbar's [`Axis`] to the previous or
///   next focusable descendant, wrapping around at either end (the same
///   default as Base UI's `loopFocus`). A non-wrapping mode would need to
///   distinguish a wrapped step from an ordinary one, which is not possible
///   with GPUI's public tab-stop API.
/// - When `disabled`, the arrow keys do nothing. Hosted controls must be
///   disabled by their owner; the flag only suppresses the toolbar's own
///   navigation.
///
/// The container is not itself a tab stop, so ordinary `Tab` traversal enters
/// and leaves the toolbar through its items, matching the ARIA toolbar
/// pattern.
///
/// An input hosted inside the toolbar keeps its own arrow-key behavior: text
/// inputs consume the arrow keys for caret movement before the toolbar sees
/// them. Place inputs at the trailing end of a horizontal toolbar, as the
/// Base UI Toolbar recommends.
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    base: Stateful<Div>,
    style: StyleRefinement,
    axis: Axis,
    disabled: bool,
    children: SmallVec<[AnyElement; 4]>,
}

impl Toolbar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            base: div().id(id.clone()),
            style: StyleRefinement::default(),
            axis: Axis::Horizontal,
            disabled: false,
            children: SmallVec::new(),
            id,
        }
    }

    /// Sets the semantic axis the arrow keys follow. Defaults to horizontal.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Disables the toolbar's own keyboard navigation. Hosted controls are
    /// not automatically disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

fn step_focus(window: &mut Window, cx: &mut App, forward: bool) {
    if forward {
        window.focus_next(cx);
    } else {
        window.focus_prev(cx);
    }
}

/// Move focus to the next (or previous) focusable element inside `container`.
///
/// Traversal walks the window's tab stops like `Root`'s focus-trap cycling:
/// step once, and if the step landed outside the container, keep stepping
/// until the focus re-enters or comes back to where it started (in which
/// case the toolbar has no other focusable item and focus stays put).
fn move_focus(container: &FocusHandle, forward: bool, window: &mut Window, cx: &mut App) {
    let Some(start) = window.focused(cx) else {
        return;
    };

    step_focus(window, cx, forward);
    if container.contains_focused(window, cx)
        && window.focused(cx).is_some_and(|focused| focused != start)
    {
        return;
    }

    for _ in 0..MAX_FOCUS_ATTEMPTS {
        step_focus(window, cx, forward);
        if container.contains_focused(window, cx)
            && window.focused(cx).is_some_and(|focused| focused != start)
        {
            return;
        }
        if window.focused(cx).is_some_and(|focused| focused == start) {
            break;
        }
    }

    window.focus(&start, cx);
}

fn handle_key_down(
    axis: Axis,
    disabled: bool,
    container: &FocusHandle,
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut App,
) {
    if disabled {
        return;
    }

    let forward = match (axis, event.keystroke.key.as_str()) {
        (Axis::Horizontal, "left") => Some(false),
        (Axis::Horizontal, "right") => Some(true),
        (Axis::Vertical, "up") => Some(false),
        (Axis::Vertical, "down") => Some(true),
        _ => None,
    };

    if let Some(forward) = forward {
        move_focus(container, forward, window, cx);
        cx.stop_propagation();
    }
}

impl Styled for Toolbar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Toolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl InteractiveElement for Toolbar {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Toolbar {}

impl RenderOnce for Toolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Self {
            base,
            style,
            axis,
            disabled,
            children,
            ..
        } = self;

        // The handle must survive across frames so containment checks and the
        // key handler refer to the same dispatch-tree node; button.rs uses
        // the same keyed-state pattern for its own focus handle.
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| {
                cx.focus_handle().tab_stop(false)
            })
            .read(cx)
            .clone();
        let orientation = match axis {
            Axis::Horizontal => accesskit::Orientation::Horizontal,
            Axis::Vertical => accesskit::Orientation::Vertical,
        };
        let key_handler = {
            let focus_handle = focus_handle.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                handle_key_down(axis, disabled, &focus_handle, event, window, cx);
            }
        };

        base.track_focus(&focus_handle)
            .role(Role::Toolbar)
            .aria_orientation(orientation)
            .on_key_down(key_handler)
            .children(children)
            .refine_style(&style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_builder() {
        let toolbar = Toolbar::new("toolbar")
            .axis(Axis::Vertical)
            .disabled(true)
            .child(div());

        assert!(toolbar.disabled);
        assert_eq!(toolbar.children.len(), 1);
    }

    #[test]
    fn test_toolbar_defaults() {
        let toolbar = Toolbar::new("toolbar");

        assert!(matches!(toolbar.axis, Axis::Horizontal));
        assert!(!toolbar.disabled);
        assert!(toolbar.children.is_empty());
    }

    #[cfg(test)]
    mod behavior {
        use super::*;
        use gpui::{Context, FocusHandle, Render, TestAppContext, VisualTestContext, px};

        struct NavHarness {
            items: [FocusHandle; 3],
        }

        impl Render for NavHarness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let [first, second, third] = &self.items;
                Toolbar::new("nav-toolbar")
                    .child(div().id("first").size(px(20.)).track_focus(first))
                    .child(div().id("second").size(px(20.)).track_focus(second))
                    .child(div().id("third").size(px(20.)).track_focus(third))
            }
        }

        fn harness(cx: &mut TestAppContext) -> ([FocusHandle; 3], &mut VisualTestContext) {
            let (state, cx) = cx.add_window_view(move |window, cx| {
                let items = [
                    cx.focus_handle().tab_stop(true),
                    cx.focus_handle().tab_stop(true),
                    cx.focus_handle().tab_stop(true),
                ];
                items[0].focus(window, cx);
                NavHarness { items }
            });
            let items = state.read_with(cx, |harness, _| harness.items.clone());
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            (items, cx)
        }

        fn assert_focused(cx: &mut VisualTestContext, expected: &FocusHandle, label: &str) {
            cx.update(|window, _| {
                assert!(
                    expected.is_focused(window),
                    "expected {label} to be focused"
                );
            });
        }

        #[gpui::test]
        fn arrow_keys_rove_focus_across_items(cx: &mut gpui::TestAppContext) {
            let ([first, second, third], cx) = harness(cx);

            cx.simulate_keystrokes("right");
            assert_focused(cx, &second, "second");
            cx.simulate_keystrokes("right");
            assert_focused(cx, &third, "third");

            // Wrapping: past the last item, focus returns to the first.
            cx.simulate_keystrokes("right");
            assert_focused(cx, &first, "first (wrapped)");

            cx.simulate_keystrokes("left");
            assert_focused(cx, &third, "third (wrapped back)");
        }

        #[gpui::test]
        fn toolbar_with_a_single_item_keeps_focus_on_it(cx: &mut gpui::TestAppContext) {
            struct SingleHarness {
                item: FocusHandle,
            }

            impl Render for SingleHarness {
                fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                    let item = &self.item;
                    Toolbar::new("single-toolbar")
                        .child(div().id("only").size(px(20.)).track_focus(item))
                }
            }

            let (state, cx) = cx.add_window_view(|window, cx| {
                let item = cx.focus_handle().tab_stop(true);
                item.focus(window, cx);
                SingleHarness { item }
            });
            let item = state.read_with(cx, |harness, _| harness.item.clone());
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });

            cx.simulate_keystrokes("right left right");
            assert_focused(cx, &item, "the only item");
        }

        #[gpui::test]
        fn disabled_toolbar_ignores_arrow_keys(cx: &mut gpui::TestAppContext) {
            struct DisabledHarness {
                items: [FocusHandle; 2],
            }

            impl Render for DisabledHarness {
                fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                    let [first, second] = &self.items;
                    Toolbar::new("disabled-toolbar")
                        .disabled(true)
                        .child(div().id("first").size(px(20.)).track_focus(first))
                        .child(div().id("second").size(px(20.)).track_focus(second))
                }
            }

            let (state, cx) = cx.add_window_view(|window, cx| {
                let items = [
                    cx.focus_handle().tab_stop(true),
                    cx.focus_handle().tab_stop(true),
                ];
                items[0].focus(window, cx);
                DisabledHarness { items }
            });
            let items = state.read_with(cx, |harness, _| harness.items.clone());
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });

            cx.simulate_keystrokes("right");
            assert_focused(cx, &items[0], "first (disabled toolbar)");
        }
    }
}
