use std::cell::Cell;

use gpui::{Action, App, Global, KeyBinding};
use serde::Deserialize;

/// The key context under which ribbon elements dispatch key actions.
pub(crate) const KEY_TIP_CONTEXT: &str = "Ribbon";

/// Toggles the ribbon key tip overlay, bound to `alt` by [`init`].
///
/// The binding is global (any focus) so the overlay works like the Office
/// ribbon; the visible flag lives on the application, so all ribbons in all
/// windows show badges together until a ribbon consumes the activation.
#[derive(Clone, Action, PartialEq, Eq, Deserialize)]
#[action(namespace = ribbon, no_json)]
pub struct ToggleKeyTips;

/// Whether the key tip overlay is currently visible, shared by every ribbon.
#[derive(Default)]
pub(crate) struct KeyTipsVisible(pub Cell<bool>);

impl Global for KeyTipsVisible {}

pub(crate) fn visible(cx: &App) -> bool {
    cx.try_global::<KeyTipsVisible>()
        .is_some_and(|state| state.0.get())
}

pub(crate) fn set_visible(cx: &mut App, visible: bool) {
    cx.set_global(KeyTipsVisible(Cell::new(visible)));
    cx.refresh_windows();
}

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        // `alt` follows the Office ribbon; `f10` is the classic fallback that
        // survives platforms where a bare alt is consumed by the system menu.
        KeyBinding::new("alt", ToggleKeyTips, None),
        KeyBinding::new("f10", ToggleKeyTips, None),
    ]);
    cx.on_action(|_: &ToggleKeyTips, cx| {
        let visible = !visible(cx);
        set_visible(cx, visible);
    });
}
