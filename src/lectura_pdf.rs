use crate::lectura_gen::{
    ExerciseMode, GeneratedSentenceExercise, ReadingWorksheetConfig, VisualStyle,
};
use crate::lexicon::Rng;
use crate::wordcloud::FontFamily;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Paleta sobria o pastel suave (sin adornos) para cada fila de ejercicio.
#[derive(Copy, Clone, Debug)]
struct RowPalette {
    bg_rgb: (f32, f32, f32),
    border_rgb: (f32, f32, f32),
    badge_bg_rgb: (f32, f32, f32),
    badge_text_rgb: (f32, f32, f32),
}

const SOBER_PALETTE: RowPalette = RowPalette {
    bg_rgb: (0.97, 0.97, 0.98),
    border_rgb: (0.72, 0.74, 0.78),
    badge_bg_rgb: (0.22, 0.25, 0.30),
    badge_text_rgb: (1.0, 1.0, 1.0),
};

const PASTEL_PALETTES: [RowPalette; 7] = [
    // 1. Lila suave
    RowPalette {
        bg_rgb: (0.95, 0.93, 0.99),
        border_rgb: (0.78, 0.72, 0.90),
        badge_bg_rgb: (0.58, 0.45, 0.78),
        badge_text_rgb: (1.0, 1.0, 1.0),
    },
    // 2. Celeste suave
    RowPalette {
        bg_rgb: (0.91, 0.96, 0.99),
        border_rgb: (0.68, 0.84, 0.94),
        badge_bg_rgb: (0.33, 0.68, 0.86),
        badge_text_rgb: (1.0, 1.0, 1.0),
    },
    // 3. Verde salvia suave
    RowPalette {
        bg_rgb: (0.91, 0.97, 0.92),
        border_rgb: (0.68, 0.86, 0.72),
        badge_bg_rgb: (0.36, 0.70, 0.45),
        badge_text_rgb: (1.0, 1.0, 1.0),
    },
    // 4. Rosa suave
    RowPalette {
        bg_rgb: (0.99, 0.92, 0.95),
        border_rgb: (0.92, 0.72, 0.82),
        badge_bg_rgb: (0.86, 0.45, 0.65),
        badge_text_rgb: (1.0, 1.0, 1.0),
    },
    // 5. Turquesa suave
    RowPalette {
        bg_rgb: (0.90, 0.97, 0.97),
        border_rgb: (0.66, 0.86, 0.86),
        badge_bg_rgb: (0.28, 0.72, 0.74),
        badge_text_rgb: (1.0, 1.0, 1.0),
    },
    // 6. Lavanda suave
    RowPalette {
        bg_rgb: (0.94, 0.92, 0.98),
        border_rgb: (0.76, 0.70, 0.88),
        badge_bg_rgb: (0.55, 0.43, 0.76),
        badge_text_rgb: (1.0, 1.0, 1.0),
    },
    // 7. Amarillo crema suave
    RowPalette {
        bg_rgb: (0.99, 0.97, 0.88),
        border_rgb: (0.90, 0.84, 0.62),
        badge_bg_rgb: (0.86, 0.68, 0.24),
        badge_text_rgb: (1.0, 1.0, 1.0),
    },
];

/// Convierte una cadena UTF-8 a literal PDF en codificación WinAnsi (Windows-1252).
fn pdf_string_win_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    out.push('(');
    for ch in text.chars() {
        let byte: u8 = match ch {
            ' '..='~' => ch as u8,
            '«' => 0xAB,
            '»' => 0xBB,
            '¡' => 0xA1,
            '¿' => 0xBF,
            'Á' => 0xC1,
            'É' => 0xC9,
            'Í' => 0xCD,
            'Ñ' => 0xD1,
            'Ó' => 0xD3,
            'Ú' => 0xDA,
            'Ü' => 0xDC,
            'á' => 0xE1,
            'é' => 0xE9,
            'í' => 0xED,
            'ñ' => 0xF1,
            'ó' => 0xF3,
            'ú' => 0xFA,
            'ü' => 0xFC,
            '•' => 0x95,
            '–' | '—' => b'-',
            _ => b'?',
        };
        match byte {
            b'(' => out.push_str("\\("),
            b')' => out.push_str("\\)"),
            b'\\' => out.push_str("\\\\"),
            0x20..=0x7E => out.push(byte as char),
            other => {
                let _ = write!(out, "\\{:03o}", other);
            }
        }
    }
    out.push(')');
    out
}

