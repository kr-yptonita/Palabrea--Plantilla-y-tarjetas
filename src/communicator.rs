use crate::cards::{generate_cards, Card, CardKind};
use crate::lectura_gen::{
    generate_reading_exercises, BlankSelectionMode, ExerciseMode, ReadingWorksheetConfig,
    SentenceTheme, VisualStyle,
};
use crate::lectura_pdf::build_reading_worksheet_pdf_bytes;
use crate::lexicon::{are_antonyms, are_synonyms, Category, Rng, WordEntry, LEXICON};
use crate::math_gen::{
    generate_worksheet_problems, AdditionConfig, DivisionConfig, DivisionNotation, MathProblem,
    MultiplicationConfig, OperationKind, SubtractionConfig, WorksheetConfig, WorksheetLanguage,
};
use crate::math_pdf::build_math_worksheet_pdf_bytes;
use crate::pdf::{build_activity_pdf_bytes, PaperSize};
use crate::wordcloud::{generate_wordcloud_sheet, FontFamily, PlacedWord};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Tipo de parámetro configurable que una herramienta didáctica expone al frontend.
#[derive(Clone, Debug)]
pub enum ParamType {
    Number { min: i64, max: i64, step: i64 },
    Select { options: Vec<(String, String)> },
    Text { placeholder: String },
}

/// Esquema de un parámetro de entrada para construir dinámicamente la UI de cada herramienta.
#[derive(Clone, Debug)]
pub struct ToolParamSchema {
    pub name: String,
    pub label: String,
    pub description: String,
    pub param_type: ParamType,
    pub default_value: String,
}

/// Metadatos descriptivos de una herramienta registrada en el Comunicador.
#[derive(Clone, Debug)]
pub struct ToolManifest {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub description: String,
    pub category: String,
    pub age_range: String,
    pub icon: String,
    pub badge: String,
    pub version: String,
    pub source_kind: String, // "native" | "external-cli"
    pub capabilities: Vec<String>,
    pub params: Vec<ToolParamSchema>,
}

/// Mapa de parámetros recibidos desde la petición HTTP o CLI.
#[derive(Clone, Debug, Default)]
pub struct ToolParams {
    pub values: HashMap<String, String>,
}

impl ToolParams {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn get_usize(&self, key: &str, default: usize) -> usize {
        self.values
            .get(key)
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(default)
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.values
            .get(key)
            .and_then(|v| {
                let t = v.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<u64>().ok()
                }
            })
    }
}

/// Información enriquecida de una tarjeta para la vista previa y hoja del docente.
#[derive(Clone, Debug)]
pub struct PreviewCard {
    pub number: usize,
    pub prompt: String,
    pub kind_id: &'static str,
    pub kind_label: &'static str,
    pub sample_solutions: Vec<String>,
    pub matched_word_indices: Vec<usize>,
}

/// Información enriquecida de una palabra posicionada en la hoja para el renderizador SVG interactivo.
#[derive(Clone, Debug)]
pub struct PreviewPlacedWord {
    pub index: usize,
    pub word: String,
    pub display_text: String,
    pub category_id: &'static str,
    pub category_label: &'static str,
    pub syllables: u8,
    pub letters: usize,
    pub vowels: usize,
    pub consonants: usize,
    pub synonyms_on_sheet: Vec<String>,
    pub antonyms_on_sheet: Vec<String>,
    pub font_family_css: &'static str,
    pub font_weight_css: &'static str,
    pub font_style_css: &'static str,
    pub font_pdf: &'static str,
    pub font_size: f32,
    pub horiz_scale: f32,
    pub outlined: bool,
    pub pdf_x: f32,
    pub pdf_y: f32,
    pub svg_x: f32,
    pub svg_y: f32,
    pub pdf_angle_deg: f32,
    pub svg_angle_deg: f32,
    pub width: f32,
    pub height: f32,
}

/// Estadísticas resumidas de la actividad generada.
#[derive(Clone, Debug)]
pub struct PreviewSummary {
    pub total_cards: usize,
    pub card_pages: usize,
    pub total_pages: usize,
    pub placed_words_count: usize,
    pub antonym_pairs_count: usize,
    pub synonym_pairs_count: usize,
    pub category_counts: Vec<(&'static str, usize)>,
}

/// Salida de vista previa devuelta por una herramienta del Comunicador.
#[derive(Clone, Debug)]
pub struct ToolPreviewOutput {
    pub tool_id: String,
    pub tool_name: String,
    pub seed: u64,
    pub paper_size_id: &'static str,
    pub paper_size_label: &'static str,
    pub page_width_pt: f32,
    pub page_height_pt: f32,
    pub cards: Vec<PreviewCard>,
    pub placed_words: Vec<PreviewPlacedWord>,
    pub summary: PreviewSummary,
}

/// Documento PDF generado por una herramienta del Comunicador.
#[derive(Clone, Debug)]
pub struct GeneratedDocument {
    pub tool_id: String,
    pub filename: String,
    pub seed: u64,
    pub paper_size_label: String,
    pub total_pages: usize,
    pub card_count: usize,
    pub word_count: usize,
    pub pdf_bytes: Vec<u8>,
}

/// Metadatos de un archivo PDF almacenado en la biblioteca local.
#[derive(Clone, Debug)]
pub struct StoredMaterial {
    pub filename: String,
    pub tool_id: String,
    pub size_bytes: u64,
    pub created_epoch_secs: u64,
    pub is_generated_dir: bool,
}

/// Trait universal que toda herramienta generadora de materiales didácticos implementa.
pub trait DidacticTool: Send + Sync {
    fn manifest(&self) -> &ToolManifest;
    fn preview(&self, params: &ToolParams) -> Result<ToolPreviewOutput, String>;
    fn generate_pdf(&self, params: &ToolParams) -> Result<GeneratedDocument, String>;
}

// =============================================================================
// UTILIDADES DE ETIQUETAS Y SERIALIZACIÓN JSON (SIN DEPENDENCIAS EXTERNAS)
// =============================================================================

pub fn json_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

pub fn card_kind_meta(kind: CardKind) -> (&'static str, &'static str) {
    match kind {
        CardKind::CategoryAndSyllables => ("CategoryAndSyllables", "Categoría y Sílabas"),
        CardKind::CategoryAndLetters => ("CategoryAndLetters", "Categoría y Letras"),
        CardKind::SyllablesAndLetters => ("SyllablesAndLetters", "Sílabas y Longitud"),
        CardKind::PhoneticAndOrthographic => ("PhoneticAndOrthographic", "Fonética y Ortografía"),
        CardKind::SynonymsAndAntonyms => ("SynonymsAndAntonyms", "Sinónimos y Antónimos"),
    }
}

pub fn category_meta(cat: Category) -> (&'static str, &'static str) {
    match cat {
        Category::Animal => ("Animal", "Animales"),
        Category::Alimento => ("Alimento", "Alimentos"),
        Category::Naturaleza => ("Naturaleza", "Naturaleza"),
        Category::Transporte => ("Transporte", "Transporte"),
        Category::Objeto => ("Objeto", "Objetos"),
        Category::Cuerpo => ("Cuerpo", "Cuerpo humano"),
        Category::Ropa => ("Ropa", "Ropa y calzado"),
        Category::Color => ("Color", "Colores"),
        Category::Numero => ("Numero", "Números"),
        Category::Adjetivo => ("Adjetivo", "Adjetivos"),
        Category::Emocion => ("Emocion", "Emociones"),
        Category::Instrumento => ("Instrumento", "Instrumentos"),
    }
}

pub fn font_css_meta(font: FontFamily) -> (&'static str, &'static str, &'static str) {
    match font {
        FontFamily::Helvetica => ("'Helvetica Neue', Helvetica, Arial, sans-serif", "400", "normal"),
        FontFamily::HelveticaBold => ("'Helvetica Neue', Helvetica, Arial, sans-serif", "700", "normal"),
        FontFamily::HelveticaOblique => ("'Helvetica Neue', Helvetica, Arial, sans-serif", "400", "italic"),
        FontFamily::HelveticaBoldOblique => ("'Helvetica Neue', Helvetica, Arial, sans-serif", "700", "italic"),
        FontFamily::TimesRoman => ("'Times New Roman', Times, Georgia, serif", "400", "normal"),
        FontFamily::TimesBold => ("'Times New Roman', Times, Georgia, serif", "700", "normal"),
        FontFamily::TimesItalic => ("'Times New Roman', Times, Georgia, serif", "400", "italic"),
        FontFamily::TimesBoldItalic => ("'Times New Roman', Times, Georgia, serif", "700", "italic"),
        FontFamily::CourierBold => ("'Courier New', Courier, monospace", "700", "normal"),
        FontFamily::CourierOblique => ("'Courier New', Courier, monospace", "400", "italic"),
    }
}

fn parse_paper_size_param(val: Option<&str>) -> (PaperSize, &'static str) {
    match val.unwrap_or("a4").trim().to_ascii_lowercase().as_str() {
        "letter" | "carta" | "us-letter" => (PaperSize::Letter, "letter"),
        _ => (PaperSize::A4, "a4"),
    }
}

fn generate_entropy_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42);
    // Mantener dentro de 53 bits seguros para JavaScript (Number.MAX_SAFE_INTEGER)
    (nanos ^ 0x5DEECE66D) & 0x000F_FFFF_FFFF_FFFF
}

