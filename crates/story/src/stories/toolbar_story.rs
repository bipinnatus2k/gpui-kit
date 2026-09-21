use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _, Size, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dock::PanelControl,
    h_flex,
    separator::Separator,
    toolbar::{Toolbar, ToolbarGroup},
    v_flex,
};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    Styled, Window, div, px,
};

use crate::section;

pub struct ToolbarStory {
    focus_handle: FocusHandle,
}

impl ToolbarStory {
    fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

fn icon_button(id: &'static str, icon: IconName, tooltip: &'static str) -> Button {
    Button::new(id)
        .ghost()
        .compact()
        .icon(icon)
        .tooltip(tooltip)
        .on_click(move |_, window, cx| window.push_notification(tooltip, cx))
}

fn size_example(
    label: &'static str,
    toolbar_id: &'static str,
    new_id: &'static str,
    undo_id: &'static str,
    redo_id: &'static str,
    search_id: &'static str,
    size: Size,
) -> impl IntoElement {
    h_flex()
        .w_full()
        .items_center()
        .gap_4()
        .child(div().w_16().child(label))
        .child(
            Toolbar::new(toolbar_id)
                .with_size(size)
                .child(
                    Button::new(new_id)
                        .ghost()
                        .compact()
                        .icon(IconName::Plus)
                        .label("New"),
                )
                .child(icon_button(undo_id, IconName::Undo2, "Undo"))
                .child(icon_button(redo_id, IconName::Redo2, "Redo"))
                .content(Separator::vertical().h_5())
                .child(icon_button(search_id, IconName::Search, "Search")),
        )
}

impl super::Story for ToolbarStory {
    fn title() -> &'static str {
        "Toolbar"
    }

    fn description() -> &'static str {
        "A transparent, sizable container for application commands."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        Self::view(window, cx)
    }

    fn zoomable() -> Option<PanelControl> {
        None
    }
}

impl Focusable for ToolbarStory {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ToolbarStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .items_center()
            .gap_6()
            .child(
                section("Document toolbar")
                    .description("A standard command bar with leading actions, a centered document name, grouped history commands, and trailing utilities.")
                    .w(px(760.))
                    .child(
                        div()
                            .w_full()
                            .border_1()
                            .rounded(cx.theme().radius)
                            .border_color(cx.theme().border)
                            .child(
                                Toolbar::new("document-toolbar")
                                    .w_full()
                                    .small()
                                    .left(
                                        Button::new("new-document")
                                            .ghost()
                                            .compact()
                                            .icon(IconName::Plus)
                                            .label("New")
                                            .on_click(|_, window, cx| window.push_notification("New document", cx)),
                                    )
                                    .left(icon_button("open-document", IconName::FolderOpen, "Open"))
                                    .left_content(Separator::vertical().h_5())
                                    .left(
                                        ToolbarGroup::new("history-commands")
                                            .label("History")
                                            .gap_1()
                                            .child(icon_button("undo", IconName::Undo2, "Undo"))
                                            .child(icon_button("redo", IconName::Redo2, "Redo")),
                                    )
                                    .content(div().text_color(cx.theme().muted_foreground).child("Quarterly report"))
                                    .right(icon_button("find", IconName::Search, "Find"))
                                    .right(icon_button("document-menu", IconName::Ellipsis, "More commands")),
                            ),
                    ),
            )
            .child(
                section("Table toolbar")
                    .description("A compact data toolbar. The surrounding table header owns the divider, while the toolbar sizes every command.")
                    .w(px(760.))
                    .child(
                        div()
                            .w_full()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                Toolbar::new("table-toolbar")
                                    .w_full()
                                    .small()
                                    .left_content("Open orders")
                                    .left_content(div().text_color(cx.theme().muted_foreground).child("24"))
                                    .right(
                                        Button::new("export-orders")
                                            .ghost()
                                            .compact()
                                            .icon(IconName::FileText)
                                            .label("Export…")
                                            .tooltip("Export orders"),
                                    )
                                    .right(icon_button("refresh-orders", IconName::RotateCw, "Refresh"))
                                    .right(icon_button("table-settings", IconName::Settings2, "Configure columns")),
                            ),
                    ),
            )
            .child(
                section("Sizes")
                    .description("The same command set at every supported density. Toolbar applies the size; its buttons use compact toolbar padding.")
                    .w(px(760.))
                    .child(
                        v_flex()
                            .w_full()
                            .gap_2()
                            .child(size_example("XSmall", "xsmall-toolbar", "xsmall-new", "xsmall-undo", "xsmall-redo", "xsmall-search", Size::XSmall))
                            .child(size_example("Small", "small-toolbar", "small-new", "small-undo", "small-redo", "small-search", Size::Small))
                            .child(size_example("Medium", "medium-toolbar", "medium-new", "medium-undo", "medium-redo", "medium-search", Size::Medium))
                            .child(size_example("Large", "large-toolbar", "large-new", "large-undo", "large-redo", "large-search", Size::Large)),
                    ),
            )
    }
}
