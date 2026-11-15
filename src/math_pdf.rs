use crate::math_gen::{MathProblem, WorksheetConfig, WorksheetLanguage};
use crate::math_layout::{build_problem_layout, CellKind, OperatorSymbol, ProblemLayout};
use crate::wordcloud::{measure_text, FontFamily};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Tema de color pastel para cada tarjeta (fondo suave + color intenso para borde, casillas e insignia).
#[derive(Copy, Clone, Debug)]
struct CardTheme {
    bg_rgb: (f32, f32, f32),
    accent_rgb: (f32, f32, f32),
}

/// Los 5 temas pastel de la lámina de referencia:
/// 0: Rosa, 1: Celeste, 2: Amarillo, 3: Verde menta, 4: Lavanda.
const CARD_THEMES: [CardTheme; 5] = [
    // 0: Rosa pastel
    CardTheme {
        bg_rgb: (1.00, 0.91, 0.95),
        accent_rgb: (0.90, 0.20, 0.52),
    },
    // 1: Celeste cielo
    CardTheme {
        bg_rgb: (0.89, 0.96, 1.00),
        accent_rgb: (0.09, 0.57, 0.89),
    },
    // 2: Amarillo sol
    CardTheme {
        bg_rgb: (1.00, 0.97, 0.85),
        accent_rgb: (0.93, 0.58, 0.08),
    },
    // 3: Verde menta
    CardTheme {
        bg_rgb: (0.89, 0.98, 0.90),
        accent_rgb: (0.14, 0.69, 0.29),
    },
    // 4: Lavanda
    CardTheme {
        bg_rgb: (0.94, 0.91, 1.00),
        accent_rgb: (0.53, 0.26, 0.89),
    },
];

/// Secuencia de temas para las 12 tarjetas de una página (idéntica a la lámina de referencia).
const PAGE_THEME_ORDER: [usize; 12] = [0, 1, 2, 3, 4, 0, 2, 3, 1, 4, 0, 2];

/// Colores arcoíris para cada letra del título principal tipo burbuja.
const TITLE_LETTER_COLORS: [(f32, f32, f32); 7] = [
    (0.98, 0.16, 0.52), // Rosa vibrante
    (0.05, 0.65, 0.98), // Azul cielo
    (0.18, 0.80, 0.28), // Verde brillante
    (0.99, 0.52, 0.10), // Naranja
    (0.99, 0.84, 0.08), // Amarillo
    (0.96, 0.22, 0.22), // Rojo coral
    (0.65, 0.20, 0.92), // Púrpura
];

