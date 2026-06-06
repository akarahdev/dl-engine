use bevy::app::{FixedUpdate, Plugin};
use bevy::camera::{Camera, Camera3d};
use bevy::camera_controller::free_camera::FreeCamera;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{EulerRot, Quat, Vec3};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{in_state, ButtonInput, Commands, CommandsStatesExt, ContainsEntity, Entity, IntoScheduleConfigs, KeyCode, MeshMaterial3d, MessageReader, MouseButton, Node, OnEnter, OnExit, Query, Res, ResMut, Resource, Scroll, Text, Transform, Update};
use bevy::ui::percent;
use crate::game::plugin::{setup_line, setup_scene, spawn_camera, spawn_meshes, GameplayObject};
use crate::game::scenes::{ColorChannel, Cuboid, SceneData, TriggerArea};
use crate::game::triggers::TriggerFunction;
use crate::state::GameState;

#[derive(Resource)]
pub struct EditorContext {
    speed: f32
}

pub struct EditScenePlugin;

impl Plugin for EditScenePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .insert_resource(EditorContext {
                speed: 0.1
            })
            .add_systems(OnEnter(GameState::Editor), (
                spawn_meshes,
                spawn_camera,
                setup_line.after(spawn_meshes),
                setup_scene.after(spawn_meshes),
                attach_free_camera.after(spawn_camera)
            ))
            .add_systems(OnExit(GameState::Editor), (save_scene_to_data, crate::game::plugin::cleanup_scene).chain())
            .add_systems(OnEnter(GameState::Editor), load_editor_ui)
            .add_systems(Update, play_level.run_if(in_state(GameState::Editor)));
    }
}

fn save_scene_to_data(
    mut commands: Commands,
    scene_data: ResMut<SceneData>,
    cubes: Query<(Entity, &Transform, &MeshMaterial3d<StandardMaterial>, &ColorChannel)>,
    triggers: Query<(Entity, &Transform, &TriggerFunction)>,
) {
    let mut new_scene = SceneData::new_simple();

    new_scene.cubes.clear();
    new_scene.trigger_areas.clear();

    for cube in cubes.iter() {
        new_scene.cubes.push(
            Cuboid::new()
                .with_position(cube.1.translation)
                .with_scale(cube.1.scale)
                .with_color_channel(cube.3.0)
        )
    }

    for trigger in triggers.iter() {
        new_scene.trigger_areas.push(
            TriggerArea::new()
                .with_position(trigger.1.translation)
                .with_scale(trigger.1.scale)
                .with_function(trigger.2.clone())
        )
    }

    commands.insert_resource(new_scene);
}

fn load_editor_ui(
    mut commands: Commands
) {
    commands.spawn((
        Text::new("Editor Mode"),
        Node {
            top: percent(5),
            left: percent(5),
            ..Default::default()
        },
        GameplayObject
    ));
}

fn play_level(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>
) {
    if input.just_pressed(KeyCode::KeyP) {
        commands.set_state(GameState::InGame);
    }
}

fn attach_free_camera(
    mut commands: Commands,
    cameras: Query<(Entity, &Camera3d)>
) {
    for camera in cameras {
        commands.entity(camera.0).insert(FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            mouse_key_cursor_grab: MouseButton::Right,
            ..Default::default()
        });
    }
}