/// Dibuja un rectángulo con esquinas redondeadas usando curvas de Bézier cúbicas.
fn append_rounded_rect_path(buf: &mut String, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w * 0.5).min(h * 0.5);
    let k = 0.552_284_8 * r;
    let _ = writeln!(
        buf,
        "{:.2} {:.2} m \
         {:.2} {:.2} l \
         {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c \
         {:.2} {:.2} l \
         {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c \
         {:.2} {:.2} l \
         {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c \
         {:.2} {:.2} l \
         {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c \
         h",
        x + r,
        y,
        x + w - r,
        y,
        x + w - r + k,
        y,
        x + w,
        y + r - k,
        x + w,
        y + r,
        x + w,
        y + h - r,
        x + w,
        y + h - r + k,
        x + w - r + k,
        y + h,
        x + w - r,
        y + h,
        x + r,
        y + h,
        x + r - k,
        y + h,
        x,
        y + h - r + k,
        x,
        y + h - r,
        x,
        y + r,
        x,
        y + r - k,
        x + r - k,
        y,
        x + r,
        y
    );
}

fn append_circle_path(buf: &mut String, cx: f32, cy: f32, r: f32) {
    append_rounded_rect_path(buf, cx - r, cy - r, 2.0 * r, 2.0 * r, r);
}

/// Ancho exacto en unidades `em` según métricas AFM de Adobe para `Helvetica-Bold`.
fn helvetica_bold_char_em(ch: char) -> f32 {
    match ch {
        'A' | 'Á' | 'B' | 'C' | 'D' | 'H' | 'K' | 'N' | 'Ñ' | 'R' | 'U' | 'Ú' | 'Ü' => 0.722,
        'E' | 'É' | 'P' | 'S' | 'V' | 'X' | 'Y' => 0.667,
        'F' | 'L' | 'T' | 'Z' => 0.611,
        'G' | 'O' | 'Ó' | 'Q' => 0.778,
        'I' | 'Í' => 0.278,
        'J' => 0.556,
        'M' => 0.833,
        'W' => 0.944,
        '-' | '(' | ')' => 0.333,
        ' ' => 0.278,
        'a' | 'á' | 'c' | 'e' | 'é' | 'o' | 'ó' | 's' | 'z' => 0.556,
        'b' | 'd' | 'g' | 'h' | 'k' | 'n' | 'ñ' | 'p' | 'q' | 'u' | 'ú' | 'ü' | 'v' | 'x' | 'y' => {
            0.611
        }
        'f' | 'r' | 't' => 0.389,
        'i' | 'í' | 'j' | 'l' | '.' | ',' | ':' | ';' | '!' | '¡' => 0.278,
        'm' => 0.889,
        'w' => 0.778,
        _ => 0.556,
    }
}

/// Ancho exacto en unidades `em` según métricas AFM de Adobe para `Helvetica` regular.
fn helvetica_regular_char_em(ch: char) -> f32 {
    match ch {
        'A' | 'Á' | 'B' | 'E' | 'É' | 'K' | 'P' | 'S' | 'V' | 'X' | 'Y' => 0.667,
        'C' | 'D' | 'H' | 'N' | 'Ñ' | 'R' | 'U' | 'Ú' | 'Ü' => 0.722,
        'F' | 'T' | 'Z' => 0.611,
        'G' | 'O' | 'Ó' | 'Q' => 0.778,
        'I' | 'Í' | 'i' | 'í' | 'j' | 'l' => 0.222,
        'J' | 'L' | 'a' | 'á' | 'b' | 'd' | 'e' | 'é' | 'g' | 'h' | 'n' | 'ñ' | 'o' | 'ó' | 'p'
        | 'q' | 'u' | 'ú' | 'ü' => 0.556,
        'M' => 0.833,
        'W' => 0.944,
        'c' | 'k' | 's' | 'v' | 'x' | 'y' | 'z' => 0.500,
        'f' | 't' | '.' | ',' | ':' | ';' | ' ' | '!' | '¡' => 0.278,
        'm' => 0.833,
        'r' | '-' | '(' | ')' => 0.333,
        'w' => 0.722,
        _ => 0.556,
    }
}

fn measure_helvetica(text: &str, bold: bool, font_size: f32) -> f32 {
    text.chars()
        .map(|c| {
            if bold {
                helvetica_bold_char_em(c) * font_size
            } else {
                helvetica_regular_char_em(c) * font_size
            }
        })
        .sum()
}

