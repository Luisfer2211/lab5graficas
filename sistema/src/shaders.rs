use nalgebra_glm::{Vec3, Vec4, Mat4};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let ndc_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * ndc_position;

    let model_mat3 = Mat4::identity();
    let normal_matrix = model_mat3;

    let normal_vector = Vec4::new(
        vertex.normal.x,
        vertex.normal.y,
        vertex.normal.z,
        0.0
    );

    let transformed_normal = normal_matrix * normal_vector;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: Vec3::new(transformed_normal.x, transformed_normal.y, transformed_normal.z).normalize(),
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, shader_type: u8) -> Color {
    match shader_type {
        1 => sun_shader(fragment, uniforms),
        2 => rocky_planet_shader(fragment, uniforms),
        3 => gas_giant_shader(fragment, uniforms),
        4 => ringed_planet_shader(fragment, uniforms),
        5 => planet_with_moon_shader(fragment, uniforms),
        6 => moon_shader(fragment, uniforms),
        _ => Color::new(255, 255, 255),
    }
}

// Funciones matemáticas rápidas para patrones procedurales
#[inline(always)]
fn fast_noise(p: Vec3) -> f32 {
    ((p.x * 12.9898 + p.y * 78.233 + p.z * 37.719).sin() * 43758.5453).fract()
}

#[inline(always)]
fn pattern1(p: Vec3, time: f32) -> f32 {
    let a = (p.x * 3.0 + time).sin();
    let b = (p.y * 3.0 - time * 0.5).cos();
    let c = (p.z * 3.0 + time * 0.3).sin();
    (a + b + c) * 0.333
}

#[inline(always)]
fn pattern2(p: Vec3, time: f32) -> f32 {
    let freq = 8.0;
    ((p.x * freq).sin() * (p.y * freq).cos() + (p.z * freq + time).sin()) * 0.5
}

#[inline(always)]
fn voronoi_simple(p: Vec3) -> f32 {
    let pi = Vec3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let pf = Vec3::new(p.x.fract(), p.y.fract(), p.z.fract());
    
    let mut min_dist: f32 = 2.0;
    for i in -1..=1 {
        for j in -1..=1 {
            let neighbor = Vec3::new(i as f32, j as f32, 0.0);
            let point = neighbor + Vec3::new(
                fast_noise(pi + neighbor),
                fast_noise(pi + neighbor + Vec3::new(0.1, 0.1, 0.1)),
                0.0
            );
            let diff = point - pf;
            let dist = diff.x * diff.x + diff.y * diff.y;
            min_dist = min_dist.min(dist);
        }
    }
    min_dist.sqrt()
}

// ===== SHADER 1: SOL CON PLASMA ANIMADO =====
fn sun_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.vertex_position * 3.0;
    let time = uniforms.time as f32 * 0.02;
    
    // Plasma con múltiples ondas
    let wave1 = ((pos.x * 4.0 + time).sin() + (pos.y * 3.0 - time * 0.7).cos()) * 0.5;
    let wave2 = ((pos.y * 5.0 + time * 1.3).sin() + (pos.z * 4.0 + time).sin()) * 0.5;
    let wave3 = ((pos.x * 2.0 - time * 0.5).cos() * (pos.y * 2.0 + time * 0.8).sin()) * 0.5;
    
    let plasma = (wave1 + wave2 + wave3) * 0.5 + 0.5;
    
    // Vórtices rotativos
    let angle = pos.y.atan2(pos.x);
    let radius = (pos.x * pos.x + pos.y * pos.y).sqrt();
    let spiral = ((angle * 8.0 + radius * 6.0 - time * 3.0).sin() + 1.0) * 0.5;
    
    // Manchas solares (zonas oscuras)
    let spot_pattern = ((pos.x * 8.0).sin() * (pos.y * 8.0).cos() + (pos.z * 8.0 + time * 0.1).sin());
    let spots = if spot_pattern > 0.8 { 0.5 } else { 1.0 };
    
    // Pulsación de corona
    let dist = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt();
    let corona = (1.0 - dist * 0.3).max(0.0).powf(3.0);
    let pulse = (time * 4.0).sin() * 0.2 + 0.8;
    
    // Gradiente de temperatura
    let temp = plasma * spiral;
    let base_color = if temp > 0.7 {
        Color::new(255, 255, 220) // Blanco caliente
    } else if temp > 0.5 {
        Color::new(255, 240, 150) // Amarillo brillante
    } else if temp > 0.3 {
        Color::new(255, 180, 80) // Naranja
    } else {
        Color::new(255, 120, 40) // Rojo-naranja
    };
    
    let with_spots = base_color.mul(spots);
    with_spots.mul(1.0 + corona * pulse * 0.8)
}

