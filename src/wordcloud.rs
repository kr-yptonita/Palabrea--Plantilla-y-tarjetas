use crate::lexicon::{are_antonyms, are_synonyms, Category, Rng, WordEntry, LEXICON};

/// Variantes de fuente estándar PDF (100% compatibles sin incrustación externa).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FontFamily {
    Helvetica,
    HelveticaBold,
    HelveticaOblique,
    HelveticaBoldOblique,
    TimesRoman,
    TimesBold,
    TimesItalic,
    TimesBoldItalic,
    CourierBold,
    CourierOblique,
}

impl FontFamily {
    pub fn pdf_name(&self) -> &'static str {
        match self {
            FontFamily::Helvetica => "F1",
            FontFamily::HelveticaBold => "F2",
            FontFamily::HelveticaOblique => "F3",
            FontFamily::HelveticaBoldOblique => "F4",
            FontFamily::TimesRoman => "F5",
            FontFamily::TimesBold => "F6",
            FontFamily::TimesItalic => "F7",
            FontFamily::TimesBoldItalic => "F8",
            FontFamily::CourierBold => "F9",
            FontFamily::CourierOblique => "F10",
        }
    }

    pub fn base_font_ps_name(&self) -> &'static str {
        match self {
            FontFamily::Helvetica => "Helvetica",
            FontFamily::HelveticaBold => "Helvetica-Bold",
            FontFamily::HelveticaOblique => "Helvetica-Oblique",
            FontFamily::HelveticaBoldOblique => "Helvetica-BoldOblique",
            FontFamily::TimesRoman => "Times-Roman",
            FontFamily::TimesBold => "Times-Bold",
            FontFamily::TimesItalic => "Times-Italic",
            FontFamily::TimesBoldItalic => "Times-BoldItalic",
            FontFamily::CourierBold => "Courier-Bold",
            FontFamily::CourierOblique => "Courier-Oblique",
        }
    }

    /// Factor de ancho promedio por carácter respecto al tamaño en puntos.
    pub fn char_width_factor(&self, c: char) -> f32 {
        match self {
            FontFamily::CourierBold | FontFamily::CourierOblique => 0.61,
            FontFamily::HelveticaBold | FontFamily::HelveticaBoldOblique => {
                base_proportional_width(c) * 1.07
            }
            FontFamily::TimesRoman | FontFamily::TimesItalic => base_proportional_width(c) * 0.94,
            FontFamily::TimesBold | FontFamily::TimesBoldItalic => {
                base_proportional_width(c) * 1.01
            }
            FontFamily::Helvetica | FontFamily::HelveticaOblique => base_proportional_width(c),
        }
    }
}

fn base_proportional_width(c: char) -> f32 {
    match c {
        'i' | 'l' | 'í' | 'I' | '!' | '.' | ',' | ':' | ';' => 0.30,
        'j' | 't' | 'f' | 'r' | 'J' => 0.38,
        'm' | 'w' => 0.84,
        'M' | 'W' => 0.92,
        'A'..='Z' | 'Á' | 'É' | 'Í' | 'Ó' | 'Ú' | 'Ñ' | 'Ü' => 0.69,
        _ => 0.55,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CaseStyle {
    Lowercase,
    Uppercase,
}

impl CaseStyle {
    pub fn apply(&self, text: &str) -> String {
        match self {
            CaseStyle::Lowercase => text.to_lowercase(),
            CaseStyle::Uppercase => spanish_uppercase(text),
        }
    }
}

pub fn spanish_uppercase(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'á' => 'Á',
            'é' => 'É',
            'í' => 'Í',
            'ó' => 'Ó',
            'ú' => 'Ú',
            'ü' => 'Ü',
            'ñ' => 'Ñ',
            other => other.to_ascii_uppercase(),
        })
        .collect()
}

/// Palabra posicionada dentro de la hoja dinámica para encerrar.
#[derive(Clone, Debug)]
pub struct PlacedWord {
    pub entry: &'static WordEntry,
    pub display_text: String,
    pub font: FontFamily,
    pub font_size: f32,
    pub horiz_scale: f32, // Porcentaje PDF Tz (ej. 85.0 .. 115.0)
    pub outlined: bool,   // Modo 2 Tr (texto hueco contorneado estilo FURGONETA)
    pub center_x: f32,
    pub center_y: f32,
    pub angle_deg: f32,
    pub width: f32,
    pub height: f32,
}