/// Renderiza el encabezado limpio y sin adornos:
/// - Izquierda: Recuadros de `Nombre: ____________` y `Fecha: ____________`
/// - Derecha: Recuadro simple con la etiqueta de la ficha (`Ficha Uno`, etc.)
/// - Centro abajo: Píldora sobria con el título (`Lectura y escritura`) y debajo la consigna (`Subraya y escribe la respuesta correcta.`).
fn render_clean_header(
    stream: &mut String,
    page_w: f32,
    page_h: f32,
    margin_x: f32,
    cfg: &ReadingWorksheetConfig,
    page_idx: usize,
    total_work_pages: usize,
    is_answer_key: bool,
) -> f32 {
    let top_y = page_h - 28.0;
    let content_w = page_w - 2.0 * margin_x;

    // 1. Bloque superior: Nombre / Fecha a la izquierda y etiqueta de Ficha a la derecha
    let badge_w = 108.0;
    let badge_h = 46.0;
    let badge_x = page_w - margin_x - badge_w;
    let badge_y = top_y - badge_h;

    let fields_w = content_w - badge_w - 14.0;
    let field_h = 20.0;

    // Recuadro Nombre
    let nombre_y = top_y - field_h;
    let _ = writeln!(stream, "q 0.99 0.99 0.99 rg 0.62 0.65 0.70 RG 0.9 w");
    append_rounded_rect_path(stream, margin_x, nombre_y, fields_w, field_h, 6.0);
    let _ = writeln!(stream, "B Q");

    let label_font_size = 11.0;
    let nombre_label = if is_answer_key {
        "Nombre: CLAVE DE RESPUESTAS (DOCENTE)"
    } else {
        "Nombre:"
    };
    let _ = writeln!(
        stream,
        "q 0.12 0.14 0.17 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
        if is_answer_key {
            FontFamily::HelveticaBold.pdf_name()
        } else {
            FontFamily::Helvetica.pdf_name()
        },
        label_font_size,
        margin_x + 9.0,
        nombre_y + 6.0,
        pdf_string_win_ansi(nombre_label)
    );
    if !is_answer_key {
        let line_x1 = margin_x + 9.0 + measure_helvetica(nombre_label, false, label_font_size) + 6.0;
        let line_x2 = margin_x + fields_w - 10.0;
        let _ = writeln!(
            stream,
            "q 0.45 0.48 0.52 RG 0.75 w {:.2} {:.2} m {:.2} {:.2} l S Q",
            line_x1,
            nombre_y + 5.2,
            line_x2,
            nombre_y + 5.2
        );
    }

    // Recuadro Fecha
    let fecha_y = nombre_y - field_h - 6.0;
    let _ = writeln!(stream, "q 0.99 0.99 0.99 rg 0.62 0.65 0.70 RG 0.9 w");
    append_rounded_rect_path(stream, margin_x, fecha_y, fields_w, field_h, 6.0);
    let _ = writeln!(stream, "B Q");

    let fecha_label = "Fecha:";
    let _ = writeln!(
        stream,
        "q 0.12 0.14 0.17 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
        FontFamily::Helvetica.pdf_name(),
        label_font_size,
        margin_x + 9.0,
        fecha_y + 6.0,
        pdf_string_win_ansi(fecha_label)
    );
    let fline_x1 = margin_x + 9.0 + measure_helvetica(fecha_label, false, label_font_size) + 6.0;
    let fline_x2 = margin_x + fields_w - 10.0;
    let _ = writeln!(
        stream,
        "q 0.45 0.48 0.52 RG 0.75 w {:.2} {:.2} m {:.2} {:.2} l S Q",
        fline_x1,
        fecha_y + 5.2,
        fline_x2,
        fecha_y + 5.2
    );

    // Recuadro derecho: Ficha (limpio y sin adornos)
    let _ = writeln!(stream, "q 0.96 0.97 0.98 rg 0.50 0.53 0.58 RG 1.1 w");
    append_rounded_rect_path(stream, badge_x, badge_y, badge_w, badge_h, 8.0);
    let _ = writeln!(stream, "B Q");

    let ficha_text = if total_work_pages > 1 {
        format!("{} ({}/{})", cfg.ficha_label, page_idx + 1, total_work_pages)
    } else {
        cfg.ficha_label.clone()
    };

    // Dividir en dos líneas si tiene espacio (ej. "Ficha" arriba y "Uno" abajo, como en la imagen)
    let words: Vec<&str> = ficha_text.split_whitespace().collect();
    if words.len() == 2 {
        let f1 = 12.5;
        let f2 = 13.5;
        let w1 = measure_helvetica(words[0], true, f1);
        let w2 = measure_helvetica(words[1], true, f2);
        let _ = writeln!(
            stream,
            "q 0.12 0.14 0.17 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
            FontFamily::HelveticaBold.pdf_name(),
            f1,
            badge_x + (badge_w - w1) * 0.5,
            badge_y + 26.0,
            pdf_string_win_ansi(words[0])
        );
        let _ = writeln!(
            stream,
            "q 0.12 0.14 0.17 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
            FontFamily::HelveticaBold.pdf_name(),
            f2,
            badge_x + (badge_w - w2) * 0.5,
            badge_y + 10.0,
            pdf_string_win_ansi(words[1])
        );
    } else {
        let mut fsize = 12.0;
        let mut tw = measure_helvetica(&ficha_text, true, fsize);
        if tw > badge_w - 12.0 && tw > 0.0 {
            fsize *= (badge_w - 12.0) / tw;
            tw = measure_helvetica(&ficha_text, true, fsize);
        }
        let _ = writeln!(
            stream,
            "q 0.12 0.14 0.17 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
            FontFamily::HelveticaBold.pdf_name(),
            fsize,
            badge_x + (badge_w - tw) * 0.5,
            badge_y + (badge_h - fsize * 0.72) * 0.5,
            pdf_string_win_ansi(&ficha_text)
        );
    }

    // 2. Píldora central con el título ("Lectura y escritura")
    let title_text = if is_answer_key {
        format!("{} - Respuestas", cfg.title)
    } else {
        cfg.title.clone()
    };
    let title_font_size = 16.5;
    let title_text_w = measure_helvetica(&title_text, true, title_font_size);
    let title_pill_w = (title_text_w + 48.0).min(content_w);
    let title_pill_h = 26.0;
    let title_pill_x = (page_w - title_pill_w) * 0.5;
    let title_pill_y = fecha_y - title_pill_h - 10.0;

    let _ = writeln!(stream, "q 0.94 0.95 0.97 rg 0.55 0.58 0.64 RG 1.0 w");
    append_rounded_rect_path(
        stream,
        title_pill_x,
        title_pill_y,
        title_pill_w,
        title_pill_h,
        13.0,
    );
    let _ = writeln!(stream, "B Q");

    let _ = writeln!(
        stream,
        "q 0.08 0.10 0.13 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
        FontFamily::HelveticaBold.pdf_name(),
        title_font_size,
        (page_w - title_text_w) * 0.5,
        title_pill_y + 7.2,
        pdf_string_win_ansi(&title_text)
    );

    // 3. Subtítulo / Consigna ("Subraya y escribe la respuesta correcta.")
    let instruction_text = if is_answer_key {
        "Solucionario: palabra correcta escrita en la línea y subrayada en las opciones."
    } else {
        cfg.effective_instruction()
    };
    let inst_font_size = 12.2;
    let inst_w = measure_helvetica(instruction_text, false, inst_font_size);
    let inst_box_w = (inst_w + 28.0).min(content_w);
    let inst_box_h = 21.0;
    let inst_box_x = (page_w - inst_box_w) * 0.5;
    let inst_box_y = title_pill_y - inst_box_h - 6.0;

    let _ = writeln!(stream, "q 0.98 0.98 0.99 rg 0.78 0.80 0.84 RG 0.75 w");
    append_rounded_rect_path(stream, inst_box_x, inst_box_y, inst_box_w, inst_box_h, 6.0);
    let _ = writeln!(stream, "B Q");

    let _ = writeln!(
        stream,
        "q 0.12 0.14 0.17 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
        FontFamily::Helvetica.pdf_name(),
        inst_font_size,
        (page_w - inst_w) * 0.5,
        inst_box_y + 6.0,
        pdf_string_win_ansi(instruction_text)
    );

    inst_box_y - 10.0
}

