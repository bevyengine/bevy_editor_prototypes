//! Pane drop area detection
//!
//! Given a rect and a cursor position, this module provides a way to determine the pane drop area.
//! There are 5 possible drop areas: Top, Bottom, Left, Right, and Center.
//! The center area takes approximately a third of the rectangle, although the logic depends on the aspect ratio.
//! The boundaries between the non-center areas are determined by 45º diagonal lines.
//!
//! ```txt
//! ____________
//! |⟍   T    ⟋|
//! |  ⟍____⟋  |
//! |   |   |   |
//! |L  | C | R |
//! |   |___|   |
//! |  ⟋ B  ⟍  |
//! |⟋________⟍|
//! ```
#![allow(unused)]

use bevy::{math::AspectRatio, prelude::*};

#[derive(Debug, PartialEq)]
pub enum PaneDropArea {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

// The center rect is created using the 1/3 of shortest side
const CENTER_SIZE: f32 = 1. / 3.;

fn get_center_rect(rect: &Rect) -> Rect {
    if let Ok(aspect_ratio) = AspectRatio::try_new(rect.width(), rect.height()) {
        let (width, height) = if aspect_ratio.is_landscape() {
            let height = rect.height() * CENTER_SIZE;
            (rect.width() - 2. * height, height)
        } else {
            let width = rect.width() * CENTER_SIZE;
            (width, rect.height() - 2. * width)
        };

        Rect::from_center_size(rect.center(), Vec2::new(width, height))
    } else {
        *rect
    }
}

pub fn get_pane_drop_area(rect: &Rect, cursor: &Vec2) -> PaneDropArea {
    let center_rect = get_center_rect(rect);
    let normalized_cursor = Vec2::new(cursor.x - rect.min.x, cursor.y - rect.min.y) / rect.size();

    if center_rect.contains(*cursor) {
        return PaneDropArea::Center;
    }

    // Check by quadrants
    // For each quadrant check if the cursor is above or below the diagonal line.
    let (is_top, is_left) = (normalized_cursor.y < 0.5, normalized_cursor.x < 0.5);

    match (is_top, is_left) {
        // TOP-LEFT
        (true, true) => {
            if cursor.y < cursor.x - rect.min.x + rect.min.y {
                PaneDropArea::Top
            } else {
                PaneDropArea::Left
            }
        }
        // BOTTOM-LEFT
        (false, true) => {
            if cursor.y < rect.max.y - cursor.x + rect.min.x {
                PaneDropArea::Left
            } else {
                PaneDropArea::Bottom
            }
        }
        // TOP-RIGHT
        (true, false) => {
            if cursor.y < rect.min.y - cursor.x + rect.max.x {
                PaneDropArea::Top
            } else {
                PaneDropArea::Right
            }
        }
        // BOTTOM-RIGHT
        (false, false) => {
            if cursor.y < cursor.x - rect.max.x + rect.max.y {
                PaneDropArea::Right
            } else {
                PaneDropArea::Bottom
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.0001;

    fn assert_approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < EPSILON, "{a} ≈ {b}");
    }

    #[test]
    fn get_center_rect_tests() {
        let square_rect = Rect::from_corners(Vec2::ZERO, Vec2::new(9., 9.));
        let landscape_rect = Rect::from_corners(Vec2::ZERO, Vec2::new(9., 3.));
        let portrait_rect = Rect::from_corners(Vec2::ZERO, Vec2::new(3., 9.));

        let square_center = get_center_rect(&square_rect);
        let landscape_center = get_center_rect(&landscape_rect);
        let portrait_center = get_center_rect(&portrait_rect);

        assert_approx_eq(square_center.width(), 3.);
        assert_approx_eq(square_center.height(), 3.);
        assert_approx_eq(landscape_center.width(), 7.);
        assert_approx_eq(landscape_center.height(), 1.);
        assert_approx_eq(portrait_center.width(), 1.);
        assert_approx_eq(portrait_center.height(), 7.);
    }

    #[test]
    fn get_pane_drop_area_tests() {
        let pane_rect = Rect::from_corners(Vec2::ZERO, Vec2::new(10., 10.));
        let center_rect = get_center_rect(&pane_rect);
        let center_inner = center_rect.inflate(-EPSILON);
        let center_outer = center_rect.inflate(EPSILON);

        let positions = [
            // Center
            (Vec2::new(5., 5.), PaneDropArea::Center),
            (Vec2::new(center_inner.min.x, 5.), PaneDropArea::Center),
            (Vec2::new(center_inner.max.x, 5.), PaneDropArea::Center),
            (Vec2::new(5., center_inner.min.y), PaneDropArea::Center),
            (Vec2::new(5., center_inner.max.y), PaneDropArea::Center),
            // Top
            (Vec2::new(5., 0.), PaneDropArea::Top),
            (Vec2::new(5., center_outer.min.y), PaneDropArea::Top),
            // Bottom
            (Vec2::new(5., center_outer.max.y), PaneDropArea::Bottom),
            (Vec2::new(5., 10.), PaneDropArea::Bottom),
            // Left
            (Vec2::new(0., 5.), PaneDropArea::Left),
            (Vec2::new(center_outer.min.x, 5.), PaneDropArea::Left),
            // Right
            (Vec2::new(center_outer.max.x, 5.), PaneDropArea::Right),
            (Vec2::new(10., 5.), PaneDropArea::Right),
            // Corner Boundaries
            // ⟍ Top-Left (favors Left)
            (Vec2::new(1., 1.), PaneDropArea::Left),
            (Vec2::new(1., 1. + EPSILON), PaneDropArea::Left),
            (Vec2::new(1., 1. - EPSILON), PaneDropArea::Top),
            // ⟋ Top-Right (favors Right)
            (Vec2::new(9., 1.), PaneDropArea::Right),
            (Vec2::new(9., 1. + EPSILON), PaneDropArea::Right),
            (Vec2::new(9., 1. - EPSILON), PaneDropArea::Top),
            // ⟋ Bottom-Left (favors Bottom)
            (Vec2::new(1., 9.), PaneDropArea::Bottom),
            (Vec2::new(1., 9. + EPSILON), PaneDropArea::Bottom),
            (Vec2::new(1., 9. - EPSILON), PaneDropArea::Left),
            // ⟍ Bottom-Right (favors Bottom)
            (Vec2::new(9., 9.), PaneDropArea::Bottom),
            (Vec2::new(9., 9. + EPSILON), PaneDropArea::Bottom),
            (Vec2::new(9., 9. - EPSILON), PaneDropArea::Right),
        ];

        for (position, expected) in positions.iter() {
            assert_eq!(
                get_pane_drop_area(&pane_rect, position),
                *expected,
                "{position:?} -> {expected:?}"
            );
        }
    }
}
