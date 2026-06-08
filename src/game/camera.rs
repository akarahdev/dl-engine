use bevy::prelude::{Commands, Query, Res, Resource, Transform, Vec3, Without};
use bevy::camera::Camera3d;
use crate::game::plugin::LiveGameDataResource;
use crate::game::plugin::LineHead;
use crate::game::utils;

#[derive(Resource, Debug, Clone)]
pub struct CameraConfig {
    pub offset: Vec3
}

impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig {
            offset: Vec3::new(-12.0, 12.0, -12.0)
        }
    }
}

pub fn update_camera(
    mut commands: Commands,
    mut heads: Query<(&LineHead, &Transform), Without<Camera3d>>,
    mut cameras: Query<(&mut Transform, &Camera3d), Without<LineHead>>,
    mut game_resource: Res<LiveGameDataResource>,
) {
    let mut avg = Vec3::ZERO;

    let mut idx = 0;
    for (head, transform) in heads.iter() {
        avg += utils::tip_of_head((&head, transform)) - Vec3::new(0.0, 0.5, 0.0);
        idx += 1;
    }
    avg /= idx as f32;

    for mut camera in cameras {
        camera.0.translation = camera.0.translation.slerp(avg+ game_resource.camera_offset, 0.01);
        let target_rot = camera.0.looking_at(avg, Vec3::Y).rotation;
        camera.0.rotation = camera.0.rotation.slerp(target_rot, 0.01);
    }
}