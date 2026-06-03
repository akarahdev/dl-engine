use std::sync::Arc;
use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::prelude::AppExtStates;
use crate::game::plugin::PlayScenePlugin;
use crate::state::GameState;

pub mod game;
pub mod state;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayScenePlugin)
        .insert_state(GameState::InGame)
        .run();
}