/// Caja envolvente orientada en 2D para detección exacta de colisiones (SAT).
#[derive(Copy, Clone, Debug)]
struct Obb {
    cx: f32,
    cy: f32,
    hx: f32,
    hy: f32,
    cos_a: f32,
    sin_a: f32,
}

impl Obb {
    fn new(cx: f32, cy: f32, width: f32, height: f32, angle_deg: f32, pad_x: f32, pad_y: f32) -> Self {
        let rad = angle_deg.to_radians();
        Self {
            cx,
            cy,
            hx: width * 0.5 + pad_x,
            hy: height * 0.5 + pad_y,
            cos_a: rad.cos(),
            sin_a: rad.sin(),
        }
    }

    fn corners(&self) -> [(f32, f32); 4] {
        let ux = (self.cos_a * self.hx, self.sin_a * self.hx);
        let uy = (-self.sin_a * self.hy, self.cos_a * self.hy);
        [
            (self.cx - ux.0 - uy.0, self.cy - ux.1 - uy.1),
            (self.cx + ux.0 - uy.0, self.cy + ux.1 - uy.1),
            (self.cx + ux.0 + uy.0, self.cy + ux.1 + uy.1),
            (self.cx - ux.0 + uy.0, self.cy - ux.1 + uy.1),
        ]
    }

    fn fits_inside(&self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> bool {
        for (x, y) in self.corners() {
            if x < min_x || x > max_x || y < min_y || y > max_y {
                return false;
            }
        }
        true
    }

    fn intersects(&self, other: &Obb) -> bool {
        let axes = [
            (self.cos_a, self.sin_a),
            (-self.sin_a, self.cos_a),
            (other.cos_a, other.sin_a),
            (-other.sin_a, other.cos_a),
        ];
        let dx = other.cx - self.cx;
        let dy = other.cy - self.cy;

        for (ax, ay) in axes {
            let dist = (dx * ax + dy * ay).abs();
            let r_self = self.hx * (self.cos_a * ax + self.sin_a * ay).abs()
                + self.hy * (-self.sin_a * ax + self.cos_a * ay).abs();
            let r_other = other.hx * (other.cos_a * ax + other.sin_a * ay).abs()
                + other.hy * (-other.sin_a * ax + other.cos_a * ay).abs();
            if dist >= r_self + r_other {
                return false;
            }
        }
        true
    }
}

/// Mide el ancho y alto estimados de una cadena con la fuente, tamaño y escala horizontal dados.
pub fn measure_text(text: &str, font: FontFamily, font_size: f32, horiz_scale: f32) -> (f32, f32) {
    let scale = horiz_scale / 100.0;
    let units: f32 = text.chars().map(|c| font.char_width_factor(c)).sum();
    let width = units * font_size * scale;
    let height = font_size * 0.82;
    (width, height)
}

/// Resultado de la generación dinámica de la hoja de palabras.
#[derive(Clone, Debug)]
pub struct WordCloudSheet {
    pub page_width: f32,
    pub page_height: f32,
    pub placed_words: Vec<PlacedWord>,
}

/// Construye la lista priorizada de palabras para la hoja, asegurando pares de
/// antónimos, sinónimos y variedad de categorías, sílabas y longitudes.
fn select_candidate_words(rng: &mut Rng, target_count: usize) -> Vec<&'static WordEntry> {
    let mut selected: Vec<&'static WordEntry> = Vec::with_capacity(target_count);
    let mut used_words: Vec<&'static str> = Vec::with_capacity(target_count);

