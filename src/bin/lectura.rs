#[path = "../cards.rs"]
pub mod cards;
#[path = "../lectura_gen.rs"]
pub mod lectura_gen;
#[path = "../lectura_pdf.rs"]
pub mod lectura_pdf;
#[path = "../lexicon.rs"]
pub mod lexicon;
#[path = "../pdf.rs"]
pub mod pdf;
#[path = "../wordcloud.rs"]
pub mod wordcloud;

use lectura_gen::{
    generate_reading_exercises, load_custom_sentences_from_file, BlankSelectionMode, ExerciseMode,
    ReadingWorksheetConfig, SentenceTheme, VisualStyle, SENTENCE_BANK,
};
use lectura_pdf::write_reading_worksheet_pdf;
use lexicon::Rng;
use pdf::PaperSize;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Default)]
struct CliArgs {
    theme: Option<SentenceTheme>,
    blank_mode: Option<BlankSelectionMode>,
    exercise_mode: Option<ExerciseMode>,
    visual_style: Option<VisualStyle>,
    exercise_count: Option<usize>,
    exercises_per_page: Option<usize>,
    options_count: Option<usize>,
    paper_size: Option<PaperSize>,
    title: Option<String>,
    subtitle: Option<String>,
    ficha_label: Option<String>,
    bank_file: Option<PathBuf>,
    output_path: Option<PathBuf>,
    seed: Option<u64>,
    no_answer_key: bool,
    interactive_wizard: bool,
    show_help: bool,
}

fn parse_theme(s: &str) -> Option<SentenceTheme> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "todos" | "todo" | "mixto" | "mix" | "all" => Some(SentenceTheme::Todos),
        "2" | "animales" | "animal" | "seres" => Some(SentenceTheme::Animales),
        "3" | "naturaleza" | "entorno" | "clima" => Some(SentenceTheme::Naturaleza),
        "4" | "cotidiano" | "vida" | "familia" | "hogar" | "juegos" => {
            Some(SentenceTheme::VidaCotidiana)
        }
        "5" | "escuela" | "ciencia" | "lectura" | "colegio" => {
            Some(SentenceTheme::EscuelaYCiencia)
        }
        _ => None,
    }
}

fn parse_blank_mode(s: &str) -> Option<BlankSelectionMode> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "clave" | "claves" | "inteligente" | "smart" => {
            Some(BlankSelectionMode::AzarPalabrasClave)
        }
        "2" | "azar" | "libre" | "cualquiera" | "todas" | "any" => {
            Some(BlankSelectionMode::AzarCualquierPalabra)
        }
        "3" | "final" | "ultima" | "última" | "clasico" | "clásico" => {
            Some(BlankSelectionMode::UltimaPalabra)
        }
        _ => None,
    }
}

fn parse_exercise_mode(s: &str) -> Option<ExerciseMode> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "opciones" | "multiple" | "incisos" | "abc" => Some(ExerciseMode::OpcionesMultiples),
        "2" | "banco" | "recuadro" | "caja" => Some(ExerciseMode::BancoDePalabras),
        "3" | "libre" | "abierto" | "completar" => Some(ExerciseMode::CompletarLibre),
        _ => None,
    }
}

fn parse_visual_style(s: &str) -> Option<VisualStyle> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "sobrio" | "bn" | "gris" | "limpio" => Some(VisualStyle::Sobrio),
        "2" | "pastel" | "color" | "suave" => Some(VisualStyle::PastelSinAdornos),
        _ => None,
    }
}

fn parse_paper_size(s: &str) -> Option<PaperSize> {
    match s.trim().to_ascii_lowercase().as_str() {
        "a4" | "1" => Some(PaperSize::A4),
        "letter" | "carta" | "us-letter" | "2" => Some(PaperSize::Letter),
        _ => None,
    }
}