/// Si la modalidad es `BancoDePalabras`, dibuja un recuadro sobrio con las palabras faltantes mezcladas.
fn render_word_bank_box(
    stream: &mut String,
    margin_x: f32,
    top_y: f32,
    content_w: f32,
    exercises_slice: &[GeneratedSentenceExercise],
) -> f32 {
    let mut words: Vec<String> = exercises_slice
        .iter()
        .map(|e| e.missing_word.clone())
        .collect();
    // Mezclar de forma determinista según la primera palabra para no mostrarlas en el mismo orden
    let mut rng = Rng::from_seed(exercises_slice.len() as u64 + 7919);
    rng.shuffle(&mut words);

    let joined = words.join("     •     ");
    let box_h = 34.0;
    let box_y = top_y - box_h;

    let _ = writeln!(stream, "q 0.97 0.98 0.99 rg 0.48 0.51 0.56 RG 1.0 w");
    append_rounded_rect_path(stream, margin_x, box_y, content_w, box_h, 7.0);
    let _ = writeln!(stream, "B Q");

    let title = "Banco de palabras:";
    let _ = writeln!(
        stream,
        "q 0.15 0.17 0.20 rg BT /{} 9.5 Tf {:.2} {:.2} Td {} Tj ET Q",
        FontFamily::HelveticaBold.pdf_name(),
        margin_x + 12.0,
        box_y + 21.0,
        pdf_string_win_ansi(title)
    );

    let mut word_font = 11.5;
    let mut w_line = measure_helvetica(&joined, true, word_font);
    let max_w = content_w - 24.0;
    if w_line > max_w && w_line > 0.0 {
        word_font *= max_w / w_line;
        w_line = measure_helvetica(&joined, true, word_font);
    }

    let _ = writeln!(
        stream,
        "q 0.08 0.10 0.13 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
        FontFamily::HelveticaBold.pdf_name(),
        word_font,
        margin_x + (content_w - w_line) * 0.5,
        box_y + 7.5,
        pdf_string_win_ansi(&joined)
    );

    box_y - 10.0
}

