use std::sync::Arc;
use bevy::prelude::{Color, Resource, Transform, Vec3};
use crate::game::camera::CameraConfig;
use crate::game::line::LineConfig;

#[derive(Resource, Debug)]
pub struct SceneData {
    pub start_pos: Vec3,
    pub camera_config: CameraConfig,
    pub line_config: LineConfig,
    pub cubes: Vec<Cuboid>,
}

impl SceneData {
    pub fn new_simple() -> Self {
        SceneData {
            start_pos: Vec3::new(0.0, 0.0, 0.0),
            camera_config: CameraConfig::default(),
            line_config: LineConfig::default(),
            cubes: vec![
                Cuboid::new()
                    .with_position(Vec3::new(0.0, -1.0, 0.0))
                    .with_scale(Vec3::new(20.0, 1.0, 20.0))
            ]
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cuboid {
    pub color: Color,
    pub position: Vec3,
    pub rotation_euler: Vec3,
    pub scale: Vec3,
}

impl Cuboid {
    pub fn new() -> Self {
        Self {
            color: Color::WHITE,
            position: Vec3::ZERO,
            rotation_euler: Vec3::ONE,
            scale: Vec3::ONE,
        }
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        return self;
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        return self;
    }

    pub fn with_rotation(mut self, rotation_euler: Vec3) -> Self {
        self.rotation_euler = rotation_euler;
        return self;
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        return self;
    }
}