fn parse_cli() -> Result<CliArgs, String> {
    let mut cli = CliArgs::default();
    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" | "--ayuda" => {
                cli.show_help = true;
                i += 1;
            }
            "--asistente" | "--interactivo" | "-i" => {
                cli.interactive_wizard = true;
                i += 1;
            }
            "-t" | "--tema" | "--theme" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el nombre del tema después de --tema".to_string())?;
                cli.theme = Some(parse_theme(val).ok_or_else(|| {
                    format!(
                        "Tema no reconocido: '{}'. Usa: todos, animales, naturaleza, cotidiano o escuela.",
                        val
                    )
                })?);
                i += 1;
            }
            "-m" | "--modo-hueco" | "--hueco" | "--seleccion" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el modo después de --modo-hueco".to_string())?;
                cli.blank_mode = Some(parse_blank_mode(val).ok_or_else(|| {
                    format!(
                        "Modo de hueco no reconocido: '{}'. Usa: clave, azar o final.",
                        val
                    )
                })?);
                i += 1;
            }
            "--modalidad" | "--tipo" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta la modalidad después de --modalidad".to_string())?;
                cli.exercise_mode = Some(parse_exercise_mode(val).ok_or_else(|| {
                    format!(
                        "Modalidad no reconocida: '{}'. Usa: opciones, banco o libre.",
                        val
                    )
                })?);
                i += 1;
            }
            "--estilo" | "--style" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el estilo después de --estilo".to_string())?;
                cli.visual_style = Some(parse_visual_style(val).ok_or_else(|| {
                    format!(
                        "Estilo no reconocido: '{}'. Usa: sobrio o pastel.",
                        val
                    )
                })?);
                i += 1;
            }
            "-c" | "--cantidad" | "--frases" | "--ejercicios" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número de ejercicios después de --cantidad".to_string())?;
                let n: usize = val
                    .parse()
                    .map_err(|_| format!("Cantidad inválida: '{}'", val))?;
                if n == 0 {
                    return Err("La cantidad de frases debe ser mayor que 0.".to_string());
                }
                cli.exercise_count = Some(n);
                i += 1;
            }
            "--por-pagina" | "--renglones" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número después de --por-pagina".to_string())?;
                let n: usize = val
                    .parse()
                    .map_err(|_| format!("Valor inválido para --por-pagina: '{}'", val))?;
                if !(3..=12).contains(&n) {
                    return Err("La cantidad de ejercicios por página debe estar entre 3 y 12.".to_string());
                }
                cli.exercises_per_page = Some(n);
                i += 1;
            }
            "--opciones" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número de opciones después de --opciones".to_string())?;
                let n: usize = val
                    .parse()
                    .map_err(|_| format!("Valor inválido para --opciones: '{}'", val))?;
                if !(2..=4).contains(&n) {
                    return Err("El número de opciones por frase debe estar entre 2 y 4.".to_string());
                }
                cli.options_count = Some(n);
                i += 1;
            }
            "-b" | "--banco" | "--archivo-frases" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta la ruta del archivo después de --banco".to_string())?;
                cli.bank_file = Some(PathBuf::from(val));
                i += 1;
            }
            "--titulo" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el texto después de --titulo".to_string())?;
                cli.title = Some(val.clone());
                i += 1;
            }
            "--consigna" | "--subtitulo" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el texto después de --consigna".to_string())?;
                cli.subtitle = Some(val.clone());
                i += 1;
            }
            "--ficha" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta la etiqueta después de --ficha".to_string())?;
                cli.ficha_label = Some(val.clone());
                i += 1;
            }
            "--sin-respuestas" => {
                cli.no_answer_key = true;
                i += 1;
            }
            "-f" | "--formato" | "--format" | "--hoja" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el formato (a4 o letter) después de --formato".to_string())?;
                cli.paper_size = Some(parse_paper_size(val).ok_or_else(|| {
                    format!("Formato no válido: '{}'. Usa 'a4' o 'letter'.", val)
                })?);
                i += 1;
            }
            "-o" | "--salida" | "--output" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta la ruta del archivo después de --salida".to_string())?;
                cli.output_path = Some(PathBuf::from(val));
                i += 1;
            }
            "-s" | "--semilla" | "--seed" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el valor numérico después de --semilla".to_string())?;
                let s: u64 = val
                    .parse()
                    .map_err(|_| format!("Semilla inválida: '{}'", val))?;
                cli.seed = Some(s);
                i += 1;
            }
            other => {
                return Err(format!(
                    "Argumento no reconocido: '{}'. Usa --ayuda para ver las opciones disponibles.",
                    other
                ));
            }
        }
    }

    Ok(cli)
}

