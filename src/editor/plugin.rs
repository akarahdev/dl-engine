use bevy::app::{Plugin, PostUpdate};
use bevy::camera::Camera3d;
use bevy::camera_controller::free_camera::FreeCamera;
use bevy::light::PointLight;
use bevy::math::Vec3;
use bevy::mesh::Mesh3d;
use bevy::pbr::StandardMaterial;
use bevy::picking::{Pickable, PickingSystems};
use bevy::prelude::{in_state, Axis, ButtonInput, Click, Commands, CommandsStatesExt, Component, ContainsEntity, Drag, DragEnd, DragStart, Entity, IntoScheduleConfigs, KeyCode, MeshMaterial3d, Message, MessageReader, MessageWriter, MouseButton, Node, On, OnEnter, OnExit, Out, Pointer, Query, Res, ResMut, Resource, Scroll, Single, Text, Transform, Update, With, Without, World};
use bevy::ui::percent;
use transform_gizmo_bevy::GizmoTarget;
use crate::game::line::{ConstLineResources, LineConfig};
use crate::game::plugin::{setup_line, setup_scene, spawn_camera, spawn_meshes, GameplayObject, LineHead};
use crate::game::scenes::{ColorChannel, Cuboid, SceneData, TriggerArea};
use crate::game::triggers::TriggerFunction;
use crate::state::GameState;

#[derive(Component, Debug)]
pub struct EditorSelection;

#[derive(Message)]
pub struct EditorSelectionUpdate;

pub struct EditScenePlugin;

impl Plugin for EditScenePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_message::<EditorSelectionUpdate>()
            .add_systems(OnEnter(GameState::Editor), (
                spawn_meshes,
                spawn_camera,
                setup_line.after(spawn_meshes),
                setup_scene.after(spawn_meshes),
                attach_free_camera.after(spawn_camera),
                load_editor_ui.after(setup_scene)
            ))
            .add_systems(OnExit(GameState::Editor), (save_scene_to_data, crate::game::plugin::cleanup_scene).chain())
            .add_systems(Update, play_level.run_if(in_state(GameState::Editor)))
            .add_systems(Update, (
                select_object,
                rehighlight_selection
            ).run_if(in_state(GameState::Editor)).after(PickingSystems::Hover))
            .add_systems(PostUpdate, (
                rehighlight_selection,
            ).run_if(in_state(GameState::Editor)).after(PickingSystems::Hover));
    }
}

fn rehighlight_selection(
    mut mr: MessageReader<EditorSelectionUpdate>,
    mut commands: Commands,
    line_resources: Res<ConstLineResources>,
    selection: Query<(Entity, &Transform), With<EditorSelection>>,
    prev_highlight: Query<Entity, With<PointLight>>
) {
    for _ in mr.read() {
        for entity in prev_highlight {
            commands.entity(entity).remove::<PointLight>();
            commands.entity(entity).remove::<GizmoTarget>();
        }

        for (entity, _) in selection.iter() {
            commands.entity(entity).insert(PointLight {
                intensity: 50000.0,
                ..Default::default()
            });
            commands.entity(entity).insert(GizmoTarget::default());
        }
    }
}

fn save_scene_to_data(
    mut commands: Commands,
    scene_data: ResMut<SceneData>,
    head: Single<(&Transform, &LineHead)>,
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

    new_scene.line_config = LineConfig {
        color: scene_data.line_config.color,
        start_pos: head.0.translation
    };

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

fn select_object(
    mut mw: MessageWriter<EditorSelectionUpdate>,
    mut commands: Commands,
    selection: Query<(Entity, &EditorSelection, &Transform, Option<&MeshMaterial3d<StandardMaterial>>)>,
    mesh_targets_query: Query<(Entity, &Transform, &MeshMaterial3d<StandardMaterial>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut recv_click: MessageReader<Pointer<Click>>,
) {
    for msg in recv_click.read() {
        println!("{:?}", msg);

        if let Ok(_) = mesh_targets_query.get(msg.entity) {
            if msg.entity.index_u32() == 0 {
                mw.write(EditorSelectionUpdate);
                continue;
            }

            if keyboard_input.pressed(KeyCode::ShiftLeft) {
                commands.entity(msg.entity).insert(EditorSelection);
            } else {
                for entity in selection.iter() {
                    commands.entity(entity.0).remove::<EditorSelection>();
                }
                commands.entity(msg.entity).insert(EditorSelection);
            }

            mw.write(EditorSelectionUpdate);
        }
    }

}