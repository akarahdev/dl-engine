use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Color, Commands, CommandsStatesExt, Entity, Handle, KeyCode, LinearRgba, Mesh, MessageReader, MessageWriter, MouseButton, Query, Res, Resource, Transform, Without};
use bevy::camera::primitives::Aabb;
use bevy::camera::Camera3d;
use bevy::input::ButtonInput;
use bevy::mesh::Mesh3d;
use crate::game::plugin::{BuildNewLine, GameplayObject, LineHead, LineTail, TransformCollidable};
use crate::game::utils;
use crate::state::GameState;

#[derive(Debug, Clone)]
pub struct LineConfig {
    pub start_pos: Vec3,
    pub color: Color
}

impl Default for LineConfig {
    fn default() -> Self {
        LineConfig {
            start_pos: Vec3::ZERO,
            color: Color::LinearRgba(LinearRgba::RED)
        }
    }
}

#[derive(Resource, Default)]
pub struct ConstLineResources {
    pub cuboid_mesh: Handle<Mesh>,
    pub line_material: Handle<StandardMaterial>
}

pub fn process_input(
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

pub fn process_line(
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
            let t = utils::forward_of_head((&head.1, &head.0)) * head.1.speed;
            head.0.translation += t / 2.0;
        } else {
            let t = utils::forward_of_head((&head.1, &head.0)) * head.1.speed;
            head.0.translation += t;
            head.0.translation.y += head.1.y_vel;
            head.1.y_vel -= 0.005;
        }
    }
}

pub fn make_line_fall(
    mut mw: MessageWriter<BuildNewLine>,
    mut heads: Query<(&mut LineHead, &Transform)>,
    collidables: Query<(Entity, &TransformCollidable, &Aabb, &Transform), Without<Camera3d>>,
) {
    for mut head in heads.iter_mut() {
        let mut is_colliding_v = false;

        for collidable in collidables {
            let head_pos = utils::tip_of_head((&*head.0, head.1)) - Vec3::new(0.0, 0.5, 0.0);
            if utils::is_colliding(&collidable.3, head_pos, Vec3::new(0.5, 0.0, 0.5)) {
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

pub fn make_new_line(
    mut mr: MessageReader<BuildNewLine>,
    mut line_resources: Res<ConstLineResources>,
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
                utils::tip_of_head((&*head.1, head.2)) - (utils::forward_of_head((&*head.1, head.2)) * 0.5)
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
                Mesh3d(line_resources.cuboid_mesh.clone()),
                MeshMaterial3d(line_resources.line_material.clone()),
                new_transform,
                head.1.clone().with_new_id(),
                LineTail,
                GameplayObject
            ));
        }

    }
}