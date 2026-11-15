use std::time::{SystemTime, UNIX_EPOCH};

/// Generador pseudoaleatorio rápido (SplitMix64) sin dependencias externas.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed },
        }
    }

    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xCAFEBABE_DEADBEEF);
        Self::from_seed(nanos ^ 0xA5A5_5A5A_F0F0_0F0F)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    pub fn gen_range(&mut self, min: usize, max_exclusive: usize) -> usize {
        if max_exclusive <= min {
            return min;
        }
        let span = (max_exclusive - min) as u64;
        min + (self.next_u64() % span) as usize
    }

    pub fn gen_f32(&mut self, min: f32, max: f32) -> f32 {
        let unit = (self.next_u64() >> 40) as f32 / ((1u64 << 24) as f32);
        min + unit * (max - min)
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.gen_range(0, i + 1);
            slice.swap(i, j);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    Animal,
    Alimento,
    Naturaleza,
    Transporte,
    Objeto,
    Cuerpo,
    Ropa,
    Color,
    Numero,
    Adjetivo,
    Emocion,
    Instrumento,
}

impl Category {
    pub const ALL: &'static [Category] = &[
        Category::Animal,
        Category::Alimento,
        Category::Naturaleza,
        Category::Transporte,
        Category::Objeto,
        Category::Cuerpo,
        Category::Ropa,
        Category::Color,
        Category::Numero,
        Category::Adjetivo,
        Category::Emocion,
        Category::Instrumento,
    ];

    /// Nombre en singular para consignas ("Encuentra un animal con...")
    pub fn singular_noun(&self) -> &'static str {
        match self {
            Category::Animal => "un animal",
            Category::Alimento => "un alimento",
            Category::Naturaleza => "un elemento de la naturaleza",
            Category::Transporte => "un medio de transporte",
            Category::Objeto => "un objeto",
            Category::Cuerpo => "una parte del cuerpo",
            Category::Ropa => "una prenda de vestir",
            Category::Color => "un color",
            Category::Numero => "un número",
            Category::Adjetivo => "un adjetivo",
            Category::Emocion => "una emoción",
            Category::Instrumento => "un instrumento musical",
        }
    }

    /// Pronombre indefinido concordado en género ("otro" / "otra")
    pub fn another_pronoun(&self) -> &'static str {
        match self {
            Category::Cuerpo | Category::Ropa | Category::Emocion => "otra",
            _ => "otro",
        }
    }

    /// Nombre en plural para consignas ("Busca dos animales con...")
    pub fn plural_noun(&self, count_word: &str) -> String {
        match self {
            Category::Animal => format!("{} animales", count_word),
            Category::Alimento => format!("{} alimentos", count_word),
            Category::Naturaleza => {
                format!("{} palabras relacionadas con la naturaleza", count_word)
            }
            Category::Transporte => format!("{} medios de transporte", count_word),
            Category::Objeto => format!("{} palabras que sean objetos", count_word),
            Category::Cuerpo => format!("{} partes del cuerpo", count_word),
            Category::Ropa => format!("{} prendas de vestir", count_word),
            Category::Color => format!("{} colores", count_word),
            Category::Numero => format!("{} números", count_word),
            Category::Adjetivo => format!("{} adjetivos (cualidades)", count_word),
            Category::Emocion => format!("{} emociones o sentimientos", count_word),
            Category::Instrumento => format!("{} instrumentos musicales", count_word),
        }
    }

    /// Frase temática ("relacionadas con la comida", etc.)
    pub fn related_phrase(&self) -> &'static str {
        match self {
            Category::Animal => "relacionadas con los animales",
            Category::Alimento => "relacionadas con la comida",
            Category::Naturaleza => "relacionadas con la naturaleza",
            Category::Transporte => "relacionadas con el transporte",
            Category::Objeto => "que sean objetos cotidianos",
            Category::Cuerpo => "relacionadas con el cuerpo humano",
            Category::Ropa => "relacionadas con la ropa",
            Category::Color => "que sean colores",
            Category::Numero => "que sean números",
            Category::Adjetivo => "que describan cómo es algo (adjetivos)",
            Category::Emocion => "relacionadas con las emociones",
            Category::Instrumento => "relacionadas con la música",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WordEntry {
    pub word: &'static str,
    pub category: Category,
    pub syllables: u8,
    pub synonym_group: Option<u16>,
    pub antonym_pair: Option<(u16, u8)>,
}

impl WordEntry {
    pub fn letters(&self) -> usize {
        self.word.chars().count()
    }

    pub fn vowels(&self) -> usize {
        self.word
            .chars()
            .filter(|&c| is_spanish_vowel(c))
            .count()
    }

    pub fn consonants(&self) -> usize {
        self.letters().saturating_sub(self.vowels())
    }

    pub fn starts_with_vowel(&self) -> bool {
        self.word
            .chars()
            .next()
            .map(is_spanish_vowel)
            .unwrap_or(false)
    }

    pub fn ends_with_vowel(&self) -> bool {
        self.word
            .chars()
            .last()
            .map(is_spanish_vowel)
            .unwrap_or(false)
    }

    pub fn first_letter_base(&self) -> char {
        self.word
            .chars()
            .next()
            .map(normalize_spanish_char)
            .unwrap_or('a')
    }

    pub fn last_letter_base(&self) -> char {
        self.word
            .chars()
            .last()
            .map(normalize_spanish_char)
            .unwrap_or('a')
    }

    pub fn has_accent(&self) -> bool {
        self.word
            .chars()
            .any(|c| matches!(c, 'á' | 'é' | 'í' | 'ó' | 'ú' | 'Á' | 'É' | 'Í' | 'Ó' | 'Ú'))
    }
}

pub fn is_spanish_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'á'
            | 'é'
            | 'í'
            | 'ó'
            | 'ú'
            | 'ü'
            | 'A'
            | 'E'
            | 'I'
            | 'O'
            | 'U'
            | 'Á'
            | 'É'
            | 'Í'
            | 'Ó'
            | 'Ú'
            | 'Ü'
    )
}

