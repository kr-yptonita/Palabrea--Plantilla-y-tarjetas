# Palabrea & Operaciones Aritméticas

Generadores dinámicos en PDF de materiales didácticos listos para imprimir, escritos íntegramente en **Rust** (edición 2021) utilizando únicamente la biblioteca estándar, sin dependencias externas.

El proyecto incluye dos binarios principales:
- **`operaciones`** (binario por defecto): Generador de fichas didácticas de operaciones aritméticas (multiplicación, división, suma, resta y mixto) con hoja de respuestas incluida.
- **`palabrea`**: Generador de tarjetas de juego léxico (*Palabrea*) y hojas de nube de palabras (*wordcloud*), además de un servidor web integrado con estudio interactivo y descarga de PDFs.

---

## Requisitos

- [Rust y Cargo](https://www.rust-lang.org/tools/install) (edición 2021 o superior).
- No requiere dependencias externas ni herramientas adicionales para generar los PDFs.

---

## Compilación y pruebas

```bash
# Compilar ambos binarios en modo release
cargo build --release

# Ejecutar todas las pruebas unitarias
cargo test
```

---

## 1. Generador de Operaciones Aritméticas (`operaciones`)

Genera fichas de ejercicios aritméticos en PDF (A4 o Letter) con diseño listo para imprimir y una hoja de respuestas al final del documento.

### Uso interactivo

```bash
cargo run --bin operaciones
```

### Uso por línea de comandos (CLI)

```bash
cargo run --bin operaciones -- --op <multiplicacion|division|suma|resta|mixto> [opciones]
```

#### Opciones generales

| Opción | Descripción | Valor por defecto |
| :--- | :--- | :--- |
| `--op <tipo>` | Tipo de operación: `multiplicacion`, `division`, `suma`, `resta` o `mixto` | — |
| `-c, --cantidad <N>` | Cantidad de ejercicios a generar | `12` |
| `-f, --formato <a4\|letter>` | Tamaño de papel del PDF (`a4` o `letter`) | `a4` |
| `-i, --idioma <es\|en>` | Idioma de los títulos (`es` o `en`) | `es` |
| `-o, --salida <ruta.pdf>` | Ruta del archivo PDF de salida | Automático según operación |
| `-s, --semilla <u64>` | Semilla numérica para reproducir exactamente la misma ficha | Aleatoria |

#### Opciones de complejidad por operación

- **Multiplicación:**
  - `--multiplicando <1..5>`: Dígitos del multiplicando (por defecto: `2`).
  - `--multiplicador <1..4>`: Dígitos del multiplicador (por defecto: `2`).
- **División:**
  - `--dividendo <2..5>`: Dígitos del dividendo (por defecto: `3`).
  - `--divisor <1..3>`: Dígitos del divisor (por defecto: `1`).
  - `--notacion <tradicional|galera>`: `tradicional` (caja hispana: `Dividendo | Divisor`) o `galera` (casita: `Divisor ⟌ Dividendo`).
  - `--exacta`: Genera únicamente divisiones exactas (residuo `0`).
- **Suma:**
  - `--sumandos <2..5>`: Cantidad de sumandos por ejercicio (por defecto: `2`).
  - `--digitos <D o D1,D2,...>`: Dígitos por sumando (ej. `3` o `3,2,3`).
- **Resta:**
  - `--minuendo <1..5>`: Dígitos del minuendo (por defecto: `3`).
  - `--sustraendo <1..5>`: Dígitos del sustraendo (por defecto: `2`).

#### Ejemplo

```bash
cargo run --bin operaciones -- --op division --dividendo 4 --divisor 2 --notacion galera --exacta --cantidad 12 --formato letter --salida ficha_divisiones.pdf
```

---

## 2. Palabrea y Plataforma Web (`palabrea`)

Genera tarjetas didácticas (6 por hoja en cuadrícula 3×2) junto con una hoja de nube de palabras, o inicia una aplicación web local para explorar el catálogo y generar materiales de forma visual.

### Modo Servidor Web

```bash
cargo run --bin palabrea -- --web --puerto 8080
```

Una vez iniciado, abre `http://localhost:8080` en tu navegador.

### Uso interactivo por terminal

```bash
cargo run --bin palabrea
```

### Uso por línea de comandos (CLI)

```bash
cargo run --bin palabrea -- --tarjetas <N> --formato <a4|letter> [--salida <archivo.pdf>] [--semilla <N>]
```

| Opción | Descripción | Valor por defecto |
| :--- | :--- | :--- |
| `-w, --web` | Inicia el servidor web y el Comunicador de Herramientas | Desactivado |
| `-p, --puerto <PUERTO>` | Puerto HTTP para la aplicación web | `8080` |
| `-t, --tarjetas <N>` | Cantidad de tarjetas a generar (6 por hoja en 3×2) | — |
| `-f, --formato <a4\|letter>` | Tamaño de las hojas del PDF (`a4` o `letter`) | — |
| `-o, --salida <ruta.pdf>` | Ruta del PDF generado | `palabrea_actividad.pdf` |
| `-s, --semilla <u64>` | Semilla opcional para reproducir la misma actividad | Aleatoria |
| `-h, --ayuda` | Muestra la ayuda en terminal | — |

---

## Estructura del repositorio

```text
.
├── Cargo.toml                  # Manifiesto del paquete y definición de binarios
├── src/
│   ├── main.rs                 # Punto de entrada de `palabrea` (CLI y modo web)
│   ├── bin/operaciones.rs      # Punto de entrada de `operaciones` (CLI e interactivo)
│   ├── lexicon.rs              # Léxico, categorías, pares de sinónimos/antónimos y RNG
│   ├── cards.rs                # Generación de tarjetas de Palabrea
│   ├── wordcloud.rs            # Algoritmo de disposición de nube de palabras sin solapamiento
│   ├── pdf.rs                  # Motor de generación PDF para Palabrea (A4 y Letter)
│   ├── math_gen.rs             # Generador de problemas aritméticos y desglose paso a paso
│   ├── math_layout.rs          # Maquetación de operaciones aritméticas en cuadrícula
│   ├── math_pdf.rs             # Motor de generación PDF para fichas de operaciones
│   ├── communicator.rs         # Registro de herramientas nativas y vistas previas
│   └── server.rs               # Servidor HTTP integrado para la interfaz web
└── web/                        # Interfaz web estática (HTML, CSS y JS)
    ├── index.html
    ├── styles.css
    └── app.js
```
