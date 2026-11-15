use crate::lexicon::Rng;
use crate::pdf::PaperSize;

/// Tipos de operación aritmética soportados por el generador.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OperationKind {
    Multiplicacion,
    Division,
    Suma,
    Resta,
    Mixto,
}

impl OperationKind {
    pub fn title_es(&self) -> &'static str {
        match self {
            OperationKind::Multiplicacion => "MULTIPLICACIÓN",
            OperationKind::Division => "DIVISIÓN",
            OperationKind::Suma => "SUMA",
            OperationKind::Resta => "RESTA",
            OperationKind::Mixto => "OPERACIONES",
        }
    }

    pub fn title_en(&self) -> &'static str {
        match self {
            OperationKind::Multiplicacion => "MULTIPLICATION",
            OperationKind::Division => "DIVISION",
            OperationKind::Suma => "ADDITION",
            OperationKind::Resta => "SUBTRACTION",
            OperationKind::Mixto => "ARITHMETIC",
        }
    }

    pub fn instruction_es(&self) -> &'static str {
        match self {
            OperationKind::Multiplicacion => "Resuelve las siguientes multiplicaciones.",
            OperationKind::Division => "Resuelve las siguientes divisiones.",
            OperationKind::Suma => "Resuelve las siguientes sumas.",
            OperationKind::Resta => "Resuelve las siguientes restas.",
            OperationKind::Mixto => "Resuelve las siguientes operaciones aritméticas.",
        }
    }

    pub fn instruction_en(&self) -> &'static str {
        match self {
            OperationKind::Multiplicacion => "Solve the following multiplications.",
            OperationKind::Division => "Solve the following divisions.",
            OperationKind::Suma => "Solve the following additions.",
            OperationKind::Resta => "Solve the following subtractions.",
            OperationKind::Mixto => "Solve the following arithmetic operations.",
        }
    }
}

/// Notación visual para la división.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DivisionNotation {
    /// Caja tradicional hispana: `Dividendo | Divisor`, cociente debajo del divisor y restos bajo el dividendo.
    Tradicional,
    /// Galera / casita: `Divisor ⟌ Dividendo`, cociente arriba del dividendo y restos debajo.
    Galera,
}

impl DivisionNotation {
    pub fn label(&self) -> &'static str {
        match self {
            DivisionNotation::Tradicional => "Caja tradicional (Dividendo | Divisor)",
            DivisionNotation::Galera => "Galera / Casita (Divisor ⟌ Dividendo)",
        }
    }
}

/// Idioma de los textos de encabezado en la lámina.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WorksheetLanguage {
    Es,
    En,
}

impl WorksheetLanguage {
    pub fn label(&self) -> &'static str {
        match self {
            WorksheetLanguage::Es => "Español",
            WorksheetLanguage::En => "Inglés (English)",
        }
    }
}

/// Configuración de complejidad para multiplicaciones.
#[derive(Clone, Debug)]
pub struct MultiplicationConfig {
    pub multiplicand_digits: usize,
    pub multiplier_digits: usize,
}

impl Default for MultiplicationConfig {
    fn default() -> Self {
        Self {
            multiplicand_digits: 2,
            multiplier_digits: 2,
        }
    }
}

/// Configuración de complejidad para divisiones.
#[derive(Clone, Debug)]
pub struct DivisionConfig {
    pub dividend_digits: usize,
    pub divisor_digits: usize,
    pub notation: DivisionNotation,
    pub exact_only: bool,
}

impl Default for DivisionConfig {
    fn default() -> Self {
        Self {
            dividend_digits: 3,
            divisor_digits: 1,
            notation: DivisionNotation::Tradicional,
            exact_only: false,
        }
    }
}

/// Configuración de complejidad para sumas.
#[derive(Clone, Debug)]
pub struct AdditionConfig {
    pub addend_count: usize,
    /// Cantidad de dígitos para cada sumando (de arriba hacia abajo).
    pub digits_per_addend: Vec<usize>,
}