fn filter_cards_by_kind(
    placed_entries: &[&'static WordEntry],
    requested_count: usize,
    kind_filter: &str,
    rng: &mut Rng,
) -> Vec<Card> {
    let target_kind = match kind_filter {
        "CategoryAndSyllables" => Some(CardKind::CategoryAndSyllables),
        "CategoryAndLetters" => Some(CardKind::CategoryAndLetters),
        "SyllablesAndLetters" => Some(CardKind::SyllablesAndLetters),
        "PhoneticAndOrthographic" => Some(CardKind::PhoneticAndOrthographic),
        "SynonymsAndAntonyms" => Some(CardKind::SynonymsAndAntonyms),
        _ => None,
    };

    if let Some(kind) = target_kind {
        // Generamos un pool amplio para filtrar por el tipo solicitado
        let pool = generate_cards(placed_entries, requested_count * 6, rng);
        let mut filtered: Vec<Card> = pool.into_iter().filter(|c| c.kind == kind).collect();
        if filtered.len() >= requested_count {
            filtered.truncate(requested_count);
            return filtered;
        }
        // Si hay menos candidatos únicos de ese tipo, completamos con el generador general
        let extra = generate_cards(placed_entries, requested_count - filtered.len(), rng);
        filtered.extend(extra);
        filtered
    } else {
        generate_cards(placed_entries, requested_count, rng)
    }
}

fn build_preview_from_sheet_and_cards(
    tool_id: &str,
    tool_name: &str,
    seed: u64,
    paper_id: &'static str,
    paper_size: PaperSize,
    page_w: f32,
    page_h: f32,
    placed_words: &[PlacedWord],
    cards: &[Card],
) -> ToolPreviewOutput {
    let entries: Vec<&'static WordEntry> = placed_words.iter().map(|pw| pw.entry).collect();

    // Contar pares de sinónimos y antónimos presentes en la hoja
    let mut ant_pairs = 0usize;
    let mut syn_pairs = 0usize;
    for (i, &a) in entries.iter().enumerate() {
        for &b in &entries[i + 1..] {
            if are_antonyms(a, b) {
                ant_pairs += 1;
            }
            if are_synonyms(a, b) {
                syn_pairs += 1;
            }
        }
    }

    // Construir palabras posicionadas enriquecidas
    let mut preview_words: Vec<PreviewPlacedWord> = Vec::with_capacity(placed_words.len());
    for (idx, pw) in placed_words.iter().enumerate() {
        let (cat_id, cat_label) = category_meta(pw.entry.category);
        let (font_family_css, font_weight_css, font_style_css) = font_css_meta(pw.font);

        let mut syns = Vec::new();
        let mut ants = Vec::new();
        for (other_idx, other_pw) in placed_words.iter().enumerate() {
            if idx == other_idx {
                continue;
            }
            if are_synonyms(pw.entry, other_pw.entry) {
                syns.push(other_pw.entry.word.to_string());
            }
            if are_antonyms(pw.entry, other_pw.entry) {
                ants.push(other_pw.entry.word.to_string());
            }
        }

        preview_words.push(PreviewPlacedWord {
            index: idx,
            word: pw.entry.word.to_string(),
            display_text: pw.display_text.clone(),
            category_id: cat_id,
            category_label: cat_label,
            syllables: pw.entry.syllables,
            letters: pw.entry.letters(),
            vowels: pw.entry.vowels(),
            consonants: pw.entry.consonants(),
            synonyms_on_sheet: syns,
            antonyms_on_sheet: ants,
            font_family_css,
            font_weight_css,
            font_style_css,
            font_pdf: pw.font.base_font_ps_name(),
            font_size: pw.font_size,
            horiz_scale: pw.horiz_scale,
            outlined: pw.outlined,
            pdf_x: pw.center_x,
            pdf_y: pw.center_y,
            svg_x: pw.center_x,
            svg_y: page_h - pw.center_y,
            pdf_angle_deg: pw.angle_deg,
            svg_angle_deg: -pw.angle_deg,
            width: pw.width,
            height: pw.height,
        });
    }

    // Construir tarjetas con índices hacia las palabras de la nube
    let mut preview_cards: Vec<PreviewCard> = Vec::with_capacity(cards.len());
    for (i, card) in cards.iter().enumerate() {
        let (kind_id, kind_label) = card_kind_meta(card.kind);
        let mut matched_indices: Vec<usize> = Vec::new();
        for &sol in &card.sample_solutions {
            if let Some(pos) = placed_words.iter().position(|pw| pw.entry.word == sol) {
                if !matched_indices.contains(&pos) {
                    matched_indices.push(pos);
                }
            }
        }

        preview_cards.push(PreviewCard {
            number: i + 1,
            prompt: card.prompt.clone(),
            kind_id,
            kind_label,
            sample_solutions: card.sample_solutions.iter().map(|s| s.to_string()).collect(),
            matched_word_indices: matched_indices,
        });
    }

    // Distribución por categorías en la hoja
    let mut category_counts: Vec<(&'static str, usize)> = Vec::new();
    for &cat in Category::ALL {
        let (_, label) = category_meta(cat);
        let count = entries.iter().filter(|e| e.category == cat).count();
        if count > 0 {
            category_counts.push((label, count));
        }
    }

    let card_pages = (cards.len() + 5) / 6;
    let total_pages = card_pages + 1;

    ToolPreviewOutput {
        tool_id: tool_id.to_string(),
        tool_name: tool_name.to_string(),
        seed,
        paper_size_id: paper_id,
        paper_size_label: paper_size.label(),
        page_width_pt: page_w,
        page_height_pt: page_h,
        cards: preview_cards,
        placed_words: preview_words,
        summary: PreviewSummary {
            total_cards: cards.len(),
            card_pages,
            total_pages,
            placed_words_count: placed_words.len(),
            antonym_pairs_count: ant_pairs,
            synonym_pairs_count: syn_pairs,
            category_counts,
        },
    }
}

// =============================================================================
// 1. HERRAMIENTA NATIVA: PALABREA (PLANTILLA Y TARJETAS)
// =============================================================================

pub struct PalabreaTool {
    manifest: ToolManifest,
}

impl PalabreaTool {
    pub fn new() -> Self {
        let params = vec![
            ToolParamSchema {
                name: "cards".to_string(),
                label: "Cantidad de tarjetas".to_string(),
                description: "Número total de tarjetas de consignas (6 tarjetas por hoja en cuadrícula 3×2).".to_string(),
                param_type: ParamType::Number {
                    min: 6,
                    max: 60,
                    step: 6,
                },
                default_value: "12".to_string(),
            },
            ToolParamSchema {
                name: "format".to_string(),
                label: "Formato de hoja".to_string(),
                description: "Tamaño de página para impresión del PDF.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("a4".to_string(), "A4 (210 × 297 mm)".to_string()),
                        ("letter".to_string(), "Letter / Carta (215.9 × 279.4 mm)".to_string()),
                    ],
                },
                default_value: "a4".to_string(),
            },
            ToolParamSchema {
                name: "kind_filter".to_string(),
                label: "Enfoque pedagógico de consignas".to_string(),
                description: "Combina todos los retos o prioriza una habilidad lingüística específica.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("all".to_string(), "Balanceado (Las 5 categorías de retos)".to_string()),
                        ("CategoryAndSyllables".to_string(), "Categoría semántica + Sílabas".to_string()),
                        ("CategoryAndLetters".to_string(), "Categoría semántica + Nº de letras".to_string()),
                        ("SyllablesAndLetters".to_string(), "Conciencia silábica y longitud".to_string()),
                        ("PhoneticAndOrthographic".to_string(), "Fonética, vocales e iniciales".to_string()),
                        ("SynonymsAndAntonyms".to_string(), "Sinónimos y Antónimos".to_string()),
                    ],
                },
                default_value: "all".to_string(),
            },
            ToolParamSchema {
                name: "seed".to_string(),
                label: "Semilla de generación (Seed)".to_string(),
                description: "Permite reproducir exactamente la misma hoja y tarjetas o generar variaciones nuevas.".to_string(),
                param_type: ParamType::Text {
                    placeholder: "Automática (clic en dado para variar)".to_string(),
                },
                default_value: "".to_string(),
            },
            ToolParamSchema {
                name: "filename".to_string(),
                label: "Nombre del archivo PDF".to_string(),
                description: "Nombre con el que se guardará en la biblioteca y descargará.".to_string(),
                param_type: ParamType::Text {
                    placeholder: "palabrea_actividad.pdf".to_string(),
                },
                default_value: "palabrea_actividad.pdf".to_string(),
            },
        ];

        Self {
            manifest: ToolManifest {
                id: "palabrea".to_string(),
                name: "Palabrea: Plantilla y Tarjetas".to_string(),
                subtitle: "Nube de palabras para encerrar + tarjetas de retos lingüísticos (3×2)".to_string(),
                description: "Genera dinámicamente una hoja apaisada repleta de palabras sin colisiones (con pares garantizados de sinónimos, antónimos y 12 categorías semánticas) junto con tarjetas verificadas listas para recortar y jugar en el aula.".to_string(),
                category: "Lengua y Vocabulario".to_string(),
                age_range: "Primaria / Secundaria (6–14 años)".to_string(),
                icon: "sparkles".to_string(),
                badge: "Motor Nativo Rust".to_string(),
                version: "1.0.0".to_string(),
                source_kind: "native".to_string(),
                capabilities: vec![
                    "live_preview".to_string(),
                    "wordcloud_svg".to_string(),
                    "teacher_key".to_string(),
                    "pdf_export".to_string(),
                ],
                params,
            },
        }
    }
}

