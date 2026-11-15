# Arquitectura del Sistema

El proyecto `palabrea` está desarrollado en **Rust (edición 2021)** sin dependencias externas (`[dependencies]` vacío en `Cargo.toml`), apoyándose únicamente en la biblioteca estándar de Rust.

## Binarios

El paquete define dos binarios en `Cargo.toml`:

1. **`operaciones`** (`src/bin/operaciones.rs`, binario por defecto):
   Generador de fichas didácticas de operaciones aritméticas en PDF con hoja de respuestas.
2. **`palabrea`** (`src/main.rs`):
   Generador de tarjetas léxicas (*Palabrea*) y hojas de nube de palabras (*wordcloud*) en PDF, además de incluir un servidor HTTP integrado para la interfaz web.

## Módulos principales (`src/`)

### Núcleo compartido y utilidades
- **`lexicon.rs`**: Contiene el generador de números pseudoaleatorios (`Rng`), el léxico de palabras (`WordEntry`), categorías y pares de sinónimos/antónimos.
- **`pdf.rs`**: Motor de generación de documentos PDF desde cero para Palabrea, con soporte para tamaños de página `PaperSize::A4` y `PaperSize::Letter`.

### Módulos de Palabrea y Plataforma Web
- **`cards.rs`**: Lógica de generación de tarjetas de actividad (`generate_cards`), garantizando que todas las tarjetas tengan soluciones válidas en la hoja.
- **`wordcloud.rs`**: Algoritmo de disposición de nube de palabras (`generate_wordcloud_sheet`) que posiciona múltiples palabras sin solapamiento.
- **`communicator.rs`**: Registro de herramientas nativas y generación de vistas previas.
- **`server.rs`**: Servidor HTTP integrado (`run_web_server`) que sirve la interfaz estática de `web/` junto con las rutas de catálogo, vista previa y descarga de PDFs.

### Módulos de Operaciones Aritméticas
- **`math_gen.rs`**: Generación de problemas aritméticos (`generate_worksheet_problems`) y desglose paso a paso para multiplicación, división, suma, resta y modo mixto.
- **`math_layout.rs`**: Maquetación y distribución espacial de las operaciones aritméticas sobre la hoja.
- **`math_pdf.rs`**: Motor de generación de PDF específico para fichas matemáticas (`write_math_worksheet_pdf`), incluyendo la hoja de respuestas al final.

## Interfaz Web (`web/`)

Contiene los recursos estáticos servidos por `server.rs` cuando se ejecuta `palabrea --web`:
- `web/index.html`
- `web/styles.css`
- `web/app.js`

## Pruebas automatizadas

El proyecto cuenta con pruebas unitarias integradas en ambos binarios, ejecutables mediante `cargo test`:

- **`operaciones`** (9 pruebas):
  - `math_gen::tests::test_addition_and_subtraction_generation`
  - `math_gen::tests::test_division_breakdown_both_notations`
  - `math_gen::tests::test_multiplication_breakdown`
  - `lexicon::tests::test_unique_words_and_valid_metadata`
  - `lexicon::tests::test_synonym_and_antonym_pairs_exist`
  - `cards::tests::test_cards_all_have_valid_solutions_on_sheet`
  - `wordcloud::tests::test_wordcloud_places_many_words_without_overlap`
  - `math_pdf::tests::test_pdf_generation_all_operations_a4_and_letter`
  - `pdf::tests::test_pdf_generation_a4_and_letter`
- **`palabrea`** (7 pruebas):
  - Incluye las pruebas de `lexicon`, `cards`, `wordcloud`, `pdf`, más:
  - `server::tests::test_server_routes_catalog_preview_and_pdf`
  - `communicator::tests::test_communicator_registers_native_tools_and_previews`
