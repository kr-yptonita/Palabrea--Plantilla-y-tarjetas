# Generador de Fichas de Operaciones Aritméticas (`operaciones`)

El binario `operaciones` (`src/bin/operaciones.rs`) genera fichas didácticas en formato PDF listas para imprimir, incluyendo automáticamente una hoja de respuestas al final del documento.

## Modos de ejecución

### 1. Asistente interactivo
Si se ejecuta sin argumentos, inicia un asistente paso a paso por terminal:
```bash
cargo run --bin operaciones
```

### 2. Línea de comandos (CLI)
```bash
cargo run --bin operaciones -- --op <multiplicacion|division|suma|resta|mixto> [opciones]
```

## Opciones generales

| Flag | Alias | Valores | Descripción | Por defecto |
| :--- | :--- | :--- | :--- | :--- |
| `--op` | — | `multiplicacion`, `division`, `suma`, `resta`, `mixto` | Tipo de operación aritmética | — |
| `-c` | `--cantidad` | Entero positivo | Cantidad de ejercicios en la ficha | `12` |
| `-f` | `--formato` | `a4`, `letter` | Tamaño de papel del PDF | `a4` |
| `-i` | `--idioma` | `es`, `en` | Idioma de los títulos de la ficha | `es` |
| `-o` | `--salida` | Ruta de archivo | Ruta del PDF generado | Automático |
| `-s` | `--semilla` | Entero `u64` | Semilla para reproducibilidad exacta | Aleatoria |

## Parámetros de complejidad por operación

### Multiplicación (`multiplicacion`)
- `--multiplicando <1..5>`: Cantidad de dígitos del multiplicando (por defecto: `2`).
- `--multiplicador <1..4>`: Cantidad de dígitos del multiplicador (por defecto: `2`).

### División (`division`)
- `--dividendo <2..5>`: Cantidad de dígitos del dividendo (por defecto: `3`).
- `--divisor <1..3>`: Cantidad de dígitos del divisor (por defecto: `1`).
- `--notacion <tradicional|galera>`:
  - `tradicional`: Notación de caja hispana (`Dividendo | Divisor`).
  - `galera`: Notación de casita anglosajona (`Divisor ⟌ Dividendo`).
- `--exacta`: Fuerza que todas las divisiones generadas tengan residuo `0`.

### Suma (`suma`)
- `--sumandos <2..5>`: Cantidad de sumandos por ejercicio (por defecto: `2`).
- `--digitos <D o D1,D2,...>`: Cantidad de dígitos por sumando. Puede indicarse un único valor (ej. `3`) o una lista separada por comas (ej. `3,2,3`).

### Resta (`resta`)
- `--minuendo <1..5>`: Cantidad de dígitos del minuendo (por defecto: `3`).
- `--sustraendo <1..5>`: Cantidad de dígitos del sustraendo (por defecto: `2`).

## Ejemplos de uso

```bash
# Ficha de 12 divisiones exactas en formato galera (Letter)
cargo run --bin operaciones -- --op division --dividendo 4 --divisor 2 --notacion galera --exacta -c 12 -f letter -o ficha_divisiones_galera_letter.pdf

# Ficha de sumas con 3 sumandos de distinta longitud (A4, inglés)
cargo run --bin operaciones -- --op suma --sumandos 3 --digitos 3,2,3 -i en -f a4 -o ficha_sumas_en.pdf

# Ficha mixta reproducible con semilla fija
cargo run --bin operaciones -- --op mixto -c 16 -s 42 -o ficha_mixta.pdf
```
