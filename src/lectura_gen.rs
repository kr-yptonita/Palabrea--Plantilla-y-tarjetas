use crate::lexicon::Rng;
use crate::pdf::PaperSize;
use std::fs;
use std::path::Path;

/// Categoría temática de las oraciones del banco de datos.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SentenceTheme {
    Todos,
    Animales,
    Naturaleza,
    VidaCotidiana,
    EscuelaYCiencia,
}

impl SentenceTheme {
    pub fn label(&self) -> &'static str {
        match self {
            SentenceTheme::Todos => "Todos los temas (Mixto)",
            SentenceTheme::Animales => "Animales y seres vivos",
            SentenceTheme::Naturaleza => "Naturaleza y entorno",
            SentenceTheme::VidaCotidiana => "Vida cotidiana, familia y juegos",
            SentenceTheme::EscuelaYCiencia => "Escuela, lectura y ciencia",
        }
    }
}

/// Modo de selección de la palabra que se ocultará al azar en cada frase.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlankSelectionMode {
    /// Elige al azar cualquiera de las palabras clave (sujeto, verbo, objeto, adjetivo, complemento) de la oración.
    AzarPalabrasClave,
    /// Elige al azar cualquier palabra de contenido (>= 3 letras) de la oración y genera distractores morfológicos.
    AzarCualquierPalabra,
    /// Oculta preferentemente la última palabra clave de la oración (estilo clásico de ficha).
    UltimaPalabra,
}

impl BlankSelectionMode {
    pub fn label(&self) -> &'static str {
        match self {
            BlankSelectionMode::AzarPalabrasClave => {
                "Al azar entre las palabras clave de la frase (sujeto, acción, objeto o cualidad)"
            }
            BlankSelectionMode::AzarCualquierPalabra => {
                "Al azar entre cualquier palabra de contenido de la frase"
            }
            BlankSelectionMode::UltimaPalabra => {
                "Palabra final de la oración (estilo ficha clásica)"
            }
        }
    }
}

/// Modalidad del ejercicio en la hoja impresa.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExerciseMode {
    /// Incluye incisos de opción múltiple (a, b, c) debajo de cada oración (como en la lámina de referencia).
    OpcionesMultiples,
    /// Incluye un recuadro con banco de palabras en la parte superior y oraciones para completar sin incisos.
    BancoDePalabras,
    /// Solo la oración con el espacio en blanco para escritura libre.
    CompletarLibre,
}

impl ExerciseMode {
    pub fn label(&self) -> &'static str {
        match self {
            ExerciseMode::OpcionesMultiples => "Opción múltiple (a, b, c) + escribir en la línea",
            ExerciseMode::BancoDePalabras => "Recuadro con banco de palabras + escribir en la línea",
            ExerciseMode::CompletarLibre => "Completar libremente la palabra faltante",
        }
    }

    pub fn default_instruction(&self) -> &'static str {
        match self {
            ExerciseMode::OpcionesMultiples => "Subraya y escribe la respuesta correcta.",
            ExerciseMode::BancoDePalabras => {
                "Elige la palabra correcta del recuadro y escríbela en cada oración."
            }
            ExerciseMode::CompletarLibre => {
                "Lee cada oración y escribe la palabra que falta en la línea."
            }
        }
    }
}

/// Estilo visual de la ficha (siempre sin adornos ni dibujos).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VisualStyle {
    /// Diseño limpio en escala de grises / tinta sobria ideal para impresión directa sin distracciones.
    Sobrio,
    /// Franjas con fondos suaves pastel para cada renglón, sin dibujos ni bordes decorativos.
    PastelSinAdornos,
}

impl VisualStyle {
    pub fn label(&self) -> &'static str {
        match self {
            VisualStyle::Sobrio => "Sobrio (blanco y negro / escala de grises limpia)",
            VisualStyle::PastelSinAdornos => "Franjas suaves de color (sin adornos ni dibujos)",
        }
    }
}

/// Configuración general para generar una ficha de Lectura y Escritura.
#[derive(Clone, Debug)]
pub struct ReadingWorksheetConfig {
    pub title: String,
    pub subtitle: Option<String>,
    pub ficha_label: String,
    pub exercise_count: usize,
    pub exercises_per_page: usize,
    pub options_count: usize,
    pub theme: SentenceTheme,
    pub blank_mode: BlankSelectionMode,
    pub exercise_mode: ExerciseMode,
    pub visual_style: VisualStyle,
    pub paper_size: PaperSize,
    pub include_answer_key: bool,
    pub custom_sentences: Vec<String>,
}

impl Default for ReadingWorksheetConfig {
    fn default() -> Self {
        Self {
            title: "Lectura y escritura".to_string(),
            subtitle: None,
            ficha_label: "Ficha Uno".to_string(),
            exercise_count: 7,
            exercises_per_page: 7,
            options_count: 3,
            theme: SentenceTheme::Todos,
            blank_mode: BlankSelectionMode::AzarPalabrasClave,
            exercise_mode: ExerciseMode::OpcionesMultiples,
            visual_style: VisualStyle::Sobrio,
            paper_size: PaperSize::A4,
            include_answer_key: true,
            custom_sentences: Vec::new(),
        }
    }
}

impl ReadingWorksheetConfig {
    pub fn effective_instruction(&self) -> &str {
        if let Some(ref custom) = self.subtitle {
            if !custom.trim().is_empty() {
                return custom.as_str();
            }
        }
        self.exercise_mode.default_instruction()
    }
}

/// Una palabra candidata dentro de una oración del banco, con distractores semántica y gramaticalmente coherentes.
#[derive(Copy, Clone, Debug)]
pub struct RemovableSlot {
    pub target_word: &'static str,
    pub distractors: &'static [&'static str],
}

/// Entrada del banco interno de oraciones didácticas.
#[derive(Copy, Clone, Debug)]
pub struct SentenceBankEntry {
    pub sentence: &'static str,
    pub theme: SentenceTheme,
    pub slots: &'static [RemovableSlot],
}

/// Un ejercicio individual ya generado con una palabra eliminada al azar y sus opciones mezcladas.
#[derive(Clone, Debug)]
pub struct GeneratedSentenceExercise {
    /// Número secuencial del ejercicio (1, 2, 3...).
    pub number: usize,
    /// Oración original completa.
    pub full_sentence: String,
    /// Fragmento de la oración antes de la palabra eliminada.
    pub prefix: String,
    /// Palabra exacta que fue eliminada al azar (respuesta correcta).
    pub missing_word: String,
    /// Fragmento de la oración después de la palabra eliminada (incluyendo puntuación final).
    pub suffix: String,
    /// Lista de opciones mezcladas (ej. para a), b), c)).
    pub options: Vec<String>,
    /// Índice dentro de `options` que corresponde a `missing_word`.
    pub correct_option_index: usize,
}

