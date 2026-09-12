//! Dock panels implemented in this example — no story gallery involved.
//!
//! A panel is a plain GPUI entity that implements:
//!
//! - `BasePanel` (the re-export of `gpui_base::dock::Panel`): behavior. Only
//!   `panel_name` has no default; it identifies the panel in saved layouts.
//! - `Panel` (`gpui_component::dock::Panel`): presentation. Every method has
//!   a default; `title` is the one most panels override.
//! - `EventEmitter<PanelEvent>`, `Focusable`, `Render`: the GPUI trio the
//!   base trait requires.

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _,
    button::{Button, ButtonVariants as _},
    dock::{BasePanel, Panel, PanelEvent, TitleStyle, panel_handle, register_panel},
    input::{Textarea, TextareaState},
};
use gpui_kit::*;

/// Register every panel of this example, so `DockArea::load` can rebuild a
/// saved layout by looking `panel_name` up in this registry.
pub(crate) fn register_panels(cx: &mut App) {
    register_panel(cx, "WelcomePanel", |_, _, cx| {
        panel_handle(WelcomePanel::new(cx))
    });
    register_panel(cx, "EditorPanel", |_, window, cx| {
        panel_handle(EditorPanel::new(window, cx))
    });
    register_panel(cx, "FilesPanel", |_, _, cx| {
        panel_handle(ListPanel::files(cx))
    });
    register_panel(cx, "OutlinePanel", |_, _, cx| {
        panel_handle(ListPanel::outline(cx))
    });
    register_panel(cx, "OutputPanel", |_, _, cx| {
        panel_handle(OutputPanel::new(cx))
    });
}

/// The element drawn inside a tab: an icon plus a label.
fn panel_title(icon: IconName, label: &'static str) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_1p5()
        .child(Icon::new(icon).xsmall())
        .child(label)
        .into_any_element()
}

/// The minimal panel: a behavior name, a title, and static content.
pub(crate) struct WelcomePanel {
    focus_handle: FocusHandle,
}

impl WelcomePanel {
    pub(crate) fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
        })
    }
}

impl BasePanel for WelcomePanel {
    fn panel_name(&self) -> &'static str {
        "WelcomePanel"
    }
}

impl Panel for WelcomePanel {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        panel_title(IconName::Info, "Welcome")
    }
}

impl EventEmitter<PanelEvent> for WelcomePanel {}

impl Focusable for WelcomePanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for WelcomePanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let features = [
            "Every tab is a plain entity implementing the Panel traits.",
            "Center, left, and bottom docks are built with DockLayout.",
            "The layout is saved to target/dock-tabs.json and restored on start.",
            "Panels only move within their zone — a cross-zone drag snaps back.",
        ];

        div()
            .id("welcome-panel")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .child(div().text_xl().child("Dock Tabs Example"))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Multi-tab dock panels without the story gallery."),
            )
            .children(features.iter().map(|line| {
                div()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(format!("• {line}"))
            }))
    }
}

/// A stateful panel: a text area, plus the `set_active` activation hook.
pub(crate) struct EditorPanel {
    focus_handle: FocusHandle,
    textarea: Entity<TextareaState>,
}

impl EditorPanel {
    pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let textarea = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Type here. Closing and reopening this tab loses the text — panel state is not part of the layout.")
        });

        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
            textarea,
        })
    }
}

impl BasePanel for EditorPanel {
    fn panel_name(&self) -> &'static str {
        "EditorPanel"
    }

    /// Called when this tab becomes, or stops being, the displayed one.
    fn set_active(&mut self, active: bool, _: &mut Window, _: &mut Context<Self>) {
        println!("EditorPanel active: {active}");
    }
}

impl Panel for EditorPanel {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        panel_title(IconName::FileText, "Editor")
    }

    /// Short name used when a collapsed tab strip has no room for the title.
    fn tab_name(&self, _: &App) -> Option<SharedString> {
        Some("Editor".into())
    }
}

impl EventEmitter<PanelEvent> for EditorPanel {}

impl Focusable for EditorPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorPanel {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_1()
            .child(Textarea::new(&self.textarea).h_full())
    }
}

/// One panel implementation serving two tabs: the persisted `panel_name`
/// picks the content when the layout is restored.
pub(crate) struct ListPanel {
    focus_handle: FocusHandle,
    name: &'static str,
    title: &'static str,
    icon: IconName,
    items: Vec<&'static str>,
}

impl ListPanel {
    pub(crate) fn files(cx: &mut App) -> Entity<Self> {
        Self::build(
            "FilesPanel",
            "Files",
            IconName::FolderOpen,
            vec![
                "main.rs",
                "panels.rs",
                "workspace.rs",
                "theme.rs",
                "layout.rs",
            ],
            cx,
        )
    }

    pub(crate) fn outline(cx: &mut App) -> Entity<Self> {
        Self::build(
            "OutlinePanel",
            "Outline",
            IconName::BookOpen,
            vec![
                "struct Workspace",
                "impl Workspace",
                "fn new",
                "fn render",
                "impl Panel",
            ],
            cx,
        )
    }

    fn build(
        name: &'static str,
        title: &'static str,
        icon: IconName,
        items: Vec<&'static str>,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
            name,
            title,
            icon,
            items,
        })
    }
}

impl BasePanel for ListPanel {
    fn panel_name(&self) -> &'static str {
        self.name
    }
}

impl Panel for ListPanel {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        panel_title(self.icon.clone(), self.title)
    }
}

impl EventEmitter<PanelEvent> for ListPanel {}

impl Focusable for ListPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ListPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id(SharedString::from(self.name))
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_0p5()
            .p_2()
            .children(self.items.iter().map(|item| {
                div()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .text_sm()
                    .hover(|this| this.bg(cx.theme().accent))
                    .child(*item)
            }))
    }
}

/// A log panel: not closable, and with a tinted tab via `title_style`.
pub(crate) struct OutputPanel {
    focus_handle: FocusHandle,
    lines: Vec<SharedString>,
}

impl OutputPanel {
    pub(crate) fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
            lines: vec![
                "[00:00] Dock tabs example started.".into(),
                "[00:00] Toggle docks from the status bar below.".into(),
                "[00:01] Drag tabs to reorder or move them between docks.".into(),
            ],
        })
    }
}

impl BasePanel for OutputPanel {
    fn panel_name(&self) -> &'static str {
        "OutputPanel"
    }

    /// The last group of a dock already refuses to close; this makes the
    /// intent explicit for the log dock.
    fn closable(&self, _: &App) -> bool {
        false
    }
}

impl Panel for OutputPanel {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Output".into_any_element()
    }

    fn title_style(&self, cx: &App) -> Option<TitleStyle> {
        Some(TitleStyle {
            background: cx.theme().secondary,
            foreground: cx.theme().secondary_foreground,
        })
    }

    /// Panel-provided buttons: the tab bar places these to the left of its
    /// own controls while this panel is the displayed tab.
    fn toolbar_buttons(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        Some(vec![
            Button::new("clear-output")
                .ghost()
                .xsmall()
                .icon(IconName::CircleX)
                .tooltip("Clear output")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.lines.clear();
                    cx.notify();
                })),
        ])
    }
}

impl EventEmitter<PanelEvent> for OutputPanel {}

impl Focusable for OutputPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for OutputPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("output-panel")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_0p5()
            .p_2()
            .font_family(cx.theme().mono_font_family.clone())
            .text_sm()
            .children(self.lines.iter().map(|line| {
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child(line.clone())
            }))
    }
}