/// Renderiza una fila de ejercicio individual sin adornos:
/// - Círculo numerado a la izquierda (`1.`, `2.`, ...)
/// - Oración con línea de respuesta (`_______________`) en la parte superior
/// - Opciones `a)`, `b)`, `c)` alineadas en columnas debajo (si `ExerciseMode::OpcionesMultiples`).
fn render_exercise_row(
    stream: &mut String,
    row_x: f32,
    row_y: f32,
    row_w: f32,
    row_h: f32,
    ex: &GeneratedSentenceExercise,
    cfg: &ReadingWorksheetConfig,
    palette: RowPalette,
    is_answer_key: bool,
) {
    // 1. Contenedor horizontal redondeado de la fila
    let (bg_r, bg_g, bg_b) = palette.bg_rgb;
    let (bd_r, bd_g, bd_b) = palette.border_rgb;
    let _ = writeln!(
        stream,
        "q {:.2} {:.2} {:.2} rg {:.2} {:.2} {:.2} RG 0.95 w",
        bg_r, bg_g, bg_b, bd_r, bd_g, bd_b
    );
    append_rounded_rect_path(stream, row_x, row_y, row_w, row_h, 9.0);
    let _ = writeln!(stream, "B Q");

    // 2. Insignia circular con el número del ejercicio
    let badge_r = (row_h * 0.24).clamp(11.0, 14.5);
    let badge_cx = row_x + 20.0;
    let badge_cy = if cfg.exercise_mode == ExerciseMode::OpcionesMultiples {
        row_y + row_h * 0.63
    } else {
        row_y + row_h * 0.50
    };

    let (bb_r, bb_g, bb_b) = palette.badge_bg_rgb;
    let _ = writeln!(stream, "q {:.2} {:.2} {:.2} rg", bb_r, bb_g, bb_b);
    append_circle_path(stream, badge_cx, badge_cy, badge_r);
    let _ = writeln!(stream, "f Q");

    let num_label = format!("{}.", ex.number);
    let num_font = if ex.number >= 10 { 11.2 } else { 12.5 };
    let num_w = measure_helvetica(&num_label, true, num_font);
    let (bt_r, bt_g, bt_b) = palette.badge_text_rgb;
    let _ = writeln!(
        stream,
        "q {:.2} {:.2} {:.2} rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
        bt_r,
        bt_g,
        bt_b,
        FontFamily::HelveticaBold.pdf_name(),
        num_font,
        badge_cx - num_w * 0.5,
        badge_cy - num_font * 0.35,
        pdf_string_win_ansi(&num_label)
    );

    // 3. Línea de la oración con el espacio en blanco vectorial
    let text_left_x = row_x + 42.0;
    let max_sentence_w = row_w - 54.0;
    let base_font_size = (row_h * 0.21).clamp(11.5, 13.8);

    // Ancho de la línea en blanco para escribir la respuesta
    let ans_w = measure_helvetica(&ex.missing_word, true, base_font_size);
    let blank_line_w = (ans_w + 38.0).max(96.0);

    let pre_w = measure_helvetica(&ex.prefix, false, base_font_size);
    let suf_w = measure_helvetica(&ex.suffix, false, base_font_size);
    let total_line_w = pre_w + blank_line_w + suf_w;

    // Si la oración es muy larga para caber en una sola línea, escalar ligeramente el tamaño de fuente y la línea
    let (sent_font, final_blank_w) = if total_line_w > max_sentence_w && total_line_w > 0.0 {
        let scale = (max_sentence_w / total_line_w).max(0.76);
        (
            base_font_size * scale,
            (blank_line_w * scale).max(ans_w * scale + 16.0),
        )
    } else {
        (base_font_size, blank_line_w)
    };

    let sentence_y = if cfg.exercise_mode == ExerciseMode::OpcionesMultiples {
        row_y + row_h * 0.58
    } else {
        row_y + (row_h - sent_font * 0.72) * 0.5
    };

    let mut cursor_x = text_left_x;

    // 3a. Prefijo de la oración
    if !ex.prefix.is_empty() {
        let _ = writeln!(
            stream,
            "q 0.08 0.10 0.13 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
            FontFamily::Helvetica.pdf_name(),
            sent_font,
            cursor_x,
            sentence_y,
            pdf_string_win_ansi(&ex.prefix)
        );
        cursor_x += measure_helvetica(&ex.prefix, false, sent_font);
    }

    // 3b. Línea horizontal en blanco para escribir la palabra eliminada
    let line_y = sentence_y - 1.5;
    let _ = writeln!(
        stream,
        "q 0.12 0.14 0.18 RG 1.15 w {:.2} {:.2} m {:.2} {:.2} l S Q",
        cursor_x,
        line_y,
        cursor_x + final_blank_w,
        line_y
    );

    // En la Hoja de Respuestas, escribir la palabra faltante centrada sobre la línea
    if is_answer_key {
        let word_w = measure_helvetica(&ex.missing_word, true, sent_font);
        let word_x = cursor_x + (final_blank_w - word_w) * 0.5;
        let _ = writeln!(
            stream,
            "q 0.08 0.28 0.72 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
            FontFamily::HelveticaBold.pdf_name(),
            sent_font,
            word_x,
            sentence_y + 1.0,
            pdf_string_win_ansi(&ex.missing_word)
        );
    }

    cursor_x += final_blank_w;

    // 3c. Sufijo de la oración (incluyendo el punto final)
    if !ex.suffix.is_empty() {
        let _ = writeln!(
            stream,
            "q 0.08 0.10 0.13 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
            FontFamily::Helvetica.pdf_name(),
            sent_font,
            cursor_x,
            sentence_y,
            pdf_string_win_ansi(&ex.suffix)
        );
    }

    // 4. Opciones múltiples debajo de la oración: a) ...   b) ...   c) ...
    if cfg.exercise_mode == ExerciseMode::OpcionesMultiples && !ex.options.is_empty() {
        let opt_font = (sent_font * 0.95).clamp(11.0, 13.0);
        let opt_y = row_y + row_h * 0.20;
        let opt_count = ex.options.len();
        let avail_opt_w = (row_w - 60.0).max(200.0);
        let col_w = avail_opt_w / (opt_count as f32);

        for (opt_idx, opt_word) in ex.options.iter().enumerate() {
            let letter = (b'a' + (opt_idx as u8)) as char;
            let prefix_label = format!("{}) ", letter);
            let opt_x = text_left_x + (opt_idx as f32) * col_w;

            let is_correct = is_answer_key && opt_idx == ex.correct_option_index;
            let prefix_w = measure_helvetica(&prefix_label, true, opt_font);
            let word_w = measure_helvetica(opt_word, is_correct, opt_font);

            let (r, g, b) = if is_correct {
                (0.08, 0.28, 0.72)
            } else {
                (0.10, 0.12, 0.15)
            };

            // Inciso "a) " en negrita
            let _ = writeln!(
                stream,
                "q {:.2} {:.2} {:.2} rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
                r,
                g,
                b,
                FontFamily::HelveticaBold.pdf_name(),
                opt_font,
                opt_x,
                opt_y,
                pdf_string_win_ansi(&prefix_label)
            );

            // Palabra de la opción
            let word_font_family = if is_correct {
                FontFamily::HelveticaBold
            } else {
                FontFamily::Helvetica
            };
            let _ = writeln!(
                stream,
                "q {:.2} {:.2} {:.2} rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
                r,
                g,
                b,
                word_font_family.pdf_name(),
                opt_font,
                opt_x + prefix_w,
                opt_y,
                pdf_string_win_ansi(opt_word)
            );

            // En la hoja de respuestas, subrayar la respuesta correcta ("Subraya y escribe...")
            if is_correct {
                let under_y = opt_y - 2.2;
                let _ = writeln!(
                    stream,
                    "q 0.08 0.28 0.72 RG 1.3 w {:.2} {:.2} m {:.2} {:.2} l S Q",
                    opt_x,
                    under_y,
                    opt_x + prefix_w + word_w,
                    under_y
                );
            }
        }
    }
}

