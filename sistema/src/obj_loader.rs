use nalgebra_glm::Vec3;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::vertex::Vertex;

pub struct Obj {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    tex_coords: Vec<Vec3>,
    faces: Vec<[usize; 9]>,
}

impl Obj {
    pub fn load(filename: &str) -> Result<Self, std::io::Error> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut faces = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse().unwrap_or(0.0);
                        let y: f32 = parts[2].parse().unwrap_or(0.0);
                        let z: f32 = parts[3].parse().unwrap_or(0.0);
                        vertices.push(Vec3::new(x, y, z));
                    }
                }
                "vn" => {
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse().unwrap_or(0.0);
                        let y: f32 = parts[2].parse().unwrap_or(0.0);
                        let z: f32 = parts[3].parse().unwrap_or(0.0);
                        normals.push(Vec3::new(x, y, z));
                    }
                }
                "vt" => {
                    if parts.len() >= 3 {
                        let u: f32 = parts[1].parse().unwrap_or(0.0);
                        let v: f32 = parts[2].parse().unwrap_or(0.0);
                        tex_coords.push(Vec3::new(u, v, 0.0));
                    }
                }
                "f" => {
                    if parts.len() >= 4 {
                        let mut face = [0; 9];
                        for (i, part) in parts.iter().skip(1).take(3).enumerate() {
                            let indices: Vec<&str> = part.split('/').collect();
                            if !indices.is_empty() {
                                face[i * 3] = indices[0].parse::<usize>().unwrap_or(1) - 1;
                            }
                            if indices.len() > 1 && !indices[1].is_empty() {
                                face[i * 3 + 1] = indices[1].parse::<usize>().unwrap_or(1) - 1;
                            }
                            if indices.len() > 2 {
                                face[i * 3 + 2] = indices[2].parse::<usize>().unwrap_or(1) - 1;
                            }
                        }
                        faces.push(face);
                    }
                }
                _ => {}
            }
        }

        Ok(Obj {
            vertices,
            normals,
            tex_coords,
            faces,
        })
    }

    pub fn get_vertex_array(&self) -> Vec<Vertex> {
        let mut vertex_array = Vec::new();

        for face in &self.faces {
            for i in 0..3 {
                let pos_idx = face[i * 3];
                let tex_idx = face[i * 3 + 1];
                let norm_idx = face[i * 3 + 2];

                let position = self.vertices.get(pos_idx).copied().unwrap_or(Vec3::zeros());
                let tex_coords = self.tex_coords.get(tex_idx).copied().unwrap_or(Vec3::zeros());
                let normal = self.normals.get(norm_idx).copied().unwrap_or(Vec3::new(0.0, 1.0, 0.0));

                vertex_array.push(Vertex::new(position, normal, tex_coords));
            }
        }

        vertex_array
    }
}
