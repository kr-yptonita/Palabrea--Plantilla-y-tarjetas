# Palabrea y Plataforma Web (`palabrea`)

El binario `palabrea` (`src/main.rs`) permite generar materiales didácticos léxicos en PDF (tarjetas de juego y hoja de nube de palabras) o iniciar una aplicación web local.

## Modos de ejecución

### 1. Modo Servidor Web
Inicia el servidor HTTP integrado (`src/server.rs`) con el catálogo, estudio interactivo, comunicador de herramientas (`src/communicator.rs`) y descarga de PDFs:

```bash
cargo run --bin palabrea -- --web [--puerto 8080]
```

- `-w, --web` (alias: `--servidor`, `--server`): Activa el modo servidor web.
- `-p, --puerto <PUERTO>` (alias: `--port`): Define el puerto HTTP (por defecto: `8080`).

### 2. Modo interactivo por terminal
Ejecutando sin argumentos, solicita los parámetros paso a paso:
```bash
cargo run --bin palabrea
```

### 3. Modo CLI directo
```bash
cargo run --bin palabrea -- --tarjetas <N> --formato <a4|letter> [--salida <archivo.pdf>] [--semilla <N>]
```

| Flag | Alias | Valores | Descripción | Por defecto |
| :--- | :--- | :--- | :--- | :--- |
| `-t` | `--tarjetas`, `--cards` | Entero `> 0` | Cantidad de tarjetas a generar (6 por hoja en cuadrícula 3×2) | — |
| `-f` | `--formato`, `--format`, `--hoja` | `a4`, `letter` | Tamaño de las hojas del PDF | — |
| `-o` | `--salida`, `--output` | Ruta de archivo | Nombre o ruta del PDF generado | `palabrea_actividad.pdf` |
| `-s` | `--semilla`, `--seed` | Entero `u64` | Semilla opcional para reproducir exactamente la misma actividad | Aleatoria |
| `-h` | `--ayuda`, `--help` | — | Muestra la ayuda en terminal | — |

## Ejemplos de uso

```bash
# Generar 12 tarjetas (2 hojas de 3x2) + nube de palabras en formato A4
cargo run --bin palabrea -- -t 12 -f a4 -o palabrea_actividad.pdf

# Reproducir una actividad exacta en tamaño Letter
cargo run --bin palabrea -- -t 6 -f letter -s 12345 -o palabrea_ejemplo_letter.pdf
```
