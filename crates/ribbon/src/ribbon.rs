use std::{rc::Rc, time::Duration};

use gpui::{
    Anchor, Animation, AnimationExt as _, App, Bounds, ClickEvent, ElementId, FocusHandle,
    InteractiveElement as _, IntoElement, KeyDownEvent, Modifiers, ParentElement as _, Pixels,
    RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    div, prelude::FluentBuilder as _, px,
};
use gpui_base::{ElementExt as _, Popover, StyledExt as _, Tab, h_flex, v_flex};

use crate::{
    button::RibbonSeparator,
    dropdown::{RibbonMenuEntry, menu_panel},
    key_tip::KEY_TIP_CONTEXT,
    size::{RibbonGroupState, RibbonItemSize, solve_auto_states},
    tab::RibbonTab,
    theme,
};

type TabChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type ContentOpenHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;
type BackstageFactory = Rc<dyn Fn(&mut Window, &mut App) -> gpui::AnyElement>;

/// Per-ribbon key tip overlay state: the focus handle that receives the
/// keystrokes while the overlay (visibility on the application global) is
/// active.
struct RibbonKeyTipState {
    focus: FocusHandle,
}

/// One registered key tip: the declared key and the command it activates.
struct KeyTipEntry {
    key: SharedString,
    activate: crate::face::ClickHandler,
}

/// The application menu: a prominent button at the left end of the tab strip
/// that opens a panel of file-level commands, following Fluent Ribbon's
/// application menu.
pub struct RibbonApplicationMenu {
    label: SharedString,
    entries: Vec<RibbonMenuEntry>,
}

impl RibbonApplicationMenu {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            entries: Vec::new(),
        }
    }

    /// Appends a menu entry to the panel.
    pub fn entry(mut self, entry: impl Into<RibbonMenuEntry>) -> Self {
        self.entries.push(entry.into());
        self
    }

    /// Appends several menu entries to the panel.
    pub fn entries(
        mut self,
        entries: impl IntoIterator<Item = impl Into<RibbonMenuEntry>>,
    ) -> Self {
        self.entries.extend(entries.into_iter().map(Into::into));
        self
    }
}

/// Per-tab group states and the last measured widths of the automatic
/// width-driven collapse.
#[derive(Default)]
struct RibbonAutoState {
    available: Pixels,
    natural: Pixels,
    per_tab: Vec<Vec<RibbonGroupState>>,
}

impl RibbonAutoState {
    fn states_for(&self, tab: usize, group_count: usize) -> Vec<RibbonGroupState> {
        match self.per_tab.get(tab) {
            Some(states) if states.len() == group_count => states.clone(),
            _ => vec![RibbonGroupState::Large; group_count],
        }
    }

    /// Normalizes the stored states for the tab and returns them mutably.
    fn states_for_mut(&mut self, tab: usize, group_count: usize) -> &mut [RibbonGroupState] {
        if self.per_tab.len() <= tab {
            self.per_tab.resize(tab + 1, Vec::new());
        }
        let states = &mut self.per_tab[tab];
        if states.len() != group_count {
            *states = vec![RibbonGroupState::Large; group_count];
        }
        states
    }
}

/// The ribbon root: a tab strip over the active tab's groups, following the
/// Fluent Ribbon control model.
///
/// The ribbon is a controlled component — the application owns the selection
/// and the minimized state and mirrors them back through the builders:
///
/// - [`Ribbon::active_tab`] selects which [`RibbonTab`] is open;
///   [`Ribbon::on_tab_change`] reports strip clicks.
/// - [`Ribbon::minimized`] collapses the ribbon to the strip alone;
///   [`Ribbon::content_open`] says whether the minimized ribbon currently
///   shows its content again. Selecting another tab while minimized reports
///   both a tab change and `content_open = true`, so the strip clicks feel
///   like the Office ribbon without the crate holding any state.
///
/// Optional Fluent pieces slot into the strip: [`Ribbon::application_menu`]
/// sits at its left end and [`Ribbon::quick_access`] at its right end. With
/// [`Ribbon::auto_collapse`] enabled the ribbon keeps one group state chain
/// per tab and walks it toward `Collapsed` while the groups overflow the
/// available width, stepping back up when spare room returns.
#[derive(IntoElement)]
pub struct Ribbon {
    id: ElementId,
    tabs: Vec<RibbonTab>,
    active_tab: Option<usize>,
    on_tab_change: Option<TabChangeHandler>,
    minimized: bool,
    content_open: bool,
    on_content_open_change: Option<ContentOpenHandler>,
    quick_access: Vec<crate::item::RibbonItem>,
    application_menu: Option<RibbonApplicationMenu>,
    auto_collapse: bool,
    backstage: Option<BackstageFactory>,
    backstage_open: bool,
    on_backstage_open_change: Option<ContentOpenHandler>,
    style: StyleRefinement,
}