// ===== SHADER 2: PLANETA TIERRA CON CONTINENTES Y NUBES =====
fn rocky_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.vertex_position * 5.0;
    let time = uniforms.time as f32 * 0.005;
    
    // Generar continentes con patrón Voronoi
    let continents = voronoi_simple(pos * 0.8);
    let mountains = ((pos.x * 10.0).sin() * (pos.y * 10.0).cos() + (pos.z * 10.0).sin() + 1.0) * 0.5;
    
    let terrain_height = continents * 0.7 + mountains * 0.3;
    
    let is_ocean = terrain_height < 0.35;
    let is_land = terrain_height >= 0.35 && terrain_height < 0.55;
    let is_mountain = terrain_height >= 0.55 && terrain_height < 0.65;
    let is_snow = terrain_height >= 0.65;
    
    // Colores base del terreno
    let mut color = if is_ocean {
        let depth = (0.35 - terrain_height) * 5.0;
        if depth > 0.6 {
            Color::new(10, 40, 100) // Océano profundo
        } else {
            Color::new(30, 80, 160) // Océano normal
        }
    } else if is_snow {
        Color::new(250, 250, 255) // Nieve
    } else if is_mountain {
        Color::new(130, 110, 90) // Montañas rocosas
    } else {
        // Variación de vegetación
        let veg = ((pos.x * 15.0).sin() + (pos.y * 15.0).cos() + 1.0) * 0.5;
        if veg > 0.6 {
            Color::new(50, 140, 50) // Bosques densos
        } else if veg > 0.4 {
            Color::new(100, 160, 70) // Pastizales
        } else {
            Color::new(210, 190, 140) // Desiertos
        }
    };
    
    // Sistema de nubes dinámicas
    let cloud1 = ((pos.x * 3.0 + time * 20.0).sin() + (pos.y * 3.0).cos() + (pos.z * 3.0 + time * 15.0).sin() + 1.5) * 0.33;
    let cloud2 = ((pos.x * 6.0 - time * 15.0).cos() + (pos.y * 6.0).sin() + 1.0) * 0.5;
    let clouds = (cloud1 * 0.7 + cloud2 * 0.3).clamp(0.0, 1.0);
    
    if clouds > 0.6 {
        let density = ((clouds - 0.6) / 0.4).min(1.0);
        let cloud_color = Color::new(255, 255, 255);
        color = color.lerp(&cloud_color, density * 0.85);
    }
    
    // Atmósfera azul
    let dist = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt();
    let atmosphere = ((1.0 - dist * 0.2).max(0.0)).powf(5.0);
    if atmosphere > 0.0 {
        let atmo_color = Color::new(100, 150, 255);
        color = color.lerp(&atmo_color, atmosphere * 0.4);
    }
    
    // Iluminación
    let light_dir = Vec3::new(0.8, 0.5, 1.0).normalize();
    let normal = fragment.normal.normalize();
    let diffuse = normal.dot(&light_dir).max(0.15);
    
    color.mul(diffuse)
}

