use bevy::app::{FixedUpdate, Plugin, PostUpdate, PreUpdate};
use bevy::asset::Assets;
use bevy::camera::primitives::Aabb;
use bevy::light::light_consts::lux::{FULL_DAYLIGHT, OVERCAST_DAY};
use bevy::math::{EulerRot, Vec3};
use bevy::math::bounding::{Aabb3d, BoundingVolume};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::prelude::{in_state, ButtonInput, Camera3d, Commands, CommandsStatesExt, Component, Cuboid, DirectionalLight, Entity, IntoScheduleConfigs, KeyCode, MeshMaterial3d, Message, MessageReader, MessageWriter, MouseButton, OnEnter, OnExit, Quat, Query, Res, ResMut, StandardMaterial, Time, Transform, Update, Virtual, With, Without, World};
use crate::game::line::{LineResource, LiveGameDataResource};
use crate::game::scenes::{SceneData, TriggerArea, TriggerFunction};
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
            .add_systems(Update, update_camera.run_if(in_state(GameState::InGame)))
            .add_systems(PreUpdate, process_input.run_if(in_state(GameState::InGame)))
            .add_systems(FixedUpdate, (make_new_line, process_line, activate_triggers, make_line_fall).chain().run_if(in_state(GameState::InGame)));
            // TODO: make process_line also run after make_new_line
            // .add_systems(PostUpdate, make_new_line.run_if(in_state(GameState::InGame)));
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
    flip: bool,
    targetting: i32
}

#[derive(Message)]
pub struct ResetScene;

fn setup_scene(
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
            .with_rotation(Quat::from_euler(EulerRot::XYZ, -90.0, 0.0, 0.0)),
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
            if head.1.is_on_ground {
                ew.write(BuildNewLine { flip: true, targetting: -1 });
            }

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
    mut collidables: Query<(Entity, &TransformCollidable, &Aabb), Without<Camera3d>>,
) {
    for mut head in heads.iter_mut() {
        if head.1.frozen {
            return;
        }
        if head.1.is_on_ground {
            head.0.scale.z += head.1.speed;
            let t = forward_of_head((&head.1, &head.0)) * head.1.speed;
            head.0.translation += t / 2.0;
        } else {
            let t = forward_of_head((&head.1, &head.0)) * head.1.speed;
            head.0.translation += t;
            head.0.translation.y += head.1.y_vel;
            head.1.y_vel -= 0.005;
        }
    }
}

fn is_colliding(
    cuboid: &Transform,
    head_pos: Vec3,
    epsilon: Vec3
) -> bool {
    let min = cuboid.translation - (cuboid.scale / 2.0) - epsilon;
    let max = cuboid.translation + (cuboid.scale / 2.0) + epsilon;
    head_pos.x >= min.x && head_pos.x <= max.x
        && head_pos.y >= min.y && head_pos.y <= max.y
        && head_pos.z >= min.z && head_pos.z <= max.z
}

fn make_line_fall(
    mut mw: MessageWriter<BuildNewLine>,
    mut heads: Query<(&mut LineHead, &Transform)>,
    collidables: Query<(Entity, &TransformCollidable, &Aabb, &Transform), Without<Camera3d>>,
) {
    for mut head in heads.iter_mut() {
        let mut is_colliding_v = false;

        for collidable in collidables {
            let head_pos = tip_of_head((&*head.0, head.1)) - Vec3::new(0.0, 0.5, 0.0);
            if is_colliding(&collidable.3, head_pos, Vec3::new(0.5, 0.0, 0.5)) {
                is_colliding_v = true;
                break;
            }
        }

        if !is_colliding_v && head.0.is_on_ground {
            head.0.is_on_ground = false;
            mw.write(BuildNewLine {
                flip: false,
                targetting: head.0.unique_id
            });
        }

        if is_colliding_v && !head.0.is_on_ground {
            head.0.is_on_ground = true;
            head.0.y_vel = 0.0;
            mw.write(BuildNewLine {
                flip: false,
                targetting: head.0.unique_id
            });
        }
    }
}

fn update_camera(
    mut commands: Commands,
    mut heads: Query<(&LineHead, &Transform), Without<Camera3d>>,
    mut cameras: Query<(&mut Transform, &Camera3d), Without<LineHead>>,
    mut game_resource: Res<LiveGameDataResource>,
) {
    let mut avg = Vec3::ZERO;

    let mut idx = 0;
    for (head, transform) in heads.iter() {
        avg += tip_of_head((&head, transform)) - Vec3::new(0.0, 0.5, 0.0);
        idx += 1;
    }
    avg /= idx as f32;

    for mut camera in cameras {
        camera.0.translation = camera.0.translation.slerp(avg+ game_resource.camera_offset, 0.01);
        let target_rot = camera.0.looking_at(avg, Vec3::Y).rotation;
        camera.0.rotation = camera.0.rotation.slerp(target_rot, 0.01);
    }
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
            if msg.targetting != -1 && msg.targetting != head.1.unique_id {
                continue;
            }

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
                head.1.clone().with_new_id(),
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

fn activate_triggers(
    heads: Query<(&LineHead, &Transform)>,
    triggers: Query<(&TriggerFunction, &Transform)>,
    mut game_resource: ResMut<LiveGameDataResource>,
) {
    for head in heads {
        let head_pos = tip_of_head((head.0, head.1)) - Vec3::new(0.0, 0.5, 0.0);
        for trigger in triggers {
            if is_colliding(trigger.1, head_pos, Vec3::new(0.0, 0.0, 0.0)) {
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