impl DidacticTool for PalabreaTool {
    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn preview(&self, params: &ToolParams) -> Result<ToolPreviewOutput, String> {
        let card_count = params.get_usize("cards", 12).clamp(1, 120);
        let (paper_size, paper_id) = parse_paper_size_param(params.get_str("format"));
        let kind_filter = params.get_str("kind_filter").unwrap_or("all");
        let seed = params.get_u64("seed").unwrap_or_else(generate_entropy_seed);

        let mut rng = Rng::from_seed(seed);
        let (page_w, page_h) = paper_size.landscape_dimensions_pt();
        let sheet = generate_wordcloud_sheet(page_w, page_h, &mut rng);

        let placed_entries: Vec<&'static WordEntry> =
            sheet.placed_words.iter().map(|pw| pw.entry).collect();
        let cards = filter_cards_by_kind(&placed_entries, card_count, kind_filter, &mut rng);

        Ok(build_preview_from_sheet_and_cards(
            &self.manifest.id,
            &self.manifest.name,
            seed,
            paper_id,
            paper_size,
            page_w,
            page_h,
            &sheet.placed_words,
            &cards,
        ))
    }

    fn generate_pdf(&self, params: &ToolParams) -> Result<GeneratedDocument, String> {
        let card_count = params.get_usize("cards", 12).clamp(1, 120);
        let (paper_size, paper_id) = parse_paper_size_param(params.get_str("format"));
        let kind_filter = params.get_str("kind_filter").unwrap_or("all");
        let seed = params.get_u64("seed").unwrap_or_else(generate_entropy_seed);

        let mut rng = Rng::from_seed(seed);
        let (page_w, page_h) = paper_size.landscape_dimensions_pt();
        let sheet = generate_wordcloud_sheet(page_w, page_h, &mut rng);

        let placed_entries: Vec<&'static WordEntry> =
            sheet.placed_words.iter().map(|pw| pw.entry).collect();
        let cards = filter_cards_by_kind(&placed_entries, card_count, kind_filter, &mut rng);

        let (pdf_bytes, total_pages) =
            build_activity_pdf_bytes(paper_size, &cards, &sheet).map_err(|e| e.to_string())?;

        let raw_name = params
            .get_str("filename")
            .unwrap_or("")
            .trim();
        let default_name = format!(
            "palabrea_{}t_{}_{}.pdf",
            cards.len(),
            paper_id,
            seed % 100_000
        );
        let filename = sanitize_pdf_filename(if raw_name.is_empty() {
            &default_name
        } else {
            raw_name
        });

        Ok(GeneratedDocument {
            tool_id: self.manifest.id.clone(),
            filename,
            seed,
            paper_size_label: paper_size.label().to_string(),
            total_pages,
            card_count: cards.len(),
            word_count: sheet.placed_words.len(),
            pdf_bytes,
        })
    }
}

// =============================================================================
// 2. HERRAMIENTA NATIVA: EXPEDIENTE DE DESAFÍOS DE SINÓNIMOS Y FONÉTICA
// =============================================================================

pub struct VocabChallengeTool {
    manifest: ToolManifest,
}

impl VocabChallengeTool {
    pub fn new() -> Self {
        let params = vec![
            ToolParamSchema {
                name: "cards".to_string(),
                label: "Cantidad de tarjetas de desafío".to_string(),
                description: "Tarjetas intensivas centradas en relaciones léxicas o conciencia fonológica.".to_string(),
                param_type: ParamType::Number {
                    min: 6,
                    max: 48,
                    step: 6,
                },
                default_value: "18".to_string(),
            },
            ToolParamSchema {
                name: "format".to_string(),
                label: "Formato de hoja".to_string(),
                description: "Tamaño de página para impresión del PDF.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("a4".to_string(), "A4 (210 × 297 mm)".to_string()),
                        ("letter".to_string(), "Letter / Carta (215.9 × 279.4 mm)".to_string()),
                    ],
                },
                default_value: "a4".to_string(),
            },
            ToolParamSchema {
                name: "kind_filter".to_string(),
                label: "Modo de taller".to_string(),
                description: "Selecciona el tipo de taller especializado para la sesión.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("SynonymsAndAntonyms".to_string(), "Taller de Sinónimos y Antónimos".to_string()),
                        ("PhoneticAndOrthographic".to_string(), "Taller de Conciencia Fonológica y Vocales".to_string()),
                        ("SyllablesAndLetters".to_string(), "Taller de Conteo Silábico y Longitud".to_string()),
                    ],
                },
                default_value: "SynonymsAndAntonyms".to_string(),
            },
            ToolParamSchema {
                name: "seed".to_string(),
                label: "Semilla de generación (Seed)".to_string(),
                description: "Semilla numérica para conservar el mismo set de taller.".to_string(),
                param_type: ParamType::Text {
                    placeholder: "Automática".to_string(),
                },
                default_value: "".to_string(),
            },
            ToolParamSchema {
                name: "filename".to_string(),
                label: "Nombre del archivo PDF".to_string(),
                description: "Nombre del archivo exportado.".to_string(),
                param_type: ParamType::Text {
                    placeholder: "taller_lexico.pdf".to_string(),
                },
                default_value: "taller_lexico.pdf".to_string(),
            },
        ];

        Self {
            manifest: ToolManifest {
                id: "taller-lexico".to_string(),
                name: "Taller Intensivo: Sinónimos, Antónimos y Fonología".to_string(),
                subtitle: "Batería temática focalizada en semántica y estructura ortográfica".to_string(),
                description: "Variante especializada del motor léxico que prioriza parejas de sinónimos, opuestos semánticos y patrones vocálicos/consonánticos para sesiones de refuerzo específico.".to_string(),
                category: "Semántica y Ortografía".to_string(),
                age_range: "Primaria Alta / Secundaria (8–15 años)".to_string(),
                icon: "book-open".to_string(),
                badge: "Motor Nativo Rust".to_string(),
                version: "1.0.0".to_string(),
                source_kind: "native".to_string(),
                capabilities: vec![
                    "live_preview".to_string(),
                    "wordcloud_svg".to_string(),
                    "teacher_key".to_string(),
                    "pdf_export".to_string(),
                ],
                params,
            },
        }
    }
}

impl DidacticTool for VocabChallengeTool {
    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn preview(&self, params: &ToolParams) -> Result<ToolPreviewOutput, String> {
        let card_count = params.get_usize("cards", 18).clamp(1, 120);
        let (paper_size, paper_id) = parse_paper_size_param(params.get_str("format"));
        let kind_filter = params
            .get_str("kind_filter")
            .unwrap_or("SynonymsAndAntonyms");
        let seed = params.get_u64("seed").unwrap_or_else(generate_entropy_seed);

        let mut rng = Rng::from_seed(seed);
        let (page_w, page_h) = paper_size.landscape_dimensions_pt();
        let sheet = generate_wordcloud_sheet(page_w, page_h, &mut rng);

        let placed_entries: Vec<&'static WordEntry> =
            sheet.placed_words.iter().map(|pw| pw.entry).collect();
        let cards = filter_cards_by_kind(&placed_entries, card_count, kind_filter, &mut rng);

        Ok(build_preview_from_sheet_and_cards(
            &self.manifest.id,
            &self.manifest.name,
            seed,
            paper_id,
            paper_size,
            page_w,
            page_h,
            &sheet.placed_words,
            &cards,
        ))
    }

