use std::array;
use bevy::app::{FixedUpdate, Plugin, PreUpdate};
use bevy::asset::Assets;
use bevy::camera::Projection;
use bevy::light::light_consts::lux::OVERCAST_DAY;
use bevy::math::{EulerRot, Vec3};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::picking::Pickable;
use bevy::prelude::{in_state, ButtonInput, Camera3d, Commands, CommandsStatesExt, Component, Cuboid, DirectionalLight, Entity, Handle, IntoScheduleConfigs, KeyCode, MeshMaterial3d, Message, OnEnter, OnExit, PerspectiveProjection, Quat, Query, Res, ResMut, Resource, StandardMaterial, State, Time, Transform, Update, Virtual, With};
use transform_gizmo_bevy::GizmoCamera;
use crate::game::line::ConstLineResources;
use crate::game::scenes::SceneData;
use crate::game::{camera, line, utils};
use crate::game::triggers::TriggerFunction;
use crate::state::GameState;

pub struct PlayScenePlugin;

impl Plugin for PlayScenePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .insert_resource(SceneData::new_simple())
            .insert_resource(ConstLineResources::default())
            .insert_resource(LiveGameDataResource {
                camera_offset: Vec3::new(-6.0, 6.0, -6.0),
                materials_to_colors: array::from_fn(|_| Handle::default()),
            })
            .add_message::<BuildNewLine>()
            .add_systems(OnEnter(GameState::InGame), (
                spawn_meshes,
                spawn_camera,
                setup_line.after(spawn_meshes),
                setup_scene.after(spawn_meshes)
            ))
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

pub fn spawn_meshes(
    mut line_resources: ResMut<ConstLineResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_resource: ResMut<LiveGameDataResource>,
    scene: Res<SceneData>
) {
    line_resources.cuboid_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    line_resources.line_material = materials.add(scene.line_config.color);

    for idx in 0..u8::MAX {
        let idx = idx as usize;
        game_resource.materials_to_colors[idx] = materials.add(scene.color_channels[idx])
    }
}

pub fn spawn_camera(
    mut commands: Commands,
    mut game_resource: ResMut<LiveGameDataResource>,
    scene: Res<SceneData>
) {
    let mut camera_transform = Transform::from_translation(scene.camera_config.offset);
    camera_transform.look_at(scene.line_config.start_pos, Vec3::Y);

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        GameplayObject,
        GizmoCamera
    ));

    game_resource.camera_offset = scene.camera_config.offset;

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
}

pub fn setup_line(
    mut commands: Commands,
    line_resources: Res<ConstLineResources>,
    scene: Res<SceneData>
) {
    commands.spawn((
        Mesh3d(line_resources.cuboid_mesh.clone()),
        MeshMaterial3d(line_resources.line_material.clone()),
        Transform::from_translation(scene.line_config.start_pos),
        LineHead::default(),
        LineTail,
        GameplayObject,
        Pickable { should_block_lower: true, is_hoverable: true }
    ));
}

pub fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    line_resources: Res<ConstLineResources>,
    game_resource: Res<LiveGameDataResource>,
    scene: Res<SceneData>,
    state: Res<State<GameState>>
) {
    for cube in &scene.cubes {
        cube.place(&mut commands, &line_resources.cuboid_mesh, &game_resource);
    }

    for trigger in &scene.trigger_areas {
        trigger.place(
            &mut commands,
            &state,
            &line_resources.cuboid_mesh,
            &mut materials
        );
    }
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
    mut commands: Commands,
    heads: Query<(&LineHead, &Transform, &MeshMaterial3d<StandardMaterial>)>,
    lines: Query<(Entity, &LineTail)>,
    triggers: Query<(&TriggerFunction, &Transform)>,
    mut game_resource: ResMut<LiveGameDataResource>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_resources: ResMut<ConstLineResources>,
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
                    TriggerFunction::RecolorLine(color) => {
                        let color_mat = materials.add(*color);
                        line_resources.line_material = color_mat.clone();
                        for tail in lines {
                            commands.entity(tail.0).insert(MeshMaterial3d(color_mat.clone()));
                        }
                    }
                    TriggerFunction::ChangeColorOfChannel(channel, color) => {
                        let handle = game_resource.materials_to_colors[*channel as usize].clone();
                        if let Some(material) = materials.get_mut(&handle) {
                            material.base_color = *color;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct LiveGameDataResource {
    pub camera_offset: Vec3,
    pub materials_to_colors: [Handle<StandardMaterial>; u8::MAX as usize],
}