use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement as _, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div, hsla,
    prelude::FluentBuilder as _, px, relative,
};
use gpui_base::{StyledExt as _, h_flex, v_flex};

use crate::{size::RibbonItemSize, theme};

pub(crate) type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
pub(crate) type ChangeHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// The interactive face shared by every ribbon item: icon plus label laid out
/// at the item's size, with hover, highlight and disabled states resolved from
/// the active semantic tokens.
///
/// State lives with the application; the face is a pure view function, so the
/// item types (button, toggle, dropdown trigger) only decide which handlers
/// and which highlight state to project onto it.
pub(crate) struct ItemFace {
    pub(crate) id: ElementId,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<AnyElement>,
    pub(crate) size: RibbonItemSize,
    pub(crate) disabled: bool,
    pub(crate) highlighted: bool,
    pub(crate) key_tip: Option<SharedString>,
    pub(crate) key_tips_visible: bool,
    pub(crate) on_click: Option<ClickHandler>,
    pub(crate) style: StyleRefinement,
}

impl ItemFace {
    pub(crate) fn render(self, cx: &App) -> gpui::Stateful<gpui::Div> {
        let tokens = theme::tokens(cx);
        let label = self.label.clone();
        let disabled = self.disabled;
        let large = self.size == RibbonItemSize::Large;

        let (icon_area_size, label_size, label_line_height) = if large {
            (px(32.), tokens.typography.xs.size, relative(1.1))
        } else {
            (px(16.), tokens.typography.sm.size, relative(1.))
        };

        let layout = match self.size {
            RibbonItemSize::Large => v_flex()
                .items_center()
                .justify_center()
                .gap_1()
                .min_w(px(64.))
                .px_2()
                .py_1(),
            RibbonItemSize::Medium => h_flex().items_center().gap_1().px_2().h(px(26.)),
            RibbonItemSize::Small => h_flex().items_center().justify_center().size(px(26.)),
        };

        let icon_area = || {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(icon_area_size)
        };

        // Office 2010 items are flat at rest; hovering raises them with a
        // background wash and a border. The border is always present but
        // transparent at rest so hovering does not shift the layout.
        layout
            .id(self.id)
            .role(gpui::Role::Button)
            .aria_label(label.clone())
            .line_height(label_line_height)
            .text_size(label_size)
            .text_color(tokens.colors.foreground)
            .rounded(tokens.radius.sm)
            .border_1()
            .border_color(hsla(0., 0., 0., 0.))
            .relative()
            .when(self.highlighted, |el| {
                el.bg(tokens.colors.accent)
                    .text_color(tokens.colors.accent_foreground)
                    .border_color(tokens.colors.ring)
            })
            .when(!disabled, |el| {
                el.hover(move |style| {
                    style
                        .bg(tokens.colors.muted)
                        .border_color(tokens.colors.ring)
                })
            })
            .when(disabled, |el| el.opacity(0.5))
            .when_some(
                (!disabled).then_some(self.on_click).flatten(),
                |el, on_click| el.on_click(move |event, window, cx| on_click(event, window, cx)),
            )
            .when_some(self.icon, |el, icon| el.child(icon_area().child(icon)))
            .when(self.size != RibbonItemSize::Small, |el| el.child(label))
            .when(
                self.key_tips_visible && !disabled && self.key_tip.is_some(),
                |el| {
                    el.child(key_tip_badge(
                        self.key_tip.expect("checked above"),
                        tokens.colors.surface,
                        tokens.colors.border,
                        tokens.colors.foreground,
                        tokens.radius.sm,
                    ))
                },
            )
            .refine_style(&self.style)
    }
}

/// The small key tip badge shown over an item while the overlay is active.
///
/// The badge covers the item's center, the way the Office ribbon overlays
/// key tips on their targets: it never leaves the item's bounds, so sibling
/// hover surfaces cannot paint over it.
pub(crate) fn key_tip_badge(
    key: SharedString,
    background: gpui::Hsla,
    border: gpui::Hsla,
    foreground: gpui::Hsla,
    radius: gpui::Pixels,
) -> gpui::Stateful<gpui::Div> {
    h_flex()
        .id(SharedString::from(format!("key-tip-{key}")))
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .items_center()
        .justify_center()
        .child(
            h_flex()
                .id(SharedString::from(format!("key-tip-chip-{key}")))
                .justify_center()
                .min_w(px(18.))
                .h(px(18.))
                .px_1()
                .bg(background)
                .border_1()
                .border_color(border)
                .rounded(radius)
                .text_size(px(11.))
                .text_color(foreground)
                .line_height(relative(1.))
                .child(key),
        )
}