impl Default for AdditionConfig {
    fn default() -> Self {
        Self {
            addend_count: 2,
            digits_per_addend: vec![3, 3],
        }
    }
}

/// Configuración de complejidad para restas.
#[derive(Clone, Debug)]
pub struct SubtractionConfig {
    pub minuend_digits: usize,
    pub subtrahend_digits: usize,
}

impl Default for SubtractionConfig {
    fn default() -> Self {
        Self {
            minuend_digits: 3,
            subtrahend_digits: 3,
        }
    }
}

/// Configuración general de la ficha didáctica a generar.
#[derive(Clone, Debug)]
pub struct WorksheetConfig {
    pub operation: OperationKind,
    pub multiplication: MultiplicationConfig,
    pub division: DivisionConfig,
    pub addition: AdditionConfig,
    pub subtraction: SubtractionConfig,
    pub problem_count: usize,
    pub paper_size: PaperSize,
    pub language: WorksheetLanguage,
}

impl Default for WorksheetConfig {
    fn default() -> Self {
        Self {
            operation: OperationKind::Multiplicacion,
            multiplication: MultiplicationConfig::default(),
            division: DivisionConfig::default(),
            addition: AdditionConfig::default(),
            subtraction: SubtractionConfig::default(),
            problem_count: 12,
            paper_size: PaperSize::A4,
            language: WorksheetLanguage::Es,
        }
    }
}

/// Un producto parcial dentro de una multiplicación de varias cifras.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartialProductStep {
    pub multiplier_digit: u8,
    pub value: u64,
    /// Desplazamiento hacia la izquierda (en columnas de valor posicional: 0 = unidades, 1 = decenas...).
    pub shift: usize,
    /// Cantidad de casillas asignadas a esta fila de producto parcial.
    pub box_count: usize,
}

/// Problema de multiplicación con su desglose completo paso a paso.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiplicationProblem {
    pub multiplicand: u64,
    pub multiplicand_digits: usize,
    pub multiplier: u64,
    pub multiplier_digits: usize,
    pub partial_products: Vec<PartialProductStep>,
    pub product: u64,
    pub product_boxes: usize,
}

/// Fila de resto parcial (o residuo final) bajo el dividendo en una división paso a paso.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DivisionStep {
    /// Índice de columna del dividendo (0 = dígito más significativo) donde termina esta fila.
    pub end_col: usize,
    /// Dígitos que ocupan las casillas de esta fila (de izquierda a derecha, terminando en `end_col`).
    pub digits: Vec<u8>,
    /// Indica si es el residuo final de la división.
    pub is_final_remainder: bool,
}

/// Problema de división con todos sus pasos intermedios para casillas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DivisionProblem {
    pub dividend: u64,
    pub dividend_digits: Vec<u8>,
    pub divisor: u64,
    pub divisor_digits: Vec<u8>,
    pub quotient: u64,
    /// Pares `(columna_del_dividendo, digito_del_cociente)`.
    pub quotient_digits: Vec<(usize, u8)>,
    pub steps: Vec<DivisionStep>,
    pub remainder: u64,
    pub notation: DivisionNotation,
}

/// Problema de suma con múltiples sumandos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdditionProblem {
    /// Lista de `(valor_sumando, cantidad_de_casillas_del_sumando)`.
    pub addends: Vec<(u64, usize)>,
    pub total: u64,
    pub total_boxes: usize,
}

/// Problema de resta (minuendo - sustraendo = diferencia).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubtractionProblem {
    pub minuend: u64,
    pub minuend_digits: usize,
    pub subtrahend: u64,
    pub subtrahend_digits: usize,
    pub difference: u64,
    pub difference_boxes: usize,
}

/// Enumeración que encapsula cualquier ejercicio aritmético generado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MathProblem {
    Multiplicacion(MultiplicationProblem),
    Division(DivisionProblem),
    Suma(AdditionProblem),
    Resta(SubtractionProblem),
}

