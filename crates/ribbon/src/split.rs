use std::rc::Rc;

use gpui::{
    Anchor, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement,
    Styled, Window, div, hsla, prelude::FluentBuilder as _, px,
};
use gpui_base::{Popover, h_flex};

use crate::{
    dropdown::menu_panel,
    face::{ChangeHandler, ClickHandler, ItemFace},
    size::RibbonItemSize,
    theme,
};

/// A ribbon item with two hit areas, following Fluent Ribbon's split button:
/// the main face runs the command, the arrow opens a menu of related
/// commands. Activating an enabled menu entry dismisses the menu.
#[derive(IntoElement)]
pub struct RibbonSplitButton {
    face: ItemFace,
    entries: Vec<crate::dropdown::RibbonMenuEntry>,
    on_click: Option<ClickHandler>,
    on_open_change: Option<ChangeHandler>,
}

impl RibbonSplitButton {
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
            entries: Vec::new(),
            on_click: None,
            on_open_change: None,
        }
    }

    /// The button label, also used as the accessibility label.
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.face.label = header.into();
        self
    }

    /// The key tip shown on this item while the key tip overlay is active.
    /// Key tip activation is not wired for split buttons; the badge is
    /// display-only.
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

    /// Runs when the main face is activated (the arrow only opens the menu).
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Appends a menu entry to the arrow's popup.
    pub fn entry(mut self, entry: impl Into<crate::dropdown::RibbonMenuEntry>) -> Self {
        self.entries.push(entry.into());
        self
    }

    /// Appends several menu entries to the arrow's popup.
    pub fn entries(
        mut self,
        entries: impl IntoIterator<Item = impl Into<crate::dropdown::RibbonMenuEntry>>,
    ) -> Self {
        self.entries.extend(entries.into_iter().map(Into::into));
        self
    }

    /// Reports whether the menu popup is open after each transition.
    pub fn on_open_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
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
}

impl Styled for RibbonSplitButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.face.style
    }
}

impl RenderOnce for RibbonSplitButton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let disabled = self.face.disabled;
        let menu_id = (self.face.id.clone(), "split");

        Popover::new(menu_id)
            .anchor(Anchor::BottomLeft)
            .when_some(self.on_open_change, |popover, handler| {
                popover.on_open_change(move |open, window, cx| handler(open, window, cx))
            })
            .when(disabled, |popover| {
                popover.trigger_with(|_, _, _| div().into_any_element())
            })
            .when(!disabled, |popover| {
                popover
                    .trigger_with(move |open, _, cx| {
                        let tokens = theme::tokens(cx);
                        let mut face = self.face;
                        face.highlighted = open;

                        // The main face swallows the press so the popover
                        // trigger only toggles from the arrow area.
                        let main = face
                            .render(cx)
                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                cx.stop_propagation();
                            })
                            .when_some(self.on_click.clone(), |main, on_click| {
                                main.on_click(move |event, window, cx| on_click(event, window, cx))
                            });

                        let arrow = h_flex()
                            .id("arrow")
                            .justify_center()
                            .w(px(14.))
                            .self_stretch()
                            .text_color(tokens.colors.muted_foreground)
                            .hover(move |style| style.bg(tokens.colors.muted))
                            .child("▾");

                        // At rest the compound control is flat like every
                        // other item. The border is always laid out (trans-
                        // parent at rest) so raising it on open does not
                        // shift the arrow or the main face.
                        h_flex()
                            .items_stretch()
                            .rounded(tokens.radius.sm)
                            .border_1()
                            .border_color(hsla(0., 0., 0., 0.))
                            .when(open, |split| {
                                split
                                    .bg(tokens.colors.accent)
                                    .border_color(tokens.colors.ring)
                                    .overflow_hidden()
                            })
                            .child(main)
                            .child(
                                div()
                                    .w(px(1.))
                                    .self_stretch()
                                    .when(open, |divider| divider.bg(tokens.colors.border)),
                            )
                            .child(arrow)
                            .into_any_element()
                    })
                    .content(move |_, _, cx| {
                        let popup = cx.entity();
                        menu_panel(self.entries, popup, cx)
                    })
            })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, Modifiers, Render, point, px};

    use super::*;
    use crate::dropdown::RibbonMenuItem;

    struct SplitHarness {
        clicks: Rc<Cell<usize>>,
        open_changes: Rc<Cell<Option<bool>>>,
    }

    impl Render for SplitHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            RibbonSplitButton::new("sort", "Sort")
                .on_click({
                    let clicks = self.clicks.clone();
                    move |_, _, _| clicks.set(clicks.get() + 1)
                })
                .entry(RibbonMenuItem::new("asc", "Ascending").on_click(|_, _, _| {}))
                .on_open_change({
                    let open_changes = self.open_changes.clone();
                    move |open, _, _| open_changes.set(Some(*open))
                })
        }
    }

    #[gpui::test]
    fn main_face_click_runs_the_command_without_opening_the_menu(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| crate::init(cx));

        let clicks = Rc::new(Cell::new(0));
        let open_changes = Rc::new(Cell::new(None));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            let open_changes = open_changes.clone();
            move |_, _| SplitHarness {
                clicks,
                open_changes,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));

        cx.simulate_click(point(px(30.), px(14.)), Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));

        assert_eq!(clicks.get(), 1);
        assert_eq!(open_changes.get(), None);
    }
}
