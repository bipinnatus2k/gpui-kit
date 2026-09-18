use gpui::{ElementId, Hsla, SharedString, StyleRefinement, Styled};

use crate::group::RibbonGroup;

/// One page of the [`Ribbon`](crate::Ribbon): a header shown on the tab strip
/// and a set of [`RibbonGroup`]s shown while the tab is active.
///
/// Tabs are data, not standalone elements: a `RibbonTab` is passed to
/// [`Ribbon::tab`](crate::Ribbon::tab) (or [`Ribbon::tabs`]), which renders
/// its strip entry and, when selected, its groups.
pub struct RibbonTab {
    pub(crate) id: ElementId,
    pub(crate) header: String,
    pub(crate) groups: Vec<RibbonGroup>,
    pub(crate) contextual: Option<Hsla>,
    pub(crate) style: StyleRefinement,
}

impl RibbonTab {
    pub fn new(id: impl Into<ElementId>, header: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            header: header.into().to_string(),
            groups: Vec::new(),
            contextual: None,
            style: StyleRefinement::default(),
        }
    }

    /// The tab label shown on the strip.
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.header = header.into().to_string();
        self
    }

    /// Marks the tab as belonging to a contextual tab group by coloring its
    /// strip accent, as in classic ribbon layouts. Visibility of the whole
    /// set stays with the application: simply pass only the tabs that are
    /// currently available.
    pub fn contextual(mut self, color: impl Into<Hsla>) -> Self {
        self.contextual = Some(color.into());
        self
    }

    /// Appends a group to the tab.
    pub fn group(mut self, group: RibbonGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// Appends several groups to the tab.
    pub fn groups(mut self, groups: impl IntoIterator<Item = RibbonGroup>) -> Self {
        self.groups.extend(groups);
        self
    }
}

impl Styled for RibbonTab {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