impl MathProblem {
    /// Resumen legible en una línea para mostrar en consola al finalizar.
    pub fn summary_line(&self) -> String {
        match self {
            MathProblem::Multiplicacion(m) => {
                format!("{} × {} = {}", m.multiplicand, m.multiplier, m.product)
            }
            MathProblem::Division(d) => {
                if d.remainder == 0 {
                    format!("{} ÷ {} = {} (exacta)", d.dividend, d.divisor, d.quotient)
                } else {
                    format!(
                        "{} ÷ {} = {} (residuo: {})",
                        d.dividend, d.divisor, d.quotient, d.remainder
                    )
                }
            }
            MathProblem::Suma(a) => {
                let terms: Vec<String> = a.addends.iter().map(|(v, _)| v.to_string()).collect();
                format!("{} = {}", terms.join(" + "), a.total)
            }
            MathProblem::Resta(s) => {
                format!("{} - {} = {}", s.minuend, s.subtrahend, s.difference)
            }
        }
    }
}

/// Descompone un entero en su vector de dígitos decimales (de izquierda a derecha).
pub fn number_to_digits(mut n: u64) -> Vec<u8> {
    if n == 0 {
        return vec![0];
    }
    let mut digits = Vec::with_capacity(10);
    while n > 0 {
        digits.push((n % 10) as u8);
        n /= 10;
    }
    digits.reverse();
    digits
}

/// Descompone un entero en `width` casillas alineadas a la derecha.
/// Si el número tiene menos dígitos que `width`, las casillas sobrantes a la izquierda quedan en `None`
/// (salvo cuando `pad_zero` es `true`, en cuyo caso se rellenan con `Some(0)`).
pub fn number_to_boxed_digits(n: u64, width: usize, pad_zero: bool) -> Vec<Option<u8>> {
    let raw = number_to_digits(n);
    let mut out = vec![if pad_zero { Some(0) } else { None }; width.max(raw.len())];
    let offset = out.len() - raw.len();
    for (i, d) in raw.into_iter().enumerate() {
        out[offset + i] = Some(d);
    }
    if out.len() > width {
        out.split_off(out.len() - width)
    } else {
        out
    }
}

fn pow10(exp: usize) -> u64 {
    10u64.saturating_pow(exp as u32)
}

/// Genera un número aleatorio con exactamente `digits` cifras (`1..=6`).
fn gen_number_with_digits(rng: &mut Rng, digits: usize, min_first_digit: u8) -> u64 {
    let d = digits.clamp(1, 6);
    if d == 1 {
        let low = (min_first_digit.max(1) as usize).min(9);
        return rng.gen_range(low, 10) as u64;
    }
    let min_lead = (min_first_digit.max(1) as u64).min(9);
    let low = min_lead * pow10(d - 1);
    let high = pow10(d);
    let span = high - low;
    low + (rng.next_u64() % span)
}

/// Construye un `MultiplicationProblem` completo a partir de sus dos operandos.
pub fn build_multiplication_problem(
    multiplicand: u64,
    multiplicand_digits: usize,
    multiplier: u64,
    multiplier_digits: usize,
) -> MultiplicationProblem {
    let d1 = multiplicand_digits.max(number_to_digits(multiplicand).len());
    let d2 = multiplier_digits.max(number_to_digits(multiplier).len());
    let mult_digits = number_to_digits(multiplier);

    let mut partial_products = Vec::new();
    if d2 > 1 {
        for (step_idx, &digit) in mult_digits.iter().rev().enumerate() {
            let val = multiplicand * (digit as u64);
            partial_products.push(PartialProductStep {
                multiplier_digit: digit,
                value: val,
                shift: step_idx,
                box_count: d1 + 1,
            });
        }
    }

    let product = multiplicand * multiplier;
    let product_boxes = if d2 == 1 { d1 + 1 } else { d1 + d2 };

    MultiplicationProblem {
        multiplicand,
        multiplicand_digits: d1,
        multiplier,
        multiplier_digits: d2,
        partial_products,
        product,
        product_boxes,
    }
}