    let add_entry = |entry: &'static WordEntry,
                     selected: &mut Vec<&'static WordEntry>,
                     used: &mut Vec<&'static str>| {
        if !used.contains(&entry.word) {
            used.push(entry.word);
            selected.push(entry);
        }
    };

    // 1. Seleccionar 8 pares completos de antónimos
    let mut antonym_pairs: Vec<(&'static WordEntry, &'static WordEntry)> = Vec::new();
    for (i, a) in LEXICON.iter().enumerate() {
        for b in &LEXICON[i + 1..] {
            if are_antonyms(a, b) {
                antonym_pairs.push((a, b));
            }
        }
    }
    rng.shuffle(&mut antonym_pairs);
    for &(a, b) in antonym_pairs.iter().take(8) {
        add_entry(a, &mut selected, &mut used_words);
        add_entry(b, &mut selected, &mut used_words);
    }

    // 2. Seleccionar 8 pares completos de sinónimos
    let mut synonym_pairs: Vec<(&'static WordEntry, &'static WordEntry)> = Vec::new();
    for (i, a) in LEXICON.iter().enumerate() {
        for b in &LEXICON[i + 1..] {
            if are_synonyms(a, b) {
                synonym_pairs.push((a, b));
            }
        }
    }
    rng.shuffle(&mut synonym_pairs);
    for &(a, b) in synonym_pairs.iter().take(8) {
        add_entry(a, &mut selected, &mut used_words);
        add_entry(b, &mut selected, &mut used_words);
    }

    // 3. Asegurar al menos 5 palabras de cada categoría semántica
    for &cat in Category::ALL {
        let mut cat_words: Vec<&'static WordEntry> =
            LEXICON.iter().filter(|e| e.category == cat).collect();
        rng.shuffle(&mut cat_words);
        let mut count_for_cat = selected.iter().filter(|e| e.category == cat).count();
        for entry in cat_words {
            if count_for_cat >= 5 {
                break;
            }
            if !used_words.contains(&entry.word) {
                add_entry(entry, &mut selected, &mut used_words);
                count_for_cat += 1;
            }
        }
    }

    // 4. Rellenar con el resto del vocabulario barajado hasta alcanzar target_count
    let mut remaining: Vec<&'static WordEntry> = LEXICON
        .iter()
        .filter(|e| !used_words.contains(&e.word))
        .collect();
    rng.shuffle(&mut remaining);
    for entry in remaining {
        if selected.len() >= target_count {
            break;
        }
        add_entry(entry, &mut selected, &mut used_words);
    }

    selected
}

