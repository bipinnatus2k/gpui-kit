use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement,
    Styled, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_base::{Popover, h_flex, v_flex};

use crate::{
    face::{ChangeHandler, ClickHandler, ItemFace},
    size::RibbonItemSize,
    theme,
};

/// One command row inside a [`RibbonDropDownButton`] popup.
pub struct RibbonMenuItem {
    id: ElementId,
    label: String,
    icon: Option<AnyElement>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl RibbonMenuItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into().to_string(),
            icon: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into().to_string();
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

/// A horizontal rule between [`RibbonMenuItem`]s.
#[derive(Default)]
pub struct RibbonMenuSeparator;

/// One entry of a [`RibbonDropDownButton`] popup: an item or a separator.
pub enum RibbonMenuEntry {
    Item(RibbonMenuItem),
    Separator,
}

impl From<RibbonMenuItem> for RibbonMenuEntry {
    fn from(item: RibbonMenuItem) -> Self {
        Self::Item(item)
    }
}

impl From<RibbonMenuSeparator> for RibbonMenuEntry {
    fn from(_: RibbonMenuSeparator) -> Self {
        Self::Separator
    }
}

/// A ribbon item that opens a menu of commands instead of running one.
///
/// The button face follows the shared ribbon item presentation; the menu
/// opens below the trigger and closes after an enabled entry is activated.
#[derive(IntoElement)]
pub struct RibbonDropDownButton {
    face: ItemFace,
    entries: Vec<RibbonMenuEntry>,
    on_open_change: Option<ChangeHandler>,
}

impl RibbonDropDownButton {
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
            on_open_change: None,
        }
    }

    /// The button label, also used as the accessibility label.
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.face.label = header.into();
        self
    }

    /// The key tip shown on this item while the key tip overlay is active.
    /// Key tip activation is not wired for dropdowns; the badge is
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

    /// Reports whether the menu popup is open after each transition.
    pub fn on_open_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }

    /// Appends a menu entry to the popup.
    pub fn entry(mut self, entry: impl Into<RibbonMenuEntry>) -> Self {
        self.entries.push(entry.into());
        self
    }

    /// Appends several menu entries to the popup.
    pub fn entries(
        mut self,
        entries: impl IntoIterator<Item = impl Into<RibbonMenuEntry>>,
    ) -> Self {
        self.entries.extend(entries.into_iter().map(Into::into));
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

impl Styled for RibbonDropDownButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.face.style
    }
}

impl RenderOnce for RibbonDropDownButton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let disabled = self.face.disabled;
        let menu_id = (self.face.id.clone(), "menu");

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
                        let mut face = self.face;
                        face.highlighted = open;
                        face.render(cx).into_any_element()
                    })
                    .content(move |_, _window, cx| {
                        let popup = cx.entity();
                        menu_panel(self.entries, popup, cx)
                    })
            })
    }
}

/// The shared dropdown panel: a bordered surface listing menu entries, where
/// activating an enabled entry runs its command and dismisses the popup.
pub(crate) fn menu_panel(
    entries: Vec<RibbonMenuEntry>,
    popup: gpui::Entity<gpui_base::PopoverState>,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    let tokens = theme::tokens(cx);
    let mut menu = v_flex()
        .id("menu-panel")
        .min_w(px(180.))
        .py_1()
        .bg(tokens.colors.surface)
        .border_1()
        .border_color(tokens.colors.border)
        .rounded(tokens.radius.md)
        .shadow(tokens.shadow.md.clone())
        .overflow_hidden();

    for entry in entries {
        menu = match entry {
            RibbonMenuEntry::Separator => {
                menu.child(div().h(px(1.)).mx_2().my_1().bg(tokens.colors.border))
            }
            RibbonMenuEntry::Item(item) => menu.child(menu_item(
                item,
                tokens.typography.sm.size,
                tokens.colors.surface_foreground,
                tokens.colors.muted,
                popup.clone(),
            )),
        };
    }

    menu
}

fn menu_item(
    item: RibbonMenuItem,
    text_size: gpui::Pixels,
    foreground: gpui::Hsla,
    hover: gpui::Hsla,
    popup: gpui::Entity<gpui_base::PopoverState>,
) -> gpui::Stateful<gpui::Div> {
    let RibbonMenuItem {
        id,
        label,
        icon,
        disabled,
        on_click,
    } = item;

    h_flex()
        .id(id)
        .role(gpui::Role::MenuItem)
        .aria_label(label.clone())
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .mx_1()
        .rounded(px(3.))
        .text_size(text_size)
        .text_color(foreground)
        .line_height(gpui::relative(1.))
        .when(!disabled, |row| row.hover(move |style| style.bg(hover)))
        .when(disabled, |row| row.opacity(0.5))
        .when_some(
            (!disabled).then_some(on_click).flatten(),
            |row, on_click| {
                row.on_click(move |event, window, cx| {
                    on_click(event, window, cx);
                    popup.update(cx, |popup, cx| popup.dismiss(window, cx));
                })
            },
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(16.))
                .when_some(icon, |area, icon| area.child(icon)),
        )
        .child(label)
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, Modifiers, Render, point, px};

    use super::*;

    struct DropdownHarness(Rc<Cell<Option<bool>>>);

    impl Render for DropdownHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let open_changes = self.0.clone();
            RibbonDropDownButton::new("case", "Case")
                .entry(RibbonMenuItem::new("upper", "UPPERCASE").on_click(|_, _, _| {}))
                .on_open_change(move |open, _, _| open_changes.set(Some(*open)))
        }
    }

    #[gpui::test]
    fn menu_item_click_reports_the_popover_closing(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| crate::init(cx));

        let open_changes = Rc::new(Cell::new(None));
        let (_, cx) = cx.add_window_view({
            let open_changes = open_changes.clone();
            move |_, _| DropdownHarness(open_changes)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));

        // Open the dropdown by clicking its trigger, then click the menu item
        // rendered below it.
        cx.simulate_click(point(px(14.), px(14.)), Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(open_changes.get(), Some(true));

        cx.simulate_click(point(px(30.), px(44.)), Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));

        assert_eq!(open_changes.get(), Some(false));
    }
}
