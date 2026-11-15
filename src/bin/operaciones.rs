#[path = "../cards.rs"]
pub mod cards;
#[path = "../lexicon.rs"]
pub mod lexicon;
#[path = "../math_gen.rs"]
pub mod math_gen;
#[path = "../math_layout.rs"]
pub mod math_layout;
#[path = "../math_pdf.rs"]
pub mod math_pdf;
#[path = "../pdf.rs"]
pub mod pdf;
#[path = "../wordcloud.rs"]
pub mod wordcloud;

use lexicon::Rng;
use math_gen::{
    generate_worksheet_problems, AdditionConfig, DivisionConfig, DivisionNotation,
    MultiplicationConfig, OperationKind, SubtractionConfig, WorksheetConfig, WorksheetLanguage,
};
use math_pdf::write_math_worksheet_pdf;
use pdf::PaperSize;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Default)]
struct CliArgs {
    operation: Option<OperationKind>,
    multiplicand_digits: Option<usize>,
    multiplier_digits: Option<usize>,
    dividend_digits: Option<usize>,
    divisor_digits: Option<usize>,
    division_notation: Option<DivisionNotation>,
    exact_division: Option<bool>,
    addend_count: Option<usize>,
    addend_digits: Option<Vec<usize>>,
    minuend_digits: Option<usize>,
    subtrahend_digits: Option<usize>,
    problem_count: Option<usize>,
    paper_size: Option<PaperSize>,
    language: Option<WorksheetLanguage>,
    output_path: Option<PathBuf>,
    seed: Option<u64>,
    show_help: bool,
}

fn parse_operation(s: &str) -> Option<OperationKind> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "m" | "mul" | "mult" | "multiplicacion" | "multiplicación" | "multiplication"
        | "x" | "*" => Some(OperationKind::Multiplicacion),
        "2" | "d" | "div" | "division" | "división" | "/" => Some(OperationKind::Division),
        "3" | "s" | "sum" | "suma" | "sumas" | "adicion" | "adición" | "addition" | "+" => {
            Some(OperationKind::Suma)
        }
        "4" | "r" | "res" | "resta" | "restas" | "sustraccion" | "sustracción" | "subtraction"
        | "-" => Some(OperationKind::Resta),
        "5" | "mix" | "mixto" | "todas" | "combinado" => Some(OperationKind::Mixto),
        _ => None,
    }
}

fn parse_notation(s: &str) -> Option<DivisionNotation> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "tradicional" | "caja" | "española" | "espanola" | "hispana" => {
            Some(DivisionNotation::Tradicional)
        }
        "2" | "galera" | "casita" | "anglosajona" | "inglesa" => Some(DivisionNotation::Galera),
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

fn parse_language(s: &str) -> Option<WorksheetLanguage> {
    match s.trim().to_ascii_lowercase().as_str() {
        "es" | "esp" | "español" | "espanol" | "1" => Some(WorksheetLanguage::Es),
        "en" | "eng" | "english" | "ingles" | "inglés" | "2" => Some(WorksheetLanguage::En),
        _ => None,
    }
}

