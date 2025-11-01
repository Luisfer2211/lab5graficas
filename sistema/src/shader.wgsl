// shader.wgsl - Shaders de planetas procedurales en GPU

struct Uniforms {
    time: f32,
    shader_type: u32,
    resolution: vec2<f32>,
    planet_position: vec2<f32>,
    planet_scale: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Aplicar rotación
    let angle = uniforms.time * 0.3;
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    
    let rot_y = mat3x3<f32>(
        vec3<f32>(cos_a, 0.0, sin_a),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(-sin_a, 0.0, cos_a)
    );
    
    // Escalar y rotar
    let scaled_pos = input.position * uniforms.planet_scale;
    let rotated_pos = rot_y * scaled_pos;
    let rotated_normal = rot_y * input.normal;
    
    // Proyección simple con offset de posición
    let pos = rotated_pos * vec3<f32>(1.0, 1.0, 0.5);
    output.clip_position = vec4<f32>(pos.xy + uniforms.planet_position, 0.5, 1.0);
    output.world_pos = rotated_pos;
    output.normal = normalize(rotated_normal);
    
    return output;
}

// ===== FUNCIONES DE RUIDO =====

fn hash(p: vec3<f32>) -> f32 {
    let h = sin(p.x * 127.1 + p.y * 311.7 + p.z * 74.7) * 43758.5453;
    return fract(h);
}

fn noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let u = f * f * (3.0 - 2.0 * f);
    
    let a = hash(i + vec3<f32>(0.0, 0.0, 0.0));
    let b = hash(i + vec3<f32>(1.0, 0.0, 0.0));
    let c = hash(i + vec3<f32>(0.0, 1.0, 0.0));
    let d = hash(i + vec3<f32>(1.0, 1.0, 0.0));
    
    let x1 = mix(a, b, u.x);
    let x2 = mix(c, d, u.x);
    
    return mix(x1, x2, u.y);
}

