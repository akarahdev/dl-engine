use bevy::app::{FixedUpdate, Plugin};
use bevy::camera::{Camera, Camera3d};
use bevy::camera_controller::free_camera::FreeCamera;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{EulerRot, Quat, Vec3};
use bevy::prelude::{in_state, ButtonInput, Commands, CommandsStatesExt, ContainsEntity, Entity, IntoScheduleConfigs, KeyCode, MessageReader, MouseButton, Node, OnEnter, OnExit, Query, Res, ResMut, Resource, Scroll, Text, Transform, Update};
use bevy::ui::percent;
use crate::game::plugin::GameplayObject;
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
            .add_systems(OnEnter(GameState::Editor), (crate::game::plugin::setup_scene, attach_free_camera).chain())
            .add_systems(OnExit(GameState::Editor), crate::game::plugin::cleanup_scene)
            .add_systems(OnEnter(GameState::Editor), load_editor_ui)
            .add_systems(Update, play_level.run_if(in_state(GameState::Editor)));
    }
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
            ..Default::default()
        });
    }
}