/// Genera un problema aleatorio de multiplicación según la configuración.
/// Procura que los dígitos del multiplicador no sean 0 ni 1 cuando es posible para que los
/// productos parciales aprovechen todas las casillas como en las láminas escolares.
pub fn generate_multiplication(
    cfg: &MultiplicationConfig,
    rng: &mut Rng,
) -> MultiplicationProblem {
    let d1 = cfg.multiplicand_digits.clamp(1, 5);
    let d2 = cfg.multiplier_digits.clamp(1, 4);

    let mut best = None;
    for _ in 0..25 {
        let multiplicand = gen_number_with_digits(rng, d1, 3);
        // Construir multiplicador evitando ceros intermedios y prefiriendo dígitos >= 2
        let mut multiplier = 0u64;
        for pos in 0..d2 {
            let min_d = if d2 == 1 || pos == 0 { 2 } else { 2 };
            let digit = rng.gen_range(min_d, 10) as u64;
            multiplier = multiplier * 10 + digit;
        }

        let prob = build_multiplication_problem(multiplicand, d1, multiplier, d2);
        // Preferimos que todos los productos parciales y el total llenen sus casillas
        let all_partials_full = prob
            .partial_products
            .iter()
            .all(|pp| number_to_digits(pp.value).len() == pp.box_count);
        let total_full = number_to_digits(prob.product).len() == prob.product_boxes;

        if all_partials_full && total_full {
            return prob;
        }
        if best.is_none() {
            best = Some(prob);
        }
    }

    best.unwrap()
}

/// Construye un `DivisionProblem` calculando todos los restos parciales y dígitos del cociente.
pub fn build_division_problem(
    dividend: u64,
    divisor: u64,
    notation: DivisionNotation,
) -> DivisionProblem {
    let divisor = divisor.max(1);
    let dividend = dividend.max(divisor);

    let dividend_digits = number_to_digits(dividend);
    let divisor_digits = number_to_digits(divisor);
    let n = dividend_digits.len();

    // 1. Encontrar el prefijo inicial del dividendo que sea >= divisor
    let mut idx = 0usize;
    let mut current = dividend_digits[0] as u64;
    while current < divisor && idx + 1 < n {
        idx += 1;
        current = current * 10 + (dividend_digits[idx] as u64);
    }

    let first_end = idx;
    let mut quotient_digits = Vec::new();
    let mut steps = Vec::new();

    let first_q = (current / divisor) as u8;
    quotient_digits.push((first_end, first_q));
    let mut rem = current % divisor;

    // 2. Bajar cada dígito restante uno por uno
    let rem_width = divisor_digits.len();
    for next_col in (first_end + 1)..n {
        let next_d = dividend_digits[next_col];
        let mut row_digits = number_to_digits(rem);
        // Asegurar ancho consistente sin exceder las columnas disponibles a la izquierda
        let max_rem_boxes = next_col;
        let target_rem_boxes = row_digits.len().max(1).min(max_rem_boxes);
        while row_digits.len() < target_rem_boxes {
            row_digits.insert(0, 0);
        }
        row_digits.push(next_d);

        steps.push(DivisionStep {
            end_col: next_col,
            digits: row_digits,
            is_final_remainder: false,
        });

        current = rem * 10 + (next_d as u64);
        let q_digit = (current / divisor) as u8;
        quotient_digits.push((next_col, q_digit));
        rem = current % divisor;
    }

    // 3. Fila final con el residuo
    let mut final_rem_digits = number_to_digits(rem);
    let target_final = final_rem_digits.len().max(1).min(rem_width).min(n);
    while final_rem_digits.len() < target_final {
        final_rem_digits.insert(0, 0);
    }
    steps.push(DivisionStep {
        end_col: n - 1,
        digits: final_rem_digits,
        is_final_remainder: true,
    });

    DivisionProblem {
        dividend,
        dividend_digits,
        divisor,
        divisor_digits,
        quotient: dividend / divisor,
        quotient_digits,
        steps,
        remainder: rem,
        notation,
    }
}

