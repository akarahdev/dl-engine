use bevy::prelude::Transform;
use bevy::math::Vec3;
use crate::game::plugin::LineHead;

pub fn is_colliding(
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

pub fn forward_of_head(head: (&LineHead, &Transform)) -> Vec3 {
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

pub fn tip_of_head(head: (&LineHead, &Transform)) -> Vec3 {
    tip_of_head_offset_from_center(head) + head.1.translation
}