use crate::cards::Card;
use crate::wordcloud::{measure_text, FontFamily, PlacedWord, WordCloudSheet};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Formatos de hoja soportados.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PaperSize {
    A4,
    Letter,
}

impl PaperSize {
    /// Dimensiones en puntos PDF (1 pt = 1/72 pulgada) en orientación vertical (Portrait).
    pub fn portrait_dimensions_pt(&self) -> (f32, f32) {
        match self {
            PaperSize::A4 => (595.28, 841.89),     // 210 mm x 297 mm
            PaperSize::Letter => (612.00, 792.00), // 8.5 in x 11.0 in (215.9 mm x 279.4 mm)
        }
    }

    /// Dimensiones en puntos PDF en orientación apaisada (Landscape) para la hoja de palabras.
    pub fn landscape_dimensions_pt(&self) -> (f32, f32) {
        let (w, h) = self.portrait_dimensions_pt();
        (h, w)
    }

    pub fn label(&self) -> &'static str {
        match self {
            PaperSize::A4 => "A4 (210 x 297 mm)",
            PaperSize::Letter => "Letter / Carta (215.9 x 279.4 mm)",
        }
    }
}

/// Convierte una cadena UTF-8 en una cadena literal PDF codificada en WinAnsiEncoding (Windows-1252).
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

/// Dibuja un rectángulo con esquinas redondeadas usando curvas de Bézier cúbicas en PDF.
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

/// Dibuja el pequeño icono decorativo (abejita/flor estilizada) en la base de cada tarjeta.
fn append_card_ornament(buf: &mut String, cx: f32, cy: f32) {
    let _ = writeln!(buf, "q 0.58 0.61 0.64 RG 0.85 w");
    // Cuerpo central ovalado
    append_rounded_rect_path(buf, cx - 3.2, cy - 2.0, 6.4, 4.0, 2.0);
    let _ = writeln!(buf, "S");
    // Alitas laterales con curvas Bézier
    let _ = writeln!(
        buf,
        "{:.2} {:.2} m {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c S",
        cx - 1.0,
        cy + 1.8,
        cx - 6.5,
        cy + 6.8,
        cx - 9.0,
        cy + 1.0,
        cx - 2.5,
        cy - 0.5
    );
    let _ = writeln!(
        buf,
        "{:.2} {:.2} m {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c S",
        cx + 1.0,
        cy + 1.8,
        cx + 6.5,
        cy + 6.8,
        cx + 9.0,
        cy + 1.0,
        cx + 2.5,
        cy - 0.5
    );
    // Antenitas
    let _ = writeln!(
        buf,
        "{:.2} {:.2} m {:.2} {:.2} l {:.2} {:.2} m {:.2} {:.2} l S Q",
        cx - 1.2,
        cy + 2.0,
        cx - 2.8,
        cy + 4.8,
        cx + 1.2,
        cy + 2.0,
        cx + 2.8,
        cy + 4.8
    );
}

