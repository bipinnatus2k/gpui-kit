//! A dock workspace whose tabs are panels implemented in this example —
//! it does not depend on the `gpui-component-story` gallery.
//!
//! Compare `examples/dock`, which builds the same shell on top of
//! `StoryContainer::panel::<T>()`.

mod panels;
mod tab_bar;

use anyhow::Context as _;
use gpui_kit::component::{
    ActiveTheme as _, IconName, Root, Sizable as _, TitleBar, WindowExt,
    button::{Button, ButtonVariants as _},
    dock::{
        ClosePanel, DockArea, DockAreaState, DockEvent, DockLayout, DockPlacement, DockSkin,
        InsertTarget, PaneRef, PanelStyle, ToggleZoom, panel_handle,
    },
    status_bar::StatusBar,
};
use gpui_kit::*;
use std::time::Duration;

const DOCK_AREA_ID: &str = "dock-tabs";
const DOCK_AREA_VERSION: usize = 1;

#[cfg(debug_assertions)]
const STATE_FILE: &str = "target/dock-tabs.json";
#[cfg(not(debug_assertions))]
const STATE_FILE: &str = "dock-tabs.json";

/// Which region of the layout a panel kind belongs to. The rule is keyed by
/// `panel_name`, so it survives save/load round trips.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Zone {
    /// Files and projects — the center workspace.
    Workspace,
    /// Tool panels — the side and bottom docks.
    Tools,
}

fn zone_of(panel_name: &str) -> Option<Zone> {
    match panel_name {
        "EditorPanel" | "WelcomePanel" => Some(Zone::Workspace),
        "FilesPanel" | "OutlinePanel" | "OutputPanel" => Some(Zone::Tools),
        _ => None,
    }
}

fn zone_of_placement(placement: DockPlacement) -> Option<Zone> {
    match placement {
        DockPlacement::Center => Some(Zone::Workspace),
        DockPlacement::Left | DockPlacement::Right | DockPlacement::Bottom => Some(Zone::Tools),
    }
}

/// A drop target inside the panel's home zone: its first tab group. A dock
/// can never be emptied by dragging — the last visible panel of a lone group
/// is not draggable — so the home zone always has a group to receive the
/// panel back.
fn home_target(area: &DockArea, home: Zone) -> Option<InsertTarget> {
    let placements: &[DockPlacement] = match home {
        Zone::Workspace => &[DockPlacement::Center],
        Zone::Tools => &[DockPlacement::Left, DockPlacement::Bottom],
    };
    for placement in placements {
        let Some(tree) = area.layout(*placement) else {
            continue;
        };
        for node_id in tree.node_ids() {
            let Some(node) = tree.find_node(node_id) else {
                continue;
            };
            if matches!(node.kind(), PaneRef::Tabs { .. }) {
                return Some(InsertTarget::Tabs {
                    node: node_id,
                    ix: None,
                    activate: true,
                });
            }
        }
    }
    None
}

pub fn init(cx: &mut App) {
    // Installs gpui_component::init — components are ready after this.
    gpui_kit::init(cx);
    panels::register_panels(cx);

    cx.bind_keys(vec![
        KeyBinding::new("shift-escape", ToggleZoom, None),
        KeyBinding::new("ctrl-w", ClosePanel, None),
    ]);

    cx.activate(true);
}

pub struct Workspace {
    dock_area: Entity<DockArea>,
    last_layout_state: Option<DockAreaState>,
    _save_layout_task: Option<Task<()>>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Build the area with the application-owned tab bar: the custom
        // renderer wraps the built-in DockSkin and only takes over the tab
        // bar, adding a close button and a right-click menu to every tab.
        // The returned skin handle is what an app keeps to change the panel
        // style, the toggle-button visibility, or the tiles scrollbar mode.
        let mut skin = None;
        let dock_area = cx.new(|cx| {
            let built = DockSkin::new(cx);
            let renderer = tab_bar::TabBarSkin::new(built.clone());
            skin = Some(built);
            DockArea::new(DOCK_AREA_ID, Some(DOCK_AREA_VERSION), window, cx).with_renderer(renderer)
        });
        let skin = skin.expect("DockSkin::new ran inside the constructor");
        // The custom tab bar always draws real tabs; keep the stock style in
        // sync for the one case still delegated to it: the collapsed strip.
        skin.set_panel_style(PanelStyle::TabBar, cx);
        let weak_dock_area = dock_area.downgrade();

