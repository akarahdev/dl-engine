use bevy::prelude::{Color, Component};
use bevy::math::Vec3;

#[derive(Debug, Clone, Component)]
pub enum TriggerFunction {
    None,
    SetCameraOffset(Vec3),
    RecolorLine(Color),
    ChangeColorOfChannel(u8, Color)
}