pub fn normalize_spanish_char(c: char) -> char {
    match c {
        'á' | 'Á' | 'A' => 'a',
        'é' | 'É' | 'E' => 'e',
        'í' | 'Í' | 'I' => 'i',
        'ó' | 'Ó' | 'O' => 'o',
        'ú' | 'Ú' | 'ü' | 'Ü' | 'U' => 'u',
        'ñ' | 'Ñ' => 'ñ',
        other => other.to_ascii_lowercase(),
    }
}

const fn w(
    word: &'static str,
    category: Category,
    syllables: u8,
    synonym_group: Option<u16>,
    antonym_pair: Option<(u16, u8)>,
) -> WordEntry {
    WordEntry {
        word,
        category,
        syllables,
        synonym_group,
        antonym_pair,
    }
}

/// Diccionario curado de palabras en español para la actividad Palabrea.
pub static LEXICON: &[WordEntry] = &[
    // ==================== ANIMALES ====================
    w("tigre", Category::Animal, 2, None, None),
    w("rinoceronte", Category::Animal, 5, None, None),
    w("pato", Category::Animal, 2, None, None),
    w("abeja", Category::Animal, 3, None, None),
    w("pez", Category::Animal, 1, None, None),
    w("lobo", Category::Animal, 2, None, None),
    w("cabra", Category::Animal, 2, None, None),
    w("cebra", Category::Animal, 2, None, None),
    w("sapo", Category::Animal, 2, None, None),
    w("cerdo", Category::Animal, 2, Some(7), None),
    w("marrano", Category::Animal, 3, Some(7), None),
    w("gato", Category::Animal, 2, None, None),
    w("perro", Category::Animal, 2, None, None),
    w("mono", Category::Animal, 2, None, None),
    w("escarabajo", Category::Animal, 5, None, None),
    w("elefante", Category::Animal, 4, None, None),
    w("jirafa", Category::Animal, 3, None, None),
    w("ballena", Category::Animal, 3, None, None),
    w("delfín", Category::Animal, 2, None, None),
    w("águila", Category::Animal, 3, None, None),
    w("tortuga", Category::Animal, 3, None, None),
    w("conejo", Category::Animal, 3, None, None),
    w("caballo", Category::Animal, 3, None, None),
    w("oveja", Category::Animal, 3, None, None),
    w("mariposa", Category::Animal, 4, None, None),

    // ==================== ALIMENTOS ====================
    w("pan", Category::Alimento, 1, None, None),
    w("miel", Category::Alimento, 1, None, None),
    w("sal", Category::Alimento, 1, None, None),
    w("pera", Category::Alimento, 2, None, None),
    w("manzana", Category::Alimento, 3, None, None),
    w("cebolla", Category::Alimento, 3, None, None),
    w("coliflor", Category::Alimento, 3, None, None),
    w("albaricoque", Category::Alimento, 5, None, None),
    w("plátano", Category::Alimento, 3, Some(25), None),
    w("banana", Category::Alimento, 3, Some(25), None),
    w("coco", Category::Alimento, 2, None, None),
    w("queso", Category::Alimento, 2, None, None),
    w("leche", Category::Alimento, 2, None, None),
    w("huevo", Category::Alimento, 2, None, None),
    w("tomate", Category::Alimento, 3, None, None),
    w("naranja", Category::Alimento, 3, None, None),
    w("limón", Category::Alimento, 2, None, None),
    w("fresa", Category::Alimento, 2, None, None),
    w("chocolate", Category::Alimento, 4, None, None),
    w("zanahoria", Category::Alimento, 4, None, None),
    w("arroz", Category::Alimento, 2, None, None),
    w("galleta", Category::Alimento, 3, None, None),

    // ==================== NATURALEZA ====================
    w("árbol", Category::Naturaleza, 2, None, None),
    w("flor", Category::Naturaleza, 1, None, None),
    w("sol", Category::Naturaleza, 1, None, None),
    w("agua", Category::Naturaleza, 2, None, None),
    w("seta", Category::Naturaleza, 2, None, None),
    w("granja", Category::Naturaleza, 2, None, None),
    w("hielo", Category::Naturaleza, 2, None, Some((19, 0))),
    w("fuego", Category::Naturaleza, 2, None, Some((19, 1))),
    w("piedra", Category::Naturaleza, 2, Some(23), None),
    w("roca", Category::Naturaleza, 2, Some(23), None),
    w("bosque", Category::Naturaleza, 2, Some(24), None),
    w("selva", Category::Naturaleza, 2, Some(24), None),
    w("día", Category::Naturaleza, 2, None, Some((20, 0))),
    w("noche", Category::Naturaleza, 2, None, Some((20, 1))),
    w("estrella", Category::Naturaleza, 3, None, None),
    w("montaña", Category::Naturaleza, 3, None, None),
    w("nube", Category::Naturaleza, 2, None, None),
    w("lluvia", Category::Naturaleza, 2, None, None),
    w("viento", Category::Naturaleza, 2, None, None),
    w("arena", Category::Naturaleza, 3, None, None),
    w("volcán", Category::Naturaleza, 2, None, None),
    w("océano", Category::Naturaleza, 4, None, None),

    // ==================== TRANSPORTE ====================
    w("helicóptero", Category::Transporte, 5, None, None),
    w("furgoneta", Category::Transporte, 4, None, None),
    w("coche", Category::Transporte, 2, Some(2), None),
    w("auto", Category::Transporte, 2, Some(2), None),
    w("barco", Category::Transporte, 2, Some(20), None),
    w("buque", Category::Transporte, 2, Some(20), None),
    w("bicicleta", Category::Transporte, 4, None, None),
    w("avión", Category::Transporte, 2, None, None),
    w("camión", Category::Transporte, 2, None, None),
    w("tren", Category::Transporte, 1, None, None),
    w("moto", Category::Transporte, 2, None, None),
    w("cohete", Category::Transporte, 3, None, None),
    w("submarino", Category::Transporte, 4, None, None),
    w("canoa", Category::Transporte, 3, None, None),
    w("patinete", Category::Transporte, 4, None, None),
    w("autobús", Category::Transporte, 3, None, None),

    // ==================== OBJETOS ====================
    w("sofá", Category::Objeto, 2, Some(11), None),
    w("sillón", Category::Objeto, 2, Some(11), None),
    w("mesa", Category::Objeto, 2, None, None),
    w("cama", Category::Objeto, 2, None, None),
    w("armario", Category::Objeto, 3, None, None),
    w("cortina", Category::Objeto, 3, None, None),
    w("ordenador", Category::Objeto, 4, Some(10), None),
    w("computadora", Category::Objeto, 5, Some(10), None),
    w("vaso", Category::Objeto, 2, None, None),
    w("copa", Category::Objeto, 2, None, None),
    w("botella", Category::Objeto, 3, None, None),
    w("peine", Category::Objeto, 2, None, None),
    w("libreta", Category::Objeto, 3, Some(9), None),
    w("cuaderno", Category::Objeto, 3, Some(9), None),
    w("lápiz", Category::Objeto, 2, None, None),
    w("estuche", Category::Objeto, 3, None, None),
    w("bolso", Category::Objeto, 2, Some(12), None),
    w("mochila", Category::Objeto, 3, Some(12), None),
    w("dedal", Category::Objeto, 2, None, None),
    w("reloj", Category::Objeto, 2, None, None),
    w("globo", Category::Objeto, 2, None, None),
    w("cometa", Category::Objeto, 3, None, None),
    w("trompo", Category::Objeto, 2, None, None),
    w("pelota", Category::Objeto, 3, Some(1), None),
    w("balón", Category::Objeto, 2, Some(1), None),
    w("gafas", Category::Objeto, 2, Some(8), None),
    w("lentes", Category::Objeto, 2, Some(8), None),
    w("espejo", Category::Objeto, 3, None, None),
    w("llave", Category::Objeto, 2, None, None),
    w("paraguas", Category::Objeto, 3, None, None),
    w("tijeras", Category::Objeto, 3, None, None),
    w("campana", Category::Objeto, 3, None, None),
    w("flotador", Category::Objeto, 3, None, None),
    w("piscina", Category::Objeto, 3, None, None),

    // ==================== CUERPO ====================
    w("mano", Category::Cuerpo, 2, None, None),
    w("rodilla", Category::Cuerpo, 3, None, None),
    w("cabeza", Category::Cuerpo, 3, None, None),
    w("cejas", Category::Cuerpo, 2, None, None),
    w("diente", Category::Cuerpo, 2, None, None),
    w("dedos", Category::Cuerpo, 2, None, None),
    w("pies", Category::Cuerpo, 1, None, None),
    w("pelo", Category::Cuerpo, 2, Some(21), None),
    w("cabello", Category::Cuerpo, 3, Some(21), None),
    w("rostro", Category::Cuerpo, 2, Some(22), None),
    w("cara", Category::Cuerpo, 2, Some(22), None),
    w("hombro", Category::Cuerpo, 2, None, None),
    w("corazón", Category::Cuerpo, 3, None, None),
    w("espalda", Category::Cuerpo, 3, None, None),
    w("nariz", Category::Cuerpo, 2, None, None),
    w("oreja", Category::Cuerpo, 3, None, None),
    w("cuello", Category::Cuerpo, 2, None, None),

    // ==================== ROPA ====================
    w("chaqueta", Category::Ropa, 3, Some(13), None),
    w("abrigo", Category::Ropa, 3, Some(13), None),
    w("gorro", Category::Ropa, 2, Some(14), None),
    w("sombrero", Category::Ropa, 3, Some(14), None),
    w("zapato", Category::Ropa, 3, None, None),
    w("guantes", Category::Ropa, 2, None, None),
    w("capa", Category::Ropa, 2, None, None),
    w("vestido", Category::Ropa, 3, None, None),
    w("pantalón", Category::Ropa, 3, None, None),
    w("camisa", Category::Ropa, 3, None, None),
    w("bufanda", Category::Ropa, 3, None, None),
    w("calcetín", Category::Ropa, 3, None, None),
    w("falda", Category::Ropa, 2, None, None),
    w("botas", Category::Ropa, 2, None, None),

    // ==================== COLORES ====================
    w("azul", Category::Color, 2, None, None),
    w("negro", Category::Color, 2, None, Some((18, 0))),
    w("blanco", Category::Color, 2, None, Some((18, 1))),
    w("amarillo", Category::Color, 4, None, None),
    w("rosa", Category::Color, 2, None, None),
    w("rojo", Category::Color, 2, None, None),
    w("verde", Category::Color, 2, None, None),
    w("morado", Category::Color, 3, Some(15), None),
    w("violeta", Category::Color, 3, Some(15), None),
    w("gris", Category::Color, 1, None, None),
    w("marrón", Category::Color, 2, None, None),
    w("dorado", Category::Color, 3, None, None),

    // ==================== NÚMEROS ====================
    w("uno", Category::Numero, 2, None, None),
    w("dos", Category::Numero, 1, None, None),
    w("tres", Category::Numero, 1, None, None),
    w("cuatro", Category::Numero, 2, None, None),
    w("cinco", Category::Numero, 2, None, None),
    w("seis", Category::Numero, 1, None, None),
    w("siete", Category::Numero, 2, None, None),
    w("ocho", Category::Numero, 2, None, None),
    w("nueve", Category::Numero, 2, None, None),
    w("diez", Category::Numero, 1, None, None),
    w("once", Category::Numero, 2, None, None),
    w("doce", Category::Numero, 2, None, None),
    w("veinte", Category::Numero, 2, None, None),
    w("cien", Category::Numero, 1, None, None),

    // ==================== ADJETIVOS (con Sinónimos y Antónimos) ====================
    w("grande", Category::Adjetivo, 2, None, Some((1, 0))),
    w("pequeño", Category::Adjetivo, 3, None, Some((1, 1))),
    w("duro", Category::Adjetivo, 2, None, Some((2, 0))),
    w("blando", Category::Adjetivo, 2, None, Some((2, 1))),
    w("fuerte", Category::Adjetivo, 2, None, Some((3, 0))),
    w("débil", Category::Adjetivo, 2, None, Some((3, 1))),
    w("viejo", Category::Adjetivo, 2, None, Some((4, 0))),
    w("joven", Category::Adjetivo, 2, None, Some((4, 1))),
    w("oscuro", Category::Adjetivo, 3, None, Some((5, 0))),
    w("claro", Category::Adjetivo, 2, None, Some((5, 1))),
    w("rápido", Category::Adjetivo, 3, Some(3), Some((7, 0))),
    w("veloz", Category::Adjetivo, 2, Some(3), None),
    w("lento", Category::Adjetivo, 2, None, Some((7, 1))),
    w("alto", Category::Adjetivo, 2, None, Some((8, 0))),
    w("bajo", Category::Adjetivo, 2, None, Some((8, 1))),
    w("lleno", Category::Adjetivo, 2, None, Some((9, 0))),
    w("vacío", Category::Adjetivo, 3, None, Some((9, 1))),
    w("pesado", Category::Adjetivo, 3, None, Some((10, 0))),
    w("ligero", Category::Adjetivo, 3, None, Some((10, 1))),
    w("dulce", Category::Adjetivo, 2, None, Some((11, 0))),
    w("salado", Category::Adjetivo, 3, None, Some((11, 1))),
    w("limpio", Category::Adjetivo, 2, None, Some((12, 0))),
    w("sucio", Category::Adjetivo, 2, None, Some((12, 1))),
    w("frío", Category::Adjetivo, 2, None, Some((13, 0))),
    w("caliente", Category::Adjetivo, 3, None, Some((13, 1))),
    w("sano", Category::Adjetivo, 2, None, Some((14, 0))),
    w("enfermo", Category::Adjetivo, 3, None, Some((14, 1))),
    w("bueno", Category::Adjetivo, 2, None, Some((15, 0))),
    w("malo", Category::Adjetivo, 2, None, Some((15, 1))),
    w("suave", Category::Adjetivo, 2, None, Some((16, 0))),
    w("áspero", Category::Adjetivo, 3, None, Some((16, 1))),
    w("diferente", Category::Adjetivo, 4, None, Some((17, 0))),
    w("igual", Category::Adjetivo, 2, None, Some((17, 1))),
    w("delgado", Category::Adjetivo, 3, Some(4), None),
    w("flaco", Category::Adjetivo, 2, Some(4), None),
    w("bonito", Category::Adjetivo, 3, Some(5), None),
    w("hermoso", Category::Adjetivo, 3, Some(5), None),
    w("alegre", Category::Adjetivo, 3, Some(6), None),
    w("contento", Category::Adjetivo, 3, Some(6), None),
    w("esponjoso", Category::Adjetivo, 4, None, None),
    w("soleado", Category::Adjetivo, 4, None, None),
    w("imposible", Category::Adjetivo, 4, None, None),
    w("inteligente", Category::Adjetivo, 5, None, None),
    w("mojado", Category::Adjetivo, 3, None, None),

    // ==================== EMOCIONES ====================
    w("tristeza", Category::Emocion, 3, None, Some((6, 0))),
    w("alegría", Category::Emocion, 4, None, Some((6, 1))),
    w("miedo", Category::Emocion, 2, Some(16), None),
    w("temor", Category::Emocion, 2, Some(16), None),
    w("calma", Category::Emocion, 2, Some(17), None),
    w("paz", Category::Emocion, 1, Some(17), None),
    w("enfado", Category::Emocion, 3, Some(18), None),
    w("enojo", Category::Emocion, 3, Some(18), None),
    w("amor", Category::Emocion, 2, Some(19), None),
    w("cariño", Category::Emocion, 3, Some(19), None),
    w("sorpresa", Category::Emocion, 3, None, None),

    // ==================== INSTRUMENTOS ====================
    w("guitarra", Category::Instrumento, 3, None, None),
    w("piano", Category::Instrumento, 2, None, None),
    w("trompeta", Category::Instrumento, 3, None, None),
    w("flauta", Category::Instrumento, 2, None, None),
    w("violín", Category::Instrumento, 2, None, None),
    w("tambor", Category::Instrumento, 2, None, None),
    w("arpa", Category::Instrumento, 2, None, None),
    w("maracas", Category::Instrumento, 3, None, None),
    w("acordeón", Category::Instrumento, 4, None, None),
    w("saxofón", Category::Instrumento, 3, None, None),
    w("xilófono", Category::Instrumento, 4, None, None),

    // ==================== VOCABULARIO ADICIONAL ====================
    w("kilo", Category::Objeto, 2, None, None),
    w("polo", Category::Ropa, 2, None, None),
    w("gol", Category::Objeto, 1, None, None),
    w("torre", Category::Objeto, 2, None, None),
    w("librería", Category::Objeto, 4, None, None),
    w("carretera", Category::Transporte, 4, None, None),
    w("yate", Category::Transporte, 2, None, None),
    w("grúa", Category::Transporte, 2, None, None),
    w("oro", Category::Naturaleza, 2, None, None),
    w("campo", Category::Naturaleza, 2, None, None),
    w("río", Category::Naturaleza, 2, None, None),
    w("playa", Category::Naturaleza, 2, None, None),
    w("ratón", Category::Animal, 2, None, None),
    w("león", Category::Animal, 2, None, None),
    w("oso", Category::Animal, 2, None, None),
    w("pulpo", Category::Animal, 2, None, None),
    w("tiburón", Category::Animal, 3, None, None),
    w("camello", Category::Animal, 3, None, None),
    w("ardilla", Category::Animal, 3, None, None),
    w("pingüino", Category::Animal, 3, None, None),
    w("sandía", Category::Alimento, 3, None, None),
    w("melón", Category::Alimento, 2, None, None),
    w("uva", Category::Alimento, 2, None, None),
    w("sopa", Category::Alimento, 2, None, None),
    w("pastel", Category::Alimento, 2, None, None),
];