impl GeneratedSentenceExercise {
    /// Devuelve la letra del inciso correcto ('a', 'b', 'c', 'd').
    pub fn correct_letter(&self) -> char {
        (b'a' + (self.correct_option_index as u8)) as char
    }

    /// Representación en texto plano para vista previa en consola.
    pub fn summary_line(&self) -> String {
        let blanked = format!("{}______{}", self.prefix, self.suffix);
        if self.options.is_empty() {
            format!("{}  -> [Resp: {}]", blanked, self.missing_word)
        } else {
            let opts_str: Vec<String> = self
                .options
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    let letter = (b'a' + (i as u8)) as char;
                    if i == self.correct_option_index {
                        format!("{}) *{}*", letter, w)
                    } else {
                        format!("{}) {}", letter, w)
                    }
                })
                .collect();
            format!("{}   [{}]", blanked, opts_str.join("  "))
        }
    }
}

// =============================================================================
// BANCO DE DATOS DE ORACIONES DIDÁCTICAS (MÚLTIPLES PALABRAS REMOVIBLES POR FRASE)
// =============================================================================

pub const SENTENCE_BANK: &[SentenceBankEntry] = &[
    // --- Las 7 frases clásicas de la lámina de referencia (con múltiples huecos posibles) ---
    SentenceBankEntry {
        sentence: "El gato está durmiendo.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "gato",
                distractors: &["perro", "conejo", "caballo", "pato"],
            },
            RemovableSlot {
                target_word: "durmiendo",
                distractors: &["corriendo", "saltando", "nadando", "volando"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La niña tiene un globo rojo.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "niña",
                distractors: &["maestra", "abuela", "doctora", "prima"],
            },
            RemovableSlot {
                target_word: "tiene",
                distractors: &["busca", "mira", "pinta", "guarda"],
            },
            RemovableSlot {
                target_word: "globo",
                distractors: &["sombrero", "cuaderno", "zapato", "reloj"],
            },
            RemovableSlot {
                target_word: "rojo",
                distractors: &["azul", "verde", "amarillo", "blanco"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El elefante tiene una trompa larga.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "elefante",
                distractors: &["ratón", "conejo", "pollito", "gato"],
            },
            RemovableSlot {
                target_word: "trompa",
                distractors: &["pluma", "aleta", "cresta", "antena"],
            },
            RemovableSlot {
                target_word: "larga",
                distractors: &["corta", "grande", "pequeña", "redonda"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El sol es una fuente de luz.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "sol",
                distractors: &["viento", "bosque", "río", "mar"],
            },
            RemovableSlot {
                target_word: "fuente",
                distractors: &["sombra", "piedra", "rama", "hoja"],
            },
            RemovableSlot {
                target_word: "luz",
                distractors: &["agua", "comida", "tierra", "arena"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El niño juega con una pelota.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "niño",
                distractors: &["abuelo", "médico", "bombero", "panadero"],
            },
            RemovableSlot {
                target_word: "juega",
                distractors: &["duerme", "cocina", "estudia", "escribe"],
            },
            RemovableSlot {
                target_word: "pelota",
                distractors: &["bicicleta", "muñeca", "cuchara", "lámpara"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La mariposa tiene alas de color amarillo.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "mariposa",
                distractors: &["tortuga", "ballena", "serpiente", "lombríz"],
            },
            RemovableSlot {
                target_word: "alas",
                distractors: &["patas", "orejas", "ruedas", "ramas"],
            },
            RemovableSlot {
                target_word: "amarillo",
                distractors: &["azul", "rojo", "verde", "morado"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La vaca da leche.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "vaca",
                distractors: &["gallina", "abeja", "paloma", "rana"],
            },
            RemovableSlot {
                target_word: "leche",
                distractors: &["huevos", "miel", "plumas", "semillas"],
            },
        ],
    },
    // --- Nuevas oraciones de Animales ---
    SentenceBankEntry {
        sentence: "El perro ladra fuerte en el jardín.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "perro",
                distractors: &["gato", "conejo", "canario", "pez"],
            },
            RemovableSlot {
                target_word: "ladra",
                distractors: &["maúlla", "vuela", "nada", "escribe"],
            },
            RemovableSlot {
                target_word: "fuerte",
                distractors: &["lento", "suave", "dulce", "corto"],
            },
            RemovableSlot {
                target_word: "jardín",
                distractors: &["tejado", "océano", "cuaderno", "bolsillo"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La abeja produce miel dulce en la colmena.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "abeja",
                distractors: &["hormiga", "araña", "tortuga", "jirafa"],
            },
            RemovableSlot {
                target_word: "produce",
                distractors: &["compra", "rompe", "pinta", "corta"],
            },
            RemovableSlot {
                target_word: "miel",
                distractors: &["leche", "sal", "harina", "arena"],
            },
            RemovableSlot {
                target_word: "dulce",
                distractors: &["salada", "amarga", "fría", "dura"],
            },
            RemovableSlot {
                target_word: "colmena",
                distractors: &["pecera", "escuela", "cocina", "mochila"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Los peces nadan rápido en el río limpio.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "peces",
                distractors: &["pájaros", "gatos", "caballos", "conejos"],
            },
            RemovableSlot {
                target_word: "nadan",
                distractors: &["vuelan", "caminan", "cantan", "tejen"],
            },
            RemovableSlot {
                target_word: "río",
                distractors: &["desierto", "tejado", "árbol", "camino"],
            },
            RemovableSlot {
                target_word: "limpio",
                distractors: &["seco", "vacío", "roto", "cuadrado"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El conejo blanco come una zanahoria fresca.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "conejo",
                distractors: &["tiburón", "águila", "león", "delfín"],
            },
            RemovableSlot {
                target_word: "blanco",
                distractors: &["verde", "azul", "morado", "rojo"],
            },
            RemovableSlot {
                target_word: "come",
                distractors: &["escribe", "lee", "dibuja", "cose"],
            },
            RemovableSlot {
                target_word: "zanahoria",
                distractors: &["piedra", "silla", "moneda", "llave"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La gallina pone huevos blancos en el nido.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "gallina",
                distractors: &["oveja", "vaca", "cebra", "cabra"],
            },
            RemovableSlot {
                target_word: "huevos",
                distractors: &["libros", "zapatos", "vasos", "botones"],
            },
            RemovableSlot {
                target_word: "nido",
                distractors: &["río", "coche", "armario", "teclado"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El caballo galopa veloz por la pradera verde.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "caballo",
                distractors: &["caracol", "tortuga", "pingüino", "cangrejo"],
            },
            RemovableSlot {
                target_word: "galopa",
                distractors: &["vuela", "bucea", "rema", "pinta"],
            },
            RemovableSlot {
                target_word: "pradera",
                distractors: &["ventana", "lámpara", "mesa", "pizarra"],
            },
            RemovableSlot {
                target_word: "verde",
                distractors: &["azul", "gris", "violeta", "negra"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La tortuga camina lento con su caparazón duro.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "tortuga",
                distractors: &["gacela", "águila", "liebre", "paloma"],
            },
            RemovableSlot {
                target_word: "lento",
                distractors: &["rápido", "alto", "lejos", "fuerte"],
            },
            RemovableSlot {
                target_word: "caparazón",
                distractors: &["sombrero", "paraguas", "abrigo", "pañuelo"],
            },
            RemovableSlot {
                target_word: "duro",
                distractors: &["blando", "dulce", "líquido", "suave"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El pájaro construye su nido en la rama alta.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "pájaro",
                distractors: &["delfín", "caballo", "perro", "pulpo"],
            },
            RemovableSlot {
                target_word: "construye",
                distractors: &["rompe", "borra", "apaga", "congela"],
            },
            RemovableSlot {
                target_word: "nido",
                distractors: &["barco", "zapato", "reloj", "plato"],
            },
            RemovableSlot {
                target_word: "rama",
                distractors: &["cuchara", "alfombra", "mochila", "silla"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La oveja tiene mucha lana suave y blanca.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "oveja",
                distractors: &["rana", "serpiente", "sardina", "tortuga"],
            },
            RemovableSlot {
                target_word: "lana",
                distractors: &["madera", "arena", "piedra", "cristal"],
            },
            RemovableSlot {
                target_word: "suave",
                distractors: &["áspera", "dura", "amarga", "pesada"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El león ruge con fuerza en la selva.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "león",
                distractors: &["pollito", "canario", "ratón", "conejo"],
            },
            RemovableSlot {
                target_word: "ruge",
                distractors: &["pía", "maúlla", "zumba", "croa"],
            },
            RemovableSlot {
                target_word: "selva",
                distractors: &["cocina", "biblioteca", "oficina", "farmacia"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La rana verde salta sobre la hoja del estanque.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "rana",
                distractors: &["jirafa", "ballena", "vaca", "cebra"],
            },
            RemovableSlot {
                target_word: "salta",
                distractors: &["escribe", "plancha", "maneja", "habla"],
            },
            RemovableSlot {
                target_word: "hoja",
                distractors: &["nube", "estrella", "lámpara", "ventana"],
            },
            RemovableSlot {
                target_word: "estanque",
                distractors: &["armario", "tejado", "comedor", "cuaderno"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El delfín salta alegre sobre las olas del mar.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "delfín",
                distractors: &["camello", "gato", "topo", "loro"],
            },
            RemovableSlot {
                target_word: "olas",
                distractors: &["ramas", "montañas", "paredes", "escaleras"],
            },
            RemovableSlot {
                target_word: "mar",
                distractors: &["desierto", "bosque", "patio", "jardín"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La ardilla guarda bellotas secas en el árbol.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "ardilla",
                distractors: &["ballena", "medusa", "foca", "estrella"],
            },
            RemovableSlot {
                target_word: "guarda",
                distractors: &["cocina", "escribe", "dibuja", "canta"],
            },
            RemovableSlot {
                target_word: "bellotas",
                distractors: &["monedas", "tijeras", "cucharas", "llaves"],
            },
            RemovableSlot {
                target_word: "árbol",
                distractors: &["autobús", "teléfono", "espejo", "zapato"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El pato amarillo nada tranquilo en la laguna.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "pato",
                distractors: &["gato", "camello", "halcón", "león"],
            },
            RemovableSlot {
                target_word: "nada",
                distractors: &["camina", "escribe", "habla", "pinta"],
            },
            RemovableSlot {
                target_word: "laguna",
                distractors: &["montaña", "biblioteca", "ventana", "escalera"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La jirafa alcanza las hojas altas con su cuello largo.",
        theme: SentenceTheme::Animales,
        slots: &[
            RemovableSlot {
                target_word: "jirafa",
                distractors: &["tortuga", "hormiga", "rana", "conejo"],
            },
            RemovableSlot {
                target_word: "hojas",
                distractors: &["raíces", "piedras", "conchas", "arenas"],
            },
            RemovableSlot {
                target_word: "cuello",
                distractors: &["pico", "caparazón", "aleta", "pluma"],
            },
            RemovableSlot {
                target_word: "largo",
                distractors: &["corto", "redondo", "cuadrado", "ancho"],
            },
        ],
    },
    // --- Naturaleza y Entorno ---
    SentenceBankEntry {
        sentence: "La lluvia moja las flores del jardín en primavera.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "lluvia",
                distractors: &["arena", "sombra", "piedra", "música"],
            },
            RemovableSlot {
                target_word: "moja",
                distractors: &["seca", "quema", "escribe", "dobla"],
            },
            RemovableSlot {
                target_word: "flores",
                distractors: &["nubes", "estrellas", "lámparas", "sillas"],
            },
            RemovableSlot {
                target_word: "primavera",
                distractors: &["cuaderno", "mochila", "ventana", "cocina"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La luna brilla de noche en el cielo oscuro.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "luna",
                distractors: &["silla", "raíz", "mesa", "puerta"],
            },
            RemovableSlot {
                target_word: "noche",
                distractors: &["mañana", "tarde", "escuela", "mesa"],
            },
            RemovableSlot {
                target_word: "cielo",
                distractors: &["suelo", "bolsillo", "cajón", "plato"],
            },
            RemovableSlot {
                target_word: "oscuro",
                distractors: &["verde", "amarillo", "salado", "blando"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Los árboles tienen raíces profundas bajo la tierra.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "árboles",
                distractors: &["coches", "aviones", "zapatos", "relojes"],
            },
            RemovableSlot {
                target_word: "raíces",
                distractors: &["ruedas", "alas", "ventanas", "antenas"],
            },
            RemovableSlot {
                target_word: "tierra",
                distractors: &["nube", "mesa", "pizarra", "lámpara"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "En invierno cae nieve blanca sobre las montañas.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "invierno",
                distractors: &["verano", "recreo", "almuerzo", "domingo"],
            },
            RemovableSlot {
                target_word: "nieve",
                distractors: &["arena", "harina", "espuma", "tinta"],
            },
            RemovableSlot {
                target_word: "blanca",
                distractors: &["negra", "roja", "verde", "morada"],
            },
            RemovableSlot {
                target_word: "montañas",
                distractors: &["alfombras", "libretas", "cucharas", "botellas"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El arcoíris aparece después de la lluvia con siete colores.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "arcoíris",
                distractors: &["relámpago", "remolino", "viento", "granizo"],
            },
            RemovableSlot {
                target_word: "lluvia",
                distractors: &["cena", "lectura", "tarea", "siesta"],
            },
            RemovableSlot {
                target_word: "siete",
                distractors: &["dos", "cien", "mil", "cero"],
            },
            RemovableSlot {
                target_word: "colores",
                distractors: &["sabores", "sonidos", "botones", "zapatos"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El viento mueve las hojas secas en otoño.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "viento",
                distractors: &["hielo", "fuego", "suelo", "espejo"],
            },
            RemovableSlot {
                target_word: "mueve",
                distractors: &["pinta", "escribe", "cocina", "cose"],
            },
            RemovableSlot {
                target_word: "hojas",
                distractors: &["piedras", "montañas", "casas", "mesas"],
            },
            RemovableSlot {
                target_word: "otoño",
                distractors: &["cocina", "escuela", "mercado", "hospital"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Las semillas necesitan agua y tierra para crecer.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "semillas",
                distractors: &["piedras", "monedas", "canicas", "llaves"],
            },
            RemovableSlot {
                target_word: "agua",
                distractors: &["humo", "pintura", "papel", "plástico"],
            },
            RemovableSlot {
                target_word: "tierra",
                distractors: &["vidrio", "metal", "cartón", "tela"],
            },
            RemovableSlot {
                target_word: "crecer",
                distractors: &["dormir", "hablar", "correr", "leer"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El río lleva agua fresca desde la montaña hasta el mar.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "río",
                distractors: &["camino", "puente", "árbol", "edificio"],
            },
            RemovableSlot {
                target_word: "agua",
                distractors: &["fuego", "arena", "humo", "algodón"],
            },
            RemovableSlot {
                target_word: "montaña",
                distractors: &["cocina", "mochila", "ventana", "silla"],
            },
            RemovableSlot {
                target_word: "mar",
                distractors: &["tejado", "armario", "cuaderno", "reloj"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El girasol busca la luz del sol durante el día.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "girasol",
                distractors: &["caracol", "paraguas", "sombrero", "zapato"],
            },
            RemovableSlot {
                target_word: "luz",
                distractors: &["nieve", "sombra", "arena", "sal"],
            },
            RemovableSlot {
                target_word: "sol",
                distractors: &["viento", "trueno", "relámpago", "granizo"],
            },
            RemovableSlot {
                target_word: "día",
                distractors: &["invierno", "sueño", "silencio", "bolsillo"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Las nubes blancas flotan suaves en el cielo azul.",
        theme: SentenceTheme::Naturaleza,
        slots: &[
            RemovableSlot {
                target_word: "nubes",
                distractors: &["piedras", "raíces", "tortugas", "sillas"],
            },
            RemovableSlot {
                target_word: "flotan",
                distractors: &["caminan", "escarban", "corren", "escriben"],
            },
            RemovableSlot {
                target_word: "cielo",
                distractors: &["suelo", "sótano", "jardín", "camino"],
            },
            RemovableSlot {
                target_word: "azul",
                distractors: &["verde", "marrón", "amarillo", "cuadrado"],
            },
        ],
    },
    // --- Vida Cotidiana, Familia y Juegos ---
    SentenceBankEntry {
        sentence: "Mi mamá prepara una sopa caliente en la cocina.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "mamá",
                distractors: &["mascota", "muñeca", "bicicleta", "pelota"],
            },
            RemovableSlot {
                target_word: "prepara",
                distractors: &["escribe", "dibuja", "barre", "recorta"],
            },
            RemovableSlot {
                target_word: "sopa",
                distractors: &["piedra", "libreta", "camisa", "escoba"],
            },
            RemovableSlot {
                target_word: "caliente",
                distractors: &["cuadrada", "azul", "vacía", "rota"],
            },
            RemovableSlot {
                target_word: "cocina",
                distractors: &["banqueta", "azotea", "cochera", "biblioteca"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El abuelo lee un cuento interesante en el sillón.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "abuelo",
                distractors: &["perro", "canario", "florero", "reloj"],
            },
            RemovableSlot {
                target_word: "lee",
                distractors: &["cocina", "riega", "barre", "salta"],
            },
            RemovableSlot {
                target_word: "cuento",
                distractors: &["zapato", "plato", "sombrero", "tenedor"],
            },
            RemovableSlot {
                target_word: "sillón",
                distractors: &["tejado", "árbol", "refrigerador", "horno"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Los niños corren alegres por el parque en la tarde.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "niños",
                distractors: &["árboles", "edificios", "muebles", "libros"],
            },
            RemovableSlot {
                target_word: "corren",
                distractors: &["duermen", "nadan", "vuelan", "cocinan"],
            },
            RemovableSlot {
                target_word: "parque",
                distractors: &["armario", "tejado", "horno", "cajón"],
            },
            RemovableSlot {
                target_word: "tarde",
                distractors: &["mochila", "mesa", "ventana", "puerta"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Me lavo las manos con agua y jabón antes de comer.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "manos",
                distractors: &["zapatos", "cuadernos", "lápices", "juguetes"],
            },
            RemovableSlot {
                target_word: "jabón",
                distractors: &["azúcar", "harina", "pintura", "pegamento"],
            },
            RemovableSlot {
                target_word: "comer",
                distractors: &["dormir", "correr", "saltar", "gritar"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El panadero hornea pan fresco todas las mañanas.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "panadero",
                distractors: &["carpintero", "bombero", "cartero", "marinero"],
            },
            RemovableSlot {
                target_word: "hornea",
                distractors: &["clava", "pinta", "barre", "lava"],
            },
            RemovableSlot {
                target_word: "pan",
                distractors: &["papel", "carbón", "hielo", "algodón"],
            },
            RemovableSlot {
                target_word: "mañanas",
                distractors: &["ventanas", "escaleras", "esquinas", "paredes"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La familia desayuna fruta dulce y leche en la mesa.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "familia",
                distractors: &["lámpara", "ventana", "escalera", "bicicleta"],
            },
            RemovableSlot {
                target_word: "fruta",
                distractors: &["madera", "tierra", "pintura", "lana"],
            },
            RemovableSlot {
                target_word: "leche",
                distractors: &["tinta", "arena", "jabón", "aceite"],
            },
            RemovableSlot {
                target_word: "mesa",
                distractors: &["calle", "banqueta", "escalera", "ventana"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Cuando llueve usamos el paraguas para no mojarnos.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "llueve",
                distractors: &["amanece", "oscurece", "calienta", "sopla"],
            },
            RemovableSlot {
                target_word: "paraguas",
                distractors: &["abanico", "tenedor", "cepillo", "espejo"],
            },
            RemovableSlot {
                target_word: "mojarnos",
                distractors: &["dormirnos", "peinarnos", "sentarnos", "reírnos"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "En la noche me cepillo los dientes antes de dormir.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "noche",
                distractors: &["escuela", "calle", "plaza", "tienda"],
            },
            RemovableSlot {
                target_word: "cepillo",
                distractors: &["pinto", "amarro", "recorto", "guardo"],
            },
            RemovableSlot {
                target_word: "dientes",
                distractors: &["zapatos", "guantes", "bolsillos", "botones"],
            },
            RemovableSlot {
                target_word: "dormir",
                distractors: &["correr", "nadar", "saltar", "barrer"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Mi hermana vuela una cometa de colores en el campo.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "hermana",
                distractors: &["tortuga", "lámpara", "mochila", "ventana"],
            },
            RemovableSlot {
                target_word: "cometa",
                distractors: &["cuchara", "almohada", "butaca", "carpeta"],
            },
            RemovableSlot {
                target_word: "campo",
                distractors: &["armario", "pasillo", "sótano", "baño"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El reloj de la pared marca la hora con sus agujas.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "reloj",
                distractors: &["espejo", "cuadro", "florero", "sillón"],
            },
            RemovableSlot {
                target_word: "pared",
                distractors: &["sopa", "lluvia", "nube", "arena"],
            },
            RemovableSlot {
                target_word: "hora",
                distractors: &["temperatura", "distancia", "altura", "canción"],
            },
            RemovableSlot {
                target_word: "agujas",
                distractors: &["ruedas", "plumas", "hojas", "raíces"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Guardamos la ropa limpia dentro del armario de madera.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "ropa",
                distractors: &["comida", "sopa", "leche", "fruta"],
            },
            RemovableSlot {
                target_word: "limpia",
                distractors: &["mojada", "rota", "líquida", "salada"],
            },
            RemovableSlot {
                target_word: "armario",
                distractors: &["horno", "jardín", "tejado", "parque"],
            },
            RemovableSlot {
                target_word: "madera",
                distractors: &["papel", "algodón", "espuma", "hielo"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El bombero apaga el fuego con una manguera grande.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "bombero",
                distractors: &["panadero", "músico", "sastre", "pintor"],
            },
            RemovableSlot {
                target_word: "apaga",
                distractors: &["enciende", "pinta", "corta", "dibuja"],
            },
            RemovableSlot {
                target_word: "fuego",
                distractors: &["viento", "hielo", "sonido", "reloj"],
            },
            RemovableSlot {
                target_word: "manguera",
                distractors: &["cuchara", "tijera", "bufanda", "regla"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "En el mercado compramos manzanas rojas y plátanos.",
        theme: SentenceTheme::VidaCotidiana,
        slots: &[
            RemovableSlot {
                target_word: "mercado",
                distractors: &["hospital", "teatro", "museo", "estadio"],
            },
            RemovableSlot {
                target_word: "compramos",
                distractors: &["barremos", "dormimos", "volamos", "nadamos"],
            },
            RemovableSlot {
                target_word: "manzanas",
                distractors: &["libretas", "piedras", "lámparas", "sillas"],
            },
            RemovableSlot {
                target_word: "plátanos",
                distractors: &["zapatos", "martillos", "espejos", "paraguas"],
            },
        ],
    },
    // --- Escuela, Lectura y Ciencia ---
    SentenceBankEntry {
        sentence: "La maestra escribe palabras nuevas en el pizarrón.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "maestra",
                distractors: &["enfermera", "cocinera", "piloto", "jardinera"],
            },
            RemovableSlot {
                target_word: "escribe",
                distractors: &["cocina", "riega", "maneja", "cose"],
            },
            RemovableSlot {
                target_word: "palabras",
                distractors: &["manzanas", "pelotas", "ventana", "escaleras"],
            },
            RemovableSlot {
                target_word: "pizarrón",
                distractors: &["refrigerador", "colchón", "jardín", "paraguas"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El alumno guarda sus cuadernos y lápices en la mochila.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "alumno",
                distractors: &["marinero", "bombero", "panadero", "plomero"],
            },
            RemovableSlot {
                target_word: "cuadernos",
                distractors: &["platillos", "floreros", "cojines", "patines"],
            },
            RemovableSlot {
                target_word: "lápices",
                distractors: &["tenedores", "cucharones", "zapatos", "calcetines"],
            },
            RemovableSlot {
                target_word: "mochila",
                distractors: &["cocina", "pecera", "maceta", "ventana"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Usamos las tijeras para recortar figuras de papel.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "tijeras",
                distractors: &["cucharas", "almohadas", "botellas", "linternas"],
            },
            RemovableSlot {
                target_word: "recortar",
                distractors: &["pegar", "calentar", "beber", "escuchar"],
            },
            RemovableSlot {
                target_word: "papel",
                distractors: &["agua", "humo", "viento", "sonido"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "En la biblioteca leemos libros de cuentos en silencio.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "biblioteca",
                distractors: &["panadería", "cancha", "alberca", "cocina"],
            },
            RemovableSlot {
                target_word: "leemos",
                distractors: &["saltamos", "gritamos", "pateamos", "cocinamos"],
            },
            RemovableSlot {
                target_word: "libros",
                distractors: &["balones", "zapatos", "platos", "floreros"],
            },
            RemovableSlot {
                target_word: "silencio",
                distractors: &["ruido", "carrera", "lluvia", "viento"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El corazón late todo el día para impulsar la sangre.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "corazón",
                distractors: &["cabello", "tobillo", "codo", "hombro"],
            },
            RemovableSlot {
                target_word: "late",
                distractors: &["mastica", "camina", "parpadea", "escribe"],
            },
            RemovableSlot {
                target_word: "sangre",
                distractors: &["saliva", "lágrima", "sombra", "voz"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Con los ojos podemos ver los colores y las formas.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "ojos",
                distractors: &["oídos", "dientes", "codos", "pies"],
            },
            RemovableSlot {
                target_word: "ver",
                distractors: &["oler", "masticar", "escuchar", "patear"],
            },
            RemovableSlot {
                target_word: "colores",
                distractors: &["sonidos", "olores", "sabores", "ecos"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La Tierra gira alrededor del Sol durante todo el año.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "Tierra",
                distractors: &["nube", "montaña", "ciudad", "escuela"],
            },
            RemovableSlot {
                target_word: "gira",
                distractors: &["duerme", "salta", "canta", "escribe"],
            },
            RemovableSlot {
                target_word: "Sol",
                distractors: &["mar", "bosque", "viento", "río"],
            },
            RemovableSlot {
                target_word: "año",
                distractors: &["minuto", "segundo", "recreo", "cuaderno"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El termómetro sirve para medir la temperatura.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "termómetro",
                distractors: &["telescopio", "micrófono", "calendario", "sacapuntas"],
            },
            RemovableSlot {
                target_word: "medir",
                distractors: &["pintar", "recortar", "coser", "barrer"],
            },
            RemovableSlot {
                target_word: "temperatura",
                distractors: &["distancia", "música", "lectura", "velocidad"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Pintamos un paisaje hermoso con acuarelas y pincel.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "paisaje",
                distractors: &["sonido", "silencio", "minuto", "secreto"],
            },
            RemovableSlot {
                target_word: "acuarelas",
                distractors: &["tijeras", "monedas", "canicas", "llaves"],
            },
            RemovableSlot {
                target_word: "pincel",
                distractors: &["tenedor", "martillo", "candado", "silbato"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La guitarra produce música cuando tocamos sus cuerdas.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "guitarra",
                distractors: &["trompeta", "flauta", "campana", "tambor"],
            },
            RemovableSlot {
                target_word: "música",
                distractors: &["pintura", "harina", "sombra", "nieve"],
            },
            RemovableSlot {
                target_word: "cuerdas",
                distractors: &["ruedas", "ventanas", "páginas", "manecillas"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "La regla nos ayuda a trazar líneas rectas en el cuaderno.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "regla",
                distractors: &["goma", "mochila", "campana", "esponja"],
            },
            RemovableSlot {
                target_word: "líneas",
                distractors: &["canciones", "comidas", "plantas", "bebidas"],
            },
            RemovableSlot {
                target_word: "rectas",
                distractors: &["dulces", "calientes", "blandas", "ruidosas"],
            },
            RemovableSlot {
                target_word: "cuaderno",
                distractors: &["zapato", "sombrero", "plato", "vaso"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "Con los oídos escuchamos el canto de las aves.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "oídos",
                distractors: &["ojos", "dedos", "labios", "hombros"],
            },
            RemovableSlot {
                target_word: "escuchamos",
                distractors: &["pintamos", "masticamos", "tocamos", "olemos"],
            },
            RemovableSlot {
                target_word: "canto",
                distractors: &["color", "sabor", "tamaño", "peso"],
            },
            RemovableSlot {
                target_word: "aves",
                distractors: &["piedras", "mesas", "nubes", "puertas"],
            },
        ],
    },
    SentenceBankEntry {
        sentence: "El agua se convierte en hielo cuando hace mucho frío.",
        theme: SentenceTheme::EscuelaYCiencia,
        slots: &[
            RemovableSlot {
                target_word: "agua",
                distractors: &["papel", "carbón", "madera", "algodón"],
            },
            RemovableSlot {
                target_word: "hielo",
                distractors: &["fuego", "arena", "harina", "humo"],
            },
            RemovableSlot {
                target_word: "frío",
                distractors: &["calor", "ruido", "sueño", "viento"],
            },
        ],
    },
];

// =============================================================================
// GENERADOR DE DISTRACTORES MORFOLÓGICOS PARA PALABRAS LIBRES / BANCOS EXTERNOS
// =============================================================================

const POOL_GERUNDS: &[&str] = &[
    "durmiendo", "corriendo", "saltando", "jugando", "nadando", "volando", "cantando",
    "leyendo", "escribiendo", "comiendo", "caminando", "pintando",
];

const POOL_INFINITIVES: &[&str] = &[
    "comer", "dormir", "correr", "jugar", "leer", "escribir", "cantar", "nadar", "crecer",
    "recortar", "medir", "viajar",
];

const POOL_COLORS: &[&str] = &[
    "rojo", "azul", "verde", "amarillo", "blanco", "negro", "morado", "rosa", "gris", "naranja",
];

const POOL_FEM_PLURAL: &[&str] = &[
    "flores", "hojas", "manzanas", "nubes", "montañas", "semillas", "estrellas", "ventanas",
    "palabras", "cuerdas", "abejas", "mariposas",
];

const POOL_MASC_PLURAL: &[&str] = &[
    "árboles", "peces", "libros", "cuadernos", "niños", "colores", "pájaros", "zapatos",
    "lápices", "globos", "conejos", "caballos",
];

const POOL_FEM_SINGULAR: &[&str] = &[
    "pelota", "escuela", "montaña", "ventana", "mochila", "tortuga", "mariposa", "estrella",
    "cuchara", "campana", "manzana", "laguna",
];

const POOL_MASC_SINGULAR: &[&str] = &[
    "perro", "gato", "árbol", "jardín", "cuaderno", "camino", "castillo", "caballo", "globo",
    "reloj", "sombrero", "parque",
];

/// Genera distractores plausibles para cualquier palabra cuando se usa un banco externo
/// o cuando se elige el modo `AzarCualquierPalabra`.
pub fn generate_morphological_distractors(
    target_word: &str,
    count: usize,
    rng: &mut Rng,
) -> Vec<String> {
    let lower = target_word.to_lowercase();

    // 1. Verificar si la palabra ya existe en algún slot curado de nuestro banco interno
    for entry in SENTENCE_BANK {
        for slot in entry.slots {
            if slot.target_word.eq_ignore_ascii_case(target_word) {
                let mut cands: Vec<String> =
                    slot.distractors.iter().map(|s| (*s).to_string()).collect();
                rng.shuffle(&mut cands);
                cands.truncate(count);
                if cands.len() == count {
                    return cands;
                }
            }
        }
    }

    // 2. Seleccionar el pool morfológico más adecuado según la terminación de la palabra
    let pool: &[&str] = if lower.ends_with("ando") || lower.ends_with("iendo") || lower.ends_with("yendo") {
        POOL_GERUNDS
    } else if lower.ends_with("ar") || lower.ends_with("er") || lower.ends_with("ir") {
        POOL_INFINITIVES
    } else if POOL_COLORS.iter().any(|&c| c == lower) {
        POOL_COLORS
    } else if lower.ends_with("as") || lower.ends_with("ciones") {
        POOL_FEM_PLURAL
    } else if lower.ends_with("os") || lower.ends_with("es") {
        POOL_MASC_PLURAL
    } else if lower.ends_with('a') {
        POOL_FEM_SINGULAR
    } else {
        POOL_MASC_SINGULAR
    };

    let is_capitalized = target_word
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false);

    let mut candidates: Vec<String> = pool
        .iter()
        .filter(|&&w| !w.eq_ignore_ascii_case(target_word))
        .map(|&w| {
            if is_capitalized {
                capitalize_first(w)
            } else {
                w.to_string()
            }
        })
        .collect();

    rng.shuffle(&mut candidates);
    candidates.truncate(count);
    candidates
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => {
            let up: String = first.to_uppercase().collect();
            format!("{}{}", up, chars.as_str())
        }
        None => String::new(),
    }
}

/// Divide una oración en `(prefijo, palabra_objetivo, sufijo)` buscando la palabra completa.
pub fn split_sentence_around_word(sentence: &str, target_word: &str) -> Option<(String, String, String)> {
    let mut byte_idx = 0;
    for token in sentence.split_whitespace() {
        let pos = sentence[byte_idx..].find(token)? + byte_idx;
        let trimmed = token.trim_matches(|c: char| !c.is_alphabetic() && c != 'á' && c != 'é' && c != 'í' && c != 'ó' && c != 'ú' && c != 'ü' && c != 'ñ' && c != 'Á' && c != 'É' && c != 'Í' && c != 'Ó' && c != 'Ú' && c != 'Ü' && c != 'Ñ');
        if trimmed == target_word {
            // Encontrar inicio exacto de `trimmed` dentro de `token`
            let inner_offset = token.find(trimmed)?;
            let word_start = pos + inner_offset;
            let word_end = word_start + trimmed.len();
            let prefix = sentence[..word_start].to_string();
            let found = sentence[word_start..word_end].to_string();
            let suffix = sentence[word_end..].to_string();
            return Some((prefix, found, suffix));
        }
        byte_idx = pos + token.len();
    }
    None
}

/// Extrae todas las palabras de contenido (>= 3 letras, excluyendo nexos muy comunes) de una oración arbitraria.
pub fn extract_candidate_words(sentence: &str) -> Vec<String> {
    const STOPWORDS: &[&str] = &[
        "el", "la", "los", "las", "un", "una", "unos", "unas", "de", "del", "al", "en", "con",
        "por", "para", "sin", "sobre", "bajo", "entre", "hasta", "desde", "como", "cuando",
        "que", "muy", "más", "sus", "mis", "tus", "son", "está", "están", "nos",
    ];

    let mut out = Vec::new();
    for raw in sentence.split_whitespace() {
        let clean = raw.trim_matches(|c: char| !c.is_alphabetic());
        let char_count = clean.chars().count();
        if char_count >= 3 {
            let low = clean.to_lowercase();
            if !STOPWORDS.iter().any(|&sw| sw == low) {
                out.push(clean.to_string());
            }
        }
    }
    out
}

/// Carga un banco de frases personalizado desde un archivo de texto (una oración por línea).
/// Soporta líneas simples (`El perro ladra en el jardín.`) o con distractores opcionales tras `|`
/// (`El perro ladra en el jardín. | perro: gato, conejo | jardín: parque, patio`).
pub fn load_custom_sentences_from_file(path: &Path) -> Result<Vec<String>, String> {
    let raw = fs::read_to_string(path).map_err(|e| {
        format!(
            "No se pudo leer el archivo de banco de frases '{}': {}",
            path.display(),
            e
        )
    })?;

    let mut sentences = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        sentences.push(trimmed.to_string());
    }

    if sentences.is_empty() {
        return Err(format!(
            "El archivo '{}' no contiene oraciones válidas.",
            path.display()
        ));
    }

    Ok(sentences)
}

/// Construye un ejercicio individual a partir de una entrada del banco interno quitando una palabra al azar.
fn build_exercise_from_bank_entry(
    number: usize,
    entry: &SentenceBankEntry,
    cfg: &ReadingWorksheetConfig,
    rng: &mut Rng,
) -> GeneratedSentenceExercise {
    let distractor_needed = cfg.options_count.saturating_sub(1).max(1);

    // Determinar qué palabra quitar según el modo configurado
    let chosen_slot: Option<&RemovableSlot> = match cfg.blank_mode {
        BlankSelectionMode::AzarPalabrasClave => {
            if entry.slots.is_empty() {
                None
            } else {
                let idx = rng.gen_range(0, entry.slots.len());
                Some(&entry.slots[idx])
            }
        }
        BlankSelectionMode::UltimaPalabra => entry.slots.last(),
        BlankSelectionMode::AzarCualquierPalabra => None,
    };

    if let Some(slot) = chosen_slot {
        if let Some((prefix, missing_word, suffix)) =
            split_sentence_around_word(entry.sentence, slot.target_word)
        {
            let mut distractors: Vec<String> =
                slot.distractors.iter().map(|s| (*s).to_string()).collect();
            rng.shuffle(&mut distractors);
            distractors.truncate(distractor_needed);

            let (options, correct_option_index) = if cfg.exercise_mode == ExerciseMode::OpcionesMultiples {
                assemble_shuffled_options(&missing_word, distractors, rng)
            } else {
                (Vec::new(), 0)
            };

            return GeneratedSentenceExercise {
                number,
                full_sentence: entry.sentence.to_string(),
                prefix,
                missing_word,
                suffix,
                options,
                correct_option_index,
            };
        }
    }

    // Fallback o modo `AzarCualquierPalabra`: elegir cualquier palabra candidata de la oración
    build_exercise_from_custom_line(number, entry.sentence, cfg, rng)
}

/// Construye un ejercicio quitando una palabra al azar de una oración arbitraria (o con formato `Oración | palabra: d1, d2`).
fn build_exercise_from_custom_line(
    number: usize,
    raw_line: &str,
    cfg: &ReadingWorksheetConfig,
    rng: &mut Rng,
) -> GeneratedSentenceExercise {
    let distractor_needed = cfg.options_count.saturating_sub(1).max(1);
    let parts: Vec<&str> = raw_line.split('|').map(|s| s.trim()).collect();
    let sentence_text = parts[0];

    // Revisar si la línea incluye especificaciones explícitas `palabra: dist1, dist2`
    let mut explicit_slots: Vec<(String, Vec<String>)> = Vec::new();
    for spec in parts.iter().skip(1) {
        if let Some((target, rest)) = spec.split_once(':') {
            let t = target.trim().to_string();
            let dists: Vec<String> = rest
                .split(',')
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty())
                .collect();
            if !t.is_empty() {
                explicit_slots.push((t, dists));
            }
        }
    }

    if !explicit_slots.is_empty() {
        let slot_idx = match cfg.blank_mode {
            BlankSelectionMode::UltimaPalabra => explicit_slots.len() - 1,
            _ => rng.gen_range(0, explicit_slots.len()),
        };
        let (ref target, ref dists) = explicit_slots[slot_idx];
        if let Some((prefix, missing_word, suffix)) =
            split_sentence_around_word(sentence_text, target)
        {
            let mut chosen_dists = dists.clone();
            rng.shuffle(&mut chosen_dists);
            if chosen_dists.len() < distractor_needed {
                let extra = generate_morphological_distractors(
                    &missing_word,
                    distractor_needed - chosen_dists.len(),
                    rng,
                );
                for e in extra {
                    if !chosen_dists.iter().any(|x| x.eq_ignore_ascii_case(&e)) {
                        chosen_dists.push(e);
                    }
                }
            }
            chosen_dists.truncate(distractor_needed);
            let (options, correct_option_index) = if cfg.exercise_mode == ExerciseMode::OpcionesMultiples {
                assemble_shuffled_options(&missing_word, chosen_dists, rng)
            } else {
                (Vec::new(), 0)
            };
            return GeneratedSentenceExercise {
                number,
                full_sentence: sentence_text.to_string(),
                prefix,
                missing_word,
                suffix,
                options,
                correct_option_index,
            };
        }
    }

    let mut candidates = extract_candidate_words(sentence_text);
    if candidates.is_empty() {
        // Si la oración es muy corta, tomar cualquier palabra alfabética
        for raw in sentence_text.split_whitespace() {
            let clean = raw.trim_matches(|c: char| !c.is_alphabetic());
            if !clean.is_empty() {
                candidates.push(clean.to_string());
            }
        }
    }

    let chosen_word = if candidates.is_empty() {
        "palabra".to_string()
    } else {
        match cfg.blank_mode {
            BlankSelectionMode::UltimaPalabra => candidates.last().unwrap().clone(),
            _ => {
                let idx = rng.gen_range(0, candidates.len());
                candidates[idx].clone()
            }
        }
    };

    let (prefix, missing_word, suffix) = split_sentence_around_word(sentence_text, &chosen_word)
        .unwrap_or_else(|| (format!("{} ", sentence_text), chosen_word.clone(), ".".to_string()));

    let distractors = generate_morphological_distractors(&missing_word, distractor_needed, rng);
    let (options, correct_option_index) = if cfg.exercise_mode == ExerciseMode::OpcionesMultiples {
        assemble_shuffled_options(&missing_word, distractors, rng)
    } else {
        (Vec::new(), 0)
    };

    GeneratedSentenceExercise {
        number,
        full_sentence: sentence_text.to_string(),
        prefix,
        missing_word,
        suffix,
        options,
        correct_option_index,
    }
}

fn assemble_shuffled_options(
    correct_word: &str,
    distractors: Vec<String>,
    rng: &mut Rng,
) -> (Vec<String>, usize) {
    let mut opts = Vec::with_capacity(1 + distractors.len());
    opts.push(correct_word.to_string());
    for d in distractors {
        if !opts.iter().any(|existing| existing.eq_ignore_ascii_case(&d)) {
            opts.push(d);
        }
    }
    rng.shuffle(&mut opts);
    let correct_idx = opts
        .iter()
        .position(|w| w == correct_word)
        .unwrap_or(0);
    (opts, correct_idx)
}

/// Genera la lista completa de ejercicios únicos seleccionando frases del banco de datos
/// y quitando palabras al azar en cada una.
pub fn generate_reading_exercises(
    cfg: &ReadingWorksheetConfig,
    rng: &mut Rng,
) -> Vec<GeneratedSentenceExercise> {
    let mut exercises = Vec::with_capacity(cfg.exercise_count);

    // 1. Si el usuario proporcionó un banco de frases externo/personalizado, usarlo
    if !cfg.custom_sentences.is_empty() {
        let mut pool_indices: Vec<usize> = (0..cfg.custom_sentences.len()).collect();
        rng.shuffle(&mut pool_indices);

        for i in 0..cfg.exercise_count {
            if i > 0 && i % pool_indices.len() == 0 {
                rng.shuffle(&mut pool_indices);
            }
            let line = &cfg.custom_sentences[pool_indices[i % pool_indices.len()]];
            exercises.push(build_exercise_from_custom_line(i + 1, line, cfg, rng));
        }
        return exercises;
    }

    // 2. Filtrar el banco de datos interno por tema
    let mut filtered: Vec<&'static SentenceBankEntry> = SENTENCE_BANK
        .iter()
        .filter(|e| cfg.theme == SentenceTheme::Todos || e.theme == cfg.theme)
        .collect();

    if filtered.is_empty() {
        filtered = SENTENCE_BANK.iter().collect();
    }

    rng.shuffle(&mut filtered);

    for i in 0..cfg.exercise_count {
        if i > 0 && i % filtered.len() == 0 {
            rng.shuffle(&mut filtered);
        }
        let entry = filtered[i % filtered.len()];
        let ex = build_exercise_from_bank_entry(i + 1, entry, cfg, rng);
        exercises.push(ex);
    }

    exercises
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_word_removal_creates_unique_materials() {
        let cfg = ReadingWorksheetConfig {
            exercise_count: 14,
            ..ReadingWorksheetConfig::default()
        };

        let mut rng1 = Rng::from_seed(101);
        let mut rng2 = Rng::from_seed(202);

        let list1 = generate_reading_exercises(&cfg, &mut rng1);
        let list2 = generate_reading_exercises(&cfg, &mut rng2);

        assert_eq!(list1.len(), 14);
        assert_eq!(list2.len(), 14);

        for ex in &list1 {
            // Verificar que al reconstruir prefix + missing_word + suffix obtenemos la oración original
            let reconstructed = format!("{}{}{}", ex.prefix, ex.missing_word, ex.suffix);
            assert_eq!(reconstructed, ex.full_sentence);
            assert_eq!(ex.options.len(), 3);
            assert_eq!(ex.options[ex.correct_option_index], ex.missing_word);
        }

        // Dos semillas distintas generan selecciones y huecos distintos
        let diffs = list1
            .iter()
            .zip(list2.iter())
            .filter(|(a, b)| a.full_sentence != b.full_sentence || a.missing_word != b.missing_word)
            .count();
        assert!(diffs > 0, "Las fichas con distinta semilla deben ser únicas");
    }

    #[test]
    fn test_same_sentence_can_remove_different_words_randomly() {
        let entry = &SENTENCE_BANK[1]; // "La niña tiene un globo rojo." (4 palabras clave)
        let cfg = ReadingWorksheetConfig::default();
        let mut removed_words = std::collections::HashSet::new();

        for seed in 1..40 {
            let mut rng = Rng::from_seed(seed);
            let ex = build_exercise_from_bank_entry(1, entry, &cfg, &mut rng);
            removed_words.insert(ex.missing_word);
        }

        assert!(
            removed_words.len() >= 3,
            "Una misma frase debe permitir ocultar distintas palabras al azar (obtenidas: {:?})",
            removed_words
        );
    }
}
