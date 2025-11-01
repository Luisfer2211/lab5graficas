use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;

fn main() {
    generate_sphere("assets/sphere.obj", 1.0, 30, 30).unwrap();
    println!("✓ Esfera generada exitosamente");
}

fn generate_sphere(filename: &str, radius: f32, rings: usize, sectors: usize) -> std::io::Result<()> {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();

    // Polo norte
    vertices.push((0.0, radius, 0.0));
    normals.push((0.0, 1.0, 0.0));

    // Generar vértices y normales
    for r in 1..rings {
        let theta = PI * r as f32 / rings as f32;
        for s in 0..sectors {
            let phi = 2.0 * PI * s as f32 / sectors as f32;
            let x = radius * theta.sin() * phi.cos();
            let y = radius * theta.cos();
            let z = radius * theta.sin() * phi.sin();
            vertices.push((x, y, z));
            normals.push((x / radius, y / radius, z / radius));
        }
    }

    // Polo sur
    vertices.push((0.0, -radius, 0.0));
    normals.push((0.0, -1.0, 0.0));

    // Generar caras para el polo norte
    for s in 0..sectors {
        let next_s = (s + 1) % sectors;
        let v1 = 1;
        let v2 = s + 2;
        let v3 = next_s + 2;
        faces.push((v1, v2, v3, v1, v2, v3));
    }

    // Generar caras para el cuerpo
    for r in 1..(rings - 1) {
        for s in 0..sectors {
            let next_s = (s + 1) % sectors;

            let current_ring_start = (r - 1) * sectors + 2;
            let next_ring_start = r * sectors + 2;

            let v1 = current_ring_start + s;
            let v2 = current_ring_start + next_s;
            let v3 = next_ring_start + next_s;
            let v4 = next_ring_start + s;

            // Primer triángulo
            faces.push((v1, v2, v3, v1, v2, v3));
            // Segundo triángulo
            faces.push((v1, v3, v4, v1, v3, v4));
        }
    }

    // Generar caras para el polo sur
    let last_vertex = vertices.len();
    let last_ring_start = (rings - 2) * sectors + 2;
    for s in 0..sectors {
        let next_s = (s + 1) % sectors;
        let v1 = last_ring_start + s;
        let v2 = last_ring_start + next_s;
        let v3 = last_vertex;
        faces.push((v1, v2, v3, v1, v2, v3));
    }

    // Escribir archivo OBJ
    let mut file = File::create(filename)?;
    writeln!(file, "# Sphere OBJ file")?;
    writeln!(file, "# Vertices: {}", vertices.len())?;
    writeln!(file, "# Faces: {}\n", faces.len())?;

    // Escribir vértices
    for v in &vertices {
        writeln!(file, "v {:.6} {:.6} {:.6}", v.0, v.1, v.2)?;
    }

    writeln!(file)?;

    // Escribir normales
    for n in &normals {
        writeln!(file, "vn {:.6} {:.6} {:.6}", n.0, n.1, n.2)?;
    }

    writeln!(file)?;

    // Escribir caras
    for face in &faces {
        writeln!(
            file,
            "f {}//{} {}//{} {}//{}",
            face.0, face.3, face.1, face.4, face.2, face.5
        )?;
    }

    println!("Esfera creada: {}", filename);
    println!("  Vértices: {}", vertices.len());
    println!("  Normales: {}", normals.len());
    println!("  Caras: {}", faces.len());

    Ok(())
}