// ===== SHADER 3: JÚPITER CON BANDAS Y GRAN MANCHA ROJA =====
fn gas_giant_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.vertex_position * 3.5;
    let time = uniforms.time as f32 * 0.01;
    
    // Bandas horizontales con turbulencia
    let base_bands = pos.y * 18.0;
    let turb1 = ((pos.x * 5.0 + time * 2.0).sin() + (pos.z * 5.0 + time).cos()) * 0.8;
    let turb2 = ((pos.x * 10.0 - time).cos() + (pos.z * 10.0).sin()) * 0.3;
    
    let band_pos = base_bands + turb1 + turb2;
    let bands = (band_pos.sin() + 1.0) * 0.5;
    
    // Más turbulencia atmosférica
    let atmosphere_chaos = ((pos.x * 8.0 + time * 1.5).sin() * (pos.y * 6.0).cos() + (pos.z * 7.0 - time * 0.8).sin() + 1.5) * 0.33;
    let band_value = (bands * 0.6 + atmosphere_chaos * 0.4).clamp(0.0, 1.0);
    
    // Paleta joviana
    let color1 = Color::new(250, 230, 190); // Beige muy claro
    let color2 = Color::new(170, 120, 80);  // Marrón
    let color3 = Color::new(210, 180, 140); // Beige medio
    let color4 = Color::new(255, 245, 220); // Blanco cremoso
    
    let mut final_color = if band_value < 0.25 {
        color1.lerp(&color2, band_value * 4.0)
    } else if band_value < 0.5 {
        color2.lerp(&color3, (band_value - 0.25) * 4.0)
    } else if band_value < 0.75 {
        color3.lerp(&color4, (band_value - 0.5) * 4.0)
    } else {
        color4.lerp(&color1, (band_value - 0.75) * 4.0)
    };
    
    // GRAN MANCHA ROJA visible y animada
    let spot_center = Vec3::new(0.6, -0.3, 0.0);
    let dx = pos.x - spot_center.x;
    let dy = (pos.y - spot_center.y) * 1.4; // Óvalo alargado
    let dz = pos.z - spot_center.z;
    let dist_to_spot = (dx * dx + dy * dy + dz * dz).sqrt();
    
    if dist_to_spot < 0.5 {
        let spot_factor = (1.0 - dist_to_spot / 0.5).max(0.0);
        
        // Vórtice rotativo en la mancha
        let angle = dy.atan2(dx);
        let swirl = ((angle * 5.0 + dist_to_spot * 15.0 - time * 3.0).sin() + 1.0) * 0.5;
        
        let red_intensity = spot_factor * (0.7 + swirl * 0.3);
        let red_color = if swirl > 0.6 {
            Color::new(240, 100, 70) // Rojo brillante
        } else {
            Color::new(190, 60, 40) // Rojo oscuro
        };
        
        final_color = final_color.lerp(&red_color, red_intensity * 0.95);
    }
    
    // Iluminación
    let light_dir = Vec3::new(1.0, 0.3, 0.8).normalize();
    let normal = fragment.normal.normalize();
    let diffuse = normal.dot(&light_dir).max(0.2);
    
    final_color.mul(diffuse)
}

// ===== SHADER 4: SATURNO CON ANILLOS ESPECTACULARES Y VISIBLES =====
fn ringed_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.vertex_position * 3.0;
    let time = uniforms.time as f32 * 0.006;
    
    // Planeta con bandas suaves
    let bands = ((pos.y * 20.0 + ((pos.x * 3.0).sin() + (pos.z * 3.0).cos()) * 0.5).sin() + 1.0) * 0.5;
    let color1 = Color::new(255, 240, 210);
    let color2 = Color::new(240, 220, 180);
    let mut planet_color = color1.lerp(&color2, bands);
    
    // ANILLOS ULTRA VISIBLES
    let ring_dist = (pos.x * pos.x + pos.z * pos.z).sqrt();
    let y_abs = pos.y.abs();
    
    // Zona de anillos expandida
    if y_abs < 0.18 && ring_dist > 0.75 && ring_dist < 2.0 {
        // Patrones de anillos concéntricos
        let ring_freq = ring_dist * 50.0;
        let ring_bands = (ring_freq.sin() + 1.0) * 0.5;
        
        // Variación de brillo por anillo
        let brightness_var = ((ring_dist * 30.0 + time * 3.0).sin() + 1.0) * 0.5;
        
        // Divisiones de Cassini (gaps oscuros)
        let is_gap = (ring_dist > 1.0 && ring_dist < 1.15) ||
                     (ring_dist > 1.5 && ring_dist < 1.55) ||
                     (ring_dist > 1.75 && ring_dist < 1.78);
        
        if is_gap {
            // Gaps con transparencia (mostrar planeta oscurecido)
            planet_color = planet_color.mul(0.4);
        } else {
            // Anillos visibles con colores variados
            let ring_color = if ring_bands > 0.7 {
                Color::new(245, 225, 190) // Anillos claros
            } else if ring_bands > 0.4 {
                Color::new(210, 185, 145) // Anillos medios
            } else {
                Color::new(180, 160, 125) // Anillos oscuros
            };
            
            // Transparencia basada en distancia al plano
            let ring_alpha = (1.0 - (y_abs / 0.18).powf(1.2)) * 0.95;
            let ring_bright = 0.9 + brightness_var * 0.2;
            
            planet_color = planet_color.lerp(&ring_color, ring_alpha);
            planet_color = planet_color.mul(ring_bright);
        }
    }
    
    // Sombra de anillos sobre el planeta
    if y_abs < 0.2 && ring_dist < 0.9 {
        let shadow_bands = (ring_dist * 50.0).sin() * 0.5 + 0.5;
        let shadow = 0.6 + shadow_bands * 0.3;
        planet_color = planet_color.mul(shadow);
    }
    
    // Iluminación
    let light_dir = Vec3::new(1.0, 0.6, 0.8).normalize();
    let normal = fragment.normal.normalize();
    let diffuse = normal.dot(&light_dir).max(0.25);
    
    planet_color.mul(diffuse)
}