/// Genera dinámicamente la hoja de nube de palabras (orientación apaisada de página completa).
pub fn generate_wordcloud_sheet(
    page_width: f32,
    page_height: f32,
    rng: &mut Rng,
) -> WordCloudSheet {
    let margin = 22.0;
    let inner_pad = 8.0;
    let min_x = margin + inner_pad;
    let min_y = margin + inner_pad;
    let max_x = page_width - margin - inner_pad;
    let max_y = page_height - margin - inner_pad;

    let candidates = select_candidate_words(rng, 165);
    let mut placed_words: Vec<PlacedWord> = Vec::with_capacity(160);
    let mut placed_obbs: Vec<Obb> = Vec::with_capacity(160);

    // Cuadrícula de celdas para repartir las palabras de manera homogénea en toda la hoja
    let cols = 8usize;
    let rows = 6usize;
    let mut cell_order: Vec<(usize, usize)> = Vec::with_capacity(cols * rows);
    for r in 0..rows {
        for c in 0..cols {
            cell_order.push((c, r));
        }
    }
    rng.shuffle(&mut cell_order);

    let fonts = [
        FontFamily::HelveticaBold,
        FontFamily::Helvetica,
        FontFamily::TimesBold,
        FontFamily::TimesRoman,
        FontFamily::TimesItalic,
        FontFamily::HelveticaOblique,
        FontFamily::TimesBoldItalic,
        FontFamily::HelveticaBoldOblique,
        FontFamily::CourierBold,
        FontFamily::CourierOblique,
    ];

    for (idx, entry) in candidates.into_iter().enumerate() {
        // Elegir estilo de caja (alternando mayúsculas y minúsculas como en la imagen)
        let case_style = if rng.gen_range(0, 100) < 40 {
            CaseStyle::Uppercase
        } else {
            CaseStyle::Lowercase
        };
        let display_text = case_style.apply(entry.word);

        let mut font = fonts[rng.gen_range(0, fonts.len())];
        let horiz_scale = match rng.gen_range(0, 5) {
            0 => 82.0,  // condensada
            1 => 92.0,  // semicondensada
            2 => 100.0, // normal
            3 => 108.0, // semiexpandida
            _ => 114.0, // expandida
        };

        // Las primeras palabras incluyen algunas palabras grandes huecas/contorneadas (estilo FURGONETA / GORRO)
        let outlined = idx < 45
            && case_style == CaseStyle::Uppercase
            && rng.gen_range(0, 100) < 28;
        if outlined {
            font = FontFamily::HelveticaBold;
        }

        // Elegir orientación inicial: ~68% horizontal, ~20% vertical (±90°), ~12% inclinada
        let preferred_angle: f32 = {
            let roll = rng.gen_range(0, 100);
            if roll < 68 {
                0.0
            } else if roll < 78 {
                90.0
            } else if roll < 88 {
                -90.0
            } else if roll < 94 {
                rng.gen_f32(14.0, 22.0)
            } else {
                rng.gen_f32(-22.0, -14.0)
            }
        };

        // Tamaño inicial variado (jerarquía visual como en la foto: grandes, medianas y pequeñas)
        let base_size = if outlined {
            rng.gen_f32(21.0, 26.5)
        } else if idx < 22 {
            rng.gen_f32(20.0, 26.5)
        } else if idx < 70 {
            rng.gen_f32(14.5, 19.5)
        } else {
            rng.gen_f32(10.8, 14.5)
        };

        let size_steps = [
            base_size,
            base_size * 0.86,
            base_size * 0.75,
            12.0,
            10.2,
        ];
        let angle_fallbacks = [preferred_angle, 0.0, 90.0];

        let (cell_c, cell_r) = cell_order[idx % cell_order.len()];
        let cell_w = (max_x - min_x) / (cols as f32);
        let cell_h = (max_y - min_y) / (rows as f32);
        let anchor_x = min_x + (cell_c as f32 + 0.5) * cell_w;
        let anchor_y = min_y + (cell_r as f32 + 0.5) * cell_h;

        'size_loop: for &sz in &size_steps {
            let font_size = sz.max(9.8);
            let (w, h) = measure_text(&display_text, font, font_size, horiz_scale);

            for &angle_deg in &angle_fallbacks {
                for attempt in 0..220 {
                    let (cx, cy) = if attempt < 130 {
                        let t = attempt as f32 * 0.52;
                        let radius = 3.1 * (attempt as f32).sqrt();
                        let jitter_x = rng.gen_f32(-3.5, 3.5);
                        let jitter_y = rng.gen_f32(-3.5, 3.5);
                        (
                            anchor_x + radius * 1.35 * t.cos() + jitter_x,
                            anchor_y + radius * t.sin() + jitter_y,
                        )
                    } else {
                        (rng.gen_f32(min_x, max_x), rng.gen_f32(min_y, max_y))
                    };

                    let obb = Obb::new(cx, cy, w, h, angle_deg, 4.0, 2.8);
                    if !obb.fits_inside(min_x, min_y, max_x, max_y) {
                        continue;
                    }
                    if placed_obbs.iter().all(|other| !obb.intersects(other)) {
                        placed_obbs.push(obb);
                        placed_words.push(PlacedWord {
                            entry,
                            display_text: display_text.clone(),
                            font,
                            font_size,
                            horiz_scale,
                            outlined,
                            center_x: cx,
                            center_y: cy,
                            angle_deg,
                            width: w,
                            height: h,
                        });
                        break 'size_loop;
                    }
                }
            }
        }
    }

    WordCloudSheet {
        page_width,
        page_height,
        placed_words,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wordcloud_places_many_words_without_overlap() {
        let mut rng = Rng::from_seed(42);
        // Tamaño A4 apaisado en puntos
        let sheet = generate_wordcloud_sheet(841.89, 595.28, &mut rng);
        assert!(
            sheet.placed_words.len() >= 85,
            "Se colocaron muy pocas palabras: {}",
            sheet.placed_words.len()
        );

        // Verificar que ninguna pareja de palabras se solape
        let obbs: Vec<Obb> = sheet
            .placed_words
            .iter()
            .map(|pw| {
                Obb::new(
                    pw.center_x,
                    pw.center_y,
                    pw.width,
                    pw.height,
                    pw.angle_deg,
                    2.0,
                    1.5,
                )
            })
            .collect();

        for i in 0..obbs.len() {
            for j in (i + 1)..obbs.len() {
                assert!(
                    !obbs[i].intersects(&obbs[j]),
                    "Solapamiento detectado entre '{}' y '{}'",
                    sheet.placed_words[i].display_text,
                    sheet.placed_words[j].display_text
                );
            }
        }
    }
}