    fn generate_pdf(&self, params: &ToolParams) -> Result<GeneratedDocument, String> {
        let card_count = params.get_usize("cards", 18).clamp(1, 120);
        let (paper_size, paper_id) = parse_paper_size_param(params.get_str("format"));
        let kind_filter = params
            .get_str("kind_filter")
            .unwrap_or("SynonymsAndAntonyms");
        let seed = params.get_u64("seed").unwrap_or_else(generate_entropy_seed);

        let mut rng = Rng::from_seed(seed);
        let (page_w, page_h) = paper_size.landscape_dimensions_pt();
        let sheet = generate_wordcloud_sheet(page_w, page_h, &mut rng);

        let placed_entries: Vec<&'static WordEntry> =
            sheet.placed_words.iter().map(|pw| pw.entry).collect();
        let cards = filter_cards_by_kind(&placed_entries, card_count, kind_filter, &mut rng);

        let (pdf_bytes, total_pages) =
            build_activity_pdf_bytes(paper_size, &cards, &sheet).map_err(|e| e.to_string())?;

        let raw_name = params.get_str("filename").unwrap_or("").trim();
        let default_name = format!(
            "taller_lexico_{}t_{}_{}.pdf",
            cards.len(),
            paper_id,
            seed % 100_000
        );
        let filename = sanitize_pdf_filename(if raw_name.is_empty() {
            &default_name
        } else {
            raw_name
        });

        Ok(GeneratedDocument {
            tool_id: self.manifest.id.clone(),
            filename,
            seed,
            paper_size_label: paper_size.label().to_string(),
            total_pages,
            card_count: cards.len(),
            word_count: sheet.placed_words.len(),
            pdf_bytes,
        })
    }
}

// =============================================================================
// 3. HERRAMIENTA NATIVA: OPERACIONES ARITMÉTICAS CON CASILLAS PASO A PASO
// =============================================================================

pub struct OperacionesMathTool {
    manifest: ToolManifest,
}

impl OperacionesMathTool {
    pub fn new() -> Self {
        let params = vec![
            ToolParamSchema {
                name: "operation".to_string(),
                label: "Tipo de operación".to_string(),
                description: "Operación aritmética principal de la lámina.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("multiplicacion".to_string(), "Multiplicación paso a paso".to_string()),
                        ("division".to_string(), "División paso a paso".to_string()),
                        ("suma".to_string(), "Suma con casillas".to_string()),
                        ("resta".to_string(), "Resta con casillas".to_string()),
                        ("mixto".to_string(), "Mixto (Combinación de las 4)".to_string()),
                    ],
                },
                default_value: "multiplicacion".to_string(),
            },
            ToolParamSchema {
                name: "cards".to_string(),
                label: "Cantidad de ejercicios".to_string(),
                description: "Número total de tarjetas de operaciones a generar.".to_string(),
                param_type: ParamType::Number {
                    min: 4,
                    max: 48,
                    step: 4,
                },
                default_value: "12".to_string(),
            },
            ToolParamSchema {
                name: "digits_a".to_string(),
                label: "Cifras del 1er número (Multiplicando / Dividendo / Minuendo)".to_string(),
                description: "Cantidad de dígitos del operando superior o dividendo (1 a 5).".to_string(),
                param_type: ParamType::Number {
                    min: 1,
                    max: 5,
                    step: 1,
                },
                default_value: "3".to_string(),
            },
            ToolParamSchema {
                name: "digits_b".to_string(),
                label: "Cifras del 2º número (Multiplicador / Divisor / Sustraendo)".to_string(),
                description: "Cantidad de dígitos del operando inferior o divisor (1 a 3).".to_string(),
                param_type: ParamType::Number {
                    min: 1,
                    max: 3,
                    step: 1,
                },
                default_value: "2".to_string(),
            },
            ToolParamSchema {
                name: "notation".to_string(),
                label: "Notación de la división".to_string(),
                description: "Estilo de representación para las divisiones.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("tradicional".to_string(), "Caja tradicional (Dividendo | Divisor)".to_string()),
                        ("galera".to_string(), "Galera / Casita (Divisor ⟌ Dividendo)".to_string()),
                    ],
                },
                default_value: "tradicional".to_string(),
            },
            ToolParamSchema {
                name: "lang".to_string(),
                label: "Idioma del encabezado".to_string(),
                description: "Idioma para los títulos y campos del alumno en la hoja.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("es".to_string(), "Español".to_string()),
                        ("en".to_string(), "Inglés (English)".to_string()),
                    ],
                },
                default_value: "es".to_string(),
            },
            ToolParamSchema {
                name: "format".to_string(),
                label: "Formato de hoja".to_string(),
                description: "Tamaño de página del documento PDF.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("a4".to_string(), "A4 (210 × 297 mm)".to_string()),
                        ("letter".to_string(), "Letter / Carta (215.9 × 279.4 mm)".to_string()),
                    ],
                },
                default_value: "a4".to_string(),
            },
            ToolParamSchema {
                name: "seed".to_string(),
                label: "Semilla de generación (Seed)".to_string(),
                description: "Semilla numérica para reproducir exactamente los mismos ejercicios.".to_string(),
                param_type: ParamType::Text {
                    placeholder: "Automática".to_string(),
                },
                default_value: "".to_string(),
            },
            ToolParamSchema {
                name: "filename".to_string(),
                label: "Nombre del archivo PDF".to_string(),
                description: "Nombre con el que se guardará y descargará la ficha.".to_string(),
                param_type: ParamType::Text {
                    placeholder: "operaciones_ficha.pdf".to_string(),
                },
                default_value: "operaciones_ficha.pdf".to_string(),
            },
        ];

        Self {
            manifest: ToolManifest {
                id: "operaciones".to_string(),
                name: "Operaciones con Casillas: Multiplicación, División, Suma y Resta".to_string(),
                subtitle: "Láminas a color con cuadrícula posicional paso a paso + Hoja de Respuestas".to_string(),
                description: "Generador automático de fichas de cálculo vertical con casillas guía para productos parciales, restos de división (en notación de caja hispana o galera/casita), sumas y restas, incluyendo solucionario completo.".to_string(),
                category: "Matemáticas y Cálculo".to_string(),
                age_range: "Primaria (6–12 años)".to_string(),
                icon: "calculator".to_string(),
                badge: "Motor Nativo Rust".to_string(),
                version: "1.0.0".to_string(),
                source_kind: "native".to_string(),
                capabilities: vec![
                    "live_preview".to_string(),
                    "teacher_key".to_string(),
                    "pdf_export".to_string(),
                ],
                params,
            },
        }
    }

    fn build_config(&self, params: &ToolParams) -> (WorksheetConfig, &'static str, u64) {
        let op = match params
            .get_str("operation")
            .unwrap_or("multiplicacion")
            .to_ascii_lowercase()
            .as_str()
        {
            "division" | "div" => OperationKind::Division,
            "suma" | "sum" => OperationKind::Suma,
            "resta" | "res" => OperationKind::Resta,
            "mixto" | "mix" => OperationKind::Mixto,
            _ => OperationKind::Multiplicacion,
        };

        let problem_count = params.get_usize("cards", 12).clamp(1, 60);
        let digits_a = params.get_usize("digits_a", 3).clamp(1, 5);
        let digits_b = params.get_usize("digits_b", 2).clamp(1, 4);

        let notation = match params
            .get_str("notation")
            .unwrap_or("tradicional")
            .to_ascii_lowercase()
            .as_str()
        {
            "galera" | "casita" => DivisionNotation::Galera,
            _ => DivisionNotation::Tradicional,
        };

        let language = match params
            .get_str("lang")
            .unwrap_or("es")
            .to_ascii_lowercase()
            .as_str()
        {
            "en" | "english" => WorksheetLanguage::En,
            _ => WorksheetLanguage::Es,
        };

        let (paper_size, paper_id) = parse_paper_size_param(params.get_str("format"));
        let seed = params.get_u64("seed").unwrap_or_else(generate_entropy_seed);

        let div_dividend = digits_a.clamp(2, 5);
        let div_divisor = digits_b.clamp(1, 3).min(div_dividend - 1).max(1);

        let cfg = WorksheetConfig {
            operation: op,
            multiplication: MultiplicationConfig {
                multiplicand_digits: digits_a,
                multiplier_digits: digits_b.clamp(1, 4),
            },
            division: DivisionConfig {
                dividend_digits: div_dividend,
                divisor_digits: div_divisor,
                notation,
                exact_only: false,
            },
            addition: AdditionConfig {
                addend_count: 2,
                digits_per_addend: vec![digits_a, digits_b.clamp(1, 5)],
            },
            subtraction: SubtractionConfig {
                minuend_digits: digits_a,
                subtrahend_digits: digits_b.clamp(1, digits_a),
            },
            problem_count,
            paper_size,
            language,
        };

        (cfg, paper_id, seed)
    }
}