/// Genera un problema aleatorio de división con la cantidad de dígitos solicitada.
pub fn generate_division(cfg: &DivisionConfig, rng: &mut Rng) -> DivisionProblem {
    let div_d = cfg.dividend_digits.clamp(2, 5);
    let dsr_d = cfg.divisor_digits.clamp(1, 3).min(div_d);

    for _ in 0..40 {
        let divisor = gen_number_with_digits(rng, dsr_d, 2);
        let mut dividend = gen_number_with_digits(rng, div_d, 2);

        if cfg.exact_only {
            let q = (dividend / divisor).max(2);
            let candidate = q * divisor;
            if number_to_digits(candidate).len() == div_d {
                dividend = candidate;
            } else {
                continue;
            }
        }

        if dividend <= divisor {
            dividend = divisor * (rng.gen_range(2, 9) as u64);
        }

        // Preferimos que el primer dígito (o prefijo) genere escalones vistosos
        if number_to_digits(dividend).len() == div_d {
            return build_division_problem(dividend, divisor, cfg.notation);
        }
    }

    let divisor = gen_number_with_digits(rng, dsr_d, 2);
    let dividend = gen_number_with_digits(rng, div_d, 3).max(divisor * 2);
    build_division_problem(dividend, divisor, cfg.notation)
}

/// Genera un problema aleatorio de suma según la cantidad de sumandos y sus dígitos.
pub fn generate_addition(cfg: &AdditionConfig, rng: &mut Rng) -> AdditionProblem {
    let count = cfg.addend_count.clamp(2, 5);
    let mut addends = Vec::with_capacity(count);
    let mut total = 0u64;
    let mut max_digits = 1usize;

    for i in 0..count {
        let d = cfg
            .digits_per_addend
            .get(i)
            .copied()
            .or_else(|| cfg.digits_per_addend.last().copied())
            .unwrap_or(3)
            .clamp(1, 5);
        max_digits = max_digits.max(d);
        let val = gen_number_with_digits(rng, d, 2);
        addends.push((val, d));
        total += val;
    }

    let total_len = number_to_digits(total).len();
    let total_boxes = total_len.max(max_digits + 1);

    AdditionProblem {
        addends,
        total,
        total_boxes,
    }
}

/// Genera un problema aleatorio de resta garantizando `minuendo > sustraendo`.
pub fn generate_subtraction(cfg: &SubtractionConfig, rng: &mut Rng) -> SubtractionProblem {
    let min_d = cfg.minuend_digits.clamp(1, 5);
    let sub_d = cfg.subtrahend_digits.clamp(1, min_d);

    let mut minuend = gen_number_with_digits(rng, min_d, 3);
    let mut subtrahend = gen_number_with_digits(rng, sub_d, 1);

    if subtrahend >= minuend {
        if min_d == sub_d {
            std::mem::swap(&mut minuend, &mut subtrahend);
            if minuend == subtrahend {
                minuend += 1;
            }
        } else {
            subtrahend = minuend / 2;
        }
    }

    let difference = minuend - subtrahend;
    SubtractionProblem {
        minuend,
        minuend_digits: min_d,
        subtrahend,
        subtrahend_digits: sub_d,
        difference,
        difference_boxes: min_d,
    }
}

