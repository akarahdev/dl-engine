use bevy::color::LinearRgba;
use bevy::prelude::{Color, Resource, Vec3};
use crate::game::camera::CameraConfig;
use crate::game::line::LineConfig;
use crate::game::triggers::TriggerFunction;

#[derive(Resource, Debug)]
pub struct SceneData {
    pub start_pos: Vec3,
    pub camera_config: CameraConfig,
    pub line_config: LineConfig,

    pub cubes: Vec<Cuboid>,
    pub trigger_areas: Vec<TriggerArea>,
}

impl SceneData {
    pub fn new_simple() -> Self {
        let mut cubes = vec![
            Cuboid::new()
                .with_position(Vec3::new(0.0, -1.0, 0.0))
                .with_scale(Vec3::new(20.0, 1.0, 20.0))
        ];
        let mut trigger_areas = vec![];

        let size = 2.0;
        let mut x = 0.0;
        let mut z = 0.0;
        for _ in 0..1000 {
            if rand::random::<f32>() > 0.5 {
                x += size;
            } else {
                z += size;
            }
            cubes.push(
                Cuboid::new()
                    .with_position(Vec3::new(x, -1.0, z))
                    .with_scale(Vec3::new(size, 1.0, size))
                    .with_color(Color::LinearRgba(LinearRgba::BLUE))
            );

            if rand::random::<f32>() < 0.1 {
                let off_x = rand::random::<f32>() * 24.0 - 12.0;
                let off_z = rand::random::<f32>() * 24.0 - 12.0;
                trigger_areas.push(
                    TriggerArea::new()
                        .with_position(Vec3::new(x, 0.0, z))
                        .with_scale(Vec3::new(size, 10.0, size))
                        .with_function(TriggerFunction::SetCameraOffset(Vec3::new(off_x, 12.0, off_z)))
                )
            }
        }


        SceneData {
            start_pos: Vec3::new(0.0, 0.0, 0.0),
            camera_config: CameraConfig::default(),
            line_config: LineConfig::default(),
            cubes,
            trigger_areas
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

#[derive(Debug, Clone)]
pub struct TriggerArea {
    pub position: Vec3,
    pub scale: Vec3,
    pub function: TriggerFunction,
}

impl TriggerArea {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            function: TriggerFunction::None,
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

    pub fn with_function(mut self, function: TriggerFunction) -> Self {
        self.function = function;
        return self;
    }
}

