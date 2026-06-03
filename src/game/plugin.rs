use bevy::app::{FixedUpdate, Plugin, PostUpdate, PreUpdate};
use bevy::asset::Assets;
use bevy::light::light_consts::lux::{FULL_DAYLIGHT, OVERCAST_DAY};
use bevy::math::{EulerRot, Vec3};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::prelude::{in_state, ButtonInput, Camera3d, Commands, CommandsStatesExt, Component, Cuboid, DirectionalLight, Entity, IntoScheduleConfigs, KeyCode, MeshMaterial3d, Message, MessageReader, MessageWriter, MouseButton, OnEnter, OnExit, Quat, Query, Res, ResMut, StandardMaterial, Time, Transform, Update, Virtual, With, Without, World};
use crate::game::line::LineResource;
use crate::game::scenes::SceneData;
use crate::state::GameState;

pub struct PlayScenePlugin;

impl Plugin for PlayScenePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .insert_resource(SceneData::new_simple())
            .insert_resource(LineResource::default())
            .add_message::<BuildNewLine>()
            .add_systems(OnEnter(GameState::InGame), setup_scene)
            .add_systems(OnExit(GameState::InGame), cleanup_scene)
            .add_systems(PreUpdate, process_input.run_if(in_state(GameState::InGame)))
            .add_systems(FixedUpdate, (make_new_line, process_line).chain().run_if(in_state(GameState::InGame)));
            // TODO: make process_line also run after make_new_line
            // .add_systems(PostUpdate, make_new_line.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component, Clone)]
pub struct LineHead {
    pub frozen: bool,
    pub speed: f32,
    pub base_rot: f32,
    pub is_in_alternated_rot: bool
}

impl Default for LineHead {
    fn default() -> Self {
        LineHead {
            frozen: true,
            speed: 0.2,
            base_rot: 0.0,
            is_in_alternated_rot: false
        }
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
    flip: bool
}

#[derive(Message)]
pub struct ResetScene;

fn setup_scene(
    mut commands: Commands,
    mut line_resources: ResMut<LineResource>,
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

    commands.spawn((
        DirectionalLight {
            illuminance: FULL_DAYLIGHT,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(-100.0, 100.0, -100.0)),
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

fn process_input(
    mut commands: Commands,
    mut ew: MessageWriter<BuildNewLine>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    heads: Query<(Entity, &mut LineHead)>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        for mut head in heads {
            if head.1.frozen {
                head.1.frozen = false;
                continue;
            }

            ew.write(BuildNewLine { flip: true });
        }
    }

    if key_input.just_pressed(KeyCode::KeyR) {
        commands.set_state(GameState::Menu);
        commands.set_state(GameState::InGame);
    }
}

fn process_line(
    mut mr: MessageReader<BuildNewLine>,
    mut heads: Query<(&mut Transform, &mut LineHead), Without<Camera3d>>,
) {
    for mut head in heads.iter_mut() {
        process_head((&mut head.0, &mut head.1));
    }
}

fn process_head(head: (&mut Transform, &mut LineHead)) {
    if head.1.frozen {
        return;
    }
    head.0.scale.z += head.1.speed;
    let t = forward_of_head((&head.1, &head.0)) * head.1.speed;
    head.0.translation += t / 2.0;
}
fn forward_of_head(head: (&LineHead, &Transform)) -> Vec3 {
    let f_rot =
        head.0.base_rot.to_radians()
            + if head.0.is_in_alternated_rot { 90.0_f32.to_radians() } else { 0.0 };
    return Vec3::new(
        f_rot.sin(),
        0.0,
        f_rot.cos()
    )
}

fn tip_of_head_offset_from_center(head: (&LineHead, &Transform)) -> Vec3 {
    forward_of_head(head) * (head.1.scale.z / 2.0)
}

fn tip_of_head(head: (&LineHead, &Transform)) -> Vec3 {
    tip_of_head_offset_from_center(head) + head.1.translation
}

fn make_new_line(
    mut mr: MessageReader<BuildNewLine>,
    mut line_resources: Res<LineResource>,
    mut commands: Commands,
    mut heads: Query<(Entity, &mut LineHead, &Transform)>,
) {
    for msg in mr.read() {
        for mut head in heads.iter_mut() {
            commands.entity(head.0).remove::<LineHead>();

            let mut new_transform = Transform::from_translation(
                tip_of_head((&*head.1, head.2)) - (forward_of_head((&*head.1, head.2)) * 0.5)
            );

            if msg.flip {
                head.1.is_in_alternated_rot = !head.1.is_in_alternated_rot;
            }

            let factor = if head.1.is_in_alternated_rot {
                90.0_f32.to_radians()
            } else {
                0.0
            };
            new_transform.rotate_y(head.1.base_rot.to_radians() + factor);
            commands.spawn((
                Mesh3d(line_resources.line_mesh.clone()),
                MeshMaterial3d(line_resources.line_material.clone()),
                new_transform,
                head.1.clone(),
                GameplayObject
            ));
        }

    }
}

fn cleanup_scene(
    mut commands: Commands,
    objects: Query<Entity, With<GameplayObject>>,
) {
    for object in objects {
        commands.entity(object).despawn();
    }
}