fn print_help() {
    println!("=========================================================================");
    println!("  GENERADOR DE FICHAS DE LECTURA Y ESCRITURA (ORACIONES INCOMPLETAS)");
    println!("  Diseño limpio sin adornos + extracción aleatoria de palabras del banco");
    println!("=========================================================================");
    println!();
    println!("Uso rápido (genera ficha con valores por defecto o asistente):");
    println!("  cargo run --bin lectura");
    println!("  cargo run --bin lectura -- --asistente");
    println!();
    println!("Uso directo por argumentos CLI:");
    println!("  cargo run --bin lectura -- [opciones]");
    println!();
    println!("Opciones principales:");
    println!("  -c, --cantidad <N>             Cantidad de oraciones a generar (por defecto: 7)");
    println!("  --por-pagina <3..12>           Oraciones por página (por defecto: 7)");
    println!("  -t, --tema <tema>              Tema del banco de datos:");
    println!("                                 'todos', 'animales', 'naturaleza', 'cotidiano', 'escuela'");
    println!("  -m, --modo-hueco <modo>        Cómo elegir la palabra que se quita en cada frase:");
    println!("                                 'clave' : Al azar entre las palabras clave de la oración (por defecto)");
    println!("                                 'azar'  : Al azar cualquier palabra de contenido de la frase");
    println!("                                 'final' : Siempre la última palabra clave de la frase");
    println!("  --modalidad <tipo>             Formato de respuesta:");
    println!("                                 'opciones' : Incisos a), b), c) + línea (por defecto)");
    println!("                                 'banco'    : Caja con banco de palabras arriba + línea");
    println!("                                 'libre'    : Solo línea en blanco para completar");
    println!("  --opciones <2..4>              Cantidad de incisos por oración (por defecto: 3 -> a, b, c)");
    println!("  --estilo <sobrio|pastel>       'sobrio' (blanco/gris sin adornos) o 'pastel' (franjas suaves)");
    println!("  -b, --banco <archivo.txt>      Cargar frases propias desde un archivo de texto (1 por línea)");
    println!("  --titulo <texto>               Título principal (por defecto: 'Lectura y escritura')");
    println!("  --consigna <texto>             Subtítulo / instrucción personalizada");
    println!("  --ficha <texto>                Etiqueta superior derecha (por defecto: 'Ficha Uno')");
    println!("  --sin-respuestas               No añadir la hoja de respuestas al final del PDF");
    println!("  -f, --formato <a4|letter>      Tamaño de hoja: 'a4' o 'letter' (por defecto: a4)");
    println!("  -o, --salida <archivo.pdf>     Archivo PDF de salida (por defecto: ficha_lectura_escritura.pdf)");
    println!("  -s, --semilla <u64>            Semilla numérica para reproducir o variar el material");
    println!("  -i, --asistente                Ejecutar asistente interactivo paso a paso");
    println!("  -h, --ayuda                    Muestra esta ayuda");
}

fn prompt_line(msg: &str) -> String {
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
    buf.trim().to_string()
}

fn run_interactive_wizard(cfg: &mut ReadingWorksheetConfig) -> PathBuf {
    println!("=========================================================================");
    println!("  ASISTENTE: GENERADOR DE FICHAS DE LECTURA Y ESCRITURA (SIN ADORNOS)");
    println!("=========================================================================");

    println!("\n1. Selecciona el tema del banco de datos ({} frases internas):", SENTENCE_BANK.len());
    println!("   1) Todos los temas (Mixto)");
    println!("   2) Animales y seres vivos");
    println!("   3) Naturaleza y entorno");
    println!("   4) Vida cotidiana, familia y juegos");
    println!("   5) Escuela, lectura y ciencia");
    let t_in = prompt_line("Tema (1-5) [por defecto: 1]: ");
    if let Some(t) = parse_theme(&t_in) {
        cfg.theme = t;
    }

    println!("\n2. ¿Cómo deseas quitar las palabras al azar de cada frase?");
    println!("   1) Al azar entre las palabras clave de cada frase (sujeto, verbo, objeto o adjetivo)");
    println!("   2) Al azar entre cualquier palabra de contenido de la frase");
    println!("   3) Quitar la última palabra clave de cada frase");
    let m_in = prompt_line("Modo de extracción (1-3) [por defecto: 1]: ");
    if let Some(m) = parse_blank_mode(&m_in) {
        cfg.blank_mode = m;
    }

    println!("\n3. Modalidad del ejercicio:");
    println!("   1) Opción múltiple a), b), c) debajo de cada frase + escribir en la línea");
    println!("   2) Recuadro superior con banco de palabras + escribir en la línea");
    println!("   3) Completar libremente (solo la línea en blanco)");
    let mod_in = prompt_line("Modalidad (1-3) [por defecto: 1]: ");
    if let Some(em) = parse_exercise_mode(&mod_in) {
        cfg.exercise_mode = em;
    }

    let c_in = prompt_line("\n4. ¿Cuántas oraciones deseas generar? (ej. 7, 14, 21) [por defecto: 7]: ");
    if let Ok(n) = c_in.parse::<usize>() {
        if n > 0 {
            cfg.exercise_count = n;
        }
    }

    let f_in = prompt_line("5. ¿Formato de hoja del PDF? (a4 / letter) [por defecto: a4]: ");
    if let Some(p) = parse_paper_size(&f_in) {
        cfg.paper_size = p;
    }

    let out_in = prompt_line(
        "6. Nombre del archivo PDF de salida [por defecto: ficha_lectura_escritura.pdf]: ",
    );
    if out_in.is_empty() {
        PathBuf::from("ficha_lectura_escritura.pdf")
    } else if out_in.to_ascii_lowercase().ends_with(".pdf") {
        PathBuf::from(out_in)
    } else {
        PathBuf::from(format!("{}.pdf", out_in))
    }
}