impl DidacticTool for OperacionesMathTool {
    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn preview(&self, params: &ToolParams) -> Result<ToolPreviewOutput, String> {
        let (cfg, paper_id, seed) = self.build_config(params);
        let mut rng = Rng::from_seed(seed);
        let problems = generate_worksheet_problems(&cfg, &mut rng);
        let (page_w, page_h) = cfg.paper_size.portrait_dimensions_pt();

        let mut cards = Vec::with_capacity(problems.len());
        for (i, prob) in problems.iter().enumerate() {
            let (kind_id, kind_label, prompt, solutions) = match prob {
                MathProblem::Multiplicacion(m) => {
                    let mut sols = vec![format!("Producto total: {}", m.product)];
                    for (idx, step) in m.partial_products.iter().enumerate() {
                        sols.push(format!(
                            "Fila {} (×{}): {}",
                            idx + 1,
                            step.multiplier_digit,
                            step.value
                        ));
                    }
                    (
                        "CategoryAndSyllables",
                        "Multiplicación",
                        format!("Resuelve paso a paso:  {} × {}", m.multiplicand, m.multiplier),
                        sols,
                    )
                }
                MathProblem::Division(d) => {
                    let mut sols = vec![
                        format!("Cociente: {}", d.quotient),
                        format!("Residuo final: {}", d.remainder),
                    ];
                    let steps_str: Vec<String> = d
                        .steps
                        .iter()
                        .map(|s| {
                            s.digits
                                .iter()
                                .map(|dg| dg.to_string())
                                .collect::<Vec<_>>()
                                .join("")
                        })
                        .collect();
                    if !steps_str.is_empty() {
                        sols.push(format!("Bajadas/Restos: {}", steps_str.join(" → ")));
                    }
                    (
                        "CategoryAndLetters",
                        "División",
                        format!("Resuelve en casillas:  {} ÷ {}", d.dividend, d.divisor),
                        sols,
                    )
                }
                MathProblem::Suma(a) => {
                    let terms: Vec<String> = a.addends.iter().map(|(v, _)| v.to_string()).collect();
                    (
                        "SyllablesAndLetters",
                        "Suma",
                        format!("Calcula la suma:  {}", terms.join(" + ")),
                        vec![format!("Total: {}", a.total)],
                    )
                }
                MathProblem::Resta(s) => (
                    "SynonymsAndAntonyms",
                    "Resta",
                    format!("Calcula la diferencia:  {} − {}", s.minuend, s.subtrahend),
                    vec![format!("Diferencia: {}", s.difference)],
                ),
            };

            cards.push(PreviewCard {
                number: i + 1,
                prompt,
                kind_id,
                kind_label,
                sample_solutions: solutions,
                matched_word_indices: Vec::new(),
            });
        }

        let card_pages = (problems.len() + 11) / 12;
        let total_pages = card_pages * 2;

        Ok(ToolPreviewOutput {
            tool_id: self.manifest.id.clone(),
            tool_name: self.manifest.name.clone(),
            seed,
            paper_size_id: paper_id,
            paper_size_label: cfg.paper_size.label(),
            page_width_pt: page_w,
            page_height_pt: page_h,
            cards,
            placed_words: Vec::new(),
            summary: PreviewSummary {
                total_cards: problems.len(),
                card_pages,
                total_pages,
                placed_words_count: problems.len(),
                antonym_pairs_count: card_pages,
                synonym_pairs_count: card_pages,
                category_counts: vec![(cfg.operation.title_es(), problems.len())],
            },
        })
    }

    fn generate_pdf(&self, params: &ToolParams) -> Result<GeneratedDocument, String> {
        let (cfg, paper_id, seed) = self.build_config(params);
        let mut rng = Rng::from_seed(seed);
        let problems = generate_worksheet_problems(&cfg, &mut rng);

        let (pdf_bytes, total_pages, _, _) =
            build_math_worksheet_pdf_bytes(&cfg, &problems).map_err(|e| e.to_string())?;

        let raw_name = params.get_str("filename").unwrap_or("").trim();
        let default_name = format!(
            "operaciones_{}ej_{}_{}.pdf",
            problems.len(),
            paper_id,
            seed % 100_000
        );
        let filename = sanitize_pdf_filename(if raw_name.is_empty() {
            &default_name
        } else {
            raw_name
        });

        Ok(GeneratedDocument {
            tool_id: self.manifest.id.clone(),
            filename,
            seed,
            paper_size_label: cfg.paper_size.label().to_string(),
            total_pages,
            card_count: problems.len(),
            word_count: problems.len(),
            pdf_bytes,
        })
    }
}

// =============================================================================
// 3b. HERRAMIENTA NATIVA: LECTURA Y ESCRITURA (ORACIONES INCOMPLETAS SIN ADORNOS)
// =============================================================================

pub struct LecturaEscrituraTool {
    manifest: ToolManifest,
}

impl LecturaEscrituraTool {
    pub fn new() -> Self {
        let params = vec![
            ToolParamSchema {
                name: "cards".to_string(),
                label: "Cantidad de oraciones".to_string(),
                description: "Número total de oraciones incompletas a generar (7 por página por defecto).".to_string(),
                param_type: ParamType::Number {
                    min: 3,
                    max: 42,
                    step: 1,
                },
                default_value: "7".to_string(),
            },
            ToolParamSchema {
                name: "theme".to_string(),
                label: "Tema del banco de frases".to_string(),
                description: "Filtra las oraciones del banco interno por categoría temática.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("todos".to_string(), "Todos los temas (Mixto)".to_string()),
                        ("animales".to_string(), "Animales y seres vivos".to_string()),
                        ("naturaleza".to_string(), "Naturaleza y entorno".to_string()),
                        ("cotidiano".to_string(), "Vida cotidiana, familia y juegos".to_string()),
                        ("escuela".to_string(), "Escuela, lectura y ciencia".to_string()),
                    ],
                },
                default_value: "todos".to_string(),
            },
            ToolParamSchema {
                name: "blank_mode".to_string(),
                label: "Extracción aleatoria de palabra".to_string(),
                description: "Define qué palabra de cada oración se oculta al azar para generar materiales únicos.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("clave".to_string(), "Al azar entre palabras clave de la frase".to_string()),
                        ("azar".to_string(), "Al azar cualquier palabra de contenido".to_string()),
                        ("final".to_string(), "Última palabra clave de la frase".to_string()),
                    ],
                },
                default_value: "clave".to_string(),
            },
            ToolParamSchema {
                name: "exercise_mode".to_string(),
                label: "Modalidad de respuesta".to_string(),
                description: "Presentación con incisos a), b), c), banco de palabras o línea libre.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("opciones".to_string(), "Opción múltiple a), b), c) + línea".to_string()),
                        ("banco".to_string(), "Recuadro superior con banco de palabras".to_string()),
                        ("libre".to_string(), "Completar libremente en la línea".to_string()),
                    ],
                },
                default_value: "opciones".to_string(),
            },
            ToolParamSchema {
                name: "format".to_string(),
                label: "Formato de hoja".to_string(),
                description: "Tamaño de página del documento PDF.".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("a4".to_string(), "A4 (210 × 297 mm)".to_string()),
                        ("letter".to_string(), "Letter / Carta (215.9 × 279.4 mm)".to_string()),
                    ],
                },
                default_value: "a4".to_string(),
            },
            ToolParamSchema {
                name: "seed".to_string(),
                label: "Semilla de generación (Seed)".to_string(),
                description: "Semilla numérica para reproducir o variar las frases y palabras ocultas.".to_string(),
                param_type: ParamType::Text {
                    placeholder: "Automática".to_string(),
                },
                default_value: "".to_string(),
            },
            ToolParamSchema {
                name: "filename".to_string(),
                label: "Nombre del archivo PDF".to_string(),
                description: "Nombre con el que se guardará y descargará la ficha.".to_string(),
                param_type: ParamType::Text {
                    placeholder: "ficha_lectura_escritura.pdf".to_string(),
                },
                default_value: "ficha_lectura_escritura.pdf".to_string(),
            },
        ];

        Self {
            manifest: ToolManifest {
                id: "lectura-escritura".to_string(),
                name: "Lectura y Escritura: Oraciones Incompletas".to_string(),
                subtitle: "Fichas limpias sin adornos con extracción aleatoria de palabras + Hoja de Respuestas".to_string(),
                description: "Genera fichas únicas de lectura y escritura a partir de un banco de datos de oraciones, quitando palabras al azar en cada frase con incisos a), b), c) mezclados o banco de palabras.".to_string(),
                category: "Lectura y Escritura".to_string(),
                age_range: "Primaria (6–11 años)".to_string(),
                icon: "book".to_string(),
                badge: "Motor Nativo Rust".to_string(),
                version: "1.0.0".to_string(),
                source_kind: "native".to_string(),
                capabilities: vec![
                    "live_preview".to_string(),
                    "teacher_key".to_string(),
                    "pdf_export".to_string(),
                ],
                params,
            },
        }
    }

    fn build_config(&self, params: &ToolParams) -> (ReadingWorksheetConfig, &'static str, u64) {
        let exercise_count = params.get_usize("cards", 7).clamp(1, 60);
        let theme = match params
            .get_str("theme")
            .unwrap_or("todos")
            .to_ascii_lowercase()
            .as_str()
        {
            "animales" => SentenceTheme::Animales,
            "naturaleza" => SentenceTheme::Naturaleza,
            "cotidiano" => SentenceTheme::VidaCotidiana,
            "escuela" => SentenceTheme::EscuelaYCiencia,
            _ => SentenceTheme::Todos,
        };
        let blank_mode = match params
            .get_str("blank_mode")
            .unwrap_or("clave")
            .to_ascii_lowercase()
            .as_str()
        {
            "azar" | "libre" => BlankSelectionMode::AzarCualquierPalabra,
            "final" => BlankSelectionMode::UltimaPalabra,
            _ => BlankSelectionMode::AzarPalabrasClave,
        };
        let exercise_mode = match params
            .get_str("exercise_mode")
            .unwrap_or("opciones")
            .to_ascii_lowercase()
            .as_str()
        {
            "banco" => ExerciseMode::BancoDePalabras,
            "libre" => ExerciseMode::CompletarLibre,
            _ => ExerciseMode::OpcionesMultiples,
        };
        let (paper_size, paper_id) = parse_paper_size_param(params.get_str("format"));
        let seed = params.get_u64("seed").unwrap_or_else(generate_entropy_seed);

        let cfg = ReadingWorksheetConfig {
            exercise_count,
            theme,
            blank_mode,
            exercise_mode,
            visual_style: VisualStyle::Sobrio,
            paper_size,
            ..ReadingWorksheetConfig::default()
        };

        (cfg, paper_id, seed)
    }
}

