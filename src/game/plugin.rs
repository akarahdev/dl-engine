use std::collections::HashMap;
use std::sync::Arc;
use bevy::app::{FixedUpdate, Plugin};
use bevy::asset::Assets;
use bevy::light::light_consts::lux::{FULL_DAYLIGHT, OVERCAST_DAY};
use bevy::math::{EulerRot, Vec3};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::prelude::{in_state, ButtonInput, Camera3d, Commands, Component, Cuboid, DirectionalLight, IntoScheduleConfigs, KeyCode, MeshMaterial3d, OnEnter, Quat, Query, Res, ResMut, StandardMaterial, Transform, With};
use crate::game::scenes::SceneData;
use crate::state::GameState;

pub struct PlayScenePlugin;

impl Plugin for PlayScenePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .insert_resource(SceneData::new_simple())
            .add_systems(OnEnter(GameState::InGame), setup_scene)
            .add_systems(FixedUpdate, print_scene.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct LineHead;

#[derive(Component)]
pub struct LineTail;

#[derive(Component)]
pub struct TransformCollidable;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene: Res<SceneData>,
) {
    let cuboid_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    let mut camera_transform = Transform::from_translation(scene.camera_config.offset);
    camera_transform.look_at(scene.line_config.start_pos, Vec3::Y);

    commands.spawn((
        Camera3d::default(),
        camera_transform
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
            TransformCollidable
        ));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: FULL_DAYLIGHT,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(-100.0, 100.0, -100.0))
    ));

    commands.spawn((
        Mesh3d(cuboid_mesh.clone()),
        MeshMaterial3d(materials.add(scene.line_config.color)),
        Transform::from_translation(scene.line_config.start_pos),
        LineHead
    ));
}

fn print_scene(
    scene: Res<SceneData>,
    cameras: Query<&mut Transform, With<Camera3d>>,
    input: Res<ButtonInput<KeyCode>>
) {
    for mut camera in cameras {
        let rot = camera.rotation.to_euler(EulerRot::XYZ);
        println!(
            "{:#?} {:#?} {:#?}",
            rot.0.to_degrees(), rot.1.to_degrees(), rot.2.to_degrees()
        );
        if input.pressed(KeyCode::ArrowLeft) {
            let (mut yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::XYZ);
            yaw -= 1_f32.to_radians();
            camera.rotation = Quat::from_euler(EulerRot::XYZ, yaw, pitch, roll);
            println!("{yaw:?} {pitch:?} {roll:?}");
        }
        if input.pressed(KeyCode::ArrowRight) {
            let (mut yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::XYZ);
            yaw += 1_f32.to_radians();
            camera.rotation = Quat::from_euler(EulerRot::XYZ, yaw, pitch, roll);
            println!("{yaw:?} {pitch:?} {roll:?}");
        }
        if input.pressed(KeyCode::ArrowUp) {
            let (mut yaw, mut pitch, roll) = camera.rotation.to_euler(EulerRot::XYZ);
            pitch += 1_f32.to_radians();
            camera.rotation = Quat::from_euler(EulerRot::XYZ, yaw, pitch, roll);
            println!("{yaw:?} {pitch:?} {roll:?}");
        }
        if input.pressed(KeyCode::ArrowDown) {
            let (mut yaw, mut pitch, roll) = camera.rotation.to_euler(EulerRot::XYZ);
            pitch -= 1_f32.to_radians();
            camera.rotation = Quat::from_euler(EulerRot::XYZ, yaw, pitch, roll);
            println!("{yaw:?} {pitch:?} {roll:?}");
        }
    }
}