use gpui_kit::component::{Icon, IconName, Sizable as _, Size, button::Button, h_flex, v_flex};
use gpui_kit::{
    App, AppContext, ClickEvent, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement as _, Render, Styled, Window, div, hsla,
};

use gpui_ribbon::{
    Ribbon, RibbonApplicationMenu, RibbonButton, RibbonDropDownButton, RibbonGallery,
    RibbonGalleryEntry, RibbonGroup, RibbonGroupState, RibbonItemSize, RibbonMenuItem,
    RibbonMenuSeparator, RibbonSplitButton, RibbonTab, RibbonToggleButton,
};

use crate::section;

pub struct RibbonStory {
    focus_handle: FocusHandle,
    active_tab: usize,
    minimized: bool,
    content_open: bool,
    auto_collapse: bool,
    backstage_open: bool,
    home_state: RibbonGroupState,
    insert_state: RibbonGroupState,
    bold: bool,
    italic: bool,
    last_action: Option<&'static str>,
}

impl super::Story for RibbonStory {
    fn title() -> &'static str {
        "Ribbon"
    }

    fn description() -> &'static str {
        "Fluent-style ribbon: tab strip, labeled groups with large/medium/small \
        items, group scaling states, quick access toolbar, application menu, \
        split buttons and a minimized mode."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
        Self::view(window, cx)
    }
}

impl RibbonStory {
    pub fn view(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
            active_tab: 0,
            minimized: false,
            content_open: false,
            auto_collapse: false,
            backstage_open: false,
            home_state: RibbonGroupState::Large,
            insert_state: RibbonGroupState::Large,
            bold: false,
            italic: true,
            last_action: None,
        })
    }
}

impl RibbonStory {
    /// Applies a manual state transition to the active tab's groups.
    fn scale_active_tab(&mut self, reduce: bool) {
        let state = match self.active_tab {
            0 => &mut self.home_state,
            _ => &mut self.insert_state,
        };
        let next = if reduce {
            state.reduce()
        } else {
            state.enlarge()
        };
        if let Some(next) = next {
            *state = next;
        }
    }
}

impl Focusable for RibbonStory {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn record_click(
    entity: Entity<RibbonStory>,
    action: &'static str,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    move |_, _, cx| {
        entity.update(cx, |this, cx| {
            this.last_action = Some(action);
            cx.notify();
        });
    }
}

fn record_toggle(
    entity: Entity<RibbonStory>,
    name: &'static str,
) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
    move |checked, _, cx| {
        entity.update(cx, |this, cx| {
            if *checked {
                this.last_action = Some(name);
            }
            cx.notify();
        });
    }
}