impl DidacticTool for LecturaEscrituraTool {
    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn preview(&self, params: &ToolParams) -> Result<ToolPreviewOutput, String> {
        let (cfg, paper_id, seed) = self.build_config(params);
        let mut rng = Rng::from_seed(seed);
        let exercises = generate_reading_exercises(&cfg, &mut rng);
        let (page_w, page_h) = cfg.paper_size.portrait_dimensions_pt();

        let cards: Vec<PreviewCard> = exercises
            .iter()
            .map(|ex| PreviewCard {
                number: ex.number,
                prompt: format!("{}____________{}", ex.prefix, ex.suffix),
                kind_id: "CategoryAndSyllables",
                kind_label: cfg.theme.label(),
                sample_solutions: if ex.options.is_empty() {
                    vec![format!("Respuesta: {}", ex.missing_word)]
                } else {
                    vec![
                        format!("{}) {} (correcta)", ex.correct_letter(), ex.missing_word),
                        format!("Opciones: {}", ex.options.join(" / ")),
                    ]
                },
                matched_word_indices: Vec::new(),
            })
            .collect();

        let card_pages = (exercises.len() + cfg.exercises_per_page - 1) / cfg.exercises_per_page;
        let total_pages = card_pages * 2;

        Ok(ToolPreviewOutput {
            tool_id: self.manifest.id.clone(),
            tool_name: self.manifest.name.clone(),
            seed,
            paper_size_id: paper_id,
            paper_size_label: cfg.paper_size.label(),
            page_width_pt: page_w,
            page_height_pt: page_h,
            cards,
            placed_words: Vec::new(),
            summary: PreviewSummary {
                total_cards: exercises.len(),
                card_pages,
                total_pages,
                placed_words_count: exercises.len(),
                antonym_pairs_count: card_pages,
                synonym_pairs_count: card_pages,
                category_counts: vec![(cfg.theme.label(), exercises.len())],
            },
        })
    }

    fn generate_pdf(&self, params: &ToolParams) -> Result<GeneratedDocument, String> {
        let (cfg, paper_id, seed) = self.build_config(params);
        let mut rng = Rng::from_seed(seed);
        let exercises = generate_reading_exercises(&cfg, &mut rng);

        let (pdf_bytes, total_pages, _, _) =
            build_reading_worksheet_pdf_bytes(&cfg, &exercises).map_err(|e| e.to_string())?;

        let raw_name = params.get_str("filename").unwrap_or("").trim();
        let default_name = format!(
            "ficha_lectura_{}frases_{}_{}.pdf",
            exercises.len(),
            paper_id,
            seed % 100_000
        );
        let filename = sanitize_pdf_filename(if raw_name.is_empty() {
            &default_name
        } else {
            raw_name
        });

        Ok(GeneratedDocument {
            tool_id: self.manifest.id.clone(),
            filename,
            seed,
            paper_size_label: cfg.paper_size.label().to_string(),
            total_pages,
            card_count: exercises.len(),
            word_count: exercises.len(),
            pdf_bytes,
        })
    }
}

// =============================================================================
// 4. ADAPTADOR PARA HERRAMIENTAS CLI EXTERNAS (DESCUBRIMIENTO AUTOMÁTICO)
// =============================================================================

/// Conector que permite ejecutar cualquier binario o script externo en la carpeta
/// `Materiales Dídacticos` que exponga un archivo descriptor `didactic-tool.json`.
pub struct ExternalCliTool {
    manifest: ToolManifest,
    working_dir: PathBuf,
    command_bin: String,
    command_args: Vec<String>,
}

impl ExternalCliTool {
    pub fn from_descriptor(
        manifest: ToolManifest,
        working_dir: PathBuf,
        command_bin: String,
        command_args: Vec<String>,
    ) -> Self {
        Self {
            manifest,
            working_dir,
            command_bin,
            command_args,
        }
    }
}

impl DidacticTool for ExternalCliTool {
    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn preview(&self, params: &ToolParams) -> Result<ToolPreviewOutput, String> {
        // Para herramientas CLI externas, invocamos el proceso o generamos una vista previa base
        let seed = params.get_u64("seed").unwrap_or_else(generate_entropy_seed);
        let (paper_size, paper_id) = parse_paper_size_param(params.get_str("format"));
        let (page_w, page_h) = paper_size.landscape_dimensions_pt();

        Ok(ToolPreviewOutput {
            tool_id: self.manifest.id.clone(),
            tool_name: self.manifest.name.clone(),
            seed,
            paper_size_id: paper_id,
            paper_size_label: paper_size.label(),
            page_width_pt: page_w,
            page_height_pt: page_h,
            cards: vec![PreviewCard {
                number: 1,
                prompt: format!(
                    "Herramienta externa '{}' lista para ejecutar en '{}'.",
                    self.manifest.name,
                    self.working_dir.display()
                ),
                kind_id: "External",
                kind_label: "Herramienta Externa",
                sample_solutions: vec!["PDF directo".to_string()],
                matched_word_indices: vec![],
            }],
            placed_words: vec![],
            summary: PreviewSummary {
                total_cards: 1,
                card_pages: 1,
                total_pages: 1,
                placed_words_count: 0,
                antonym_pairs_count: 0,
                synonym_pairs_count: 0,
                category_counts: vec![],
            },
        })
    }

    fn generate_pdf(&self, params: &ToolParams) -> Result<GeneratedDocument, String> {
        let seed = params.get_u64("seed").unwrap_or_else(generate_entropy_seed);
        let tmp_out = std::env::temp_dir().join(format!("ext_{}_{}.pdf", self.manifest.id, seed));

        let mut cmd = Command::new(&self.command_bin);
        cmd.current_dir(&self.working_dir);
        for arg in &self.command_args {
            cmd.arg(arg);
        }
        cmd.env("DIDACTIC_OUTPUT_PDF", &tmp_out);
        cmd.env("DIDACTIC_SEED", seed.to_string());
        for (k, v) in &params.values {
            cmd.env(format!("DIDACTIC_PARAM_{}", k.to_ascii_uppercase()), v);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("No se pudo ejecutar '{}': {}", self.command_bin, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("El proceso externo falló: {}", stderr));
        }

        let pdf_bytes = fs::read(&tmp_out)
            .map_err(|e| format!("El proceso externo no generó el archivo PDF esperado: {}", e))?;
        let _ = fs::remove_file(&tmp_out);

        Ok(GeneratedDocument {
            tool_id: self.manifest.id.clone(),
            filename: sanitize_pdf_filename(&format!("{}_{}.pdf", self.manifest.id, seed % 100_000)),
            seed,
            paper_size_label: "A4 / Personalizado".to_string(),
            total_pages: 1,
            card_count: 0,
            word_count: 0,
            pdf_bytes,
        })
    }
}

// =============================================================================
// COMUNICADOR CENTRAL DE HERRAMIENTAS (ORQUESTADOR + BIBLIOTECA)
// =============================================================================

pub struct ToolCommunicator {
    tools: Vec<Box<dyn DidacticTool>>,
    workspace_dir: PathBuf,
    generated_dir: PathBuf,
}