/// Renderiza una página completa de la ficha de Lectura y Escritura.
fn render_reading_page_stream(
    exercises_slice: &[GeneratedSentenceExercise],
    page_w: f32,
    page_h: f32,
    cfg: &ReadingWorksheetConfig,
    page_idx: usize,
    total_work_pages: usize,
    is_answer_key: bool,
) -> String {
    let mut stream = String::with_capacity(16384);
    let margin_x = 34.0;
    let margin_bottom = 28.0;
    let content_w = page_w - 2.0 * margin_x;

    // 1. Encabezado limpio (Nombre, Fecha, Ficha, Título y Consigna)
    let mut current_top_y = render_clean_header(
        &mut stream,
        page_w,
        page_h,
        margin_x,
        cfg,
        page_idx,
        total_work_pages,
        is_answer_key,
    );

    // 2. Si el modo es Banco de Palabras, dibujar el recuadro de palabras arriba
    if cfg.exercise_mode == ExerciseMode::BancoDePalabras && !is_answer_key {
        current_top_y = render_word_bank_box(
            &mut stream,
            margin_x,
            current_top_y,
            content_w,
            exercises_slice,
        );
    }

    // 3. Distribuir las filas de ejercicios verticalmente
    let rows_count = cfg.exercises_per_page.max(exercises_slice.len()).max(1);
    let avail_h = (current_top_y - margin_bottom).max(200.0);
    let gap_y = (avail_h * 0.022).clamp(7.0, 12.0);
    let total_gaps = gap_y * ((rows_count.saturating_sub(1)) as f32);
    let row_h = ((avail_h - total_gaps) / (rows_count as f32)).clamp(38.0, 74.0);

    for (idx, ex) in exercises_slice.iter().enumerate() {
        let row_y = current_top_y - ((idx + 1) as f32) * row_h - (idx as f32) * gap_y;
        let palette = match cfg.visual_style {
            VisualStyle::Sobrio => SOBER_PALETTE,
            VisualStyle::PastelSinAdornos => PASTEL_PALETTES[idx % PASTEL_PALETTES.len()],
        };

        render_exercise_row(
            &mut stream,
            margin_x,
            row_y,
            content_w,
            row_h,
            ex,
            cfg,
            palette,
            is_answer_key,
        );
    }

    stream
}