impl Ribbon {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            tabs: Vec::new(),
            active_tab: None,
            on_tab_change: None,
            minimized: false,
            content_open: false,
            on_content_open_change: None,
            quick_access: Vec::new(),
            application_menu: None,
            auto_collapse: false,
            backstage: None,
            backstage_open: false,
            on_backstage_open_change: None,
            style: StyleRefinement::default(),
        }
    }

    /// Appends a tab page to the ribbon.
    pub fn tab(mut self, tab: RibbonTab) -> Self {
        self.tabs.push(tab);
        self
    }

    /// Appends several tab pages to the ribbon.
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = RibbonTab>) -> Self {
        self.tabs.extend(tabs);
        self
    }

    /// The index of the currently active tab.
    pub fn active_tab(mut self, index: usize) -> Self {
        self.active_tab = Some(index);
        self
    }

    /// Reports clicks on the tab strip with the clicked tab's index.
    pub fn on_tab_change(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_tab_change = Some(Rc::new(handler));
        self
    }

    /// Whether the ribbon is minimized to the tab strip alone.
    pub fn minimized(mut self, minimized: bool) -> Self {
        self.minimized = minimized;
        self
    }

    /// Whether the minimized ribbon currently shows its content again.
    pub fn content_open(mut self, content_open: bool) -> Self {
        self.content_open = content_open;
        self
    }

    /// Reports minimized-content visibility changes.
    pub fn on_content_open_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_content_open_change = Some(Rc::new(handler));
        self
    }

    /// Appends an item to the quick access toolbar at the right end of the
    /// tab strip. Items render in the small, icon-only size.
    pub fn quick_access_item(mut self, item: impl Into<crate::item::RibbonItem>) -> Self {
        self.quick_access.push(item.into());
        self
    }

    /// Appends several items to the quick access toolbar.
    pub fn quick_access(
        mut self,
        items: impl IntoIterator<Item = impl Into<crate::item::RibbonItem>>,
    ) -> Self {
        self.quick_access.extend(items.into_iter().map(Into::into));
        self
    }

    /// Places an application menu button at the left end of the tab strip.
    pub fn application_menu(mut self, menu: RibbonApplicationMenu) -> Self {
        self.application_menu = Some(menu);
        self
    }

    /// Enables the automatic width-driven collapse: while the active tab's
    /// groups overflow the available width they walk their state chain
    /// toward `Collapsed`, one step per frame, and step back up when room
    /// returns. Explicit per-group states are ignored while enabled.
    pub fn auto_collapse(mut self, auto_collapse: bool) -> Self {
        self.auto_collapse = auto_collapse;
        self
    }

    /// Replaces the ribbon with a full-area backstage view while open. Once
    /// configured, the application menu button toggles the backstage instead
    /// of opening its popup.
    pub fn backstage(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.backstage = Some(Rc::new(content));
        self
    }

    /// Whether the backstage view is currently open.
    pub fn backstage_open(mut self, open: bool) -> Self {
        self.backstage_open = open;
        self
    }

    /// Reports backstage visibility changes (application menu button and the
    /// backstage back button).
    pub fn on_backstage_open_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_backstage_open_change = Some(Rc::new(handler));
        self
    }
}

impl Styled for Ribbon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Ribbon {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = theme::tokens(cx);
        let Ribbon {
            id,
            tabs,
            active_tab,
            on_tab_change,
            minimized,
            content_open,
            on_content_open_change,
            quick_access,
            application_menu,
            auto_collapse,
            backstage,
            backstage_open,
            on_backstage_open_change,
            style,
        } = self;

        let auto_state = auto_collapse.then(|| {
            window.use_keyed_state((id.clone(), "auto"), cx, |_, _| RibbonAutoState::default())
        });

