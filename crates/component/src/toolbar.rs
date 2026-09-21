use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, prelude::FluentBuilder as _,
};
use smallvec::SmallVec;

use gpui_base::{Toolbar as BaseToolbar, ToolbarGroup as BaseToolbarGroup};

use crate::{Sizable, Size, StyledExt as _, h_flex};

enum ToolbarItem {
    Sized(Box<dyn FnOnce(Size) -> AnyElement>),
    Content(AnyElement),
}

impl ToolbarItem {
    fn sized(item: impl Sizable + IntoElement + 'static) -> Self {
        Self::Sized(Box::new(move |size| {
            item.with_size(size).into_any_element()
        }))
    }

    fn content(content: impl IntoElement) -> Self {
        Self::Content(content.into_any_element())
    }

    fn into_element(self, size: Size) -> AnyElement {
        match self {
            Self::Sized(item) => item(size),
            Self::Content(content) => content,
        }
    }
}

/// A semantic subgroup of toolbar controls that shares one accessible label
/// and propagates its size to every control added with `child` or `children`.
#[derive(IntoElement)]
pub struct ToolbarGroup {
    id: ElementId,
    style: StyleRefinement,
    size: Size,
    label: Option<SharedString>,
    children: SmallVec<[ToolbarItem; 4]>,
}

impl ToolbarGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            size: Size::default(),
            label: None,
            children: SmallVec::new(),
        }
    }

    /// Sets the accessible name announced for the group.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Append a control that inherits the group's final size.
    pub fn child(mut self, child: impl Sizable + IntoElement + 'static) -> Self {
        self.children.push(ToolbarItem::sized(child));
        self
    }

    /// Append controls that inherit the group's final size.
    pub fn children<T>(mut self, children: impl IntoIterator<Item = T>) -> Self
    where
        T: Sizable + IntoElement + 'static,
    {
        self.children
            .extend(children.into_iter().map(ToolbarItem::sized));
        self
    }

    /// Append non-sized content to the group.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.children.push(ToolbarItem::content(content));
        self
    }
}

impl ParentElement for ToolbarGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(ToolbarItem::Content));
    }
}

impl Styled for ToolbarGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Sizable for ToolbarGroup {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for ToolbarGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let size = self.size;
        BaseToolbarGroup::new(self.id)
            .when_some(self.label, |this, label| this.label(label))
            .children(
                self.children
                    .into_iter()
                    .map(|item| item.into_element(size)),
            )
            .refine_style(&self.style)
    }
}

/// A transparent horizontal container for commands in a window, pane, or
/// section.
///
/// The toolbar owns layout and keyboard behavior while the surrounding header,
/// tab strip, or custom surface owns its background and border.
///
/// The bar exposes `Toolbar` semantics to assistive technology and owns
/// roving keyboard focus: when focus is on one of its controls, the arrow
/// keys move focus along the bar, wrapping around at the ends. Hosted inputs
/// keep their own arrow-key caret behavior; place them at the trailing end of
/// the bar.
///
/// `left`, `right`, and `child` accept [`Sizable`] controls and automatically
/// apply the toolbar's final size, regardless of builder order. Use
/// `left_content`, `right_content`, and `content` for separators, labels, and
/// custom layout. An icon-only button must carry a tooltip and an accessible
/// name.
///
/// `left` and `right` pin items to each end; `child`/`children` add to the
/// middle, whose alignment follows the pinned ends: centered with both `left`
/// and `right`, end-aligned with only `left`, and start-aligned otherwise
/// (only `right`, or neither — like a plain bar).
///
/// The id keeps the toolbar's keyboard-focus state stable across frames;
/// give each toolbar in a window a distinct id.
///
/// ```
/// # mod gpui_kit { pub extern crate gpui_component as component; }
/// use gpui_kit::component::toolbar::Toolbar;
///
/// let _ = Toolbar::new("document-toolbar")
///     .left_content("Document")
///     .right_content("Ready");
/// ```
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    style: StyleRefinement,
    size: Size,
    left: SmallVec<[ToolbarItem; 1]>,
    right: SmallVec<[ToolbarItem; 1]>,
    children: SmallVec<[ToolbarItem; 1]>,
}

impl Toolbar {
    /// Create a new, empty [`Toolbar`] at [`Size::Medium`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            size: Size::default(),
            left: SmallVec::new(),
            right: SmallVec::new(),
            children: SmallVec::new(),
        }
    }

    /// Append an element to the left (leading) region. Call multiple times to
    /// add more.
    pub fn left(mut self, child: impl Sizable + IntoElement + 'static) -> Self {
        self.left.push(ToolbarItem::sized(child));
        self
    }

    /// Append non-sized content to the left (leading) region.
    pub fn left_content(mut self, content: impl IntoElement) -> Self {
        self.left.push(ToolbarItem::content(content));
        self
    }

    /// Append an element to the right (trailing) region. Call multiple times
    /// to add more.
    pub fn right(mut self, child: impl Sizable + IntoElement + 'static) -> Self {
        self.right.push(ToolbarItem::sized(child));
        self
    }

    /// Append non-sized content to the right (trailing) region.
    pub fn right_content(mut self, content: impl IntoElement) -> Self {
        self.right.push(ToolbarItem::content(content));
        self
    }

    /// Append a sized control to the middle region. The toolbar applies its
    /// final size when it renders, so builder call order does not matter.
    pub fn child(mut self, child: impl Sizable + IntoElement + 'static) -> Self {
        self.children.push(ToolbarItem::sized(child));
        self
    }

    /// Append sized controls to the middle region.
    pub fn children<T>(mut self, children: impl IntoIterator<Item = T>) -> Self
    where
        T: Sizable + IntoElement + 'static,
    {
        self.children
            .extend(children.into_iter().map(ToolbarItem::sized));
        self
    }

    /// Append non-sized content to the middle region.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.children.push(ToolbarItem::content(content));
        self
    }

    /// Append non-sized content to the middle region.
    pub fn contents(mut self, contents: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children
            .extend(contents.into_iter().map(ToolbarItem::Content));
        self
    }
}