/// Ajusta un texto en múltiples líneas según el ancho máximo en puntos.
fn wrap_text_lines(text: &str, font: FontFamily, font_size: f32, max_width: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current, word)
        };
        let (w, _) = measure_text(&candidate, font, font_size, 100.0);
        if w <= max_width || current.is_empty() {
            current = candidate;
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Renderiza una página de hasta 6 tarjetas en disposición 3 filas x 2 columnas.
fn render_cards_page_stream(cards_slice: &[Card], page_width: f32, page_height: f32) -> String {
    let mut stream = String::with_capacity(8192);

    let margin_x = 26.0;
    let margin_y = 28.0;
    let gap_x = 16.0;
    let gap_y = 18.0;

    let cols = 2usize;
    let rows = 3usize;

    let total_w = page_width - 2.0 * margin_x - gap_x * ((cols - 1) as f32);
    let total_h = page_height - 2.0 * margin_y - gap_y * ((rows - 1) as f32);
    let card_w = total_w / (cols as f32);
    let card_h = total_h / (rows as f32);

    for (idx, card) in cards_slice.iter().take(6).enumerate() {
        let col = idx % cols;
        let row = idx / cols;

        let x = margin_x + (col as f32) * (card_w + gap_x);
        // Fila 0 arriba, fila 2 abajo
        let y = page_height - margin_y - ((row + 1) as f32) * card_h - (row as f32) * gap_y;

        // 1. Recuadro exterior de la tarjeta (borde de corte suave)
        let _ = writeln!(stream, "q 0.99 0.99 0.99 rg 0.35 0.37 0.40 RG 0.8 w");
        append_rounded_rect_path(&mut stream, x, y, card_w, card_h, 6.0);
        let _ = writeln!(stream, "B Q");

        // 2. Marco interior grueso con esquinas redondeadas (estilo plantilla Palabrea)
        let inset = 9.5;
        let inner_x = x + inset;
        let inner_y = y + inset;
        let inner_w = card_w - 2.0 * inset;
        let inner_h = card_h - 2.0 * inset;

        let _ = writeln!(stream, "q 0.62 0.64 0.67 RG 3.2 w");
        append_rounded_rect_path(&mut stream, inner_x, inner_y, inner_w, inner_h, 14.0);
        let _ = writeln!(stream, "S Q");

        // 3. Pequeño adorno inferior (abejita/flor como en la foto original)
        append_card_ornament(&mut stream, x + card_w * 0.5, inner_y + 12.0);

        // 4. Texto centrado de la consigna en negrita
        let font = FontFamily::HelveticaBold;
        let font_size = 15.2;
        let line_height = 21.0;
        let max_text_w = inner_w - 36.0;
        let lines = wrap_text_lines(&card.prompt, font, font_size, max_text_w);

        let block_h = (lines.len() as f32) * line_height;
        let center_y = inner_y + inner_h * 0.54;
        let first_baseline_y = center_y + (block_h * 0.5) - font_size * 0.8;

        for (line_idx, line) in lines.iter().enumerate() {
            let (line_w, _) = measure_text(line, font, font_size, 100.0);
            let text_x = x + (card_w - line_w) * 0.5;
            let text_y = first_baseline_y - (line_idx as f32) * line_height;

            let _ = writeln!(
                stream,
                "q 0.10 0.11 0.13 rg BT /{} {:.2} Tf {:.2} {:.2} Td {} Tj ET Q",
                font.pdf_name(),
                font_size,
                text_x,
                text_y,
                pdf_string_win_ansi(line)
            );
        }
    }

    stream
}

/// Renderiza la página final completa con la nube dinámica de palabras para encerrar.
fn render_wordcloud_page_stream(sheet: &WordCloudSheet) -> String {
    let mut stream = String::with_capacity(32768);

    // Marco perimetral suave alrededor de toda la hoja de palabras
    let border_margin = 16.0;
    let _ = writeln!(stream, "q 0.72 0.74 0.77 RG 1.4 w");
    append_rounded_rect_path(
        &mut stream,
        border_margin,
        border_margin,
        sheet.page_width - 2.0 * border_margin,
        sheet.page_height - 2.0 * border_margin,
        10.0,
    );
    let _ = writeln!(stream, "S Q");

    // Dibujar cada palabra en su posición, ángulo, escala horizontal y estilo
    for pw in &sheet.placed_words {
        draw_placed_word(&mut stream, pw);
    }

    stream
}

fn draw_placed_word(stream: &mut String, pw: &PlacedWord) {
    let rad = pw.angle_deg.to_radians();
    let cos_a = rad.cos();
    let sin_a = rad.sin();

    // Centrar la palabra en el origen local antes de rotar y trasladar a (center_x, center_y)
    let local_x = -pw.width * 0.5;
    let local_y = -pw.height * 0.36;

    let _ = writeln!(
        stream,
        "q {:.5} {:.5} {:.5} {:.5} {:.2} {:.2} cm",
        cos_a,
        sin_a,
        -sin_a,
        cos_a,
        pw.center_x,
        pw.center_y
    );

    if pw.outlined {
        // Texto hueco/contorneado estilo display ("2 Tr") como FURGONETA en la plantilla
        let _ = writeln!(
            stream,
            "0.12 0.13 0.15 RG 0.75 w BT 2 Tr /{} {:.2} Tf {:.1} Tz {:.2} {:.2} Td {} Tj 0 Tr ET Q",
            pw.font.pdf_name(),
            pw.font_size,
            pw.horiz_scale,
            local_x,
            local_y,
            pdf_string_win_ansi(&pw.display_text)
        );
    } else {
        let _ = writeln!(
            stream,
            "0.08 0.09 0.11 rg BT /{} {:.2} Tf {:.1} Tz {:.2} {:.2} Td {} Tj ET Q",
            pw.font.pdf_name(),
            pw.font_size,
            pw.horiz_scale,
            local_x,
            local_y,
            pdf_string_win_ansi(&pw.display_text)
        );
    }
}

/// Construye en memoria los bytes del documento PDF completo y devuelve `(bytes, total_pages)`:
/// - Primeras páginas: Tarjetas en cuadrícula 3x2 (6 por hoja).
/// - Última página: Hoja de encerrar palabras ocupando una hoja completa.
pub fn build_activity_pdf_bytes(
    paper_size: PaperSize,
    cards: &[Card],
    sheet: &WordCloudSheet,
) -> io::Result<(Vec<u8>, usize)> {
    let (card_page_w, card_page_h) = paper_size.landscape_dimensions_pt();

    // Construir los flujos de contenido de cada página
    let mut page_streams: Vec<(f32, f32, String)> = Vec::new();

    // Páginas de tarjetas (6 por hoja en 3x2)
    for chunk in cards.chunks(6) {
        let stream = render_cards_page_stream(chunk, card_page_w, card_page_h);
        page_streams.push((card_page_w, card_page_h, stream));
    }

    // Si por alguna razón se pidieron 0 tarjetas, aún se incluye la hoja final
    // Última página: Hoja completa de palabras para encerrar
    let word_stream = render_wordcloud_page_stream(sheet);
    page_streams.push((sheet.page_width, sheet.page_height, word_stream));

    let total_pages = page_streams.len();

    // Asignación de IDs de objetos PDF:
    // 1: Catalog
    // 2: Pages
    // 3..=12: 10 Fuentes estándar con WinAnsiEncoding (F1..F10)
    // 13..: Pares de (Page, ContentStream) para cada página
    let fonts = [
        FontFamily::Helvetica,
        FontFamily::HelveticaBold,
        FontFamily::HelveticaOblique,
        FontFamily::HelveticaBoldOblique,
        FontFamily::TimesRoman,
        FontFamily::TimesBold,
        FontFamily::TimesItalic,
        FontFamily::TimesBoldItalic,
        FontFamily::CourierBold,
        FontFamily::CourierOblique,
    ];

    let mut objects: Vec<Vec<u8>> = Vec::new();

    // Objeto 1: Catalog
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());

    // Objeto 2: Pages (se rellenará con los Kids)
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

    // Objetos 3..=12: Definición de las 10 fuentes estándar PDF con WinAnsiEncoding
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

    // Objetos de cada página y su respectivo stream de contenido
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

    // Escribir el archivo PDF con tabla xref válida
    let mut pdf_bytes: Vec<u8> = Vec::with_capacity(65536);
    pdf_bytes.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets: Vec<usize> = Vec::with_capacity(objects.len() + 1);
    offsets.push(0); // Objeto 0 libre

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

    Ok((pdf_bytes, total_pages))
}