/// Construye en memoria los bytes del PDF de Lectura y Escritura y devuelve
/// `(pdf_bytes, total_pages, worksheet_pages, answer_pages)`.
pub fn build_reading_worksheet_pdf_bytes(
    cfg: &ReadingWorksheetConfig,
    exercises: &[GeneratedSentenceExercise],
) -> io::Result<(Vec<u8>, usize, usize, usize)> {
    let (page_w, page_h) = cfg.paper_size.portrait_dimensions_pt();
    let per_page = cfg.exercises_per_page.max(1);

    let chunks: Vec<&[GeneratedSentenceExercise]> = exercises.chunks(per_page).collect();
    let work_pages_count = chunks.len().max(1);

    let mut page_streams: Vec<(f32, f32, String)> = Vec::new();

    // 1. Hojas de trabajo para el alumno
    for (page_idx, chunk) in chunks.iter().enumerate() {
        let stream = render_reading_page_stream(
            chunk,
            page_w,
            page_h,
            cfg,
            page_idx,
            work_pages_count,
            false,
        );
        page_streams.push((page_w, page_h, stream));
    }

    // 2. Hoja(s) de respuestas al final del PDF (si está habilitada)
    let mut answer_pages_count = 0usize;
    if cfg.include_answer_key {
        for (page_idx, chunk) in chunks.iter().enumerate() {
            let stream = render_reading_page_stream(
                chunk,
                page_w,
                page_h,
                cfg,
                page_idx,
                work_pages_count,
                true,
            );
            page_streams.push((page_w, page_h, stream));
            answer_pages_count += 1;
        }
    }

    let total_pages = page_streams.len();

    // Ensamblar documento PDF 1.4 con fuentes estándar Type1 (Helvetica y Helvetica-Bold)
    let fonts = [FontFamily::Helvetica, FontFamily::HelveticaBold];
    let mut objects: Vec<Vec<u8>> = Vec::new();

    // Objeto 1: Catalog
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());

    // Objeto 2: Pages
    let first_page_obj_id = 3 + fonts.len();
    let mut kids_refs = String::new();
    for i in 0..total_pages {
        let page_obj_id = first_page_obj_id + i * 2;
        let _ = write!(kids_refs, "{} 0 R ", page_obj_id);
    }
    let pages_dict = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids_refs.trim_end(),
        total_pages
    );
    objects.push(pages_dict.into_bytes());

    // Objetos 3..=4: Fuentes
    let mut font_resource_entries = String::new();
    for (idx, font) in fonts.iter().enumerate() {
        let obj_id = 3 + idx;
        let font_dict = format!(
            "<< /Type /Font /Subtype /Type1 /BaseFont /{} /Encoding /WinAnsiEncoding >>",
            font.base_font_ps_name()
        );
        objects.push(font_dict.into_bytes());
        let _ = write!(font_resource_entries, "/{} {} 0 R ", font.pdf_name(), obj_id);
    }

    // Objetos de página y streams
    for (i, (w, h, content)) in page_streams.into_iter().enumerate() {
        let page_obj_id = first_page_obj_id + i * 2;
        let stream_obj_id = page_obj_id + 1;

        let page_dict = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /Font << {} >> >> /Contents {} 0 R >>",
            w,
            h,
            font_resource_entries.trim_end(),
            stream_obj_id
        );
        objects.push(page_dict.into_bytes());

        let content_bytes = content.into_bytes();
        let mut stream_obj = Vec::with_capacity(content_bytes.len() + 64);
        write!(
            stream_obj,
            "<< /Length {} >>\nstream\n",
            content_bytes.len()
        )?;
        stream_obj.extend_from_slice(&content_bytes);
        if !content_bytes.ends_with(b"\n") {
            stream_obj.push(b'\n');
        }
        stream_obj.extend_from_slice(b"endstream");
        objects.push(stream_obj);
    }

    // Serializar archivo PDF con tabla xref
    let mut pdf_bytes: Vec<u8> = Vec::with_capacity(32768);
    pdf_bytes.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets: Vec<usize> = Vec::with_capacity(objects.len() + 1);
    offsets.push(0);

    for (idx, obj_data) in objects.iter().enumerate() {
        let obj_id = idx + 1;
        offsets.push(pdf_bytes.len());
        write!(pdf_bytes, "{} 0 obj\n", obj_id)?;
        pdf_bytes.extend_from_slice(obj_data);
        pdf_bytes.extend_from_slice(b"\nendobj\n");
    }

    let xref_offset = pdf_bytes.len();
    write!(pdf_bytes, "xref\n0 {}\n", offsets.len())?;
    pdf_bytes.extend_from_slice(b"0000000000 65535 f \n");
    for &off in &offsets[1..] {
        write!(pdf_bytes, "{:010} 00000 n \n", off)?;
    }

    write!(
        pdf_bytes,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        offsets.len(),
        xref_offset
    )?;

    Ok((
        pdf_bytes,
        total_pages,
        work_pages_count,
        answer_pages_count,
    ))
}