impl Render for RibbonStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let bold = self.bold;
        let italic = self.italic;

        let home_state = self.home_state;
        let insert_state = self.insert_state;

        let home_tab = RibbonTab::new("home", "Home")
            .group(
                RibbonGroup::new("clipboard", "Clipboard")
                    .state(home_state)
                    .icon(Icon::new(IconName::Copy))
                    .item(
                        RibbonButton::new("paste", "Paste")
                            .icon(Icon::new(IconName::Copy).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .key_tip("V")
                            .on_click(record_click(entity.clone(), "Paste")),
                    )
                    .item(
                        RibbonButton::new("copy", "Copy")
                            .icon(Icon::new(IconName::Copy))
                            .size(RibbonItemSize::Medium)
                            .key_tip("C")
                            .on_click(record_click(entity.clone(), "Copy")),
                    )
                    .item(
                        RibbonButton::new("undo", "Undo")
                            .icon(Icon::new(IconName::Undo))
                            .size(RibbonItemSize::Medium)
                            .key_tip("Z")
                            .on_click(record_click(entity.clone(), "Undo")),
                    )
                    .item(
                        RibbonButton::new("delete", "Delete")
                            .icon(Icon::new(IconName::Delete))
                            .size(RibbonItemSize::Medium)
                            .key_tip("X")
                            .disabled(true),
                    ),
            )
            .group(
                RibbonGroup::new("font", "Font")
                    .state(home_state)
                    .icon(Icon::new(IconName::CaseSensitive))
                    .item(
                        RibbonToggleButton::new("bold", "Bold")
                            .icon(Icon::new(IconName::CaseSensitive))
                            .size(RibbonItemSize::Medium)
                            .checked(bold)
                            .on_change({
                                let entity = entity.clone();
                                move |checked, window, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.bold = *checked;
                                        cx.notify();
                                    });
                                    record_toggle(entity.clone(), "Bold")(checked, window, cx);
                                }
                            }),
                    )
                    .item(
                        RibbonToggleButton::new("italic", "Italic")
                            .size(RibbonItemSize::Medium)
                            .checked(italic)
                            .on_change({
                                let entity = entity.clone();
                                move |checked, window, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.italic = *checked;
                                        cx.notify();
                                    });
                                    record_toggle(entity.clone(), "Italic")(checked, window, cx);
                                }
                            }),
                    )
                    .item(
                        RibbonDropDownButton::new("case", "Case")
                            .icon(Icon::new(IconName::ALargeSmall))
                            .size(RibbonItemSize::Medium)
                            .entry(
                                RibbonMenuItem::new("upper", "UPPERCASE")
                                    .on_click(record_click(entity.clone(), "UPPERCASE")),
                            )
                            .entry(
                                RibbonMenuItem::new("lower", "lowercase")
                                    .on_click(record_click(entity.clone(), "lowercase")),
                            )
                            .entry(RibbonMenuSeparator)
                            .entry(
                                RibbonMenuItem::new("title", "Title Case")
                                    .on_click(record_click(entity.clone(), "Title Case")),
                            ),
                    ),
            )
            .group(
                RibbonGroup::new("editing", "Editing")
                    .item(
                        RibbonButton::new("find", "Find")
                            .icon(Icon::new(IconName::Search).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .key_tip("F")
                            .on_click(record_click(entity.clone(), "Find")),
                    )
                    .item(
                        RibbonGallery::new("shapes", "Shapes")
                            .icon(Icon::new(IconName::Frame))
                            .size(RibbonItemSize::Medium)
                            .entry(
                                RibbonGalleryEntry::new("rect", "Rectangle")
                                    .icon(Icon::new(IconName::Frame))
                                    .on_click(record_click(entity.clone(), "Rectangle")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("chart", "Chart")
                                    .icon(Icon::new(IconName::ChartPie))
                                    .on_click(record_click(entity.clone(), "Chart tile")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("globe", "Globe")
                                    .icon(Icon::new(IconName::Globe))
                                    .on_click(record_click(entity.clone(), "Globe")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("star", "Star")
                                    .icon(Icon::new(IconName::Star))
                                    .on_click(record_click(entity.clone(), "Star")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("calendar", "Calendar")
                                    .icon(Icon::new(IconName::Calendar))
                                    .on_click(record_click(entity.clone(), "Calendar tile")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("bot", "Bot")
                                    .icon(Icon::new(IconName::Bot))
                                    .disabled(true),
                            ),
                    )
                    .item(
                        RibbonSplitButton::new("sort", "Sort")
                            .icon(Icon::new(IconName::SortAscending))
                            .size(RibbonItemSize::Medium)
                            .on_click(record_click(entity.clone(), "Sort"))
                            .entry(
                                RibbonMenuItem::new("sort-asc", "Ascending")
                                    .icon(Icon::new(IconName::SortAscending))
                                    .on_click(record_click(entity.clone(), "Ascending")),
                            )
                            .entry(
                                RibbonMenuItem::new("sort-desc", "Descending")
                                    .icon(Icon::new(IconName::SortDescending))
                                    .on_click(record_click(entity.clone(), "Descending")),
                            ),
                    )
                    .item(
                        RibbonButton::new("replace", "Replace")
                            .icon(Icon::new(IconName::Replace))
                            .size(RibbonItemSize::Medium)
                            .on_click(record_click(entity.clone(), "Replace")),
                    )
                    .launcher(record_click(entity.clone(), "Font dialog")),
            );

        let insert_tab = RibbonTab::new("insert", "Insert")
            .contextual(hsla(0.58, 0.65, 0.55, 1.))
            .group(
                RibbonGroup::new("pages", "Pages")
                    .item(
                        RibbonButton::new("cover", "Cover Page")
                            .icon(Icon::new(IconName::FileText).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .on_click(record_click(entity.clone(), "Cover Page")),
                    )
                    .item(
                        RibbonButton::new("blank-page", "Blank Page")
                            .icon(Icon::new(IconName::File))
                            .size(RibbonItemSize::Medium)
                            .on_click(record_click(entity.clone(), "Blank Page")),
                    ),
            )
            .group(
                RibbonGroup::new("illustrations", "Illustrations")
                    .state(insert_state)
                    .item(
                        RibbonButton::new("chart", "Chart")
                            .icon(Icon::new(IconName::ChartPie).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .on_click(record_click(entity.clone(), "Chart")),
                    )
                    .item(
                        RibbonButton::new("calendar", "Calendar")
                            .icon(Icon::new(IconName::Calendar))
                            .size(RibbonItemSize::Medium)
                            .on_click(record_click(entity.clone(), "Calendar")),
                    ),
            );

        v_flex().gap_6().child(
            section("Ribbon")
                .description(
                    "A controlled Fluent-style ribbon. Click tabs to switch pages, \
                    press Alt to show key tips, click the File button for the \
                    backstage view, and use the toolbar below to minimize the \
                    ribbon, scale the Clipboard and Font groups, or toggle \
                    automatic width-driven collapse.",
                )
                .child(
                    v_flex().gap_3().child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .child(
                                Button::new("toggle-minimize")
                                    .label(if self.minimized {
                                        "Expand ribbon"
                                    } else {
                                        "Minimize ribbon"
                                    })
                                    .on_click({
                                        let entity = entity.clone();
                                        move |_, _, cx| {
                                            entity.update(cx, |this, cx| {
                                                this.minimized = !this.minimized;
                                                this.content_open = !this.minimized;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(Button::new("reduce-group").label("Reduce group").on_click({
                                let entity = entity.clone();
                                move |_, _, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.scale_active_tab(true);
                                        cx.notify();
                                    });
                                }
                            }))
                            .child(
                                Button::new("enlarge-group")
                                    .label("Enlarge group")
                                    .on_click({
                                        let entity = entity.clone();
                                        move |_, _, cx| {
                                            entity.update(cx, |this, cx| {
                                                this.scale_active_tab(false);
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new("toggle-auto-collapse")
                                    .label(if self.auto_collapse {
                                        "Disable auto collapse"
                                    } else {
                                        "Enable auto collapse"
                                    })
                                    .on_click({
                                        let entity = entity.clone();
                                        move |_, _, cx| {
                                            entity.update(cx, |this, cx| {
                                                this.auto_collapse = !this.auto_collapse;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(div().child(match self.last_action {
                                Some(action) => format!("Last action: {action}"),
                                None => "Click a ribbon item.".to_string(),
                            })),
                    ),
                )
                .child(
                    Ribbon::new("story-ribbon")
                        .active_tab(self.active_tab)
                        .minimized(self.minimized)
                        .content_open(self.content_open)
                        .auto_collapse(self.auto_collapse)
                        .backstage({
                            let entity = entity.clone();
                            move |_, _| {
                                v_flex()
                                    .gap_2()
                                    .p_4()
                                    .child(div().child(
                                        "The backstage view replaces the ribbon \
                                        surface with file-level commands.",
                                    ))
                                    .child(
                                        RibbonButton::new("bs-new", "New")
                                            .icon(
                                                Icon::new(IconName::FileText)
                                                    .with_size(Size::Large),
                                            )
                                            .size(RibbonItemSize::Large)
                                            .on_click(record_click(
                                                entity.clone(),
                                                "Backstage New",
                                            )),
                                    )
                                    .child(
                                        RibbonButton::new("bs-open", "Open")
                                            .icon(
                                                Icon::new(IconName::Folder).with_size(Size::Large),
                                            )
                                            .size(RibbonItemSize::Large)
                                            .on_click(record_click(
                                                entity.clone(),
                                                "Backstage Open",
                                            )),
                                    )
                                    .into_any_element()
                            }
                        })
                        .backstage_open(self.backstage_open)
                        .on_backstage_open_change({
                            let entity = entity.clone();
                            move |open, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.backstage_open = *open;
                                    cx.notify();
                                });
                            }
                        })
                        .application_menu(
                            RibbonApplicationMenu::new("File")
                                .entry(
                                    RibbonMenuItem::new("new", "New")
                                        .on_click(record_click(entity.clone(), "New")),
                                )
                                .entry(
                                    RibbonMenuItem::new("open", "Open…")
                                        .on_click(record_click(entity.clone(), "Open")),
                                )
                                .entry(RibbonMenuSeparator)
                                .entry(
                                    RibbonMenuItem::new("exit", "Exit")
                                        .on_click(record_click(entity.clone(), "Exit")),
                                ),
                        )
                        .quick_access([
                            RibbonButton::new("qat-undo", "Undo")
                                .icon(Icon::new(IconName::Undo))
                                .on_click(record_click(entity.clone(), "QAT Undo")),
                            RibbonButton::new("qat-redo", "Redo")
                                .icon(Icon::new(IconName::Redo))
                                .on_click(record_click(entity.clone(), "QAT Redo")),
                        ])
                        .on_tab_change({
                            let entity = entity.clone();
                            move |index, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.active_tab = index;
                                    cx.notify();
                                });
                            }
                        })
                        .on_content_open_change({
                            let entity = entity.clone();
                            move |open, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.content_open = *open;
                                    cx.notify();
                                });
                            }
                        })
                        .tab(home_tab)
                        .tab(insert_tab),
                ),
        )
    }
}
