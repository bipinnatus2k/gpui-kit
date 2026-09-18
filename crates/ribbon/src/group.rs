use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement,
    Styled, Window, div, hsla, prelude::FluentBuilder as _, px,
};
use gpui_base::{Popover, StyledExt as _, h_flex, v_flex};

use crate::{face::ClickHandler, item::RibbonItem, size::RibbonGroupState, theme};

/// A labeled cluster of items inside a [`RibbonTab`](crate::RibbonTab).
///
/// The group owns the Fluent Ribbon scaling model: at [`RibbonGroupState::Large`]
/// items render at their own sizes; while the state walks toward
/// [`RibbonGroupState::Collapsed`] the group caps scalable item sizes; when
/// collapsed, the whole group becomes a single button that offers the items
/// in a popup. The group header sits below the items, as in classic ribbon
/// layouts.
#[derive(IntoElement)]
pub struct RibbonGroup {
    id: ElementId,
    header: String,
    icon: Option<AnyElement>,
    state: RibbonGroupState,
    pub(crate) items: Vec<RibbonItem>,
    launcher: Option<ClickHandler>,
    pub(crate) key_tips_visible: bool,
    style: StyleRefinement,
}

impl RibbonGroup {
    pub fn new(id: impl Into<ElementId>, header: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            header: header.into().to_string(),
            icon: None,
            state: RibbonGroupState::default(),
            items: Vec::new(),
            launcher: None,
            key_tips_visible: false,
            style: StyleRefinement::default(),
        }
    }

    /// The group label shown beneath the items.
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.header = header.into().to_string();
        self
    }

    /// The group icon, shown on the collapsed trigger.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// The current layout state of the group.
    pub fn state(mut self, state: RibbonGroupState) -> Self {
        self.state = state;
        self
    }

    /// The dialog launcher: a small button shown next to the group header
    /// that opens the group's full dialog, as in classic ribbon layouts.
    pub fn launcher(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.launcher = Some(Rc::new(on_click));
        self
    }

    /// Appends an item to the group.
    pub fn item(mut self, item: impl Into<RibbonItem>) -> Self {
        self.items.push(item.into());
        self
    }

    /// Appends several items to the group.
    pub fn items(mut self, items: impl IntoIterator<Item = impl Into<RibbonItem>>) -> Self {
        self.items.extend(items.into_iter().map(Into::into));
        self
    }
}

impl Styled for RibbonGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use gpui::div;

    use super::*;
    use crate::button::{RibbonButton, RibbonSeparator};

    #[test]
    fn items_convert_from_concrete_controls() {
        let group = RibbonGroup::new("clipboard", "Clipboard")
            .item(RibbonButton::new("paste", "Paste"))
            .item(RibbonSeparator::new())
            .item(div().into_any_element());

        assert_eq!(group.items.len(), 3);
        assert!(matches!(group.items[0], RibbonItem::Button(_)));
        assert!(matches!(group.items[1], RibbonItem::Separator(_)));
        assert!(matches!(group.items[2], RibbonItem::Custom(_)));
    }
}

impl RenderOnce for RibbonGroup {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = theme::tokens(cx);
        let RibbonGroup {
            id,
            header,
            icon,
            state,
            items,
            launcher,
            key_tips_visible,
            style,
        } = self;

        match state {
            RibbonGroupState::Collapsed => {
                let has_icon = icon.is_some();
                let header_for_popup = header.clone();
                Popover::new((id, "collapsed"))
                    .anchor(Anchor::BottomLeft)
                    .trigger_with(move |_, _, _cx| {
                        h_flex()
                            .id("trigger")
                            .items_center()
                            .gap_1()
                            .px_2()
                            .h(px(26.))
                            .rounded(tokens.radius.sm)
                            .border_1()
                            .border_color(hsla(0., 0., 0., 0.))
                            .text_size(tokens.typography.sm.size)
                            .text_color(tokens.colors.foreground)
                            .hover(move |style| {
                                style
                                    .bg(tokens.colors.muted)
                                    .border_color(tokens.colors.ring)
                            })
                            .when_some(icon, |trigger, icon| {
                                trigger.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(px(16.))
                                        .child(icon),
                                )
                            })
                            .when(!has_icon, |trigger| trigger.child(header.clone()))
                            .child(div().text_color(tokens.colors.muted_foreground).child("▾"))
                            .into_any_element()
                    })
                    .content(move |_, _, cx| {
                        let popup = cx.entity();
                        v_flex()
                            .items_start()
                            .min_w(px(140.))
                            .py_1()
                            .bg(tokens.colors.surface)
                            .border_1()
                            .border_color(tokens.colors.border)
                            .rounded(tokens.radius.md)
                            .shadow(tokens.shadow.md.clone())
                            .overflow_hidden()
                            .children(items.into_iter().map(|item| {
                                div()
                                    .px_1()
                                    .py_1()
                                    .child(item.render_in_popup(None, popup.clone()))
                            }))
                            .child(
                                div()
                                    .w_full()
                                    .text_center()
                                    .px_2()
                                    .py_1()
                                    .text_size(px(10.))
                                    .text_color(tokens.colors.muted_foreground)
                                    .child(header_for_popup),
                            )
                    })
                    .into_any_element()
            }
            _ => {
                let cap = state.item_size_cap();
                // The header is pinned to the bottom of the group (`justify_
                // between` + the row's `items_stretch`), so group headers stay
                // aligned across the tab no matter how many lines the items
                // wrap into.
                v_flex()
                    .justify_between()
                    .gap_1()
                    .py_1()
                    .child(
                        h_flex().items_stretch().flex_wrap().gap_1().children(
                            items
                                .into_iter()
                                .map(|item| item.render_at(cap, key_tips_visible)),
                        ),
                    )
                    .child({
                        let has_launcher = launcher.is_some();
                        let launcher_on_click = launcher.clone();
                        let header_text = header.clone();
                        h_flex()
                            .id("group-header")
                            .w_full()
                            .items_center()
                            .justify_center()
                            .gap_1()
                            .text_size(px(10.))
                            .text_color(tokens.colors.muted_foreground)
                            .when_some(launcher_on_click, |header_row, on_click| {
                                header_row.child(header_text).child(
                                    h_flex()
                                        .id("launcher")
                                        .items_center()
                                        .justify_center()
                                        .size(px(14.))
                                        .rounded(px(3.))
                                        .text_size(px(10.))
                                        .text_color(tokens.colors.muted_foreground)
                                        .hover(move |style| style.bg(tokens.colors.muted))
                                        .child("▸")
                                        .on_click(move |event, window, cx| {
                                            on_click(event, window, cx)
                                        }),
                                )
                            })
                            .when(!has_launcher, |header_row| header_row.child(header))
                    })
                    .refine_style(&style)
                    .into_any_element()
            }
        }
    }
}
