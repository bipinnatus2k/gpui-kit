//! Size vocabulary shared by ribbon items and groups, following Fluent
//! Ribbon's `RibbonControlSize` and `RibbonGroupBoxState` chains.

use gpui::{Pixels, px};

/// One step of the automatic width-driven collapse, mirroring Fluent
/// Ribbon's group `ReduceOrder` walk: when the groups overflow the available
/// width the leftmost reducible group steps one state toward `Collapsed`;
/// when there is comfortable spare room the rightmost collapsible group
/// steps back up. The asymmetric enlarge threshold gives the chain
/// hysteresis so it settles instead of oscillating.
///
/// Returns whether any state changed.
pub(crate) fn solve_auto_states(
    states: &mut [RibbonGroupState],
    natural: Pixels,
    available: Pixels,
) -> bool {
    if states.is_empty() {
        return false;
    }

    if natural > available {
        for state in states.iter_mut() {
            if let Some(next) = state.reduce() {
                *state = next;
                return true;
            }
        }
        false
    } else if available - natural > px(48.) {
        for state in states.iter_mut().rev() {
            if let Some(next) = state.enlarge() {
                *state = next;
                return true;
            }
        }
        false
    } else {
        false
    }
}

/// The presentation size of a single ribbon item.
///
/// A large item stacks its 32px icon above its label; medium and small items
/// sit inline, with small hiding the label entirely. Items are built with a
/// preferred size and a containing [`RibbonGroupState`](crate::RibbonGroupState)
/// may cap them while the group scales down.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum RibbonItemSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl RibbonItemSize {
    /// The next smaller size, or `None` when already the smallest.
    pub fn reduce(self) -> Option<Self> {
        match self {
            Self::Large => Some(Self::Medium),
            Self::Medium => Some(Self::Small),
            Self::Small => None,
        }
    }

    /// The next larger size, or `None` when already the largest.
    pub fn enlarge(self) -> Option<Self> {
        match self {
            Self::Large => None,
            Self::Medium => Some(Self::Large),
            Self::Small => Some(Self::Medium),
        }
    }

    /// The smaller of the two sizes.
    pub fn min(self, other: Self) -> Self {
        if other < self { other } else { self }
    }
}

/// The layout state of a [`RibbonGroup`](crate::RibbonGroup).
///
/// The chain mirrors Fluent Ribbon's group state definition: reducing walks
/// `Large → Middle → Small → Collapsed`. While `Middle` or `Small` caps the
/// sizes of the group's scalable items, `Collapsed` replaces the whole group
/// with a single button that offers the items in a popup.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RibbonGroupState {
    #[default]
    Large,
    Middle,
    Small,
    Collapsed,
}

impl RibbonGroupState {
    /// The next state toward `Collapsed`, or `None` when already collapsed.
    pub fn reduce(self) -> Option<Self> {
        match self {
            Self::Large => Some(Self::Middle),
            Self::Middle => Some(Self::Small),
            Self::Small => Some(Self::Collapsed),
            Self::Collapsed => None,
        }
    }

    /// The previous state toward `Large`, or `None` when already at `Large`.
    pub fn enlarge(self) -> Option<Self> {
        match self {
            Self::Large => None,
            Self::Middle => Some(Self::Large),
            Self::Small => Some(Self::Middle),
            Self::Collapsed => Some(Self::Small),
        }
    }

    /// The largest item size this state still renders inline, or `None` when
    /// items keep their own sizes (`Large` state and the collapsed popup).
    pub fn item_size_cap(self) -> Option<RibbonItemSize> {
        match self {
            Self::Large | Self::Collapsed => None,
            Self::Middle => Some(RibbonItemSize::Medium),
            Self::Small => Some(RibbonItemSize::Small),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_size_walks_large_to_small() {
        assert_eq!(RibbonItemSize::Large.reduce(), Some(RibbonItemSize::Medium));
        assert_eq!(RibbonItemSize::Medium.reduce(), Some(RibbonItemSize::Small));
        assert_eq!(RibbonItemSize::Small.reduce(), None);
        assert_eq!(
            RibbonItemSize::Small.enlarge(),
            Some(RibbonItemSize::Medium)
        );
        assert_eq!(RibbonItemSize::Large.enlarge(), None);
    }

    #[test]
    fn item_size_min_prefers_the_smaller_side() {
        assert_eq!(
            RibbonItemSize::Large.min(RibbonItemSize::Small),
            RibbonItemSize::Small
        );
        assert_eq!(
            RibbonItemSize::Medium.min(RibbonItemSize::Large),
            RibbonItemSize::Medium
        );
    }

    #[test]
    fn group_state_walks_large_to_collapsed() {
        let mut state = RibbonGroupState::Large;
        let walked = std::iter::from_fn(|| {
            let next = state.reduce()?;
            state = next;
            Some(next)
        })
        .collect::<Vec<_>>();

        assert_eq!(
            walked,
            vec![
                RibbonGroupState::Middle,
                RibbonGroupState::Small,
                RibbonGroupState::Collapsed,
            ]
        );
        assert_eq!(RibbonGroupState::Collapsed.reduce(), None);
        assert_eq!(RibbonGroupState::Large.enlarge(), None);
    }

    #[test]
    fn group_state_caps_item_sizes_while_scaling() {
        assert_eq!(RibbonGroupState::Large.item_size_cap(), None);
        assert_eq!(
            RibbonGroupState::Middle.item_size_cap(),
            Some(RibbonItemSize::Medium)
        );
        assert_eq!(
            RibbonGroupState::Small.item_size_cap(),
            Some(RibbonItemSize::Small)
        );
        assert_eq!(RibbonGroupState::Collapsed.item_size_cap(), None);
    }

    #[test]
    fn auto_solver_reduces_from_the_left_until_it_fits() {
        let mut states = [RibbonGroupState::Large; 3];

        assert!(solve_auto_states(&mut states, px(500.), px(400.)));
        assert_eq!(
            states,
            [
                RibbonGroupState::Middle,
                RibbonGroupState::Large,
                RibbonGroupState::Large
            ]
        );
    }

    #[test]
    fn auto_solver_enlarges_from_the_right_only_with_spare_room() {
        let mut states = [RibbonGroupState::Middle, RibbonGroupState::Collapsed];

        // No comfortable spare room: nothing changes.
        assert!(!solve_auto_states(&mut states, px(390.), px(400.)));
        assert_eq!(
            states,
            [RibbonGroupState::Middle, RibbonGroupState::Collapsed]
        );

        // Comfortable spare room: the rightmost group steps back up.
        assert!(solve_auto_states(&mut states, px(200.), px(400.)));
        assert_eq!(states, [RibbonGroupState::Middle, RibbonGroupState::Small]);
    }

    #[test]
    fn auto_solver_is_done_when_the_chain_ends() {
        let mut states = [RibbonGroupState::Collapsed, RibbonGroupState::Collapsed];
        assert!(!solve_auto_states(&mut states, px(500.), px(100.)));

        let mut expanded = [RibbonGroupState::Large];
        assert!(!solve_auto_states(&mut expanded, px(10.), px(1000.)));
    }
}
