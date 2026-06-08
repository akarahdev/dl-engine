extern crate core;

use bevy::app::App;
use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::DefaultPlugins;
use bevy::picking::DefaultPickingPlugins;
use bevy::prelude::{AppExtStates, MeshPickingPlugin};
use bevy_simple_text_input::{TextInput, TextInputPlugin};
use transform_gizmo_bevy::TransformGizmoPlugin;
use crate::editor::plugin::EditScenePlugin;
use crate::game::plugin::PlayScenePlugin;
use crate::state::GameState;

pub mod game;
pub mod state;
pub mod editor;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TransformGizmoPlugin)
        .add_plugins(TextInputPlugin)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(FreeCameraPlugin)
        .add_plugins(PlayScenePlugin)
        .add_plugins(EditScenePlugin)
        .insert_state(GameState::InGame)
        .run();
}