/// Genera y guarda en disco el archivo PDF de la ficha de Lectura y Escritura.
pub fn write_reading_worksheet_pdf(
    output_path: &Path,
    cfg: &ReadingWorksheetConfig,
    exercises: &[GeneratedSentenceExercise],
) -> io::Result<(usize, usize, usize)> {
    let (pdf_bytes, total_pages, work_pages, ans_pages) =
        build_reading_worksheet_pdf_bytes(cfg, exercises)?;
    let mut file = File::create(output_path)?;
    file.write_all(&pdf_bytes)?;
    file.flush()?;
    Ok((total_pages, work_pages, ans_pages))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lectura_gen::generate_reading_exercises;
    use crate::pdf::PaperSize;

    #[test]
    fn test_reading_pdf_generation_a4_and_letter() {
        for paper in [PaperSize::A4, PaperSize::Letter] {
            let cfg = ReadingWorksheetConfig {
                paper_size: paper,
                exercise_count: 7,
                ..ReadingWorksheetConfig::default()
            };
            let mut rng = Rng::from_seed(42);
            let exercises = generate_reading_exercises(&cfg, &mut rng);
            let (bytes, total, work, ans) =
                build_reading_worksheet_pdf_bytes(&cfg, &exercises).expect("Error creando PDF");
            assert!(bytes.starts_with(b"%PDF-1.4"));
            assert_eq!(work, 1);
            assert_eq!(ans, 1);
            assert_eq!(total, 2);
        }
    }
}