impl ToolCommunicator {
    pub fn new(workspace_dir: PathBuf) -> Self {
        let generated_dir = workspace_dir.join("generated");
        let _ = fs::create_dir_all(&generated_dir);

        let mut comm = Self {
            tools: Vec::new(),
            workspace_dir,
            generated_dir,
        };

        // 1. Registrar las herramientas nativas integradas en el motor Rust
        comm.register_tool(Box::new(PalabreaTool::new()));
        comm.register_tool(Box::new(OperacionesMathTool::new()));
        comm.register_tool(Box::new(LecturaEscrituraTool::new()));
        comm.register_tool(Box::new(VocabChallengeTool::new()));

        // 2. Descubrir herramientas externas en la carpeta padre ("Materiales Dídacticos")
        comm.discover_external_tools();

        comm
    }

    pub fn register_tool(&mut self, tool: Box<dyn DidacticTool>) {
        let id = tool.manifest().id.clone();
        if !self.tools.iter().any(|t| t.manifest().id == id) {
            self.tools.push(tool);
        }
    }

    /// Escanea el directorio padre en busca de subcarpetas con `didactic-tool.json`
    /// para incorporar automáticamente nuevas herramientas al catálogo.
    pub fn discover_external_tools(&mut self) {
        let Some(parent_dir) = self.workspace_dir.parent() else {
            return;
        };
        let Ok(entries) = fs::read_dir(parent_dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || path == self.workspace_dir {
                continue;
            }
            let descriptor_path = path.join("didactic-tool.json");
            if descriptor_path.is_file() {
                if let Ok(raw) = fs::read_to_string(&descriptor_path) {
                    if let Some(ext_tool) = parse_external_tool_descriptor(&raw, &path) {
                        self.register_tool(Box::new(ext_tool));
                    }
                }
            }
        }
    }

    pub fn list_manifests(&self) -> Vec<&ToolManifest> {
        self.tools.iter().map(|t| t.manifest()).collect()
    }

    pub fn find_tool(&self, tool_id: &str) -> Option<&dyn DidacticTool> {
        self.tools
            .iter()
            .find(|t| t.manifest().id == tool_id)
            .map(|t| t.as_ref())
    }

    pub fn preview(&self, tool_id: &str, params: &ToolParams) -> Result<ToolPreviewOutput, String> {
        let tool = self
            .find_tool(tool_id)
            .ok_or_else(|| format!("Herramienta no encontrada en el comunicador: '{}'", tool_id))?;
        tool.preview(params)
    }

    pub fn generate_pdf(
        &self,
        tool_id: &str,
        params: &ToolParams,
        save_to_library: bool,
    ) -> Result<GeneratedDocument, String> {
        let tool = self
            .find_tool(tool_id)
            .ok_or_else(|| format!("Herramienta no encontrada en el comunicador: '{}'", tool_id))?;
        let doc = tool.generate_pdf(params)?;

        if save_to_library {
            let _ = fs::create_dir_all(&self.generated_dir);
            let target_path = self.generated_dir.join(&doc.filename);
            fs::write(&target_path, &doc.pdf_bytes)
                .map_err(|e| format!("Error al guardar PDF en la biblioteca: {}", e))?;
        }

        Ok(doc)
    }

    /// Lista todos los archivos PDF disponibles en `generated/` y en la raíz del proyecto.
    pub fn list_stored_materials(&self) -> Vec<StoredMaterial> {
        let mut materials: Vec<StoredMaterial> = Vec::new();

        // 1. Archivos en generated/
        if let Ok(entries) = fs::read_dir(&self.generated_dir) {
            for entry in entries.flatten() {
                if let Some(mat) = inspect_pdf_entry(&entry.path(), true) {
                    materials.push(mat);
                }
            }
        }

        // 2. Archivos PDF de ejemplo en la raíz del workspace
        if let Ok(entries) = fs::read_dir(&self.workspace_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(mat) = inspect_pdf_entry(&path, false) {
                        if !materials.iter().any(|m| m.filename == mat.filename) {
                            materials.push(mat);
                        }
                    }
                }
            }
        }

        materials.sort_by(|a, b| b.created_epoch_secs.cmp(&a.created_epoch_secs));
        materials
    }

    /// Lee los bytes de un PDF almacenado en `generated/` o en el workspace.
    pub fn read_stored_material(&self, filename: &str) -> Result<Vec<u8>, String> {
        let clean = sanitize_pdf_filename(filename);
        let gen_path = self.generated_dir.join(&clean);
        if gen_path.is_file() {
            return fs::read(&gen_path).map_err(|e| e.to_string());
        }
        let root_path = self.workspace_dir.join(&clean);
        if root_path.is_file() {
            return fs::read(&root_path).map_err(|e| e.to_string());
        }
        Err(format!("El archivo '{}' no existe en la biblioteca.", clean))
    }

    /// Elimina un archivo PDF de la carpeta `generated/`.
    pub fn delete_stored_material(&self, filename: &str) -> Result<(), String> {
        let clean = sanitize_pdf_filename(filename);
        let gen_path = self.generated_dir.join(&clean);
        if gen_path.is_file() {
            fs::remove_file(&gen_path).map_err(|e| e.to_string())
        } else {
            Err("Solo se pueden eliminar recursos guardados en la carpeta generated/.".to_string())
        }
    }

    pub fn lexicon_count(&self) -> usize {
        LEXICON.len()
    }
}

fn inspect_pdf_entry(path: &Path, is_generated_dir: bool) -> Option<StoredMaterial> {
    let ext = path.extension()?.to_str()?;
    if !ext.eq_ignore_ascii_case("pdf") {
        return None;
    }
    let filename = path.file_name()?.to_str()?.to_string();
    let meta = fs::metadata(path).ok()?;
    let created_epoch_secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let tool_id = if filename.starts_with("taller_lexico") {
        "taller-lexico".to_string()
    } else if filename.starts_with("operaciones") || filename.starts_with("ficha_") {
        "operaciones".to_string()
    } else {
        "palabrea".to_string()
    };

    Some(StoredMaterial {
        filename,
        tool_id,
        size_bytes: meta.len(),
        created_epoch_secs,
        is_generated_dir,
    })
}

pub fn sanitize_pdf_filename(input: &str) -> String {
    let base = Path::new(input)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("material_didactico.pdf");

    let mut clean: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if clean.is_empty() || clean == ".pdf" {
        clean = "material_didactico.pdf".to_string();
    }
    if !clean.to_ascii_lowercase().ends_with(".pdf") {
        clean.push_str(".pdf");
    }
    clean
}

/// Parser ligero para descriptores `didactic-tool.json` de herramientas externas.
fn parse_external_tool_descriptor(raw: &str, folder: &Path) -> Option<ExternalCliTool> {
    let extract_field = |key: &str| -> Option<String> {
        let pattern = format!("\"{}\"", key);
        let idx = raw.find(&pattern)?;
        let after_key = &raw[idx + pattern.len()..];
        let colon = after_key.find(':')?;
        let after_colon = after_key[colon + 1..].trim_start();
        if !after_colon.starts_with('"') {
            return None;
        }
        let rest = &after_colon[1..];
        let end_quote = rest.find('"')?;
        Some(rest[..end_quote].to_string())
    };

    let id = extract_field("id")?;
    let name = extract_field("name")?;
    let subtitle = extract_field("subtitle").unwrap_or_else(|| "Herramienta CLI Externa".to_string());
    let description = extract_field("description").unwrap_or_default();
    let category = extract_field("category").unwrap_or_else(|| "General".to_string());
    let age_range = extract_field("age_range").unwrap_or_else(|| "Todos los niveles".to_string());
    let command_bin = extract_field("command")?;

    let manifest = ToolManifest {
        id,
        name,
        subtitle,
        description,
        category,
        age_range,
        icon: extract_field("icon").unwrap_or_else(|| "terminal".to_string()),
        badge: "Conector CLI Externo".to_string(),
        version: extract_field("version").unwrap_or_else(|| "1.0.0".to_string()),
        source_kind: "external-cli".to_string(),
        capabilities: vec!["pdf_export".to_string()],
        params: vec![
            ToolParamSchema {
                name: "format".to_string(),
                label: "Formato de hoja".to_string(),
                description: "Tamaño de página del documento".to_string(),
                param_type: ParamType::Select {
                    options: vec![
                        ("a4".to_string(), "A4".to_string()),
                        ("letter".to_string(), "Letter / Carta".to_string()),
                    ],
                },
                default_value: "a4".to_string(),
            },
            ToolParamSchema {
                name: "seed".to_string(),
                label: "Semilla".to_string(),
                description: "Semilla opcional".to_string(),
                param_type: ParamType::Text {
                    placeholder: "Automática".to_string(),
                },
                default_value: "".to_string(),
            },
        ],
    };

    Some(ExternalCliTool::from_descriptor(
        manifest,
        folder.to_path_buf(),
        command_bin,
        Vec::new(),
    ))
}

// =============================================================================
// SERIALIZACIÓN A JSON DE LAS ESTRUCTURAS DEL COMUNICADOR
// =============================================================================