/// Generic [`ParentElement`] extension treats elements as non-sized content.
/// Prefer the inherent `child` / `children` methods for controls that should
/// inherit the toolbar size.
impl ParentElement for Toolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(ToolbarItem::Content));
    }
}

impl Styled for Toolbar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Sizable for Toolbar {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for Toolbar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // The middle aligns by which ends are pinned: centered with both left
        // and right, end-aligned with only left, otherwise start-aligned (only
        // right, or neither) — matching `StatusBar`'s region contract. The
        // regions live inside the base toolbar so its roving arrow-key
        // navigation reaches every item across them.
        let size = self.size;
        let has_left = !self.left.is_empty();
        let has_right = !self.right.is_empty();
        let region = || {
            h_flex()
                .overflow_hidden()
                .items_center()
                .map(|this| match size {
                    Size::XSmall | Size::Small => this.gap_1(),
                    _ => this.gap_2(),
                })
        };

        let left = self.left.into_iter().map(|item| item.into_element(size));
        let right = self.right.into_iter().map(|item| item.into_element(size));
        let children = self
            .children
            .into_iter()
            .map(|item| item.into_element(size));

        BaseToolbar::new(self.id)
            .flex()
            .items_center()
            .flex_shrink_0()
            .map(|this| match size {
                Size::XSmall => this.h_7().px_2().gap_1().text_xs(),
                Size::Small => this.h_8().px_2().gap_1().text_sm(),
                Size::Large => this.h_12().px_3().gap_2().text_base(),
                _ => this.h_10().px_2().gap_2().text_sm(),
            })
            .refine_style(&self.style)
            .when(has_left, |this| this.child(region().children(left)))
            .child(
                region()
                    .flex_1()
                    .when(has_left && has_right, |this| this.justify_center())
                    .when(has_left && !has_right, |this| this.justify_end())
                    .children(children),
            )
            .when(has_right, |this| this.child(region().children(right)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, div};
    use std::sync::{Arc, Mutex};

    #[derive(IntoElement)]
    struct SizeProbe {
        size: Size,
        observed: Arc<Mutex<Option<Size>>>,
    }

    impl SizeProbe {
        fn new(observed: Arc<Mutex<Option<Size>>>) -> Self {
            Self {
                size: Size::default(),
                observed,
            }
        }
    }

    impl Sizable for SizeProbe {
        fn with_size(mut self, size: impl Into<Size>) -> Self {
            self.size = size.into();
            self
        }
    }

    impl RenderOnce for SizeProbe {
        fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
            *self.observed.lock().unwrap() = Some(self.size);
            div()
        }
    }

    struct ToolbarHarness {
        left: Arc<Mutex<Option<Size>>>,
        center: Arc<Mutex<Option<Size>>>,
        right: Arc<Mutex<Option<Size>>>,
    }

    struct ToolbarGroupHarness {
        item: Arc<Mutex<Option<Size>>>,
    }

    impl Render for ToolbarGroupHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            ToolbarGroup::new("group")
                .child(SizeProbe::new(self.item.clone()))
                .small()
        }
    }

    impl Render for ToolbarHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            Toolbar::new("toolbar")
                .left(SizeProbe::new(self.left.clone()))
                .child(SizeProbe::new(self.center.clone()))
                .right(SizeProbe::new(self.right.clone()))
                .small()
        }
    }

    #[test]
    fn test_toolbar_builder() {
        let toolbar = Toolbar::new("toolbar")
            .left_content("New")
            .left_content("Open")
            .content("Center")
            .right_content("Settings")
            .small();

        assert_eq!(toolbar.left.len(), 2);
        assert_eq!(toolbar.children.len(), 1);
        assert_eq!(toolbar.right.len(), 1);
        assert_eq!(toolbar.size, Size::Small);
    }

    #[test]
    fn test_toolbar_default() {
        let toolbar = Toolbar::new("toolbar");

        assert_eq!(toolbar.size, Size::Medium);
        assert!(toolbar.left.is_empty());
        assert!(toolbar.right.is_empty());
        assert!(toolbar.children.is_empty());
    }

    #[gpui::test]
    fn toolbar_size_propagates_to_items_independent_of_builder_order(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let left = Arc::new(Mutex::new(None));
        let center = Arc::new(Mutex::new(None));
        let right = Arc::new(Mutex::new(None));

        let expected = [left.clone(), center.clone(), right.clone()];
        let (_, cx) = cx.add_window_view(move |_, _| ToolbarHarness {
            left,
            center,
            right,
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));

        for observed in expected {
            assert_eq!(*observed.lock().unwrap(), Some(Size::Small));
        }
    }

    #[gpui::test]
    fn toolbar_group_propagates_its_size_to_items(cx: &mut TestAppContext) {
        let item = Arc::new(Mutex::new(None));
        let observed = item.clone();
        let (_, cx) = cx.add_window_view(move |_, _| ToolbarGroupHarness { item });
        cx.update(|window, cx| window.draw(cx).clear(cx));

        assert_eq!(*observed.lock().unwrap(), Some(Size::Small));
    }
}