/// Devuelve si dos palabras del banco son sinónimos entre sí.
pub fn are_synonyms(a: &WordEntry, b: &WordEntry) -> bool {
    if a.word == b.word {
        return false;
    }
    match (a.synonym_group, b.synonym_group) {
        (Some(ga), Some(gb)) => ga == gb,
        _ => false,
    }
}

/// Devuelve si dos palabras del banco son antónimos entre sí.
pub fn are_antonyms(a: &WordEntry, b: &WordEntry) -> bool {
    if a.word == b.word {
        return false;
    }
    match (a.antonym_pair, b.antonym_pair) {
        (Some((pa, sa)), Some((pb, sb))) => pa == pb && sa != sb,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_unique_words_and_valid_metadata() {
        let mut seen = HashSet::new();
        for entry in LEXICON {
            assert!(!entry.word.is_empty());
            assert!(
                seen.insert(entry.word),
                "Palabra duplicada en LEXICON: {}",
                entry.word
            );
            assert!(
                entry.syllables >= 1 && entry.syllables <= 6,
                "Sílabas fuera de rango para {}",
                entry.word
            );
            assert!(entry.letters() >= 2);
            assert!(entry.vowels() >= 1);
        }
    }

    #[test]
    fn test_synonym_and_antonym_pairs_exist() {
        let mut syn_pairs = 0;
        let mut ant_pairs = 0;
        for (i, a) in LEXICON.iter().enumerate() {
            for b in &LEXICON[i + 1..] {
                if are_synonyms(a, b) {
                    syn_pairs += 1;
                }
                if are_antonyms(a, b) {
                    ant_pairs += 1;
                }
            }
        }
        assert!(syn_pairs >= 15, "Se esperaban al menos 15 pares de sinónimos");
        assert!(ant_pairs >= 15, "Se esperaban al menos 15 pares de antónimos");
    }
}