        match Self::load_layout(dock_area.clone(), window, cx) {
            Ok(_) => {
                println!("load layout success");
            }
            Err(err) => {
                eprintln!("load layout error: {err:?}");
                Self::reset_default_layout(weak_dock_area, window, cx);
            }
        };

        cx.subscribe_in(
            &dock_area,
            window,
            |this, dock_area, ev: &DockEvent, window, cx| match ev {
                DockEvent::LayoutChanged => {
                    // Refuse cross-zone moves before persisting the layout.
                    this.enforce_zones(window, cx);
                    this.save_layout(dock_area, window, cx);
                }
                _ => {}
            },
        )
        .detach();

        cx.on_app_quit({
            let dock_area = dock_area.clone();
            move |_, cx| {
                let state = dock_area.read(cx).dump(cx);
                cx.background_executor().spawn(async move {
                    // Save layout before quitting
                    Self::save_state(&state).unwrap();
                })
            }
        })
        .detach();

        Self {
            dock_area,
            last_layout_state: None,
            _save_layout_task: None,
        }
    }

    fn save_layout(
        &mut self,
        dock_area: &Entity<DockArea>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let dock_area = dock_area.clone();
        self._save_layout_task = Some(cx.spawn_in(window, async move |workspace, window| {
            window
                .background_executor()
                .timer(Duration::from_secs(10))
                .await;

            _ = workspace.update_in(window, move |this, _, cx| {
                let dock_area = dock_area.read(cx);
                let state = dock_area.dump(cx);

                let last_layout_state = this.last_layout_state.clone();
                if Some(&state) == last_layout_state.as_ref() {
                    return;
                }

                Self::save_state(&state).unwrap();
                this.last_layout_state = Some(state);
            });
        }));
    }

    /// Roll back any panel sitting in a zone its kind does not belong to.
    /// Runs on every `LayoutChanged`; a legal layout makes it a no-op, so the
    /// revert's own `LayoutChanged` event does not loop.
    fn enforce_zones(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let misplaced = self.dock_area.update(cx, |area, cx| {
            let mut misplaced = Vec::new();
            for placement in [
                DockPlacement::Center,
                DockPlacement::Left,
                DockPlacement::Bottom,
            ] {
                let Some(zone) = zone_of_placement(placement) else {
                    continue;
                };
                let Some(tree) = area.layout(placement) else {
                    continue;
                };
                for panel_id in tree.panels() {
                    let home = area
                        .panel(panel_id)
                        .map(|panel| panel.panel_name(cx))
                        .and_then(zone_of);
                    if let Some(home) = home.filter(|home| *home != zone) {
                        misplaced.push((panel_id, home));
                    }
                }
            }
            misplaced
        });

        if misplaced.is_empty() {
            return;
        }

        let mut reverted: Vec<&'static str> = Vec::new();
        self.dock_area.update(cx, |area, cx| {
            for (panel_id, home) in misplaced {
                let name = area
                    .panel(panel_id)
                    .map(|panel| panel.panel_name(cx))
                    .unwrap_or("");
                let Some(target) = home_target(area, home) else {
                    continue;
                };
                area.move_panel(panel_id, target, window, cx);
                println!("zone violation: {name} sent back to its home zone");
                reverted.push(name);
            }
        });

        if !reverted.is_empty() {
            window.push_notification(
                format!("{} cannot live in this zone", reverted.join(", ")),
                cx,
            );
        }
    }

    fn save_state(state: &DockAreaState) -> std::io::Result<()> {
        println!("Save layout...");
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(STATE_FILE, json)?;
        Ok(())
    }

    fn load_layout(
        dock_area: Entity<DockArea>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let json = std::fs::read_to_string(STATE_FILE)?;
        let state = serde_json::from_str::<DockAreaState>(&json)?;

        // Bump DOCK_AREA_VERSION when the default layout changes; offer the
        // user a reset instead of silently restoring an outdated layout.
        if state.version != Some(DOCK_AREA_VERSION) {
            let answer = window.prompt(
                PromptLevel::Info,
                "The default layout has been updated.\n\
                Do you want to reset the layout to default?",
                None,
                &["Yes", "No"],
                cx,
            );

            let weak_dock_area = dock_area.downgrade();
            cx.spawn_in(window, async move |this, window| {
                if answer.await == Ok(0) {
                    _ = this.update_in(window, |_, window, cx| {
                        Self::reset_default_layout(weak_dock_area, window, cx);
                    });
                }
            })
            .detach();
        }

        dock_area.update(cx, |dock_area, cx| {
            dock_area.load(state, window, cx).context("load layout")?;
            for placement in [DockPlacement::Left, DockPlacement::Bottom] {
                dock_area.set_dock_collapsible(placement, true, window, cx);
            }

            Ok::<(), anyhow::Error>(())
        })
    }

    fn reset_default_layout(dock_area: WeakEntity<DockArea>, window: &mut Window, cx: &mut App) {
        // The center holds several tabs; each `.panel_view()` call adds one.
        // Panels go in through `panel_handle` — that carries the presentation
        // handle, so tabs draw the panel's `title` and not its `panel_name`.
        let center = DockLayout::tabs()
            .panel_view(panel_handle(panels::EditorPanel::new(window, cx)), cx)
            .panel_view(panel_handle(panels::WelcomePanel::new(cx)), cx)
            .active_index(0);

        // The left dock holds its own tab group.
        let left = DockLayout::tabs()
            .panel_view(panel_handle(panels::ListPanel::files(cx)), cx)
            .panel_view(panel_handle(panels::ListPanel::outline(cx)), cx);

        let bottom = DockLayout::tabs().panel_view(panel_handle(panels::OutputPanel::new(cx)), cx);

        _ = dock_area.update(cx, |view, cx| {
            view.set_center(center, window, cx);
            for (placement, layout, size) in [
                (DockPlacement::Left, left, px(240.)),
                (DockPlacement::Bottom, bottom, px(180.)),
            ] {
                view.set_dock(placement, layout, window, cx);
                view.set_dock_size(placement, size, window, cx);
            }

            Self::save_state(&view.dump(cx)).unwrap();
        });
    }

    pub fn new_local(cx: &mut App) -> Task<anyhow::Result<WindowHandle<Root>>> {
        let mut window_size = size(px(1600.0), px(1200.0));
        if let Some(display) = cx.primary_display() {
            let display_size = display.bounds().size;
            window_size.width = window_size.width.min(display_size.width * 0.85);
            window_size.height = window_size.height.min(display_size.height * 0.85);
        }

        let window_bounds = Bounds::centered(None, window_size, cx);

        cx.spawn(async move |cx| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                window_min_size: Some(gpui_kit::Size {
                    width: px(640.),
                    height: px(480.),
                }),
                #[cfg(target_os = "linux")]
                window_background: gpui_kit::WindowBackgroundAppearance::Transparent,
                #[cfg(target_os = "linux")]
                window_decorations: Some(gpui_kit::WindowDecorations::Client),
                kind: WindowKind::Normal,
                ..TitleBar::window_options()
            };

            let window = cx.open_window(options, |window, cx| {
                let workspace = cx.new(|cx| Workspace::new(window, cx));
                // The first level view of a window must be a Root.
                cx.new(|cx| Root::new(workspace, window, cx))
            })?;

            window
                .update(cx, |_, window, cx| {
                    window.activate_window();
                    window.set_window_title("Dock Tabs");
                    cx.on_release(|_, cx| {
                        // exit app
                        cx.quit();
                    })
                    .detach();
                })
                .expect("failed to update window");

            Ok(window)
        })
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .font_family(cx.theme().font_family.clone())
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(TitleBar::new().child("Dock Tabs"))
            .child(div().flex_1().min_h_0().child(self.dock_area.clone()))
            .child(
                StatusBar::new()
                    .left(
                        Button::new("toggle-left-dock")
                            .ghost()
                            .xsmall()
                            .icon(IconName::PanelLeft)
                            .tooltip("Toggle Left Dock")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.dock_area.update(cx, |area, cx| {
                                    area.toggle_dock(DockPlacement::Left, window, cx);
                                });
                            })),
                    )
                    .child(
                        Button::new("toggle-bottom-dock")
                            .ghost()
                            .xsmall()
                            .icon(IconName::PanelBottom)
                            .tooltip("Toggle Bottom Dock")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.dock_area.update(cx, |area, cx| {
                                    area.toggle_dock(DockPlacement::Bottom, window, cx);
                                });
                            })),
                    ),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}

fn main() {
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);

    app.run(move |cx| {
        init(cx);

        let task = Workspace::new_local(cx);
        cx.spawn(async move |cx| {
            if let Some(root) = task.await.ok() {
                root.update(cx, |_, _, _| {})
                    .expect("failed to open workspace");
            }
        })
        .detach();
    });
}
