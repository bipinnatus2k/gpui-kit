use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, StyleRefinement, Styled,
    Window, div, px,
};
use gpui_base::StyledExt as _;

use crate::{
    face::{ClickHandler, ItemFace},
    size::RibbonItemSize,
};

/// A ribbon button, the workhorse item of a [`RibbonGroup`](crate::RibbonGroup).
///
/// A large button stacks a 32px icon above its label; a medium button shows a
/// 16px icon next to the label; a small button is icon-only. Buttons are
/// definitive: clicking them runs a command, they hold no checked state (use
/// [`RibbonToggleButton`](crate::RibbonToggleButton) for that).
///
/// Icons are not bundled: pass any element (typically an `Icon` from the
/// styled layer) via [`RibbonButton::icon`].
#[derive(IntoElement)]
pub struct RibbonButton {
    face: ItemFace,
}

impl RibbonButton {
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
        }
    }

    /// The button label, also used as the accessibility label.
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.face.label = header.into();
        self
    }

    /// The key tip shown on this item while the key tip overlay is active;
    /// pressing the key then activates the button.
    pub fn key_tip(mut self, key_tip: impl Into<SharedString>) -> Self {
        self.face.key_tip = Some(key_tip.into());
        self
    }

    /// The button icon, rendered at the size the item's size mode dictates.
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

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.face.on_click = Some(Rc::new(handler));
        self
    }

    pub(crate) fn with_size_cap(mut self, cap: Option<RibbonItemSize>) -> Self {
        if let Some(cap) = cap {
            self.face.size = self.face.size.min(cap);
        }
        self
    }

    /// Marks whether the key tip overlay is currently visible.
    pub(crate) fn with_key_tip_visible(mut self, visible: bool) -> Self {
        self.face.key_tips_visible = visible;
        self
    }

    /// The declared key tip, if any.
    pub(crate) fn declared_key_tip(&self) -> Option<&str> {
        self.face.key_tip.as_deref()
    }

    /// The command handler, for key tip activation.
    pub(crate) fn on_click_handler(&self) -> Option<ClickHandler> {
        self.face.on_click.clone()
    }

    /// Chains the click handler with a popup dismissal, so an item rendered
    /// inside a collapsed group popup closes the popup after its command runs.
    pub(crate) fn dismiss_after_click(
        mut self,
        popup: gpui::Entity<gpui_base::PopoverState>,
    ) -> Self {
        let user_on_click = self.face.on_click.take();
        self.face.on_click = Some(Rc::new(move |event, window, cx| {
            if let Some(user_on_click) = &user_on_click {
                user_on_click(event, window, cx);
            }
            popup.update(cx, |popup, cx| popup.dismiss(window, cx));
        }));
        self
    }
}

impl Styled for RibbonButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.face.style
    }
}

impl RenderOnce for RibbonButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.face.render(cx)
    }
}

/// A 1px vertical divider between groups, stretched to the content row height.
#[derive(IntoElement)]
pub struct RibbonSeparator {
    style: StyleRefinement,
}

impl RibbonSeparator {
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
        }
    }
}

impl Default for RibbonSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl Styled for RibbonSeparator {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for RibbonSeparator {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let border = crate::theme::tokens(cx).colors.border;
        div()
            .w(px(1.))
            .self_stretch()
            .mx_1()
            .bg(border)
            .refine_style(&self.style)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::Cell,
        rc::Rc,
        sync::{Arc, Mutex},
    };

    use gpui::{Context, Element as _, Modifiers, Render, accesskit, canvas, point, px};

    use super::*;

    #[test]
    fn size_cap_only_shrinks_the_preferred_size() {
        assert_eq!(
            RibbonButton::new("paste", "Paste")
                .size(RibbonItemSize::Large)
                .with_size_cap(Some(RibbonItemSize::Medium))
                .face
                .size,
            RibbonItemSize::Medium
        );
        assert_eq!(
            RibbonButton::new("paste", "Paste")
                .size(RibbonItemSize::Large)
                .with_size_cap(None)
                .face
                .size,
            RibbonItemSize::Large
        );
        assert_eq!(
            RibbonButton::new("cut", "Cut")
                .size(RibbonItemSize::Small)
                .with_size_cap(Some(RibbonItemSize::Large))
                .face
                .size,
            RibbonItemSize::Small
        );
    }

    struct ClickHarness(Rc<Cell<usize>>);

    impl Render for ClickHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.0.clone();
            RibbonButton::new("paste", "Paste")
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
        }
    }

    #[gpui::test]
    fn pointer_activation_runs_the_click_handler(cx: &mut gpui::TestAppContext) {
        let clicks = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| ClickHarness(clicks)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));

        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());

        assert_eq!(clicks.get(), 1);
    }

    #[gpui::test]
    fn button_exposes_the_button_role_and_label(cx: &mut gpui::TestAppContext) {
        type Captured = Arc<Mutex<Option<accesskit::Node>>>;

        struct Probe(Captured);

        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let captured = self.0.clone();
                canvas(
                    move |_, window, cx| {
                        let mut node = accesskit::Node::new(accesskit::Role::Button);
                        RibbonButton::new("paste", "Paste")
                            .disabled(true)
                            .render(window, cx)
                            .into_element()
                            .write_a11y_info(&mut node);
                        *captured.lock().unwrap() = Some(node);
                    },
                    |_, _, _, _| {},
                )
            }
        }

        let captured: Captured = Arc::new(Mutex::new(None));
        let result = captured.clone();
        let (_, cx) = cx.add_window_view(move |_, _| Probe(captured));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let node = result.lock().unwrap().take().unwrap();

        assert_eq!(node.role(), accesskit::Role::Button);
        assert_eq!(node.label(), Some("Paste"));
        assert!(!node.supports_action(accesskit::Action::Click));
    }
}