fn parse_digits_list(s: &str) -> Result<Vec<usize>, String> {
    let mut list = Vec::new();
    for part in s.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let val: usize = trimmed
            .parse()
            .map_err(|_| format!("Cantidad de dígitos inválida: '{}'", trimmed))?;
        if !(1..=5).contains(&val) {
            return Err(format!(
                "La cantidad de dígitos por sumando debe estar entre 1 y 5 (recibido: {}).",
                val
            ));
        }
        list.push(val);
    }
    if list.is_empty() {
        return Err("Debes indicar al menos una cantidad de dígitos.".to_string());
    }
    Ok(list)
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
            "--op" | "--operacion" | "--operation" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el tipo de operación después de --op".to_string())?;
                cli.operation = Some(parse_operation(val).ok_or_else(|| {
                    format!(
                        "Operación no reconocida: '{}'. Usa: multiplicacion, division, suma, resta o mixto.",
                        val
                    )
                })?);
                i += 1;
            }
            "--multiplicando" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número de dígitos después de --multiplicando".to_string())?;
                let d: usize = val
                    .parse()
                    .map_err(|_| format!("Valor inválido para --multiplicando: '{}'", val))?;
                if !(1..=5).contains(&d) {
                    return Err("El multiplicando debe tener entre 1 y 5 dígitos.".to_string());
                }
                cli.multiplicand_digits = Some(d);
                i += 1;
            }
            "--multiplicador" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número de dígitos después de --multiplicador".to_string())?;
                let d: usize = val
                    .parse()
                    .map_err(|_| format!("Valor inválido para --multiplicador: '{}'", val))?;
                if !(1..=4).contains(&d) {
                    return Err("El multiplicador debe tener entre 1 y 4 dígitos.".to_string());
                }
                cli.multiplier_digits = Some(d);
                i += 1;
            }
            "--dividendo" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número de dígitos después de --dividendo".to_string())?;
                let d: usize = val
                    .parse()
                    .map_err(|_| format!("Valor inválido para --dividendo: '{}'", val))?;
                if !(2..=5).contains(&d) {
                    return Err("El dividendo debe tener entre 2 y 5 dígitos.".to_string());
                }
                cli.dividend_digits = Some(d);
                i += 1;
            }
            "--divisor" | "--divisores" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número de dígitos después de --divisor".to_string())?;
                let d: usize = val
                    .parse()
                    .map_err(|_| format!("Valor inválido para --divisor: '{}'", val))?;
                if !(1..=3).contains(&d) {
                    return Err("El divisor debe tener entre 1 y 3 dígitos.".to_string());
                }
                cli.divisor_digits = Some(d);
                i += 1;
            }
            "--notacion" | "--notation" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el tipo de notación después de --notacion".to_string())?;
                cli.division_notation = Some(parse_notation(val).ok_or_else(|| {
                    format!(
                        "Notación de división no reconocida: '{}'. Usa 'tradicional' (caja) o 'galera' (casita).",
                        val
                    )
                })?);
                i += 1;
            }
            "--exacta" | "--exact" => {
                cli.exact_division = Some(true);
                i += 1;
            }
            "--sumandos" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta la cantidad de sumandos después de --sumandos".to_string())?;
                let c: usize = val
                    .parse()
                    .map_err(|_| format!("Valor inválido para --sumandos: '{}'", val))?;
                if !(2..=5).contains(&c) {
                    return Err("La cantidad de sumandos debe estar entre 2 y 5.".to_string());
                }
                cli.addend_count = Some(c);
                i += 1;
            }
            "--digitos" | "--digitos-sumando" | "--sumando" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta la cantidad de dígitos después de --digitos".to_string())?;
                cli.addend_digits = Some(parse_digits_list(val)?);
                i += 1;
            }
            "--minuendo" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número de dígitos después de --minuendo".to_string())?;
                let d: usize = val
                    .parse()
                    .map_err(|_| format!("Valor inválido para --minuendo: '{}'", val))?;
                if !(1..=5).contains(&d) {
                    return Err("El minuendo debe tener entre 1 y 5 dígitos.".to_string());
                }
                cli.minuend_digits = Some(d);
                i += 1;
            }
            "--sustraendo" | "--sustractor" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número de dígitos después de --sustraendo".to_string())?;
                let d: usize = val
                    .parse()
                    .map_err(|_| format!("Valor inválido para --sustraendo: '{}'", val))?;
                if !(1..=5).contains(&d) {
                    return Err("El sustraendo/sustractor debe tener entre 1 y 5 dígitos.".to_string());
                }
                cli.subtrahend_digits = Some(d);
                i += 1;
            }
            "-c" | "--cantidad" | "--ejercicios" | "--count" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el número de ejercicios después de --cantidad".to_string())?;
                let n: usize = val
                    .parse()
                    .map_err(|_| format!("Cantidad inválida: '{}'", val))?;
                if n == 0 {
                    return Err("La cantidad de ejercicios debe ser mayor a 0.".to_string());
                }
                cli.problem_count = Some(n);
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
            "-i" | "--idioma" | "--lang" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| "Falta el idioma (es o en) después de --idioma".to_string())?;
                cli.language = Some(parse_language(val).ok_or_else(|| {
                    format!("Idioma no válido: '{}'. Usa 'es' o 'en'.", val)
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
                    .ok_or_else(|| "Falta el número después de --semilla".to_string())?;
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
    println!("  GENERADOR AUTOMÁTICO DE FICHAS DIDÁCTICAS DE OPERACIONES ARITMÉTICAS");
    println!("=========================================================================");
    println!();
    println!("Uso interactivo (asistente paso a paso):");
    println!("  cargo run --bin operaciones");
    println!();
    println!("Uso directo por argumentos:");
    println!("  cargo run --bin operaciones -- --op <multiplicacion|division|suma|resta|mixto> [opciones]");
    println!();
    println!("Opciones generales:");
    println!("  --op <tipo>                  Operación: multiplicacion, division, suma, resta o mixto");
    println!("  -c, --cantidad <N>           Cantidad de ejercicios (por defecto: 12)");
    println!("  -f, --formato <a4|letter>    Tamaño de papel: 'a4' o 'letter' (por defecto: a4)");
    println!("  -i, --idioma <es|en>         Idioma de los títulos: 'es' o 'en' (por defecto: es)");
    println!("  -o, --salida <archivo.pdf>   Ruta del PDF generado (incluye hoja de respuestas al final)");
    println!("  -s, --semilla <u64>          Semilla opcional para reproducir la misma ficha");
    println!();
    println!("Complejidad para Multiplicación:");
    println!("  --multiplicando <1..5>       Cantidad de dígitos en el multiplicando (por defecto: 2)");
    println!("  --multiplicador <1..4>       Cantidad de dígitos en el multiplicador (por defecto: 2)");
    println!();
    println!("Complejidad para División:");
    println!("  --dividendo <2..5>           Cantidad de dígitos en el dividendo (por defecto: 3)");
    println!("  --divisor <1..3>             Cantidad de dígitos en el divisor (por defecto: 1)");
    println!("  --notacion <tradicional|galera>");
    println!("                               'tradicional': Dividendo | Divisor (caja hispana)");
    println!("                               'galera':      Divisor ⟌ Dividendo (casita)");
    println!("  --exacta                     Generar únicamente divisiones exactas (residuo 0)");
    println!();
    println!("Complejidad para Suma:");
    println!("  --sumandos <2..5>            Cantidad de sumandos por ejercicio (por defecto: 2)");
    println!("  --digitos <D o D1,D2,...>    Dígitos por sumando (ej. '3' o '3,2,3')");
    println!();
    println!("Complejidad para Resta:");
    println!("  --minuendo <1..5>            Cantidad de dígitos del minuendo (por defecto: 3)");
    println!("  --sustraendo <1..5>          Cantidad de dígitos del sustraendo/sustractor (por defecto: 2)");
}

fn prompt_line(msg: &str) -> String {
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
    buf.trim().to_string()
}

fn prompt_usize_in_range(msg: &str, default: usize, min: usize, max: usize) -> usize {
    loop {
        let input = prompt_line(msg);
        if input.is_empty() {
            return default;
        }
        if let Ok(v) = input.parse::<usize>() {
            if (min..=max).contains(&v) {
                return v;
            }
        }
        println!(
            "  Por favor, introduce un número entre {} y {}.",
            min, max
        );
    }
}

fn prompt_operation() -> OperationKind {
    loop {
        println!("\n¿Qué tipo de operación deseas generar?");
        println!("  1) Multiplicaciones");
        println!("  2) Divisiones");
        println!("  3) Sumas");
        println!("  4) Restas");
        println!("  5) Mixto (combinación de las 4 operaciones)");
        let input = prompt_line("Selecciona una opción (1-5) [por defecto: 1]: ");
        if input.is_empty() {
            return OperationKind::Multiplicacion;
        }
        if let Some(op) = parse_operation(&input) {
            return op;
        }
        println!("  Opción no válida. Escribe un número del 1 al 5 o el nombre de la operación.");
    }
}

fn prompt_division_notation() -> DivisionNotation {
    loop {
        println!("¿Qué notación deseas para las divisiones?");
        println!("  1) Caja tradicional (Dividendo | Divisor, cociente debajo del divisor)");
        println!("  2) Galera / Casita  (Divisor ⟌ Dividendo, cociente arriba del dividendo)");
        let input = prompt_line("Selecciona la notación (1 o 2) [por defecto: 1]: ");
        if input.is_empty() {
            return DivisionNotation::Tradicional;
        }
        if let Some(not) = parse_notation(&input) {
            return not;
        }
        println!("  Escribe '1' (tradicional) o '2' (galera).");
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

    let interactive = cli.operation.is_none();
    if interactive {
        println!("=========================================================================");
        println!("  GENERADOR DE MATERIALES DIDÁCTICOS: OPERACIONES ARITMÉTICAS EN PDF");
        println!("=========================================================================");
    }

    let operation = cli.operation.unwrap_or_else(prompt_operation);
    let mut cfg = WorksheetConfig {
        operation,
        ..WorksheetConfig::default()
    };

    // Configurar parámetros de Multiplicación
    if matches!(operation, OperationKind::Multiplicacion | OperationKind::Mixto) {
        let d1 = cli.multiplicand_digits.unwrap_or_else(|| {
            if interactive {
                prompt_usize_in_range(
                    "Cantidad de dígitos del multiplicando (1-5) [por defecto: 2]: ",
                    2,
                    1,
                    5,
                )
            } else {
                2
            }
        });
        let d2 = cli.multiplier_digits.unwrap_or_else(|| {
            if interactive {
                prompt_usize_in_range(
                    "Cantidad de dígitos del multiplicador (1-4) [por defecto: 2]: ",
                    2,
                    1,
                    4,
                )
            } else {
                2
            }
        });
        cfg.multiplication = MultiplicationConfig {
            multiplicand_digits: d1,
            multiplier_digits: d2,
        };
    }

    // Configurar parámetros de División
    if matches!(operation, OperationKind::Division | OperationKind::Mixto) {
        let dvd = cli.dividend_digits.unwrap_or_else(|| {
            if interactive {
                prompt_usize_in_range(
                    "Cantidad de dígitos del dividendo (2-5) [por defecto: 3]: ",
                    3,
                    2,
                    5,
                )
            } else {
                3
            }
        });
        let max_dsr = 3.min(dvd);
        let dsr = cli.divisor_digits.unwrap_or_else(|| {
            if interactive {
                prompt_usize_in_range(
                    &format!(
                        "Cantidad de dígitos del divisor (1-{}) [por defecto: 1]: ",
                        max_dsr
                    ),
                    1,
                    1,
                    max_dsr,
                )
            } else {
                1.min(max_dsr)
            }
        });
        let notation = cli.division_notation.unwrap_or_else(|| {
            if interactive {
                prompt_division_notation()
            } else {
                DivisionNotation::Tradicional
            }
        });
        let exact_only = cli.exact_division.unwrap_or_else(|| {
            if interactive {
                let ans = prompt_line(
                    "¿Generar solo divisiones exactas sin residuo? (s/n) [por defecto: n]: ",
                );
                matches!(ans.to_ascii_lowercase().as_str(), "s" | "si" | "sí" | "y" | "yes")
            } else {
                false
            }
        });
        cfg.division = DivisionConfig {
            dividend_digits: dvd,
            divisor_digits: dsr.min(dvd),
            notation,
            exact_only,
        };
    }

    // Configurar parámetros de Suma
    if matches!(operation, OperationKind::Suma | OperationKind::Mixto) {
        let count = cli.addend_count.unwrap_or_else(|| {
            if interactive {
                prompt_usize_in_range(
                    "Cantidad de sumandos (filas a sumar, 2-5) [por defecto: 2]: ",
                    2,
                    2,
                    5,
                )
            } else {
                2
            }
        });
        let digits_vec = match cli.addend_digits {
            Some(mut list) => {
                while list.len() < count {
                    let last = *list.last().unwrap_or(&3);
                    list.push(last);
                }
                list
            }
            None => {
                if interactive {
                    loop {
                        let raw = prompt_line(&format!(
                            "Cantidad de dígitos para los {} sumandos (ej. '3' o separados por coma '3,2') [por defecto: 3]: ",
                            count
                        ));
                        if raw.is_empty() {
                            break vec![3; count];
                        }
                        match parse_digits_list(&raw) {
                            Ok(mut list) => {
                                while list.len() < count {
                                    let last = *list.last().unwrap_or(&3);
                                    list.push(last);
                                }
                                break list;
                            }
                            Err(e) => println!("  {}", e),
                        }
                    }
                } else {
                    vec![3; count]
                }
            }
        };
        cfg.addition = AdditionConfig {
            addend_count: count,
            digits_per_addend: digits_vec,
        };
    }

    // Configurar parámetros de Resta
    if matches!(operation, OperationKind::Resta | OperationKind::Mixto) {
        let min_d = cli.minuend_digits.unwrap_or_else(|| {
            if interactive {
                prompt_usize_in_range(
                    "Cantidad de dígitos del minuendo (1-5) [por defecto: 3]: ",
                    3,
                    1,
                    5,
                )
            } else {
                3
            }
        });
        let sub_d = cli.subtrahend_digits.unwrap_or_else(|| {
            if interactive {
                prompt_usize_in_range(
                    &format!(
                        "Cantidad de dígitos del sustraendo / sustractor (1-{}) [por defecto: {}]: ",
                        min_d, min_d
                    ),
                    min_d,
                    1,
                    min_d,
                )
            } else {
                min_d
            }
        });
        cfg.subtraction = SubtractionConfig {
            minuend_digits: min_d,
            subtrahend_digits: sub_d.min(min_d),
        };
    }

    // Cantidad de ejercicios, formato de papel, idioma y archivo de salida
    cfg.problem_count = cli.problem_count.unwrap_or_else(|| {
        if interactive {
            prompt_usize_in_range(
                "¿Cuántos ejercicios deseas generar? (ej. 12, 24) [por defecto: 12]: ",
                12,
                1,
                120,
            )
        } else {
            12
        }
    });

    cfg.paper_size = cli.paper_size.unwrap_or_else(|| {
        if interactive {
            loop {
                let raw = prompt_line(
                    "¿Tamaño de hoja del PDF? (a4 / letter) [por defecto: a4]: ",
                );
                if raw.is_empty() {
                    break PaperSize::A4;
                }
                if let Some(p) = parse_paper_size(&raw) {
                    break p;
                }
                println!("  Escribe 'a4' o 'letter' (carta).");
            }
        } else {
            PaperSize::A4
        }
    });

    cfg.language = cli.language.unwrap_or_else(|| {
        if interactive {
            loop {
                let raw = prompt_line(
                    "¿Idioma del encabezado? (es: Español / en: English) [por defecto: es]: ",
                );
                if raw.is_empty() {
                    break WorksheetLanguage::Es;
                }
                if let Some(l) = parse_language(&raw) {
                    break l;
                }
                println!("  Escribe 'es' o 'en'.");
            }
        } else {
            WorksheetLanguage::Es
        }
    });

    let default_filename = match operation {
        OperationKind::Multiplicacion => "ficha_multiplicaciones.pdf",
        OperationKind::Division => "ficha_divisiones.pdf",
        OperationKind::Suma => "ficha_sumas.pdf",
        OperationKind::Resta => "ficha_restas.pdf",
        OperationKind::Mixto => "ficha_operaciones_mixtas.pdf",
    };

    let output_path = cli.output_path.unwrap_or_else(|| {
        if interactive {
            let raw = prompt_line(&format!(
                "Nombre del archivo PDF de salida [por defecto: {}]: ",
                default_filename
            ));
            if raw.is_empty() {
                PathBuf::from(default_filename)
            } else if raw.to_ascii_lowercase().ends_with(".pdf") {
                PathBuf::from(raw)
            } else {
                PathBuf::from(format!("{}.pdf", raw))
            }
        } else {
            PathBuf::from(default_filename)
        }
    });

    let mut rng = match cli.seed {
        Some(s) => Rng::from_seed(s),
        None => Rng::from_entropy(),
    };

    let problems = generate_worksheet_problems(&cfg, &mut rng);

    match write_math_worksheet_pdf(&output_path, &cfg, &problems) {
        Ok((total_pages, work_pages, ans_pages)) => {
            println!();
            println!("¡Material didáctico generado con éxito!");
            println!("  • Archivo PDF:          {}", output_path.display());
            println!("  • Operación:            {}", cfg.operation.title_es());
            if matches!(cfg.operation, OperationKind::Division | OperationKind::Mixto) {
                println!("  • Notación de división: {}", cfg.division.notation.label());
            }
            println!("  • Formato de hoja:      {}", cfg.paper_size.label());
            println!("  • Idioma de lámina:     {}", cfg.language.label());
            println!("  • Total de ejercicios:  {}", problems.len());
            println!(
                "  • Estructura del PDF:   {} páginas en total ({} de actividades + {} de hoja de respuestas al final)",
                total_pages, work_pages, ans_pages
            );
            println!();
            println!("Resumen de respuestas de los primeros ejercicios:");
            for (idx, prob) in problems.iter().take(6).enumerate() {
                println!("  {:>2}. {}", idx + 1, prob.summary_line());
            }
        }
        Err(e) => {
            eprintln!(
                "Error al escribir el archivo PDF '{}': {}",
                output_path.display(),
                e
            );
            std::process::exit(1);
        }
    }
}
