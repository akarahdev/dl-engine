use bevy::prelude::Component;
use bevy::math::Vec3;

#[derive(Debug, Clone, Component)]
pub enum TriggerFunction {
    None,
    SetCameraOffset(Vec3)
}