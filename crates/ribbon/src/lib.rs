//! Ribbon UI for GPUI applications, built on `gpui-base`.
//!
//! The crate re-implements the classic Fluent Ribbon model — a tab strip over
//! labeled groups of large/medium/small commands — following the same seam
//! rules as `gpui-base` itself: every control is a stateless `RenderOnce`
//! element with private fields, builder construction, and application-owned
//! state, and all colors, radii, spacing, typography, and shadows resolve from
//! the active [`gpui_base::SemanticThemeTokens`].
//!
//! # Example
//!
//! ```ignore
//! use gpui_ribbon::{Ribbon, RibbonButton, RibbonGroup, RibbonTab};
//!
//! Ribbon::new("ribbon")
//!     .active_tab(0)
//!     .on_tab_change(move |index, _, cx| state.update(cx, |state, _| state.tab = index))
//!     .tab(
//!         RibbonTab::new("home", "Home").group(
//!             RibbonGroup::new("clipboard", "Clipboard")
//!                 .item(RibbonButton::new("paste", "Paste").on_click(|_, _, _| {}))
//!                 .item(RibbonButton::new("cut", "Cut").size(RibbonItemSize::Medium)),
//!         ),
//!     );
//! ```
//!
//! Icons are not bundled; pass any element (typically an `Icon` from a styled
//! layer) to [`RibbonButton::icon`] and friends. Theming follows the active
//! Base theme: when the application drives its palette through
//! `gpui-component`, the ribbon is re-themed with it automatically.

mod button;
mod dropdown;
mod face;
mod gallery;
mod group;
mod item;
mod key_tip;
mod ribbon;
mod size;
mod split;
mod tab;
mod theme;
mod toggle;

pub use button::{RibbonButton, RibbonSeparator};
pub use dropdown::{RibbonDropDownButton, RibbonMenuItem, RibbonMenuSeparator};
pub use gallery::{RibbonGallery, RibbonGalleryEntry};
pub use group::RibbonGroup;
pub use item::RibbonItem;
pub use key_tip::ToggleKeyTips;
pub use ribbon::{Ribbon, RibbonApplicationMenu};
pub use size::{RibbonGroupState, RibbonItemSize};
pub use split::RibbonSplitButton;
pub use tab::RibbonTab;
pub use toggle::RibbonToggleButton;

use gpui::App;

/// Initializes the ribbon: infrastructure from the Base layer plus the
/// key bindings for the key tip overlay (`alt`).
///
/// Applications that already call `gpui_component::init` (or `gpui_base::init`)
/// still need this for the key tip binding.
pub fn init(cx: &mut App) {
    gpui_base::init(cx);
    key_tip::init(cx);
}
