use bevy::prelude::{Resource, Vec3};

#[derive(Resource, Debug)]
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