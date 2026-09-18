use gpui_kit::assets::Assets;
use gpui_kit::component::{
    ActiveTheme, Icon, IconName, Root, Sizable as _, Size, button::Button, checkbox::Checkbox,
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, App, AppContext, ClickEvent, Context, Entity, FocusHandle, Focusable, FontWeight,
    Hsla, IntoElement, ParentElement as _, Render, SharedString, Styled, Window, WindowOptions,
    div, hsla, px,
};
use gpui_ribbon::{
    Ribbon, RibbonApplicationMenu, RibbonButton, RibbonDropDownButton, RibbonGallery,
    RibbonGalleryEntry, RibbonGroup, RibbonGroupState, RibbonItemSize, RibbonMenuItem,
    RibbonMenuSeparator, RibbonSplitButton, RibbonTab, RibbonToggleButton,
};

fn main() {
    let app = gpui_kit::application().with_assets(Assets);

    app.run(move |cx| {
        gpui_kit::init(cx);
        // Registers the ribbon key tip bindings (`alt` / `f10`).
        gpui_ribbon::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| RibbonDemo::new(cx));
                // The first level view of a window must be a Root.
                cx.new(|cx| Root::new(view, window, cx).bg(cx.theme().background))
            })
            .expect("Failed to open example window");
        })
        .detach();
    });
}

/// The application state behind the ribbon. Every ribbon control is a
/// stateless view; the application owns all of it and mirrors it back through
/// the builders, the way the Fluent.Ribbon showcase drives its view models.
struct RibbonDemo {
    focus_handle: FocusHandle,
    active_tab: usize,
    minimized: bool,
    content_open: bool,
    auto_collapse: bool,
    backstage_open: bool,
    table_tools_visible: bool,
    /// Manual group states, tracked per tab so Reduce/Enlarge always acts on
    /// the active tab. Unused while auto collapse is on.
    manual_states: [RibbonGroupState; 4],
    bold: bool,
    italic: bool,
    log: Vec<String>,
}

/// The tabs in display order; the Table Tools slot is only present while the
/// contextual tab is visible, which shifts View's index.
const TAB_IDS: [&str; 4] = ["home", "insert", "table-tools", "view"];
impl RibbonDemo {
    fn new(cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            active_tab: 0,
            minimized: false,
            content_open: false,
            auto_collapse: true,
            backstage_open: false,
            table_tools_visible: false,
            manual_states: [RibbonGroupState::Large; 4],
            bold: false,
            italic: false,
            log: vec!["Ready. Press Alt (or F10) to show key tips.".into()],
        }
    }

    fn record(&mut self, action: &str) {
        self.log.push(action.to_string());
        if self.log.len() > 30 {
            self.log.remove(0);
        }
    }

    /// The manual group state of the active tab, if it has one.
    fn active_tab_id(&self) -> Option<&'static str> {
        match self.active_tab {
            0 => Some("home"),
            1 => Some("insert"),
            2 if self.table_tools_visible => Some("table-tools"),
            2 => Some("view"),
            _ => Some("view"),
        }
    }

    /// Applies a state transition to the active tab and turns the automatic
    /// width-driven collapse off, which would otherwise override the manual
    /// chain on the next frames.
    fn scale_active_tab(&mut self, reduce: bool) {
        self.auto_collapse = false;
        let Some(id) = self.active_tab_id() else {
            return;
        };
        let Some(index) = TAB_IDS.iter().position(|tab| *tab == id) else {
            return;
        };
        let Some(slot) = self.manual_states.get_mut(index) else {
            return;
        };
        let next = if reduce {
            slot.reduce()
        } else {
            slot.enlarge()
        };
        if let Some(next) = next {
            *slot = next;
            self.record(&format!("{id} scaled to {next:?}"));
        }
    }
}

impl Focusable for RibbonDemo {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RibbonDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let background = cx.theme().background;
        let accent = cx.theme().accent;

        let mut ribbon = Ribbon::new("demo-ribbon")
            .active_tab(self.active_tab)
            .minimized(self.minimized)
            .content_open(self.content_open)
            .auto_collapse(self.auto_collapse)
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
            .backstage(backstage_page(entity.clone()))
            .application_menu(
                RibbonApplicationMenu::new("File")
                    .entry(
                        RibbonMenuItem::new("app-new", "New")
                            .on_click(record_click(&entity, "New document")),
                    )
                    .entry(
                        RibbonMenuItem::new("app-open", "Open…")
                            .on_click(record_click(&entity, "Open document")),
                    )
                    .entry(RibbonMenuSeparator)
                    .entry(RibbonMenuItem::new("app-exit", "Exit").on_click(|_, _, cx| cx.quit())),
            )
            .quick_access([
                RibbonButton::new("qat-undo", "Undo")
                    .icon(Icon::new(IconName::Undo))
                    .key_tip("1")
                    .on_click(record_click(&entity, "Quick undo")),
                RibbonButton::new("qat-redo", "Redo")
                    .icon(Icon::new(IconName::Redo))
                    .key_tip("2")
                    .on_click(record_click(&entity, "Quick redo")),
                RibbonButton::new("qat-save", "Save")
                    .icon(Icon::new(IconName::Check))
                    .key_tip("3")
                    .on_click(record_click(&entity, "Save")),
            ])
            .tab(self.home_tab(&entity, self.manual_states[0]))
            .tab(self.insert_tab(&entity, self.manual_states[1]));

