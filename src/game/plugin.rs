use bevy::app::{FixedUpdate, Plugin, PreUpdate};
use bevy::asset::Assets;
use bevy::light::light_consts::lux::OVERCAST_DAY;
use bevy::math::{EulerRot, Vec3};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::prelude::{in_state, ButtonInput, Camera3d, Commands, CommandsStatesExt, Component, Cuboid, DirectionalLight, Entity, IntoScheduleConfigs, KeyCode, MeshMaterial3d, Message, OnEnter, OnExit, Quat, Query, Res, ResMut, Resource, StandardMaterial, Time, Transform, Update, Virtual, With};
use crate::game::line::LineResource;
use crate::game::scenes::SceneData;
use crate::game::{camera, line, utils};
use crate::game::triggers::TriggerFunction;
use crate::state::GameState;

pub struct PlayScenePlugin;

impl Plugin for PlayScenePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .insert_resource(SceneData::new_simple())
            .insert_resource(LineResource::default())
            .insert_resource(LiveGameDataResource::default())
            .add_message::<BuildNewLine>()
            .add_systems(OnEnter(GameState::InGame), setup_scene)
            .add_systems(OnExit(GameState::InGame), cleanup_scene)
            .add_systems(Update, camera::update_camera.run_if(in_state(GameState::InGame)))
            .add_systems(PreUpdate, line::process_input.run_if(in_state(GameState::InGame)))
            .add_systems(FixedUpdate, (line::make_new_line, line::process_line, activate_triggers, line::make_line_fall).chain().run_if(in_state(GameState::InGame)))
            .add_systems(Update, switch_to_editor.run_if(in_state(GameState::InGame)));
    }
}

fn switch_to_editor(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>
) {
    if input.just_pressed(KeyCode::KeyE) {
        commands.set_state(GameState::Editor);
    }
}

#[derive(Component, Clone)]
pub struct LineHead {
    pub frozen: bool,
    pub speed: f32,
    pub base_rot: f32,
    pub is_in_alternated_rot: bool,
    pub unique_id: i32,
    pub is_on_ground: bool,
    pub y_vel: f32
}

impl LineHead {
    pub fn new(id: i32) -> Self {
        LineHead {
            frozen: true,
            speed: 0.2,
            base_rot: 0.0,
            is_in_alternated_rot: false,
            unique_id: id,
            is_on_ground: true,
            y_vel: 0.0
        }
    }

    pub fn new_rand() -> Self {
        LineHead {
            frozen: true,
            speed: 0.2,
            base_rot: 0.0,
            is_in_alternated_rot: false,
            unique_id: rand::random::<i32>(),
            is_on_ground: true,
            y_vel: 0.0
        }
    }

    pub fn with_new_id(self) -> Self {
        LineHead {
            unique_id: rand::random::<i32>(),
            ..self
        }
    }
}

impl Default for LineHead {
    fn default() -> Self {
        Self::new_rand()
    }
}

#[derive(Component)]
pub struct LineTail;

#[derive(Component)]
pub struct GameplayObject;

#[derive(Component)]
pub struct TransformCollidable;

#[derive(Message)]
pub struct BuildNewLine {
    pub flip: bool,
    pub targetting: i32
}

#[derive(Message)]
pub struct ResetScene;

pub fn setup_scene(
    mut commands: Commands,
    mut line_resources: ResMut<LineResource>,
    mut game_resource: ResMut<LiveGameDataResource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene: Res<SceneData>,
    mut time: ResMut<Time<Virtual>>
) {
    // time.set_relative_speed(0.1);

    line_resources.line_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    line_resources.line_material = materials.add(scene.line_config.color);
    let cuboid_mesh = line_resources.line_mesh.clone();

    let mut camera_transform = Transform::from_translation(scene.camera_config.offset);
    camera_transform.look_at(scene.line_config.start_pos, Vec3::Y);

    game_resource.camera_offset = scene.camera_config.offset;

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        GameplayObject
    ));

    for cube in &scene.cubes {
        let mut transform = Transform::from_translation(cube.position);
        transform = transform.with_scale(cube.scale);
        transform.rotate_x(cube.rotation_euler.x.to_radians());
        transform.rotate_y(cube.rotation_euler.y.to_radians());
        transform.rotate_z(cube.rotation_euler.z.to_radians());
        commands.spawn((
            Mesh3d(cuboid_mesh.clone()),
            MeshMaterial3d(materials.add(cube.color)),
            transform,
            TransformCollidable,
            GameplayObject
        ));
    }

    for trigger in &scene.trigger_areas {
        let mut transform = Transform::from_translation(trigger.position);
        transform = transform.with_scale(trigger.scale);
        commands.spawn((
            transform,
            trigger.function.clone(),
            GameplayObject
        ));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: OVERCAST_DAY,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_rotation(Quat::from_euler(EulerRot::XYZ, -45.0_f32.to_radians(), -45.0_f32.to_radians(), 0.0)),
        GameplayObject
    ));

    commands.spawn((
        Mesh3d(cuboid_mesh.clone()),
        MeshMaterial3d(line_resources.line_material.clone()),
        Transform::from_translation(scene.line_config.start_pos),
        LineHead::default(),
        GameplayObject
    ));
}

pub(crate) fn cleanup_scene(
    mut commands: Commands,
    objects: Query<Entity, With<GameplayObject>>,
) {
    for object in objects {
        commands.entity(object).despawn();
    }
}

fn activate_triggers(
    heads: Query<(&LineHead, &Transform)>,
    triggers: Query<(&TriggerFunction, &Transform)>,
    mut game_resource: ResMut<LiveGameDataResource>,
) {
    for head in heads {
        let head_pos = utils::tip_of_head((head.0, head.1)) - Vec3::new(0.0, 0.5, 0.0);
        for trigger in triggers {
            if utils::is_colliding(trigger.1, head_pos, Vec3::new(0.0, 0.0, 0.0)) {
                match trigger.0 {
                    TriggerFunction::None => {}
                    TriggerFunction::SetCameraOffset(offset) => {
                        game_resource.camera_offset = *offset;
                    }
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct LiveGameDataResource {
    pub camera_offset: Vec3
}