use gpui::{AnyElement, IntoElement as _};

use crate::{
    button::{RibbonButton, RibbonSeparator},
    dropdown::RibbonDropDownButton,
    gallery::RibbonGallery,
    size::RibbonItemSize,
    split::RibbonSplitButton,
    toggle::RibbonToggleButton,
};

/// One entry in a [`RibbonGroup`](crate::RibbonGroup) item list.
///
/// The enum exists so the group can re-render its items at a capped size
/// while it scales down; items added through [`RibbonGroup::item`] convert
/// from the concrete control types automatically.
pub enum RibbonItem {
    Button(RibbonButton),
    Toggle(RibbonToggleButton),
    DropDown(RibbonDropDownButton),
    Split(RibbonSplitButton),
    Gallery(RibbonGallery),
    Separator(RibbonSeparator),
    Custom(AnyElement),
}

impl From<RibbonButton> for RibbonItem {
    fn from(button: RibbonButton) -> Self {
        Self::Button(button)
    }
}

impl From<RibbonToggleButton> for RibbonItem {
    fn from(toggle: RibbonToggleButton) -> Self {
        Self::Toggle(toggle)
    }
}

impl From<RibbonDropDownButton> for RibbonItem {
    fn from(dropdown: RibbonDropDownButton) -> Self {
        Self::DropDown(dropdown)
    }
}

impl From<RibbonSplitButton> for RibbonItem {
    fn from(split: RibbonSplitButton) -> Self {
        Self::Split(split)
    }
}

impl From<RibbonGallery> for RibbonItem {
    fn from(gallery: RibbonGallery) -> Self {
        Self::Gallery(gallery)
    }
}

impl From<RibbonSeparator> for RibbonItem {
    fn from(separator: RibbonSeparator) -> Self {
        Self::Separator(separator)
    }
}

impl From<AnyElement> for RibbonItem {
    fn from(element: AnyElement) -> Self {
        Self::Custom(element)
    }
}

impl RibbonItem {
    pub(crate) fn render_at(
        self,
        cap: Option<RibbonItemSize>,
        key_tips_visible: bool,
    ) -> AnyElement {
        match self {
            Self::Button(button) => button
                .with_size_cap(cap)
                .with_key_tip_visible(key_tips_visible)
                .into_any_element(),
            Self::Toggle(toggle) => toggle
                .with_size_cap(cap)
                .with_key_tip_visible(key_tips_visible)
                .into_any_element(),
            Self::DropDown(dropdown) => dropdown
                .with_size_cap(cap)
                .with_key_tip_visible(key_tips_visible)
                .into_any_element(),
            Self::Split(split) => split
                .with_size_cap(cap)
                .with_key_tip_visible(key_tips_visible)
                .into_any_element(),
            Self::Gallery(gallery) => gallery
                .with_size_cap(cap)
                .with_key_tip_visible(key_tips_visible)
                .into_any_element(),
            Self::Separator(separator) => separator.into_any_element(),
            Self::Custom(element) => element,
        }
    }

    /// Renders the item inside a collapsed group popup: sizes stay uncapped
    /// and definitive commands dismiss the popup after they run. Custom
    /// elements keep their own handlers and do not close the popup.
    pub(crate) fn render_in_popup(
        self,
        cap: Option<RibbonItemSize>,
        popup: gpui::Entity<gpui_base::PopoverState>,
    ) -> AnyElement {
        match self {
            Self::Button(button) => button
                .with_size_cap(cap)
                .dismiss_after_click(popup)
                .into_any_element(),
            Self::Toggle(toggle) => toggle
                .with_size_cap(cap)
                .dismiss_after_click(popup)
                .into_any_element(),
            Self::DropDown(dropdown) => dropdown.with_size_cap(cap).into_any_element(),
            Self::Split(split) => split.with_size_cap(cap).into_any_element(),
            Self::Gallery(gallery) => gallery.with_size_cap(cap).into_any_element(),
            Self::Separator(separator) => separator.into_any_element(),
            Self::Custom(element) => element,
        }
    }

    /// The declared key tip and its command handler, when the item can be
    /// activated from the key tip overlay. Only buttons register an entry.
    pub(crate) fn key_tip_entry(&self) -> Option<(&str, crate::face::ClickHandler)> {
        match self {
            Self::Button(button) => Some((button.declared_key_tip()?, button.on_click_handler()?)),
            _ => None,
        }
    }
}