fn main() {
    let cli = match parse_cli() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if cli.show_help {
        print_help();
        return;
    }

    let mut cfg = ReadingWorksheetConfig::default();

    let output_path = if cli.interactive_wizard {
        run_interactive_wizard(&mut cfg)
    } else {
        if let Some(t) = cli.theme {
            cfg.theme = t;
        }
        if let Some(bm) = cli.blank_mode {
            cfg.blank_mode = bm;
        }
        if let Some(em) = cli.exercise_mode {
            cfg.exercise_mode = em;
        }
        if let Some(vs) = cli.visual_style {
            cfg.visual_style = vs;
        }
        if let Some(c) = cli.exercise_count {
            cfg.exercise_count = c;
        }
        if let Some(pp) = cli.exercises_per_page {
            cfg.exercises_per_page = pp;
        }
        if let Some(oc) = cli.options_count {
            cfg.options_count = oc;
        }
        if let Some(ps) = cli.paper_size {
            cfg.paper_size = ps;
        }
        if let Some(title) = cli.title {
            cfg.title = title;
        }
        if let Some(sub) = cli.subtitle {
            cfg.subtitle = Some(sub);
        }
        if let Some(ficha) = cli.ficha_label {
            cfg.ficha_label = ficha;
        }
        cfg.include_answer_key = !cli.no_answer_key;

        if let Some(ref bank_path) = cli.bank_file {
            match load_custom_sentences_from_file(bank_path) {
                Ok(lines) => cfg.custom_sentences = lines,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        cli.output_path
            .unwrap_or_else(|| PathBuf::from("ficha_lectura_escritura.pdf"))
    };

    let mut rng = match cli.seed {
        Some(s) => Rng::from_seed(s),
        None => Rng::from_entropy(),
    };

    let exercises = generate_reading_exercises(&cfg, &mut rng);

    match write_reading_worksheet_pdf(&output_path, &cfg, &exercises) {
        Ok((total_pages, work_pages, ans_pages)) => {
            println!("¡Ficha de Lectura y Escritura generada con éxito!");
            println!("  • Archivo PDF:          {}", output_path.display());
            println!("  • Diseño:               {} (sin adornos ni dibujos)", cfg.visual_style.label());
            println!(
                "  • Banco de datos:       {}",
                if cfg.custom_sentences.is_empty() {
                    format!(
                        "Banco interno ({} frases disponibles — Tema: {})",
                        SENTENCE_BANK.len(),
                        cfg.theme.label()
                    )
                } else {
                    format!(
                        "Archivo externo personalizado ({} frases cargadas)",
                        cfg.custom_sentences.len()
                    )
                }
            );
            println!("  • Selección de hueco:   {}", cfg.blank_mode.label());
            println!("  • Modalidad:            {}", cfg.exercise_mode.label());
            println!("  • Formato de hoja:      {}", cfg.paper_size.label());
            println!("  • Total de oraciones:   {}", exercises.len());
            println!(
                "  • Estructura del PDF:   {} páginas ({} de actividades + {} de hoja de respuestas)",
                total_pages, work_pages, ans_pages
            );
            println!();
            println!("Vista previa de las oraciones generadas (* = respuesta correcta):");
            for ex in exercises.iter().take(10) {
                println!("  {:>2}. {}", ex.number, ex.summary_line());
            }
        }
        Err(e) => {
            eprintln!(
                "Error al guardar el archivo PDF '{}': {}",
                output_path.display(),
                e
            );
            std::process::exit(1);
        }
    }
}
