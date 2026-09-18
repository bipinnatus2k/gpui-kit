use std::rc::Rc;

use gpui::{
    Anchor, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    div, prelude::FluentBuilder as _, px,
};
use gpui_base::{Popover, v_flex};

use crate::{
    face::{ChangeHandler, ClickHandler, ItemFace},
    size::RibbonItemSize,
    theme,
};

/// One tile of a [`RibbonGallery`]: an icon with a label beneath it.
pub struct RibbonGalleryEntry {
    id: ElementId,
    label: String,
    icon: Option<gpui::AnyElement>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl RibbonGalleryEntry {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into().to_string(),
            icon: None,
            disabled: false,
            on_click: None,
        }
    }

    /// The tile label, also used as the accessibility label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into().to_string();
        self
    }

    /// The tile icon, rendered above the label.
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

/// A ribbon item that opens a grid of icon tiles instead of a menu list,
/// following Fluent Ribbon's gallery control. Activating a tile runs its
/// command and closes the gallery.
#[derive(IntoElement)]
pub struct RibbonGallery {
    face: ItemFace,
    entries: Vec<RibbonGalleryEntry>,
    on_open_change: Option<ChangeHandler>,
}

impl RibbonGallery {
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

    /// The gallery label, also used as the accessibility label.
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.face.label = header.into();
        self
    }

    /// The gallery icon, rendered at the size the item's size mode dictates.
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

    /// The key tip shown on this item while the key tip overlay is active.
    /// Key tip activation is not wired for galleries; the badge is
    /// display-only.
    pub fn key_tip(mut self, key_tip: impl Into<SharedString>) -> Self {
        self.face.key_tip = Some(key_tip.into());
        self
    }

    /// Appends a tile to the gallery.
    pub fn entry(mut self, entry: RibbonGalleryEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Appends several tiles to the gallery.
    pub fn entries(mut self, entries: impl IntoIterator<Item = RibbonGalleryEntry>) -> Self {
        self.entries.extend(entries);
        self
    }

    /// Reports whether the gallery popup is open after each transition.
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

impl Styled for RibbonGallery {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.face.style
    }
}

impl RenderOnce for RibbonGallery {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let disabled = self.face.disabled;
        let gallery_id = (self.face.id.clone(), "gallery");

        Popover::new(gallery_id)
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
                    .content(move |_, _, cx| {
                        let tokens = theme::tokens(cx);
                        let popup = cx.entity();
                        let mut grid = div()
                            .id("gallery-grid")
                            .grid()
                            .grid_cols(3)
                            .gap_1()
                            .p_2()
                            .min_w(px(260.))
                            .max_w(px(340.))
                            .bg(tokens.colors.surface)
                            .border_1()
                            .border_color(tokens.colors.border)
                            .rounded(tokens.radius.md)
                            .shadow(tokens.shadow.md.clone())
                            .overflow_hidden();

                        for entry in self.entries {
                            let RibbonGalleryEntry {
                                id,
                                label,
                                icon,
                                disabled,
                                on_click,
                            } = entry;
                            let popup = popup.clone();
                            grid = grid.child(
                                v_flex()
                                    .id(id)
                                    .role(gpui::Role::MenuItem)
                                    .aria_label(label.clone())
                                    .items_center()
                                    .justify_center()
                                    .gap_1()
                                    .py_2()
                                    .px_1()
                                    .rounded(tokens.radius.sm)
                                    .text_size(tokens.typography.xs.size)
                                    .text_color(tokens.colors.surface_foreground)
                                    .line_height(gpui::relative(1.))
                                    .when(!disabled, |tile| {
                                        tile.hover(move |style| style.bg(tokens.colors.muted))
                                    })
                                    .when(disabled, |tile| tile.opacity(0.5))
                                    .when_some(
                                        (!disabled).then_some(on_click).flatten(),
                                        |tile, on_click| {
                                            tile.on_click(move |event, window, cx| {
                                                on_click(event, window, cx);
                                                popup.update(cx, |popup, cx| {
                                                    popup.dismiss(window, cx);
                                                });
                                            })
                                        },
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size(px(28.))
                                            .when_some(icon, |area, icon| area.child(icon)),
                                    )
                                    .child(label),
                            );
                        }

                        grid
                    })
            })
    }
}