        if self.table_tools_visible {
            ribbon = ribbon.tab(self.table_tools_tab(&entity, self.manual_states[2]));
        }

        ribbon = ribbon.tab(self.view_tab(&entity));

        v_flex()
            .size_full()
            .child(ribbon)
            .child(self.document_area(&entity, background, accent))
    }
}

impl RibbonDemo {
    fn home_tab(&self, entity: &Entity<Self>, state: RibbonGroupState) -> RibbonTab {
        RibbonTab::new("home", "Home")
            .group(
                RibbonGroup::new("clipboard", "Clipboard")
                    .state(state)
                    .icon(Icon::new(IconName::Copy))
                    .item(
                        RibbonButton::new("paste", "Paste")
                            .icon(Icon::new(IconName::Copy).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .key_tip("V")
                            .on_click(record_click(entity, "Paste")),
                    )
                    .item(
                        RibbonButton::new("copy", "Copy")
                            .icon(Icon::new(IconName::Copy))
                            .size(RibbonItemSize::Medium)
                            .key_tip("C")
                            .on_click(record_click(entity, "Copy")),
                    )
                    .item(
                        RibbonButton::new("cut", "Cut")
                            .icon(Icon::new(IconName::Delete))
                            .size(RibbonItemSize::Medium)
                            .key_tip("X")
                            .on_click(record_click(entity, "Cut")),
                    )
                    .item(
                        RibbonButton::new("undo", "Undo")
                            .icon(Icon::new(IconName::Undo))
                            .size(RibbonItemSize::Medium)
                            .key_tip("Z")
                            .on_click(record_click(entity, "Undo")),
                    )
                    .item(
                        RibbonButton::new("delete", "Delete")
                            .icon(Icon::new(IconName::Minus))
                            .size(RibbonItemSize::Medium)
                            .key_tip("D")
                            .disabled(true),
                    ),
            )
            .group(
                RibbonGroup::new("font", "Font")
                    .state(state)
                    .icon(Icon::new(IconName::CaseSensitive))
                    .item(
                        RibbonToggleButton::new("bold", "Bold")
                            .icon(Icon::new(IconName::CaseSensitive))
                            .size(RibbonItemSize::Medium)
                            .key_tip("B")
                            .checked(self.bold)
                            .on_change(toggle_change(entity, "Bold")),
                    )
                    .item(
                        RibbonToggleButton::new("italic", "Italic")
                            .size(RibbonItemSize::Medium)
                            .key_tip("I")
                            .checked(self.italic)
                            .on_change(toggle_change(entity, "Italic")),
                    )
                    .item(
                        RibbonDropDownButton::new("case", "Case")
                            .icon(Icon::new(IconName::ALargeSmall))
                            .size(RibbonItemSize::Medium)
                            .entry(
                                RibbonMenuItem::new("upper", "UPPERCASE")
                                    .on_click(record_click(entity, "UPPERCASE")),
                            )
                            .entry(
                                RibbonMenuItem::new("lower", "lowercase")
                                    .on_click(record_click(entity, "lowercase")),
                            )
                            .entry(RibbonMenuSeparator)
                            .entry(
                                RibbonMenuItem::new("title", "Title Case")
                                    .on_click(record_click(entity, "Title Case")),
                            ),
                    )
                    .launcher(record_click(entity, "Font dialog")),
            )
            .group(
                RibbonGroup::new("editing", "Editing")
                    .item(
                        RibbonButton::new("find", "Find")
                            .icon(Icon::new(IconName::Search))
                            .size(RibbonItemSize::Large)
                            .key_tip("F")
                            .on_click(record_click(entity, "Find")),
                    )
                    .item(
                        RibbonSplitButton::new("sort", "Sort")
                            .icon(Icon::new(IconName::SortAscending))
                            .size(RibbonItemSize::Medium)
                            .on_click(record_click(entity, "Sort"))
                            .entry(
                                RibbonMenuItem::new("sort-asc", "Ascending")
                                    .icon(Icon::new(IconName::SortAscending))
                                    .on_click(record_click(entity, "Ascending")),
                            )
                            .entry(
                                RibbonMenuItem::new("sort-desc", "Descending")
                                    .icon(Icon::new(IconName::SortDescending))
                                    .on_click(record_click(entity, "Descending")),
                            ),
                    )
                    .item(
                        RibbonButton::new("replace", "Replace")
                            .icon(Icon::new(IconName::Replace))
                            .size(RibbonItemSize::Medium)
                            .on_click(record_click(entity, "Replace")),
                    ),
            )
    }

    fn insert_tab(&self, entity: &Entity<Self>, state: RibbonGroupState) -> RibbonTab {
        RibbonTab::new("insert", "Insert")
            .contextual(hsla(0.58, 0.65, 0.55, 1.))
            .group(
                RibbonGroup::new("pages", "Pages")
                    .state(state)
                    .item(
                        RibbonButton::new("cover", "Cover Page")
                            .icon(Icon::new(IconName::FileText).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .on_click(record_click(entity, "Cover Page")),
                    )
                    .item(
                        RibbonButton::new("blank-page", "Blank Page")
                            .icon(Icon::new(IconName::File))
                            .size(RibbonItemSize::Medium)
                            .on_click(record_click(entity, "Blank Page")),
                    ),
            )
            .group(
                RibbonGroup::new("illustrations", "Illustrations")
                    .item(
                        RibbonGallery::new("shapes", "Shapes")
                            .icon(Icon::new(IconName::Frame))
                            .size(RibbonItemSize::Medium)
                            .entry(
                                RibbonGalleryEntry::new("rect", "Rectangle")
                                    .icon(Icon::new(IconName::Frame))
                                    .on_click(record_click(entity, "Rectangle shape")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("chart", "Chart")
                                    .icon(Icon::new(IconName::ChartPie))
                                    .on_click(record_click(entity, "Chart shape")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("globe", "Globe")
                                    .icon(Icon::new(IconName::Globe))
                                    .on_click(record_click(entity, "Globe shape")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("star", "Star")
                                    .icon(Icon::new(IconName::Star))
                                    .on_click(record_click(entity, "Star shape")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("heart", "Heart")
                                    .icon(Icon::new(IconName::Heart))
                                    .on_click(record_click(entity, "Heart shape")),
                            )
                            .entry(
                                RibbonGalleryEntry::new("bot", "Bot")
                                    .icon(Icon::new(IconName::Bot))
                                    .disabled(true),
                            ),
                    )
                    .item(
                        RibbonButton::new("chart", "Chart")
                            .icon(Icon::new(IconName::ChartPie).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .on_click(record_click(entity, "Chart")),
                    )
                    .item(
                        RibbonButton::new("calendar", "Calendar")
                            .icon(Icon::new(IconName::Calendar))
                            .size(RibbonItemSize::Medium)
                            .on_click(record_click(entity, "Calendar")),
                    ),
            )
    }

    fn table_tools_tab(&self, entity: &Entity<Self>, state: RibbonGroupState) -> RibbonTab {
        RibbonTab::new("table-tools", "Table Tools")
            .contextual(hsla(0.08, 0.7, 0.55, 1.))
            .group(
                RibbonGroup::new("rows", "Rows & Columns")
                    .state(state)
                    .item(
                        RibbonButton::new("row-above", "Insert Above")
                            .icon(Icon::new(IconName::Plus).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .on_click(record_click(entity, "Insert row above")),
                    )
                    .item(
                        RibbonButton::new("row-below", "Insert Below")
                            .icon(Icon::new(IconName::Plus))
                            .size(RibbonItemSize::Medium)
                            .on_click(record_click(entity, "Insert row below")),
                    )
                    .item(
                        RibbonButton::new("row-delete", "Delete Row")
                            .icon(Icon::new(IconName::Minus))
                            .size(RibbonItemSize::Medium)
                            .disabled(true),
                    ),
            )
            .group(
                RibbonGroup::new("table-styles", "Table Styles").item(
                    RibbonGallery::new("table-style-gallery", "Styles")
                        .icon(Icon::new(IconName::LayoutDashboard))
                        .size(RibbonItemSize::Medium)
                        .entry(
                            RibbonGalleryEntry::new("style-grid", "Grid")
                                .icon(Icon::new(IconName::LayoutDashboard))
                                .on_click(record_click(entity, "Grid style")),
                        )
                        .entry(
                            RibbonGalleryEntry::new("style-panels", "Panels")
                                .icon(Icon::new(IconName::PanelLeft))
                                .on_click(record_click(entity, "Panels style")),
                        )
                        .entry(
                            RibbonGalleryEntry::new("style-rows", "Rows")
                                .icon(Icon::new(IconName::PanelRight))
                                .on_click(record_click(entity, "Rows style")),
                        ),
                ),
            )
    }

    fn view_tab(&self, entity: &Entity<Self>) -> RibbonTab {
        RibbonTab::new("view", "View").group(
            RibbonGroup::new("show", "Show")
                .item(custom_checkbox(
                    "show-table-tools",
                    "Table Tools tab",
                    self.table_tools_visible,
                    |this, checked| this.table_tools_visible = checked,
                    entity,
                ))
                .item(custom_checkbox(
                    "auto-collapse",
                    "Auto collapse",
                    self.auto_collapse,
                    |this, checked| this.auto_collapse = checked,
                    entity,
                )),
        )
    }

    fn document_area(
        &self,
        entity: &Entity<Self>,
        background: Hsla,
        accent: Hsla,
    ) -> impl IntoElement {
        let recent: Vec<String> = self.log.iter().rev().take(6).rev().cloned().collect();

        h_flex()
            .flex_1()
            .min_h_0()
            .items_stretch()
            .gap_4()
            .p_4()
            .bg(background)
            .child(
                // The demo control column, mirroring the showcase's option
                // panel.
                v_flex()
                    .w(px(300.))
                    .gap_3()
                    .child(section_title("Showcase controls"))
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
                    .child(
                        Button::new("toggle-table-tools")
                            .label(if self.table_tools_visible {
                                "Hide Table Tools tab"
                            } else {
                                "Show Table Tools tab"
                            })
                            .on_click({
                                let entity = entity.clone();
                                move |_, _, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.table_tools_visible = !this.table_tools_visible;
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
                        v_flex()
                            .gap_1()
                            .child(section_title("Try"))
                            .child(div().child(
                                "Press Alt or F10 for key tips, then a badge key. Resize \
                        the window narrower to watch auto collapse. Click File \
                        for the backstage view.",
                            )),
                    ),
            )
            .child(
                // The fake document with the command log.
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .gap_3()
                    .child(section_title("Command log"))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .rounded(px(6.))
                            .border_1()
                            .border_color(accent.opacity(0.25))
                            .p_3()
                            .children(recent.into_iter().map(|line| {
                                div().py_0p5().child(SharedString::from(log_line(&line)))
                            })),
                    ),
            )
    }
}

fn backstage_page(entity: Entity<RibbonDemo>) -> impl Fn(&mut Window, &mut App) -> AnyElement {
    move |_, _| {
        v_flex()
            .gap_3()
            .p_4()
            .child(div().child(
                "The backstage view replaces the ribbon surface with file-level \
                commands.",
            ))
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_4()
                    .child(
                        RibbonButton::new("bs-new", "New")
                            .icon(Icon::new(IconName::FileText).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .on_click(record_click(&entity, "Backstage New")),
                    )
                    .child(
                        RibbonButton::new("bs-open", "Open")
                            .icon(Icon::new(IconName::Folder).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .on_click(record_click(&entity, "Backstage Open")),
                    )
                    .child(
                        RibbonButton::new("bs-export", "Export")
                            .icon(Icon::new(IconName::ExternalLink).with_size(Size::Large))
                            .size(RibbonItemSize::Large)
                            .disabled(true),
                    ),
            )
            .into_any_element()
    }
}

/// A component checkbox dropped into a ribbon group as a custom item.
fn custom_checkbox(
    id: &'static str,
    label: &'static str,
    checked: bool,
    apply: impl Fn(&mut RibbonDemo, bool) + 'static,
    entity: &Entity<RibbonDemo>,
) -> AnyElement {
    let entity = entity.clone();
    div()
        .py_1()
        .child(
            Checkbox::new(id)
                .label(label)
                .checked(checked)
                .on_click(move |checked, _, cx| {
                    entity.update(cx, |this, cx| {
                        apply(this, *checked);
                        cx.notify();
                    });
                }),
        )
        .into_any_element()
}

fn section_title(title: &'static str) -> impl IntoElement {
    div()
        .text_size(px(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(title)
}

fn log_line(action: &str) -> String {
    format!("▶ {action}")
}

fn record_click(
    entity: &Entity<RibbonDemo>,
    action: &'static str,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let entity = entity.clone();
    move |_, _, cx| {
        entity.update(cx, |this, cx| {
            this.record(action);
            cx.notify();
        });
    }
}

fn toggle_change(
    entity: &Entity<RibbonDemo>,
    name: &'static str,
) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
    let entity = entity.clone();
    move |checked, _, cx| {
        entity.update(cx, |this, cx| {
            this.record(&format!("{name}: {checked}"));
            cx.notify();
        });
    }
}