/// Genera y escribe el documento PDF completo en disco:
/// - Primeras páginas: Tarjetas en cuadrícula 3x2 (6 por hoja).
/// - Última página: Hoja de encerrar palabras ocupando una hoja completa.
pub fn write_activity_pdf(
    output_path: &Path,
    paper_size: PaperSize,
    cards: &[Card],
    sheet: &WordCloudSheet,
) -> io::Result<usize> {
    let (pdf_bytes, total_pages) = build_activity_pdf_bytes(paper_size, cards, sheet)?;
    let mut file = File::create(output_path)?;
    file.write_all(&pdf_bytes)?;
    file.flush()?;

    Ok(total_pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::generate_cards;
    use crate::lexicon::{Rng, WordEntry};
    use crate::wordcloud::generate_wordcloud_sheet;

    #[test]
    fn test_pdf_generation_a4_and_letter() {
        for paper in [PaperSize::A4, PaperSize::Letter] {
            let mut rng = Rng::from_seed(99);
            let (lw, lh) = paper.landscape_dimensions_pt();
            let sheet = generate_wordcloud_sheet(lw, lh, &mut rng);
            let placed_entries: Vec<&'static WordEntry> =
                sheet.placed_words.iter().map(|pw| pw.entry).collect();
            let cards = generate_cards(&placed_entries, 12, &mut rng);

            let tmp_path = std::env::temp_dir().join(format!("test_palabrea_{:?}.pdf", paper));
            let pages = write_activity_pdf(&tmp_path, paper, &cards, &sheet)
                .expect("Fallo al escribir el PDF de prueba");
            // 12 tarjetas = 2 páginas de 6 tarjetas + 1 página final de palabras = 3 páginas
            assert_eq!(pages, 3);

            let metadata = std::fs::metadata(&tmp_path).expect("El archivo PDF no existe");
            assert!(metadata.len() > 4000, "El PDF generado es demasiado pequeño");
            let _ = std::fs::remove_file(&tmp_path);
        }
    }
}