/// Convierte una cadena UTF-8 a literal PDF en codificación WinAnsi (Windows-1252).
fn pdf_string_win_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    out.push('(');
    for ch in text.chars() {
        let byte: u8 = match ch {
            ' '..='~' => ch as u8,
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
            '×' => 0xD7,
            '÷' => 0xF7,
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

/// Dibuja el trazado de un rectángulo con esquinas redondeadas.
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

/// Dibuja un círculo completo centrado en `(cx, cy)` con radio `r`.
fn append_circle_path(buf: &mut String, cx: f32, cy: f32, r: f32) {
    append_rounded_rect_path(buf, cx - r, cy - r, 2.0 * r, 2.0 * r, r);
}

/// Dibuja una estrella decorativa de 5 puntas centrada en `(cx, cy)`.
fn append_star_path(buf: &mut String, cx: f32, cy: f32, outer_r: f32, inner_r: f32, rot_deg: f32) {
    for i in 0..10 {
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        let angle = (rot_deg + 90.0 + (i as f32) * 36.0).to_radians();
        let px = cx + r * angle.cos();
        let py = cy + r * angle.sin();
        if i == 0 {
            let _ = write!(buf, "{:.2} {:.2} m ", px, py);
        } else {
            let _ = write!(buf, "{:.2} {:.2} l ", px, py);
        }
    }
    let _ = writeln!(buf, "h");
}

/// Determina la cuadrícula óptima `(columnas, filas)` por página según el tamaño de las operaciones.
pub fn choose_grid_dimensions(layouts: &[ProblemLayout]) -> (usize, usize) {
    let max_cols = layouts
        .iter()
        .map(|l| l.total_cols)
        .fold(4.0f32, f32::max);
    let max_rows = layouts
        .iter()
        .map(|l| l.total_rows)
        .fold(4.0f32, f32::max);

    if max_cols <= 6.5 && max_rows <= 6.2 {
        (3, 4) // 12 tarjetas por hoja (estándar idéntico a la imagen)
    } else if max_cols <= 8.2 && max_rows <= 7.5 {
        (3, 3) // 9 tarjetas por hoja para operaciones medianas-grandes
    } else {
        (2, 3) // 6 tarjetas por hoja para operaciones muy extensas
    }
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
        '-' => 0.333,
        ' ' => 0.278,
        'a' | 'á' | 'c' | 'e' | 'é' | 'o' | 'ó' | 's' | 'z' => 0.556,
        'b' | 'd' | 'g' | 'h' | 'k' | 'n' | 'ñ' | 'p' | 'q' | 'u' | 'ú' | 'ü' | 'v' | 'x' | 'y' => 0.611,
        'f' | 'r' | 't' => 0.389,
        'i' | 'í' | 'j' | 'l' | '.' | ',' | ':' => 0.278,
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
        'f' | 't' | '.' | ',' | ':' | ' ' => 0.278,
        'm' => 0.833,
        'r' | '-' => 0.333,
        'w' => 0.722,
        _ => 0.556,
    }
}

fn measure_exact_helvetica(text: &str, bold: bool, font_size: f32) -> f32 {
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

/// Dibuja el encabezado colorido de la lámina (título multicolor en nube celeste + cintillo de instrucciones).
fn render_worksheet_header(
    stream: &mut String,
    page_w: f32,
    page_h: f32,
    cfg: &WorksheetConfig,
    is_answer_key: bool,
) -> f32 {
    let top_y = page_h - 18.0;
    let banner_h = 44.0;
    let banner_y = top_y - banner_h;
    let banner_w = (page_w - 46.0).min(516.0);
    let banner_x = (page_w - banner_w) * 0.5;

    // 1. Nube/píldora celeste de fondo con círculos laterales suaves
    let _ = writeln!(stream, "q 0.85 0.94 0.99 rg 0.67 0.86 0.97 RG 1.8 w");
    append_rounded_rect_path(stream, banner_x, banner_y, banner_w, banner_h, 22.0);
    let _ = writeln!(stream, "B Q");

    // Estrellas decorativas amarillas a izquierda y derecha del título
    let _ = writeln!(stream, "q 1.00 0.84 0.16 rg 0.93 0.65 0.08 RG 1.2 w 1 j");
    append_star_path(stream, banner_x + 19.0, banner_y + 26.0, 10.0, 4.8, 12.0);
    let _ = writeln!(stream, "B");
    append_star_path(stream, banner_x + 33.0, banner_y + 13.0, 6.2, 3.0, -15.0);
    let _ = writeln!(stream, "B");
    append_star_path(
        stream,
        banner_x + banner_w - 20.0,
        banner_y + 22.0,
        10.5,
        5.0,
        -10.0,
    );
    let _ = writeln!(stream, "B Q");

    // 2. Título principal en letras tipo burbuja multicolor con borde oscuro y halo blanco
    let base_title = match cfg.language {
        WorksheetLanguage::Es => cfg.operation.title_es(),
        WorksheetLanguage::En => cfg.operation.title_en(),
    };
    let title_text = if is_answer_key {
        match cfg.language {
            WorksheetLanguage::Es => format!("{} - RESPUESTAS", base_title),
            WorksheetLanguage::En => format!("{} - ANSWER KEY", base_title),
        }
    } else {
        base_title.to_string()
    };

    let font = FontFamily::HelveticaBold;
    let max_title_w = banner_w - 92.0;
    let mut title_font_size = if is_answer_key { 21.5 } else { 28.0 };
    let tracking = 1.0;

    // Calcular ancho total letra por letra usando métricas exactas de Helvetica-Bold
    let chars: Vec<char> = title_text.chars().collect();
    let measure_tracked = |fsize: f32| -> f32 {
        let mut total = 0.0;
        for &ch in &chars {
            let cw = helvetica_bold_char_em(ch) * fsize;
            total += cw + tracking;
        }
        (total - tracking).max(0.0)
    };

    let mut total_title_w = measure_tracked(title_font_size);
    if total_title_w > max_title_w && total_title_w > 0.0 {
        title_font_size *= max_title_w / total_title_w;
        total_title_w = measure_tracked(title_font_size);
    }

    let title_start_x = (page_w - total_title_w) * 0.5;
    let title_baseline_y = banner_y + (banner_h - title_font_size * 0.72) * 0.5;

    // Capa 1: Halo exterior blanco grueso para efecto sticker
    let mut cursor_x = title_start_x;
    for &ch in &chars {
        let s = ch.to_string();
        let cw = helvetica_bold_char_em(ch) * title_font_size;
        if ch != ' ' {
            let _ = writeln!(
                stream,
                "q 1 1 1 RG {:.2} w 1 J 1 j BT 2 Tr /{} {:.2} Tf {:.2} {:.2} Td {} Tj 0 Tr ET Q",
                title_font_size * 0.22,
                font.pdf_name(),
                title_font_size,
                cursor_x,
                title_baseline_y,
                pdf_string_win_ansi(&s)
            );
        }
        cursor_x += cw + tracking;
    }

    // Capa 2: Relleno arcoíris + contorno oscuro por cada letra
    cursor_x = title_start_x;
    let mut color_idx = 0usize;
    for &ch in &chars {
        let s = ch.to_string();
        let cw = helvetica_bold_char_em(ch) * title_font_size;
        if ch != ' ' {
            let (r, g, b) = TITLE_LETTER_COLORS[color_idx % TITLE_LETTER_COLORS.len()];
            color_idx += 1;
            // Contorno oscuro detrás
            let _ = writeln!(
                stream,
                "q 0.14 0.11 0.22 RG {:.2} w 1 J 1 j BT 2 Tr /{} {:.2} Tf {:.2} {:.2} Td {} Tj 0 Tr ET Q",
                title_font_size * 0.11,
                font.pdf_name(),
                title_font_size,
                cursor_x,
                title_baseline_y,
                pdf_string_win_ansi(&s)
            );
            // Relleno brillante encima
            let _ = writeln!(
                stream,
                "q {:.2} {:.2} {:.2} rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
                r,
                g,
                b,
                font.pdf_name(),
                title_font_size,
                cursor_x,
                title_baseline_y,
                pdf_string_win_ansi(&s)
            );
        }
        cursor_x += cw + tracking;
    }

    // 3. Cintillo inferior con la consigna ("Actividad. Resuelve las siguientes...")
    let (lead_label, body_label) = match (cfg.language, is_answer_key) {
        (WorksheetLanguage::Es, false) => ("Actividad. ", cfg.operation.instruction_es()),
        (WorksheetLanguage::En, false) => ("Activity. ", cfg.operation.instruction_en()),
        (WorksheetLanguage::Es, true) => (
            "Hoja de respuestas. ",
            "Solucionario completo con pasos intermedios y resultados.",
        ),
        (WorksheetLanguage::En, true) => (
            "Answer Key. ",
            "Complete step-by-step solutions and final answers.",
        ),
    };

    let sub_size = 11.0;
    let w_lead = measure_exact_helvetica(lead_label, true, sub_size);
    let w_body = measure_exact_helvetica(body_label, false, sub_size);
    let text_total_w = w_lead + w_body;

    let pill_h = 20.0;
    let pill_y = banner_y - pill_h - 5.0;
    let pill_w = (text_total_w + 34.0).max(page_w * 0.64).min(page_w - 40.0);
    let pill_x = (page_w - pill_w) * 0.5;

    if is_answer_key {
        let _ = writeln!(stream, "q 0.88 0.98 0.90 rg 0.55 0.85 0.62 RG 1.0 w");
    } else {
        let _ = writeln!(stream, "q 1.00 0.96 0.80 rg 0.95 0.86 0.55 RG 1.0 w");
    }
    append_rounded_rect_path(stream, pill_x, pill_y, pill_w, pill_h, 10.0);
    let _ = writeln!(stream, "B Q");

    let sub_start_x = (page_w - text_total_w) * 0.5;
    let sub_y = pill_y + 6.0;

    let (lr, lg, lb) = if is_answer_key {
        (0.08, 0.52, 0.22)
    } else {
        (0.08, 0.18, 0.78)
    };

    let _ = writeln!(
        stream,
        "q {:.2} {:.2} {:.2} rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
        lr,
        lg,
        lb,
        FontFamily::HelveticaBold.pdf_name(),
        sub_size,
        sub_start_x,
        sub_y,
        pdf_string_win_ansi(lead_label)
    );
    let _ = writeln!(
        stream,
        "q 0.12 0.13 0.15 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
        FontFamily::Helvetica.pdf_name(),
        sub_size,
        sub_start_x + w_lead,
        sub_y,
        pdf_string_win_ansi(body_label)
    );

    pill_y - 10.0
}

/// Dibuja un símbolo operador (`×`, `+`, `-`) de forma vectorial dentro de una celda `(bx, by, box_size)`.
fn draw_operator_symbol(
    stream: &mut String,
    op: OperatorSymbol,
    bx: f32,
    by: f32,
    box_size: f32,
) {
    let cx = bx + box_size * 0.5;
    let cy = by + box_size * 0.5;
    let lw = (box_size * 0.13).clamp(1.6, 3.0);

    let _ = writeln!(stream, "q 0.05 0.06 0.08 RG {:.2} w 1 J", lw);
    match op {
        OperatorSymbol::Mul => {
            let arm = box_size * 0.23;
            let _ = writeln!(
                stream,
                "{:.2} {:.2} m {:.2} {:.2} l {:.2} {:.2} m {:.2} {:.2} l S",
                cx - arm,
                cy - arm,
                cx + arm,
                cy + arm,
                cx - arm,
                cy + arm,
                cx + arm,
                cy - arm
            );
        }
        OperatorSymbol::Add => {
            let arm = box_size * 0.26;
            let _ = writeln!(
                stream,
                "{:.2} {:.2} m {:.2} {:.2} l {:.2} {:.2} m {:.2} {:.2} l S",
                cx - arm,
                cy,
                cx + arm,
                cy,
                cx,
                cy - arm,
                cx,
                cy + arm
            );
        }
        OperatorSymbol::Sub => {
            let arm = box_size * 0.26;
            let _ = writeln!(
                stream,
                "{:.2} {:.2} m {:.2} {:.2} l S",
                cx - arm,
                cy,
                cx + arm,
                cy
            );
        }
    }
    let _ = writeln!(stream, "Q");
}

/// Dibuja un dígito centrado dentro de una casilla `(bx, by, box_size)`.
fn draw_centered_digit(
    stream: &mut String,
    digit: u8,
    bx: f32,
    by: f32,
    box_size: f32,
    rgb: (f32, f32, f32),
) {
    let font_size = box_size * 0.68;
    // En Helvetica-Bold todos los dígitos '0'..'9' tienen ancho 0.556 * font_size y altura cap 0.718 * font_size
    let digit_w = font_size * 0.556;
    let digit_h = font_size * 0.718;
    let tx = bx + (box_size - digit_w) * 0.5;
    let ty = by + (box_size - digit_h) * 0.5;
    let ch = (b'0' + (digit % 10)) as char;

    let _ = writeln!(
        stream,
        "q {:.2} {:.2} {:.2} rg BT /{} {:.2} Tf {:.2} {:.2} Td ({}) Tj ET Q",
        rgb.0,
        rgb.1,
        rgb.2,
        FontFamily::HelveticaBold.pdf_name(),
        font_size,
        tx,
        ty,
        ch
    );
}

/// Renderiza una tarjeta individual con su número, fondo pastel, borde discontinuo y casillas de operación.
fn render_problem_card(
    stream: &mut String,
    card_x: f32,
    card_y: f32,
    card_w: f32,
    card_h: f32,
    problem_number: usize,
    theme: CardTheme,
    problem: &MathProblem,
    layout: &ProblemLayout,
    is_answer_key: bool,
) {
    let (bg_r, bg_g, bg_b) = theme.bg_rgb;
    let (ac_r, ac_g, ac_b) = theme.accent_rgb;

    // 1. Fondo pastel redondeado
    let _ = writeln!(stream, "q {:.2} {:.2} {:.2} rg", bg_r, bg_g, bg_b);
    append_rounded_rect_path(stream, card_x, card_y, card_w, card_h, 11.0);
    let _ = writeln!(stream, "f Q");

    // 2. Borde exterior discontinuo redondeado del color de acento
    let _ = writeln!(
        stream,
        "q {:.2} {:.2} {:.2} RG 1.75 w [5.0 3.2] 0 d 1 J 1 j",
        ac_r, ac_g, ac_b
    );
    append_rounded_rect_path(
        stream,
        card_x + 1.2,
        card_y + 1.2,
        card_w - 2.4,
        card_h - 2.4,
        10.5,
    );
    let _ = writeln!(stream, "S Q");

    // 3. Insignia circular en la esquina superior izquierda con el número del ejercicio
    let badge_r = 10.2;
    let badge_cx = card_x + 16.5;
    let badge_cy = card_y + card_h - 16.5;
    let _ = writeln!(stream, "q {:.2} {:.2} {:.2} rg", ac_r, ac_g, ac_b);
    append_circle_path(stream, badge_cx, badge_cy, badge_r);
    let _ = writeln!(stream, "f Q");

    let num_str = problem_number.to_string();
    let num_font_size = if problem_number >= 100 {
        8.5
    } else if problem_number >= 10 {
        10.2
    } else {
        11.5
    };
    let (num_w, _) = measure_text(&num_str, FontFamily::HelveticaBold, num_font_size, 100.0);
    let num_x = badge_cx - num_w * 0.5;
    let num_y = badge_cy - num_font_size * 0.36;
    let _ = writeln!(
        stream,
        "q 1 1 1 rg BT /{} {:.2} Tf {:.2} {:.2} Td ({}) Tj ET Q",
        FontFamily::HelveticaBold.pdf_name(),
        num_font_size,
        num_x,
        num_y,
        num_str
    );

    // 4. Calcular tamaño y posición del bloque de casillas dentro de la tarjeta
    let is_division = matches!(problem, MathProblem::Division(_));
    let pad_left = if is_division { 26.0 } else { 14.0 };
    let pad_right = 11.0;
    let pad_top = if is_division { 14.0 } else { 9.0 };
    let pad_bottom = 9.0;

    let avail_w = (card_w - pad_left - pad_right).max(40.0);
    let avail_h = (card_h - pad_top - pad_bottom).max(40.0);

    let pitch_w = avail_w / layout.total_cols.max(1.0);
    let pitch_h = avail_h / layout.total_rows.max(1.0);
    let pitch = pitch_w.min(pitch_h).min(26.5);
    let box_fill_ratio = 0.86;
    let box_size = pitch * box_fill_ratio;

    let block_w = layout.total_cols * pitch;
    let block_h = layout.total_rows * pitch;

    let h_align = if is_division { 0.55 } else { 0.72 };
    let origin_x = card_x + pad_left + (avail_w - block_w) * h_align;
    let origin_top_y = card_y + pad_bottom + (avail_h + block_h) * 0.5;

    // 5. Dibujar cada celda (casillas de dígitos y operadores)
    for cell in &layout.cells {
        let bx = origin_x + cell.col * pitch;
        let by = origin_top_y - (cell.row + box_fill_ratio) * pitch;

        match cell.kind {
            CellKind::Operator(op) => {
                draw_operator_symbol(stream, op, bx, by, box_size);
            }
            CellKind::GivenDigit(d) => {
                // Casilla blanca con borde del color de la tarjeta
                let _ = writeln!(
                    stream,
                    "q 1 1 1 rg {:.2} {:.2} {:.2} RG 1.15 w",
                    ac_r, ac_g, ac_b
                );
                append_rounded_rect_path(stream, bx, by, box_size, box_size, box_size * 0.21);
                let _ = writeln!(stream, "B Q");

                // Dígito dado en negro intenso
                draw_centered_digit(stream, d, bx, by, box_size, (0.05, 0.06, 0.08));
            }
            CellKind::AnswerDigit(ans_opt) => {
                // Casilla blanca con borde del color de la tarjeta
                let _ = writeln!(
                    stream,
                    "q 1 1 1 rg {:.2} {:.2} {:.2} RG 1.15 w",
                    ac_r, ac_g, ac_b
                );
                append_rounded_rect_path(stream, bx, by, box_size, box_size, box_size * 0.21);
                let _ = writeln!(stream, "B Q");

                // Si es la hoja de respuestas, rellenar con el dígito de solución en azul/acento educativo
                if is_answer_key {
                    if let Some(d) = ans_opt {
                        draw_centered_digit(stream, d, bx, by, box_size, (0.07, 0.28, 0.78));
                    }
                }
            }
        }
    }

    // 6. Dibujar las líneas divisorias (barras horizontales, caja o galera de división)
    let rule_lw = (pitch * 0.085).clamp(1.5, 2.3);
    let _ = writeln!(stream, "q 0.05 0.06 0.08 RG {:.2} w 1 J 1 j", rule_lw);
    for rule in &layout.rules {
        let x1 = origin_x + rule.x1 * pitch;
        let y1 = origin_top_y - rule.y1 * pitch;
        let x2 = origin_x + rule.x2 * pitch;
        let y2 = origin_top_y - rule.y2 * pitch;
        let _ = writeln!(stream, "{:.2} {:.2} m {:.2} {:.2} l S", x1, y1, x2, y2);
    }
    let _ = writeln!(stream, "Q");
}

/// Renderiza una página completa de tarjetas de ejercicios (o de respuestas).
fn render_worksheet_page_stream(
    problems_slice: &[(usize, &MathProblem, &ProblemLayout)],
    cols: usize,
    rows: usize,
    page_w: f32,
    page_h: f32,
    cfg: &WorksheetConfig,
    is_answer_key: bool,
) -> String {
    let mut stream = String::with_capacity(32768);

    // Encabezado superior
    let grid_top_y = render_worksheet_header(&mut stream, page_w, page_h, cfg, is_answer_key);

    let margin_x = 20.0;
    let margin_bottom = 18.0;
    let gap_x = 12.0;
    let gap_y = 11.0;

    let total_grid_w = page_w - 2.0 * margin_x - gap_x * ((cols - 1) as f32);
    let total_grid_h = grid_top_y - margin_bottom - gap_y * ((rows - 1) as f32);

    let card_w = total_grid_w / (cols as f32);
    let card_h = total_grid_h / (rows as f32);

    for (local_idx, &(prob_num, problem, layout)) in problems_slice.iter().enumerate() {
        let col = local_idx % cols;
        let row = local_idx / cols;

        let card_x = margin_x + (col as f32) * (card_w + gap_x);
        let card_y = grid_top_y - ((row + 1) as f32) * card_h - (row as f32) * gap_y;

        let theme_idx = PAGE_THEME_ORDER[local_idx % PAGE_THEME_ORDER.len()];
        let theme = CARD_THEMES[theme_idx];

        render_problem_card(
            &mut stream,
            card_x,
            card_y,
            card_w,
            card_h,
            prob_num,
            theme,
            problem,
            layout,
            is_answer_key,
        );
    }

    stream
}

/// Construye en memoria los bytes del documento PDF de operaciones matemáticas y devuelve
/// `(pdf_bytes, total_paginas, paginas_ejercicios, paginas_respuestas)`.
pub fn build_math_worksheet_pdf_bytes(
    cfg: &WorksheetConfig,
    problems: &[MathProblem],
) -> io::Result<(Vec<u8>, usize, usize, usize)> {
    let (page_w, page_h) = cfg.paper_size.portrait_dimensions_pt();

    let layouts: Vec<ProblemLayout> = problems.iter().map(build_problem_layout).collect();
    let (cols, rows) = choose_grid_dimensions(&layouts);
    let per_page = cols * rows;

    let indexed: Vec<(usize, &MathProblem, &ProblemLayout)> = problems
        .iter()
        .zip(layouts.iter())
        .enumerate()
        .map(|(i, (p, l))| (i + 1, p, l))
        .collect();

    let mut page_streams: Vec<(f32, f32, String)> = Vec::new();

    // 1. Páginas de ejercicios (en blanco para resolver)
    let mut worksheet_pages = 0usize;
    for chunk in indexed.chunks(per_page) {
        let stream =
            render_worksheet_page_stream(chunk, cols, rows, page_w, page_h, cfg, false);
        page_streams.push((page_w, page_h, stream));
        worksheet_pages += 1;
    }

    // 2. Páginas de Hoja de Respuestas al final del PDF
    let mut answer_pages = 0usize;
    for chunk in indexed.chunks(per_page) {
        let stream =
            render_worksheet_page_stream(chunk, cols, rows, page_w, page_h, cfg, true);
        page_streams.push((page_w, page_h, stream));
        answer_pages += 1;
    }

    let total_pages = page_streams.len();

    // Ensamblar objetos PDF (1: Catalog, 2: Pages, 3..=4: Fuentes Helvetica y Helvetica-Bold)
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

    // Objetos de fuentes
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

    // Objetos de páginas y sus flujos
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

    // Escribir bytes PDF con tabla xref
    let mut pdf_bytes: Vec<u8> = Vec::with_capacity(65536);
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

    Ok((pdf_bytes, total_pages, worksheet_pages, answer_pages))
}

/// Genera y escribe el documento PDF completo en disco:
/// - Primeras páginas: Hojas de actividades para el alumno (casillas de respuesta en blanco).
/// - Páginas finales: Hoja de respuestas con todos los pasos intermedios y resultados resueltos.
/// Devuelve `(total_paginas, paginas_ejercicios, paginas_respuestas)`.
pub fn write_math_worksheet_pdf(
    output_path: &Path,
    cfg: &WorksheetConfig,
    problems: &[MathProblem],
) -> io::Result<(usize, usize, usize)> {
    let (pdf_bytes, total_pages, worksheet_pages, answer_pages) =
        build_math_worksheet_pdf_bytes(cfg, problems)?;
    let mut file = File::create(output_path)?;
    file.write_all(&pdf_bytes)?;
    file.flush()?;

    Ok((total_pages, worksheet_pages, answer_pages))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::Rng;
    use crate::math_gen::{
        DivisionNotation, generate_worksheet_problems, OperationKind, WorksheetConfig,
    };
    use crate::pdf::PaperSize;

    #[test]
    fn test_pdf_generation_all_operations_a4_and_letter() {
        let ops = [
            OperationKind::Multiplicacion,
            OperationKind::Division,
            OperationKind::Suma,
            OperationKind::Resta,
            OperationKind::Mixto,
        ];

        for op in ops {
            for paper in [PaperSize::A4, PaperSize::Letter] {
                let mut cfg = WorksheetConfig {
                    operation: op,
                    paper_size: paper,
                    problem_count: 12,
                    ..WorksheetConfig::default()
                };
                if op == OperationKind::Division && paper == PaperSize::Letter {
                    cfg.division.notation = DivisionNotation::Galera;
                }

                let mut rng = Rng::from_seed(12345);
                let problems = generate_worksheet_problems(&cfg, &mut rng);
                let tmp = std::env::temp_dir()
                    .join(format!("test_math_{:?}_{:?}.pdf", op, paper));

                let (total, work_p, ans_p) =
                    write_math_worksheet_pdf(&tmp, &cfg, &problems).expect("Fallo al crear PDF");
                assert_eq!(work_p, 1);
                assert_eq!(ans_p, 1);
                assert_eq!(total, 2);

                let meta = std::fs::metadata(&tmp).expect("PDF no encontrado");
                assert!(meta.len() > 8000, "PDF demasiado pequeño");
                let _ = std::fs::remove_file(&tmp);
            }
        }
    }
}