/// Genera el conjunto completo de ejercicios para la ficha asegurando que no se repitan consecutivamente.
pub fn generate_worksheet_problems(cfg: &WorksheetConfig, rng: &mut Rng) -> Vec<MathProblem> {
    let mut problems = Vec::with_capacity(cfg.problem_count);

    for idx in 0..cfg.problem_count {
        let op = match cfg.operation {
            OperationKind::Mixto => match idx % 4 {
                0 => OperationKind::Multiplicacion,
                1 => OperationKind::Division,
                2 => OperationKind::Suma,
                _ => OperationKind::Resta,
            },
            specific => specific,
        };

        let mut candidate = match op {
            OperationKind::Multiplicacion => {
                MathProblem::Multiplicacion(generate_multiplication(&cfg.multiplication, rng))
            }
            OperationKind::Division => {
                MathProblem::Division(generate_division(&cfg.division, rng))
            }
            OperationKind::Suma => MathProblem::Suma(generate_addition(&cfg.addition, rng)),
            OperationKind::Resta => {
                MathProblem::Resta(generate_subtraction(&cfg.subtraction, rng))
            }
            OperationKind::Mixto => unreachable!(),
        };

        // Evitar duplicados exactos si ya existe en la lista
        for _ in 0..10 {
            if !problems.contains(&candidate) {
                break;
            }
            candidate = match op {
                OperationKind::Multiplicacion => {
                    MathProblem::Multiplicacion(generate_multiplication(&cfg.multiplication, rng))
                }
                OperationKind::Division => {
                    MathProblem::Division(generate_division(&cfg.division, rng))
                }
                OperationKind::Suma => MathProblem::Suma(generate_addition(&cfg.addition, rng)),
                OperationKind::Resta => {
                    MathProblem::Resta(generate_subtraction(&cfg.subtraction, rng))
                }
                OperationKind::Mixto => unreachable!(),
            };
        }

        problems.push(candidate);
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplication_breakdown() {
        // Caso exacto del ejercicio #1 de la imagen de referencia: 96 x 43
        let prob = build_multiplication_problem(96, 2, 43, 2);
        assert_eq!(prob.partial_products.len(), 2);
        assert_eq!(prob.partial_products[0].value, 288); // 96 * 3
        assert_eq!(prob.partial_products[0].shift, 0);
        assert_eq!(prob.partial_products[0].box_count, 3);
        assert_eq!(prob.partial_products[1].value, 384); // 96 * 4
        assert_eq!(prob.partial_products[1].shift, 1);
        assert_eq!(prob.partial_products[1].box_count, 3);
        assert_eq!(prob.product, 4128);
        assert_eq!(prob.product_boxes, 4);
    }

    #[test]
    fn test_division_breakdown_both_notations() {
        // 847 ÷ 6 = 141, residuo 1
        let prob_trad = build_division_problem(847, 6, DivisionNotation::Tradicional);
        assert_eq!(prob_trad.quotient, 141);
        assert_eq!(prob_trad.remainder, 1);
        assert_eq!(prob_trad.quotient_digits, vec![(0, 1), (1, 4), (2, 1)]);
        assert_eq!(prob_trad.steps.len(), 3);
        assert_eq!(prob_trad.steps[0].digits, vec![2, 4]);
        assert_eq!(prob_trad.steps[0].end_col, 1);
        assert_eq!(prob_trad.steps[1].digits, vec![0, 7]);
        assert_eq!(prob_trad.steps[1].end_col, 2);
        assert_eq!(prob_trad.steps[2].digits, vec![1]);
        assert!(prob_trad.steps[2].is_final_remainder);

        let prob_gal = build_division_problem(847, 6, DivisionNotation::Galera);
        assert_eq!(prob_gal.notation, DivisionNotation::Galera);
        assert_eq!(prob_gal.quotient, 141);
    }

    #[test]
    fn test_addition_and_subtraction_generation() {
        let mut rng = Rng::from_seed(42);
        let add_cfg = AdditionConfig {
            addend_count: 3,
            digits_per_addend: vec![3, 2, 3],
        };
        let add_prob = generate_addition(&add_cfg, &mut rng);
        assert_eq!(add_prob.addends.len(), 3);
        let sum_check: u64 = add_prob.addends.iter().map(|(v, _)| *v).sum();
        assert_eq!(add_prob.total, sum_check);

        let sub_cfg = SubtractionConfig {
            minuend_digits: 4,
            subtrahend_digits: 3,
        };
        let sub_prob = generate_subtraction(&sub_cfg, &mut rng);
        assert!(sub_prob.minuend > sub_prob.subtrahend);
        assert_eq!(sub_prob.minuend - sub_prob.subtrahend, sub_prob.difference);
    }
}
