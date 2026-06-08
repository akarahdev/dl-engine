use std::array;
use bevy::asset::{Assets, Handle};
use bevy::color::LinearRgba;
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::picking::Pickable;
use bevy::prelude::{Color, Commands, Component, Resource, Transform, Vec3};
use crate::game::camera::CameraConfig;
use crate::game::line::LineConfig;
use crate::game::plugin::{GameplayObject, LiveGameDataResource, TransformCollidable};
use crate::game::triggers::TriggerFunction;
use crate::state::GameState;

#[derive(Resource, Debug)]
pub struct SceneData {
    pub start_pos: Vec3,
    pub camera_config: CameraConfig,
    pub line_config: LineConfig,

    pub cubes: Vec<Cuboid>,
    pub trigger_areas: Vec<TriggerArea>,
    pub color_channels: [Color; u8::MAX as usize],
}

impl SceneData {
    pub fn new_empty() -> Self {
        SceneData {
            start_pos: Vec3::new(0.0, 0.0, 0.0),
            camera_config: CameraConfig::default(),
            line_config: LineConfig::default(),
            cubes: vec![],
            trigger_areas: vec![],
            color_channels: array::from_fn(|_| Color::LinearRgba(LinearRgba::WHITE)),
        }
    }

    pub fn new_simple() -> Self {
        let mut cubes = vec![
            Cuboid::new()
                .with_position(Vec3::new(0.0, -1.0, 0.0))
                .with_scale(Vec3::new(20.0, 1.0, 20.0))
        ];
        let mut trigger_areas = vec![];

        let mut size = 3.0;
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
                    .with_color_channel(1)
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

            if rand::random::<f32>() < 0.1 {
                trigger_areas.push(
                    TriggerArea::new()
                        .with_position(Vec3::new(x, 0.0, z))
                        .with_scale(Vec3::new(size + 0.1, 10.0, size + 0.1))
                        .with_function(TriggerFunction::RecolorLine(
                            Color::LinearRgba(LinearRgba::new(
                                rand::random::<f32>(),
                                rand::random::<f32>(),
                                rand::random::<f32>(),
                                1.0
                            ))
                        ))
                )
            }

            if rand::random::<f32>() < 0.1 {
                trigger_areas.push(
                    TriggerArea::new()
                        .with_position(Vec3::new(x, 0.0, z))
                        .with_scale(Vec3::new(size + 0.1, 10.0, size + 0.1))
                        .with_function(TriggerFunction::ChangeColorOfChannel(
                            1,
                            Color::LinearRgba(LinearRgba::new(
                                rand::random::<f32>(),
                                rand::random::<f32>(),
                                rand::random::<f32>(),
                                1.0
                            ))
                        ))
                )
            }

            size -= 0.01;
            if size < 0.05 {
                break;
            }
        }

        let mut arr = array::from_fn(|_| Color::LinearRgba(LinearRgba::WHITE));
        arr[1] = Color::LinearRgba(LinearRgba::BLUE);

        SceneData {
            start_pos: Vec3::new(0.0, 0.0, 0.0),
            camera_config: CameraConfig::default(),
            line_config: LineConfig::default(),
            cubes,
            trigger_areas,
            color_channels: arr,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cuboid {
    pub color: u8,
    pub position: Vec3,
    pub rotation_euler: Vec3,
    pub scale: Vec3,
}

impl Cuboid {
    pub fn new() -> Self {
        Self {
            color: 0,
            position: Vec3::ZERO,
            rotation_euler: Vec3::ZERO,
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

    pub fn with_color_channel(mut self, color: u8) -> Self {
        self.color = color;
        return self;
    }

    pub fn place(
        &self,
        commands: &mut Commands,
        cuboid_mesh: &Handle<Mesh>,
        game_resource: &LiveGameDataResource
    ) {
        let mut transform = Transform::from_translation(self.position);
        transform = transform.with_scale(self.scale);
        transform.rotate_x(self.rotation_euler.x.to_radians());
        transform.rotate_y(self.rotation_euler.y.to_radians());
        transform.rotate_z(self.rotation_euler.z.to_radians());
        commands.spawn((
            Mesh3d(cuboid_mesh.clone()),
            MeshMaterial3d(game_resource.materials_to_colors[self.color as usize].clone()),
            ColorChannel(self.color),
            transform,
            TransformCollidable,
            GameplayObject,
            Pickable { should_block_lower: true, is_hoverable: true }
        ));
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

    pub fn place(
        &self,
        commands: &mut Commands,
        state: &GameState,
        cuboid_mesh: &Handle<Mesh>,
        materials: &mut Assets<StandardMaterial>
    ) {
        let mut transform = Transform::from_translation(self.position);
        transform = transform.with_scale(self.scale);

        if *state == GameState::InGame {
            commands.spawn((
                transform,
                self.function.clone(),
                GameplayObject
            ));
        } else if *state == GameState::Editor {
            commands.spawn((
                transform,
                self.function.clone(),
                GameplayObject,
                Mesh3d(cuboid_mesh.clone()),
                MeshMaterial3d(materials.add(Color::srgba(0.0, 1.0, 0.0, 0.3))),
                Pickable { should_block_lower: true, is_hoverable: true }
            ));
        }

    }
}

#[derive(Debug, Clone, Component)]
pub struct ColorChannel(pub u8);

