use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Color, Handle, LinearRgba, Material, Mesh, MeshMaterial3d, Resource};

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
pub struct LineResource {
    pub line_mesh: Handle<Mesh>,
    pub line_material: Handle<StandardMaterial>
}