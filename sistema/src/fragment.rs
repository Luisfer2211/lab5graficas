use nalgebra_glm::Vec3;

pub struct Fragment {
    pub position: Vec3,
    pub normal: Vec3,
    pub depth: f32,
    pub vertex_position: Vec3,
    pub intensity: f32,
}

impl Fragment {
    pub fn new(position: Vec3, normal: Vec3, depth: f32, vertex_position: Vec3, intensity: f32) -> Self {
        Fragment {
            position,
            normal,
            depth,
            vertex_position,
            intensity,
        }
    }
}
