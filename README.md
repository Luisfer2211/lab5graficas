# Sistema Solar Interactivo

Un sistema solar renderizado con shaders procedurales en WGPU, mostrando diferentes tipos de cuerpos celestes con efectos visuales únicos.

## Ejecución

1. Asegúrate de tener Rust instalado en tu sistema
2. Clona este repositorio
3. Ejecuta el proyecto con:
```bash
cargo run --release
```

## Controles

- **Flechas Izquierda/Derecha**: Rotar el sistema solar horizontalmente
- **Flechas Arriba/Abajo**: Rotar el sistema solar verticalmente
- **ESC**: Salir del programa

## Cuerpos Celestes

El sistema incluye varios tipos de planetas con efectos visuales únicos:
- Sol brillante (centro)
- Planeta volcánico (izquierda arriba)  
- Planeta con anillos (izquierda abajo)
- Luna de hielo (derecha arriba)
- Planeta rocoso (derecha abajo)
- Gigante gaseoso (abajo centro)

Cada planeta tiene sus propios patrones procedurales y efectos de iluminación que los hacen únicos.
