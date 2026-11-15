pub mod cards;
pub mod lexicon;
pub mod pdf;
pub mod wordcloud;

use cards::generate_cards;
use lexicon::{Rng, WordEntry};
use pdf::{write_activity_pdf, PaperSize};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use wordcloud::generate_wordcloud_sheet;

#[derive(Debug, Default)]
struct CliOptions {
    card_count: Option<usize>,
    paper_size: Option<PaperSize>,
    output_path: Option<PathBuf>,
    seed: Option<u64>,
    show_help: bool,
}

fn parse_paper_size(input: &str) -> Option<PaperSize> {
    match input.trim().to_ascii_lowercase().as_str() {
        "a4" | "1" => Some(PaperSize::A4),
        "letter" | "carta" | "us-letter" | "2" => Some(PaperSize::Letter),
        _ => None,
    }
}

fn parse_cli_args() -> Result<CliOptions, String> {
    let mut opts = CliOptions::default();
    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" | "--ayuda" => {
                opts.show_help = true;
                i += 1;
            }
            "-t" | "--tarjetas" | "--cards" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número después de --tarjetas".to_string())?;
                let parsed: usize = val
                    .parse()
                    .map_err(|_| format!("Cantidad de tarjetas inválida: '{}'", val))?;
                if parsed == 0 {
                    return Err("La cantidad de tarjetas debe ser mayor a 0.".to_string());
                }
                opts.card_count = Some(parsed);
                i += 1;
            }
            "-f" | "--formato" | "--format" | "--hoja" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el formato (a4 o letter) después de --formato".to_string())?;
                let paper = parse_paper_size(val).ok_or_else(|| {
                    format!(
                        "Formato de hoja no reconocido: '{}'. Usa 'a4' o 'letter'.",
                        val
                    )
                })?;
                opts.paper_size = Some(paper);
                i += 1;
            }
            "-o" | "--salida" | "--output" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta la ruta del archivo después de --salida".to_string())?;
                opts.output_path = Some(PathBuf::from(val));
                i += 1;
            }
            "-s" | "--semilla" | "--seed" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el valor numérico después de --semilla".to_string())?;
                let seed: u64 = val
                    .parse()
                    .map_err(|_| format!("Semilla inválida: '{}'", val))?;
                opts.seed = Some(seed);
                i += 1;
            }
            other => {
                return Err(format!(
                    "Argumento no reconocido: '{}'. Usa --ayuda para ver las opciones.",
                    other
                ));
            }
        }
    }

    Ok(opts)
}

fn print_help() {
    println!("==================================================================");
    println!("  PALABREA: Generador Dinámico de Plantilla de Palabras y Tarjetas");
    println!("==================================================================");
    println!();
    println!("Uso interactivo:");
    println!("  cargo run");
    println!();
    println!("Uso con argumentos:");
    println!("  cargo run -- --tarjetas <N> --formato <a4|letter> [--salida <archivo.pdf>] [--semilla <N>]");
    println!();
    println!("Opciones:");
    println!("  -t, --tarjetas <N>         Cantidad de tarjetas a generar (6 por hoja en 3x2)");
    println!("  -f, --formato <a4|letter>  Tamaño de las hojas del PDF: 'a4' o 'letter'");
    println!("  -o, --salida <ruta.pdf>    Nombre o ruta del PDF generado (por defecto: palabrea_actividad.pdf)");
    println!("  -s, --semilla <u64>        Semilla opcional para reproducir exactamente la misma actividad");
    println!("  -h, --ayuda                Muestra esta ayuda");
}

fn prompt_card_count() -> usize {
    loop {
        print!("¿Cuántas tarjetas deseas generar? (ej. 12, 18, 24) [por defecto: 12]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return 12;
        }
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return 12;
        }
        match trimmed.parse::<usize>() {
            Ok(n) if n > 0 => return n,
            _ => println!("  Por favor, introduce un número entero mayor que 0."),
        }
    }
}

fn prompt_paper_size() -> PaperSize {
    loop {
        print!("¿En qué tamaño de hoja deseas el PDF? (a4 / letter) [por defecto: a4]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return PaperSize::A4;
        }
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return PaperSize::A4;
        }
        if let Some(paper) = parse_paper_size(trimmed) {
            return paper;
        }
        println!("  Formato no válido. Escribe 'a4' o 'letter' (también acepta 'carta').");
    }
}

fn prompt_output_path() -> PathBuf {
    print!("Nombre del archivo PDF de salida [por defecto: palabrea_actividad.pdf]: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return PathBuf::from("palabrea_actividad.pdf");
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        PathBuf::from("palabrea_actividad.pdf")
    } else if trimmed.to_ascii_lowercase().ends_with(".pdf") {
        PathBuf::from(trimmed)
    } else {
        PathBuf::from(format!("{}.pdf", trimmed))
    }
}

fn main() {
    let opts = match parse_cli_args() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if opts.show_help {
        print_help();
        return;
    }

    let interactive = opts.card_count.is_none() || opts.paper_size.is_none();
    if interactive {
        println!("==================================================================");
        println!("  PALABREA: Generador Dinámico de Plantilla y Tarjetas en PDF");
        println!("==================================================================");
    }

    let card_count = opts.card_count.unwrap_or_else(prompt_card_count);
    let paper_size = opts.paper_size.unwrap_or_else(prompt_paper_size);
    let output_path = opts.output_path.unwrap_or_else(|| {
        if interactive {
            prompt_output_path()
        } else {
            PathBuf::from("palabrea_actividad.pdf")
        }
    });

    let mut rng = match opts.seed {
        Some(seed) => Rng::from_seed(seed),
        None => Rng::from_entropy(),
    };

    // 1. Generar dinámicamente la hoja de palabras para encerrar (página completa al final)
    let (landscape_w, landscape_h) = paper_size.landscape_dimensions_pt();
    let sheet = generate_wordcloud_sheet(landscape_w, landscape_h, &mut rng);

    // 2. Generar dinámicamente las tarjetas verificadas contra las palabras de la hoja
    let placed_entries: Vec<&'static WordEntry> =
        sheet.placed_words.iter().map(|pw| pw.entry).collect();
    let cards = generate_cards(&placed_entries, card_count, &mut rng);

    // 3. Exportar el PDF (tarjetas en 3x2 + hoja de palabras al final)
    match write_activity_pdf(&output_path, paper_size, &cards, &sheet) {
        Ok(total_pages) => {
            let card_pages = total_pages.saturating_sub(1);
            println!();
            println!("¡Actividad generada con éxito!");
            println!("  • Archivo PDF:        {}", output_path.display());
            println!("  • Formato de hoja:    {}", paper_size.label());
            println!(
                "  • Tarjetas generadas: {} ({} {} en cuadrícula 3x2 de 6 tarjetas por hoja)",
                cards.len(),
                card_pages,
                if card_pages == 1 { "hoja" } else { "hojas" }
            );
            println!(
                "  • Hoja de palabras:   1 hoja completa al final con {} palabras dinámicas",
                sheet.placed_words.len()
            );
            println!();
            println!("Ejemplos de tarjetas incluidas (con respuestas presentes en la hoja):");
            for (idx, card) in cards.iter().take(6).enumerate() {
                println!(
                    "  {}. {}  -> [ej: {}]",
                    idx + 1,
                    card.prompt,
                    card.sample_solutions.join(", ")
                );
            }
        }
        Err(err) => {
            eprintln!("Error al guardar el archivo PDF '{}': {}", output_path.display(), err);
            std::process::exit(1);
        }
    }
}