// ===== SHADER 5: PLANETA VOLCÁNICO CON LAVA BRILLANTE =====
fn planet_with_moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.vertex_position * 4.0;
    let time = uniforms.time as f32 * 0.015;
    
    // Superficie con patrón Voronoi para grietas
    let cracks = voronoi_simple(pos * 1.5);
    let fine_cracks = ((pos.x * 20.0 + time).sin() * (pos.y * 20.0).cos() + (pos.z * 20.0 - time).sin() + 1.5) * 0.33;
    
    let is_lava = cracks < 0.4 || fine_cracks > 0.8;
    
    let mut color = if is_lava {
        // Lava con pulsación de temperatura
        let heat_pattern = ((pos.x * 6.0 + time * 3.0).sin() + (pos.y * 6.0).cos() + (pos.z * 6.0 + time * 2.0).sin() + 1.5) * 0.33;
        let pulse = (time * 5.0).sin() * 0.25 + 0.75;
        
        if heat_pattern > 0.75 {
            Color::new(255, 255, 220).mul(pulse) // Lava blanca (ultra caliente)
        } else if heat_pattern > 0.55 {
            Color::new(255, 230, 120).mul(pulse) // Amarilla
        } else if heat_pattern > 0.35 {
            Color::new(255, 150, 50).mul(pulse) // Naranja
        } else {
            Color::new(220, 70, 30).mul(pulse) // Roja
        }
    } else {
        // Roca solidificada oscura
        let rock_var = ((pos.x * 25.0).sin() + (pos.y * 25.0).cos() + 1.0) * 0.5;
        if rock_var > 0.6 {
            Color::new(70, 60, 55) // Gris oscuro
        } else {
            Color::new(35, 30, 25) // Casi negro
        }
    };
    
    // Grietas ultra brillantes
    if fine_cracks > 0.88 {
        let glow_intensity = (fine_cracks - 0.88) / 0.12;
        let crack_glow = Color::new(255, 220, 100);
        color = color.lerp(&crack_glow, glow_intensity);
    }
    
    // Resplandor ambiental
    let ambient_glow = ((pos.x * 3.0 - time * 0.8).sin() + (pos.z * 3.0 + time * 0.5).cos() + 1.0) * 0.15;
    let glow_color = Color::new((ambient_glow * 255.0) as u8, (ambient_glow * 120.0) as u8, 0);
    color = color.add(&glow_color);
    
    // Iluminación con auto-emisión
    let light_dir = Vec3::new(0.8, 0.8, 1.0).normalize();
    let normal = fragment.normal.normalize();
    let diffuse = normal.dot(&light_dir).max(0.35);
    
    color.mul(diffuse * 0.5 + 0.5)
}

// ===== SHADER 6: LUNA CON CRÁTERES =====
fn moon_shader(fragment: &Fragment, _uniforms: &Uniforms) -> Color {
    let pos = fragment.vertex_position * 5.0;
    
    // Cráteres con Voronoi
    let crater_pattern = voronoi_simple(pos * 1.2);
    let is_crater = crater_pattern < 0.25;
    
    // Mares lunares (zonas oscuras)
    let mare_pattern = ((pos.x * 2.0).sin() * (pos.y * 2.0).cos() + (pos.z * 2.0).sin() + 1.0) * 0.5;
    let is_mare = mare_pattern < 0.3;
    
    // Tierras altas
    let highland_pattern = ((pos.x * 4.0).sin() + (pos.y * 4.0).cos() + (pos.z * 4.0).sin() + 1.5) * 0.33;
    let is_highland = highland_pattern > 0.7;
    
    let base_color = if is_crater {
        Color::new(60, 60, 60) // Cráteres oscuros
    } else if is_mare {
        Color::new(80, 80, 80) // Mares lunares
    } else if is_highland {
        Color::new(190, 190, 190) // Tierras altas brillantes
    } else {
        Color::new(140, 140, 140) // Gris base
    };
    
    // Detalle fino de superficie
    let fine_detail = ((pos.x * 30.0).sin() * (pos.y * 30.0).cos() + (pos.z * 30.0).sin() + 1.0) * 0.5;
    let final_color = base_color.mul(0.90 + fine_detail * 0.20);
    
    // Iluminación lunar con sombras duras
    let light_dir = Vec3::new(1.0, 0.3, 0.8).normalize();
    let normal = fragment.normal.normalize();
    let diffuse = normal.dot(&light_dir).max(0.10);
    
    final_color.mul(diffuse)
}
