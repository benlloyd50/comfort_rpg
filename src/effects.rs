use bevy::prelude::Vec2;

/// Moves start vec towards finish vec by the scalar value (same in both directions)
pub fn lerp(start: Vec2, finish: Vec2, scalar: f32) -> Vec2 {
    start + (finish - start) * scalar
}