impl ToolManifest {
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(1024);
        out.push('{');
        let _ = write!(
            out,
            "\"id\":\"{}\",\"name\":\"{}\",\"subtitle\":\"{}\",\"description\":\"{}\",\"category\":\"{}\",\"age_range\":\"{}\",\"icon\":\"{}\",\"badge\":\"{}\",\"version\":\"{}\",\"source_kind\":\"{}\",",
            json_escape(&self.id),
            json_escape(&self.name),
            json_escape(&self.subtitle),
            json_escape(&self.description),
            json_escape(&self.category),
            json_escape(&self.age_range),
            json_escape(&self.icon),
            json_escape(&self.badge),
            json_escape(&self.version),
            json_escape(&self.source_kind),
        );

        out.push_str("\"capabilities\":[");
        for (i, cap) in self.capabilities.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(out, "\"{}\"", json_escape(cap));
        }
        out.push_str("],\"params\":[");
        for (i, p) in self.params.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&p.to_json());
        }
        out.push_str("]}");
        out
    }
}

impl ToolParamSchema {
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(256);
        out.push('{');
        let _ = write!(
            out,
            "\"name\":\"{}\",\"label\":\"{}\",\"description\":\"{}\",\"default_value\":\"{}\",",
            json_escape(&self.name),
            json_escape(&self.label),
            json_escape(&self.description),
            json_escape(&self.default_value),
        );
        match &self.param_type {
            ParamType::Number { min, max, step } => {
                let _ = write!(
                    out,
                    "\"type\":\"number\",\"min\":{},\"max\":{},\"step\":{}",
                    min, max, step
                );
            }
            ParamType::Select { options } => {
                out.push_str("\"type\":\"select\",\"options\":[");
                for (i, (val, label)) in options.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    let _ = write!(
                        out,
                        "{{\"value\":\"{}\",\"label\":\"{}\"}}",
                        json_escape(val),
                        json_escape(label)
                    );
                }
                out.push(']');
            }
            ParamType::Text { placeholder } => {
                let _ = write!(
                    out,
                    "\"type\":\"text\",\"placeholder\":\"{}\"",
                    json_escape(placeholder)
                );
            }
        }
        out.push('}');
        out
    }
}

impl ToolPreviewOutput {
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(32768);
        out.push('{');
        let _ = write!(
            out,
            "\"tool_id\":\"{}\",\"tool_name\":\"{}\",\"seed\":{},\"paper_size_id\":\"{}\",\"paper_size_label\":\"{}\",\"page_width_pt\":{:.2},\"page_height_pt\":{:.2},",
            json_escape(&self.tool_id),
            json_escape(&self.tool_name),
            self.seed,
            json_escape(self.paper_size_id),
            json_escape(self.paper_size_label),
            self.page_width_pt,
            self.page_height_pt
        );

        // Summary
        let _ = write!(
            out,
            "\"summary\":{{\"total_cards\":{},\"card_pages\":{},\"total_pages\":{},\"placed_words_count\":{},\"antonym_pairs_count\":{},\"synonym_pairs_count\":{},\"categories\":[",
            self.summary.total_cards,
            self.summary.card_pages,
            self.summary.total_pages,
            self.summary.placed_words_count,
            self.summary.antonym_pairs_count,
            self.summary.synonym_pairs_count
        );
        for (i, (cat_label, count)) in self.summary.category_counts.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(
                out,
                "{{\"label\":\"{}\",\"count\":{}}}",
                json_escape(cat_label),
                count
            );
        }
        out.push_str("]},");

        // Cards
        out.push_str("\"cards\":[");
        for (i, c) in self.cards.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(
                out,
                "{{\"number\":{},\"prompt\":\"{}\",\"kind_id\":\"{}\",\"kind_label\":\"{}\",\"sample_solutions\":[",
                c.number,
                json_escape(&c.prompt),
                json_escape(c.kind_id),
                json_escape(c.kind_label)
            );
            for (j, sol) in c.sample_solutions.iter().enumerate() {
                if j > 0 {
                    out.push(',');
                }
                let _ = write!(out, "\"{}\"", json_escape(sol));
            }
            out.push_str("],\"matched_word_indices\":[");
            for (j, idx) in c.matched_word_indices.iter().enumerate() {
                if j > 0 {
                    out.push(',');
                }
                let _ = write!(out, "{}", idx);
            }
            out.push_str("]}");
        }
        out.push_str("],");

        // Placed words
        out.push_str("\"placed_words\":[");
        for (i, pw) in self.placed_words.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(
                out,
                "{{\"index\":{},\"word\":\"{}\",\"display_text\":\"{}\",\"category_id\":\"{}\",\"category_label\":\"{}\",\"syllables\":{},\"letters\":{},\"vowels\":{},\"consonants\":{},\"font_family_css\":\"{}\",\"font_weight_css\":\"{}\",\"font_style_css\":\"{}\",\"font_pdf\":\"{}\",\"font_size\":{:.2},\"horiz_scale\":{:.1},\"outlined\":{},\"pdf_x\":{:.2},\"pdf_y\":{:.2},\"svg_x\":{:.2},\"svg_y\":{:.2},\"pdf_angle_deg\":{:.2},\"svg_angle_deg\":{:.2},\"width\":{:.2},\"height\":{:.2},\"synonyms_on_sheet\":[",
                pw.index,
                json_escape(&pw.word),
                json_escape(&pw.display_text),
                json_escape(pw.category_id),
                json_escape(pw.category_label),
                pw.syllables,
                pw.letters,
                pw.vowels,
                pw.consonants,
                json_escape(pw.font_family_css),
                json_escape(pw.font_weight_css),
                json_escape(pw.font_style_css),
                json_escape(pw.font_pdf),
                pw.font_size,
                pw.horiz_scale,
                pw.outlined,
                pw.pdf_x,
                pw.pdf_y,
                pw.svg_x,
                pw.svg_y,
                pw.pdf_angle_deg,
                pw.svg_angle_deg,
                pw.width,
                pw.height
            );
            for (j, s) in pw.synonyms_on_sheet.iter().enumerate() {
                if j > 0 {
                    out.push(',');
                }
                let _ = write!(out, "\"{}\"", json_escape(s));
            }
            out.push_str("],\"antonyms_on_sheet\":[");
            for (j, a) in pw.antonyms_on_sheet.iter().enumerate() {
                if j > 0 {
                    out.push(',');
                }
                let _ = write!(out, "\"{}\"", json_escape(a));
            }
            out.push_str("]}");
        }
        out.push_str("]}");
        out
    }
}

impl StoredMaterial {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"filename\":\"{}\",\"tool_id\":\"{}\",\"size_bytes\":{},\"created_epoch_secs\":{},\"deletable\":{}}}",
            json_escape(&self.filename),
            json_escape(&self.tool_id),
            self.size_bytes,
            self.created_epoch_secs,
            self.is_generated_dir
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_communicator_registers_native_tools_and_previews() {
        let tmp_dir = std::env::temp_dir().join("palabrea_comm_test");
        let _ = fs::create_dir_all(&tmp_dir);
        let comm = ToolCommunicator::new(tmp_dir.clone());

        let manifests = comm.list_manifests();
        assert!(manifests.len() >= 3);
        assert!(manifests.iter().any(|m| m.id == "palabrea"));
        assert!(manifests.iter().any(|m| m.id == "operaciones"));
        assert!(manifests.iter().any(|m| m.id == "taller-lexico"));

        let mut params = ToolParams::new();
        params.values.insert("cards".to_string(), "12".to_string());
        params.values.insert("format".to_string(), "a4".to_string());
        params.values.insert("seed".to_string(), "777".to_string());

        let preview1 = comm.preview("palabrea", &params).expect("Fallo en preview 1");
        let preview2 = comm.preview("palabrea", &params).expect("Fallo en preview 2");

        assert_eq!(preview1.seed, 777);
        assert_eq!(preview1.cards.len(), 12);
        assert!(!preview1.placed_words.is_empty());
        // Determinismo con la misma semilla
        assert_eq!(preview1.cards[0].prompt, preview2.cards[0].prompt);
        assert_eq!(
            preview1.placed_words[0].display_text,
            preview2.placed_words[0].display_text
        );

        let doc = comm
            .generate_pdf("palabrea", &params, false)
            .expect("Fallo al generar PDF desde comunicador");
        assert!(doc.pdf_bytes.starts_with(b"%PDF-1.4"));
        assert_eq!(doc.total_pages, 3);

        // Probar también la herramienta matemática "operaciones"
        let math_preview = comm
            .preview("operaciones", &params)
            .expect("Fallo en preview de operaciones");
        assert_eq!(math_preview.cards.len(), 12);
        let math_doc = comm
            .generate_pdf("operaciones", &params, false)
            .expect("Fallo en PDF de operaciones");
        assert!(math_doc.pdf_bytes.starts_with(b"%PDF-1.4"));
        assert_eq!(math_doc.total_pages, 2);
    }
}
