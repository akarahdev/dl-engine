use bevy::math::Vec3;
use bevy::prelude::{Color, LinearRgba};

#[derive(Debug, Clone)]
pub struct LineConfig {
    pub start_pos: Vec3,
    pub color: Color
}

impl Default for LineConfig {
    fn default() -> Self {
        LineConfig {
            start_pos: Vec3::ZERO,
            color: Color::LinearRgba(LinearRgba::RED)
        }
    }
}