        // Backstage mode replaces the whole ribbon surface.
        if backstage_open && let Some(backstage) = backstage.as_ref() {
            let content = backstage(window, cx);
            return v_flex()
                .id(id)
                .w_full()
                .bg(tokens.colors.background)
                .border_b_1()
                .border_color(tokens.colors.border)
                .child(backstage_header(
                    tokens.colors.muted,
                    tokens.colors.foreground,
                    tokens.colors.muted_foreground,
                    tokens.typography.sm.size,
                    tokens.radius.sm,
                    on_backstage_open_change.clone(),
                ))
                .child(div().w_full().child(content))
                .refine_style(&style);
        }

        let key_tip_state =
            window.use_keyed_state((id.clone(), "keytips"), cx, |_, cx| RibbonKeyTipState {
                focus: cx.focus_handle(),
            });
        let key_tips_visible = crate::key_tip::visible(cx);
        let key_tip_focus = key_tip_state.read(cx).focus.clone();
        if key_tips_visible && !key_tip_focus.contains_focused(window, cx) {
            key_tip_focus.focus(window, cx);
        }

        // Collect key tip registrations for keyboard activation.
        let key_tip_entries: Vec<KeyTipEntry> = tabs
            .iter()
            .flat_map(|tab| tab.groups.iter())
            .flat_map(|group| group.items.iter())
            .filter_map(|item| {
                item.key_tip_entry().map(|(key, activate)| KeyTipEntry {
                    key: key.to_string().into(),
                    activate,
                })
            })
            .collect();

        let mut strip = h_flex()
            .id("ribbon-tabs")
            .role(gpui::Role::TabList)
            .items_stretch()
            .bg(tokens.colors.muted)
            .px(px(8.))
            .pt(px(4.))
            .gap(px(2.));

        if let Some(menu) = application_menu {
            strip = strip.child(application_menu_button(
                menu,
                backstage.as_ref(),
                backstage_open,
                on_backstage_open_change.clone(),
                cx,
            ));
        }

        for (i, tab) in tabs.iter().enumerate() {
            let selected = active_tab == Some(i);
            strip = strip.child(strip_tab(
                tab,
                i,
                selected,
                minimized,
                content_open,
                active_tab,
                on_tab_change.clone(),
                on_content_open_change.clone(),
                &tokens,
            ));
        }

        let mut root = v_flex()
            .id(id)
            .w_full()
            .key_context(KEY_TIP_CONTEXT)
            .track_focus(&key_tip_focus)
            .bg(tokens.colors.background)
            .border_b_1()
            .border_color(tokens.colors.border);

        // Office 2010 places the quick access toolbar on its own slim row
        // above the tab strip, docked to the left like the title-bar area.
        if !quick_access.is_empty() {
            root =
                root.child(
                    h_flex()
                        .id("ribbon-quick-access")
                        .role(gpui::Role::Toolbar)
                        .w_full()
                        .h(px(26.))
                        .items_center()
                        .gap_1()
                        .px(px(8.))
                        .children(quick_access.into_iter().map(|item| {
                            item.render_at(Some(RibbonItemSize::Small), key_tips_visible)
                        })),
                );
        }

        root = root.child(strip);

        let shows_content = active_tab.is_some() && (!minimized || content_open);
        if shows_content {
            let active_index = active_tab.expect("checked by shows_content");

            // One width-driven collapse step per frame: decide from the last
            // measured widths, then let the prepaint callbacks re-measure.
            if let Some(auto_state) = &auto_state {
                let group_count = tabs.get(active_index).map_or(0, |tab| tab.groups.len());
                let changed = auto_state.update(cx, |state, _| {
                    let natural = state.natural;
                    let available = state.available;
                    let states = state.states_for_mut(active_index, group_count);
                    solve_auto_states(states, natural, available)
                });
                if changed {
                    window.request_animation_frame();
                }
            }

            root = root.child(content_row(
                tabs.into_iter().nth(active_index),
                auto_state.as_ref(),
                active_index,
                key_tips_visible,
                cx,
            ));
        }