fn fbm(p: vec3<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;
    
    for (var i = 0; i < octaves; i++) {
        value += amplitude * noise(pos * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    
    return value;
}

fn voronoi(p: vec3<f32>) -> f32 {
    let pi = floor(p);
    let pf = fract(p);
    
    var min_dist = 2.0;
    
    for (var i = -1; i <= 1; i++) {
        for (var j = -1; j <= 1; j++) {
            let neighbor = vec3<f32>(f32(i), f32(j), 0.0);
            let point = neighbor + vec3<f32>(
                hash(pi + neighbor),
                hash(pi + neighbor + vec3<f32>(0.1, 0.1, 0.1)),
                0.0
            );
            let diff = point - pf;
            let dist = dot(diff.xy, diff.xy);
            min_dist = min(min_dist, dist);
        }
    }
    
    return sqrt(min_dist);
}

// ===== SHADERS DE PLANETAS =====

// SHADER 1: SOL
fn sun_shader(pos: vec3<f32>, time: f32) -> vec3<f32> {
    let p = pos * 3.0;
    
    // Plasma multicapa
    let plasma1 = fbm(p + vec3<f32>(time, time * 0.5, 0.0), 4);
    let plasma2 = fbm(p * 2.0 - vec3<f32>(time * 0.7, time * 1.1, time * 0.3), 3);
    let plasma3 = fbm(p * 0.5 + vec3<f32>(cos(time * 0.5), sin(time * 0.5), time * 0.2), 3);
    
    let combined = clamp((plasma1 * 0.5 + plasma2 * 0.3 + plasma3 * 0.2), 0.0, 1.0);
    
    // Vórtices
    let angle = atan2(p.y, p.x);
    let radius = length(p.xy);
    let swirl = sin(angle * 6.0 + radius * 4.0 - time * 2.0 + combined * 2.0) * 0.5 + 0.5;
    
    // Manchas solares
    let spot_noise = fbm(p * 3.0 + vec3<f32>(time * 0.05, 0.0, 0.0), 3);
    let spots = select(1.0, 0.4, spot_noise > 0.68);
    
    // Corona pulsante
    let dist = length(p);
    let corona = pow(1.0 - dist * 0.4, 3.0) * 1.5;
    let pulse = sin(time * 2.0) * 0.2 + 1.2;
    
    // Gradiente de temperatura
    let temp = combined * swirl * 1.2;
    var base_color: vec3<f32>;
    
    // Nueva paleta de colores para el sol - más azulado/blanco (estilo estrella joven)
    if (temp > 0.8) {
        base_color = vec3<f32>(0.9, 0.95, 1.0); // Núcleo blanco azulado
    } else if (temp > 0.6) {
        base_color = vec3<f32>(0.1, 0.3, 0.8); // Azul profundo
    } else if (temp > 0.4) {
        base_color = vec3<f32>(0.1, 0.3, 0.8); // Azul profundo
    } else {
        base_color = vec3<f32>(0.1, 0.3, 0.8); // Azul profundo
    }
    
    return base_color * spots * (1.0 + corona * pulse * 0.8);
}

// SHADER 2: PLANETA ROCOSO (TIERRA)
fn rocky_planet_shader(pos: vec3<f32>, time: f32) -> vec3<f32> {
    let p = pos * 5.0;
    
    // Continentes con Voronoi
    let continents = voronoi(p * 0.8);
    let mountains = fbm(p * 3.0, 2) * 0.3;
    let terrain_height = continents * 0.7 + mountains * 0.3;
    
    var color: vec3<f32>;
    
    // Nuevos colores para planeta tipo Marte
    if (terrain_height < 0.35) {
        // Planicies bajas
        color = vec3<f32>(0.8, 0.3, 0.1); // Rojo oxidado
    } else if (terrain_height >= 0.65) {
        // Casquetes polares
        color = vec3<f32>(0.95, 0.95, 1.0); // Hielo blanco
    } else if (terrain_height >= 0.55) {
        // Montañas
        color = vec3<f32>(0.6, 0.2, 0.1); // Rojo oscuro
    } else {
        // Terreno variado
        let veg = fbm(p * 5.0, 2);
        if (veg > 0.6) {
            color = vec3<f32>(0.7, 0.3, 0.1); // Rojo claro
        } else if (veg > 0.4) {
            color = vec3<f32>(0.5, 0.2, 0.1); // Rojo medio
        } else {
            color = vec3<f32>(0.4, 0.15, 0.1); // Rojo oscuro
        }
    }

    // Tormentas de polvo
    let cloud1 = fbm(p * 2.0 + vec3<f32>(time * 15.0, 0.0, time * 8.0), 3);
    let cloud2 = fbm(p * 4.0 - vec3<f32>(time * 10.0, 0.0, time * 5.0), 2);
    let clouds = clamp((cloud1 * 0.7 + cloud2 * 0.3), 0.0, 1.0);

    // Aplicar tormentas de polvo
    if (clouds > 0.6) {
        let density = min((clouds - 0.6) / 0.4, 1.0);
        color = mix(color, vec3<f32>(0.8, 0.4, 0.2), density * 0.5);
    }
    
    return color;
}

// SHADER 3: GIGANTE GASEOSO (JÚPITER)
fn gas_giant_shader(pos: vec3<f32>, time: f32) -> vec3<f32> {
    let p = pos * 3.5;
    
    // Bandas horizontales con turbulencia
    let base_bands = p.y * 18.0;
    let turb1 = fbm(p * 2.0 + vec3<f32>(time * 1.5, 0.0, 0.0), 3) * 2.0;
    let turb2 = fbm(p * 4.0 - vec3<f32>(time * 0.8, 0.0, time * 0.5), 2) * 0.8;
    
    let band_pos = base_bands + turb1 + turb2;
    let bands = sin(band_pos) * 0.5 + 0.5;
    
    let atmosphere_chaos = fbm(p * 3.0 + vec3<f32>(time, 0.0, 0.0), 3);
    let band_value = clamp(bands * 0.6 + atmosphere_chaos * 0.4, 0.0, 1.0);
    
    // Paleta joviana
    let color1 = vec3<f32>(0.9, 0.7, 0.5); // Warmer tones
    let color2 = vec3<f32>(0.7, 0.4, 0.2);
    let color3 = vec3<f32>(0.8, 0.6, 0.4);
    let color4 = vec3<f32>(1.0, 0.8, 0.6);
    
    var final_color: vec3<f32>;
    
    if (band_value < 0.25) {
        final_color = mix(color1, color2, band_value * 4.0);
    } else if (band_value < 0.5) {
        final_color = mix(color2, color3, (band_value - 0.25) * 4.0);
    } else if (band_value < 0.75) {
        final_color = mix(color3, color4, (band_value - 0.5) * 4.0);
    } else {
        final_color = mix(color4, color1, (band_value - 0.75) * 4.0);
    }
    
    // Manchas blancas en vez de la mancha roja
    let spot_center = vec3<f32>(0.6, -0.3, 0.0);
    let dx = p.x - spot_center.x;
    let dy = (p.y - spot_center.y) * 1.4;
    let dz = p.z - spot_center.z;
    let dist_to_spot = sqrt(dx * dx + dy * dy + dz * dz);
    
    if (dist_to_spot < 0.5) {
        let spot_factor = max(1.0 - dist_to_spot / 0.5, 0.0);
        let angle = atan2(dy, dx);
        let swirl = sin(angle * 5.0 + dist_to_spot * 15.0 - time * 3.0) * 0.5 + 0.5;
        
        let white_intensity = spot_factor * (0.7 + swirl * 0.3);
        let white_color = select(
            vec3<f32>(0.8, 0.9, 1.0),
            vec3<f32>(1.0, 1.0, 1.0),
            swirl > 0.6
        );
        
        final_color = mix(final_color, white_color, white_intensity * 0.95);
    }
    
    return final_color;
}

// SHADER 4: SATURNO CON ANILLOS
fn ringed_planet_shader(pos: vec3<f32>, time: f32) -> vec3<f32> {
    let p = pos * 3.0;
    
    // Planeta base
    let bands = sin(p.y * 20.0 + fbm(p, 2) * 0.5) * 0.5 + 0.5;
    let color1 = vec3<f32>(0.2, 0.5, 0.3); // Verde oscuro
    let color2 = vec3<f32>(0.3, 0.7, 0.4); // Verde medio
    var planet_color = mix(color1, color2, bands);

    // ANILLOS ESPECTACULARES
    let ring_dist = length(p.xz);
    let y_abs = abs(p.y);
    
    if (y_abs < 0.18 && ring_dist > 0.75 && ring_dist < 2.0) {
        let ring_freq = ring_dist * 50.0;
        let ring_bands = sin(ring_freq) * 0.5 + 0.5;
        let brightness_var = sin(ring_dist * 30.0 + time * 3.0) * 0.5 + 0.5;
        
        // Gaps de Cassini
        let is_gap = (ring_dist > 1.0 && ring_dist < 1.15) ||
                     (ring_dist > 1.5 && ring_dist < 1.55) ||
                     (ring_dist > 1.75 && ring_dist < 1.78);
        
        if (is_gap) {
            planet_color *= 0.4;
        } else {
            var ring_color: vec3<f32>;
            if (ring_bands > 0.7) {
                ring_color = vec3<f32>(0.3, 0.8, 0.5); // Verde brillante
            } else if (ring_bands > 0.4) {
                ring_color = vec3<f32>(0.2, 0.6, 0.4); // Verde medio
            } else {
                ring_color = vec3<f32>(0.1, 0.4, 0.3); // Verde oscuro
            }
            
            let ring_alpha = (1.0 - pow(y_abs / 0.18, 1.2)) * 0.95;
            let ring_bright = 0.9 + brightness_var * 0.2;
            
            planet_color = mix(planet_color, ring_color, ring_alpha);
            planet_color *= ring_bright;
        }
    }
    
    // Sombra de anillos en el planeta
    if (y_abs < 0.2 && ring_dist < 0.9) {
        let shadow_bands = sin(ring_dist * 50.0) * 0.5 + 0.5;
        let shadow = 0.6 + shadow_bands * 0.3;
        planet_color *= shadow;
    }
    
    return planet_color;
}

// SHADER 5: PLANETA VOLCÁNICO
fn volcanic_planet_shader(pos: vec3<f32>, time: f32) -> vec3<f32> {
    let p = pos * 4.0;
    
    // Superficie con Voronoi para grietas
    let cracks = voronoi(p * 1.5);
    let fine_cracks = fbm(p * 8.0 + vec3<f32>(time, 0.0, 0.0), 3);
    
    let is_lava = cracks < 0.4 || fine_cracks > 0.8;
    
    var color: vec3<f32>;
    
    // Nuevos colores para lava y roca
    if (is_lava) {
        let heat = fbm(p * 2.0 + vec3<f32>(time * 2.0, 0.0, time), 3);
        let pulse = sin(time * 5.0) * 0.25 + 0.75;
        
        if (heat > 0.75) {
            color = vec3<f32>(0.0, 1.0, 0.8) * pulse; // Lava azul-verde
        } else if (heat > 0.55) {
            color = vec3<f32>(0.0, 0.8, 0.6) * pulse; // Cian
        } else if (heat > 0.35) {
            color = vec3<f32>(0.0, 0.6, 0.4) * pulse; // Verde azulado
        } else {
            color = vec3<f32>(0.0, 0.4, 0.3) * pulse; // Verde oscuro
        }
    } else {
        let rock_var = fbm(p * 10.0, 2);
        if (rock_var > 0.6) {
            color = vec3<f32>(0.1, 0.2, 0.2); // Roca oscura
        } else {
            color = vec3<f32>(0.05, 0.1, 0.1); // Roca más oscura
        }
    }
    
    return color;
}

// SHADER 6: LUNA
fn moon_shader(pos: vec3<f32>) -> vec3<f32> {
    let p = pos * 5.0;
    
    // Cráteres con Voronoi
    let crater_pattern = voronoi(p * 1.2);
    let is_crater = crater_pattern < 0.25;
    
    // Mares lunares
    let mare_pattern = fbm(p * 0.8, 3);
    let is_mare = mare_pattern < 0.3;
    
    // Tierras altas
    let highland = fbm(p * 2.0, 2);
    let is_highland = highland > 0.7;
    
    var color: vec3<f32>;
    
    if (is_crater) {
        color = vec3<f32>(0.7, 0.8, 0.9); // Hielo agrietado
    } else if (is_mare) {
        color = vec3<f32>(0.5, 0.7, 0.8); // Hielo azulado
    } else if (is_highland) {
        color = vec3<f32>(0.9, 0.95, 1.0); // Hielo brillante
    } else {
        color = vec3<f32>(0.8, 0.9, 0.95); // Hielo base
    }
    
    return color;
}

// Add star shader (case 7)
fn star_shader(pos: vec3<f32>, time: f32) -> vec3<f32> {
    let dist = length(pos);
    let glow = pow(1.0 - dist, 8.0);
    let twinkle = sin(time * 3.0 + pos.x * 10.0 + pos.y * 8.0) * 0.5 + 0.5;
    return vec3<f32>(1.0, 1.0, 1.0) * glow * (0.7 + twinkle * 0.3);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pos = normalize(input.world_pos);
    let normal = normalize(input.normal);
    let time = uniforms.time;
    
    var color: vec3<f32>;
    
    // Seleccionar shader según tipo
    switch uniforms.shader_type {
        case 1u: { color = sun_shader(pos, time); }
        case 2u: { color = rocky_planet_shader(pos, time); }
        case 3u: { color = gas_giant_shader(pos, time); }
        case 4u: { color = ringed_planet_shader(pos, time); }
        case 5u: { color = volcanic_planet_shader(pos, time); }
        case 6u: { color = moon_shader(pos); }
        case 7u: { color = star_shader(pos, time); }
        default: { color = vec3<f32>(1.0, 0.0, 1.0); }
    }
    
    // Iluminación básica
    let light_dir = normalize(vec3<f32>(1.0, 0.5, 0.8));
    let diffuse = max(dot(normal, light_dir), 0.15);
    
    // Auto-emisión para sol y lava
    let emission = select(1.0, diffuse, uniforms.shader_type != 1u && uniforms.shader_type != 5u);
    
    return vec4<f32>(color * mix(1.0, diffuse, 0.7), 1.0);
}
