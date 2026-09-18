use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, StyleRefinement, Styled,
    Window,
};

use crate::{
    face::{ChangeHandler, ItemFace},
    size::RibbonItemSize,
};

/// A ribbon button that holds a checked state, like Bold or Italic.
///
/// A checked toggle renders with the accent face until the application clears
/// it. Toggles in the same visual set are not mutually exclusive by default;
/// keep one application state and pass the same `checked` value to each
/// member when a group semantics is wanted.
#[derive(IntoElement)]
pub struct RibbonToggleButton {
    face: ItemFace,
    checked: bool,
    on_change: Option<ChangeHandler>,
}

impl RibbonToggleButton {
    pub fn new(id: impl Into<ElementId>, header: impl Into<SharedString>) -> Self {
        Self {
            face: ItemFace {
                id: id.into(),
                label: header.into(),
                icon: None,
                size: RibbonItemSize::default(),
                disabled: false,
                highlighted: false,
                key_tip: None,
                key_tips_visible: false,
                on_click: None,
                style: StyleRefinement::default(),
            },
            checked: false,
            on_change: None,
        }
    }

    /// The toggle label, also used as the accessibility label.
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.face.label = header.into();
        self
    }

    /// The key tip shown on this item while the key tip overlay is active.
    /// Key tip activation is not wired for toggles; the badge is display-only.
    pub fn key_tip(mut self, key_tip: impl Into<SharedString>) -> Self {
        self.face.key_tip = Some(key_tip.into());
        self
    }

    /// The toggle icon, rendered at the size the item's size mode dictates.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.face.icon = Some(icon.into_any_element());
        self
    }

    /// The preferred size. A containing group state may cap it while the
    /// group scales down.
    pub fn size(mut self, size: RibbonItemSize) -> Self {
        self.face.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.face.disabled = disabled;
        self
    }

    /// Whether the toggle currently reads as active.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Called with the next checked value when the toggle is activated.
    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub(crate) fn with_size_cap(mut self, cap: Option<RibbonItemSize>) -> Self {
        if let Some(cap) = cap {
            self.face.size = self.face.size.min(cap);
        }
        self
    }

    pub(crate) fn with_key_tip_visible(mut self, visible: bool) -> Self {
        self.face.key_tips_visible = visible;
        self
    }

    /// Chains the activation handler with a popup dismissal, so a toggle
    /// rendered inside a collapsed group popup closes the popup after the
    /// application records its next checked value.
    pub(crate) fn dismiss_after_click(
        mut self,
        popup: gpui::Entity<gpui_base::PopoverState>,
    ) -> Self {
        self.on_change = match self.on_change.take() {
            Some(user_on_change) => Some(Rc::new(move |checked, window, cx| {
                user_on_change(checked, window, cx);
                popup.update(cx, |popup, cx| popup.dismiss(window, cx));
            })),
            None => Some(Rc::new(move |_, window, cx| {
                popup.update(cx, |popup, cx| popup.dismiss(window, cx));
            })),
        };
        self
    }
}

impl Styled for RibbonToggleButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.face.style
    }
}

impl RenderOnce for RibbonToggleButton {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        self.face.highlighted = checked;
        self.face.on_click = self.on_change.map(|on_change| {
            let on_click = move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                on_change(&!checked, window, cx);
            };
            Rc::new(on_click) as crate::face::ClickHandler
        });
        self.face.render(cx)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, Modifiers, Render, point, px};

    use super::*;

    struct ToggleHarness(Rc<Cell<Option<bool>>>);

    impl Render for ToggleHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let next = self.0.clone();
            RibbonToggleButton::new("bold", "Bold")
                .checked(false)
                .on_change(move |checked, _, _| next.set(Some(*checked)))
        }
    }

    #[gpui::test]
    fn activation_reports_the_next_checked_value(cx: &mut gpui::TestAppContext) {
        let next = Rc::new(Cell::new(None));
        let (_, cx) = cx.add_window_view({
            let next = next.clone();
            move |_, _| ToggleHarness(next)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));

        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());

        assert_eq!(next.get(), Some(true));
    }
}