        let root = root.on_key_down({
            move |event: &KeyDownEvent, window, cx| {
                if !crate::key_tip::visible(cx) {
                    return;
                }
                let key = event.keystroke.key.as_str();
                if key == "escape" {
                    crate::key_tip::set_visible(cx, false);
                    cx.stop_propagation();
                    return;
                }
                if event.keystroke.modifiers != Modifiers::default() {
                    return;
                }
                let entry = key_tip_entries
                    .iter()
                    .find(|entry| entry.key.as_ref().eq_ignore_ascii_case(key));
                if let Some(entry) = entry {
                    crate::key_tip::set_visible(cx, false);
                    (entry.activate)(&ClickEvent::default(), window, cx);
                    cx.stop_propagation();
                }
            }
        });

        if let Some(auto_state) = &auto_state {
            let auto_state = auto_state.clone();
            let root = root.on_prepaint(move |bounds: Bounds<Pixels>, window, cx| {
                let mut changed = false;
                auto_state.update(cx, |state, _| {
                    if state.available != bounds.size.width {
                        state.available = bounds.size.width;
                        changed = true;
                    }
                });
                if changed {
                    // The measurement feeds the next render's solve step.
                    // `request_animation_frame` explicitly schedules a
                    // platform frame, so the chain is not starved while the
                    // window sits idle.
                    window.request_animation_frame();
                }
            });
            root.refine_style(&style)
        } else {
            root.refine_style(&style)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn strip_tab(
    ribbon_tab: &RibbonTab,
    i: usize,
    selected: bool,
    minimized: bool,
    content_open: bool,
    active_tab: Option<usize>,
    on_tab_change: Option<TabChangeHandler>,
    on_content_open_change: Option<ContentOpenHandler>,
    tokens: &gpui_base::SemanticThemeTokens,
) -> Tab {
    // Office 2010 tab anatomy: unselected tabs melt into the strip band with
    // plain text (contextual tabs tinted with their group color), while the
    // selected tab brightens to the content background and joins it through a
    // colored underline along the strip's bottom edge.
    // The outline color: a contextual tab keeps its group color, a regular
    // tab falls back to the ring token (the palette's visible interactive
    // border; `accent` itself can be near-white in neutral palettes).
    let accent = ribbon_tab.contextual.unwrap_or(tokens.colors.ring);

    // Office 2010 tab anatomy: unselected tabs melt into the strip band with
    // plain text (contextual tabs tinted with their group color). The
    // selected tab is a raised folder tab — content background plus top and
    // side borders along the strip's bottom edge, open toward the content —
    // and the raised surface fades in over a few frames when the selection
    // moves. The surface is a child that only exists while selected, so its
    // animation state is discarded with it and replays on every switch.
    let raised_surface = selected.then(|| {
        // Folder-tab outline: top and side borders in the tab's accent color,
        // open at the bottom where the tab meets the content. The layer is
        // absolutely positioned so its border never affects tab layout.
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(tokens.colors.background)
            .rounded_t(tokens.radius.sm)
            .border_t(px(1.))
            .border_l(px(1.))
            .border_r(px(1.))
            .border_color(accent)
    });

    Tab::new(ribbon_tab.id.clone())
        .selected(selected)
        .px(px(12.))
        .py(px(6.))
        .rounded_t(tokens.radius.sm)
        .text_size(tokens.typography.sm.size)
        .relative()
        .when(selected, |tab| tab.text_color(tokens.colors.foreground))
        .when(!selected, |tab| {
            tab.text_color(
                ribbon_tab
                    .contextual
                    .unwrap_or(tokens.colors.muted_foreground),
            )
            .hover(move |style| style.bg(tokens.colors.secondary))
        })
        .when_some(raised_surface, |tab, surface| {
            tab.child(surface.with_animation(
                (ribbon_tab.id.clone(), "selected"),
                Animation::new(Duration::from_millis(150)),
                |surface, delta| surface.opacity(delta),
            ))
        })
        .child(ribbon_tab.header.clone())
        .on_click({
            let on_tab_change = on_tab_change.clone();
            let on_content_open_change = on_content_open_change.clone();
            move |_, window, cx| {
                if minimized {
                    if active_tab == Some(i) {
                        if let Some(handler) = &on_content_open_change {
                            handler(&!content_open, window, cx);
                        }
                    } else {
                        if let Some(handler) = &on_tab_change {
                            handler(i, window, cx);
                        }
                        if let Some(handler) = &on_content_open_change {
                            handler(&true, window, cx);
                        }
                    }
                } else if let Some(handler) = &on_tab_change {
                    handler(i, window, cx);
                }
            }
        })
}

fn application_menu_button(
    menu: RibbonApplicationMenu,
    backstage: Option<&BackstageFactory>,
    backstage_open: bool,
    on_backstage_open_change: Option<ContentOpenHandler>,
    cx: &App,
) -> gpui::AnyElement {
    // Office 2010 renders the application menu as a special tab at the left
    // end of the tab strip: solid `primary` with its own foreground, unlike
    // the document tabs next to it.
    let trigger = move |open: bool, cx: &App| {
        let tokens = theme::tokens(cx);
        h_flex()
            .id("app-menu-trigger")
            .role(gpui::Role::Button)
            .aria_label(menu.label.clone())
            .self_center()
            .items_center()
            .px_3()
            .h(px(26.))
            .rounded(tokens.radius.sm)
            .text_size(tokens.typography.sm.size)
            .bg(tokens.colors.primary)
            .text_color(tokens.colors.primary_foreground)
            .when(open, |trigger| trigger.opacity(0.85))
            .hover(move |style| style.opacity(0.85))
            .child(menu.label.clone())
    };

    // With a backstage configured, the application menu button toggles the
    // backstage view instead of opening its popup.
    if let Some(on_backstage_open_change) = on_backstage_open_change.filter(|_| backstage.is_some())
    {
        return trigger(backstage_open, cx)
            .on_click(move |_, window, cx| {
                on_backstage_open_change(&!backstage_open, window, cx);
            })
            .into_any_element();
    }

    Popover::new("ribbon-application-menu")
        .anchor(Anchor::BottomLeft)
        .trigger_with(move |open, _, cx| trigger(open, cx).into_any_element())
        .content(move |_, _, cx| {
            let popup = cx.entity();
            menu_panel(menu.entries, popup, cx)
        })
        .into_any_element()
}

fn backstage_header(
    background: gpui::Hsla,
    foreground: gpui::Hsla,
    hover: gpui::Hsla,
    text_size: gpui::Pixels,
    radius: gpui::Pixels,
    on_backstage_open_change: Option<ContentOpenHandler>,
) -> gpui::Stateful<gpui::Div> {
    h_flex()
        .id("backstage-header")
        .w_full()
        .items_center()
        .bg(background)
        .px(px(8.))
        .py(px(6.))
        .child(
            h_flex()
                .id("backstage-back")
                .role(gpui::Role::Button)
                .aria_label("Back")
                .items_center()
                .gap_1()
                .px_2()
                .h(px(26.))
                .rounded(radius)
                .text_size(text_size)
                .text_color(foreground)
                .hover(move |style| style.bg(hover))
                .child("←")
                .child("Back")
                .on_click(move |_, window, cx| {
                    if let Some(handler) = &on_backstage_open_change {
                        handler(&false, window, cx);
                    }
                }),
        )
}

fn content_row(
    tab: Option<RibbonTab>,
    auto_state: Option<&gpui::Entity<RibbonAutoState>>,
    active_index: usize,
    key_tips_visible: bool,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    let groups = tab.map(|tab| tab.groups).unwrap_or_default();

    let applied_states =
        auto_state.map(|state| state.read(cx).states_for(active_index, groups.len()));

    let mut row = h_flex()
        .id("ribbon-content")
        .role(gpui::Role::TabPanel)
        .items_stretch()
        .px(px(8.))
        .py(px(4.));

    for (i, mut group) in groups.into_iter().enumerate() {
        if i > 0 {
            row = row.child(RibbonSeparator::new());
        }
        group.key_tips_visible = key_tips_visible;
        row = row.child(match &applied_states {
            Some(states) => group.state(states[i]),
            None => group,
        });
    }

    if let Some(auto_state) = auto_state {
        let auto_state = auto_state.clone();
        row = row
            .self_start()
            .on_prepaint(move |bounds: Bounds<Pixels>, window, cx| {
                let mut changed = false;
                auto_state.update(cx, |state, _| {
                    if state.natural != bounds.size.width {
                        state.natural = bounds.size.width;
                        changed = true;
                    }
                });
                if changed {
                    // Re-measurements feed the next render's solve step;
                    // schedule a platform frame so the chain keeps converging
                    // while the window sits idle.
                    window.request_animation_frame();
                }
            });
    }

    row
}
