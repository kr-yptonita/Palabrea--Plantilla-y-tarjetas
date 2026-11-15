use crate::math_gen::{
    number_to_boxed_digits, AdditionProblem, DivisionNotation, DivisionProblem, MathProblem,
    MultiplicationProblem, SubtractionProblem,
};

/// Símbolos de operación que se dibujan junto a las casillas.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OperatorSymbol {
    Mul,
    Add,
    Sub,
}

/// Tipo de celda dentro de la retícula de una tarjeta de ejercicio.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CellKind {
    /// Casilla con un dígito dado del enunciado (visible en ejercicios y respuestas).
    GivenDigit(u8),
    /// Casilla vacía para el alumno (en la hoja de respuestas muestra `Some(digito)` si aplica).
    AnswerDigit(Option<u8>),
    /// Símbolo operador (`×`, `+`, `-`) sin recuadro.
    Operator(OperatorSymbol),
}

/// Celda posicionada en unidades de cuadrícula local `(col, row)` desde la esquina superior izquierda.
#[derive(Clone, Debug)]
pub struct GridCell {
    pub col: f32,
    pub row: f32,
    pub kind: CellKind,
}

/// Segmento de línea recta (barra de suma/multiplicación o caja/galera de división) en coordenadas de cuadrícula.
/// Nota: Una casilla en `(col, row)` ocupa `[col, col + box_fill]` horizontalmente y `[row, row + box_fill]` verticalmente.
#[derive(Clone, Debug)]
pub struct RuleLine {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

/// Maquetación geométrica completa de un problema matemático dentro de una tarjeta.
#[derive(Clone, Debug)]
pub struct ProblemLayout {
    pub cells: Vec<GridCell>,
    pub rules: Vec<RuleLine>,
    /// Ancho total en unidades de celda.
    pub total_cols: f32,
    /// Alto total en unidades de celda.
    pub total_rows: f32,
}

/// Construye la maquetación de casillas y líneas para cualquier `MathProblem`.
pub fn build_problem_layout(problem: &MathProblem) -> ProblemLayout {
    match problem {
        MathProblem::Multiplicacion(m) => layout_multiplication(m),
        MathProblem::Division(d) => layout_division(d),
        MathProblem::Suma(a) => layout_addition(a),
        MathProblem::Resta(s) => layout_subtraction(s),
    }
}

fn layout_multiplication(prob: &MultiplicationProblem) -> ProblemLayout {
    let mut cells = Vec::new();
    let mut rules = Vec::new();

    let d1 = prob.multiplicand_digits;
    let d2 = prob.multiplier_digits;
    let p_boxes = prob.product_boxes;

    // Columna 0 reservada para el signo '+' (cuando d2 > 1) o '×' (cuando d2 == 1).
    // Las casillas de dígitos van de la columna 1.0 a `p_boxes as f32`.
    let units_col = p_boxes as f32;

    // Fila 0: Multiplicando (alineado a la derecha en `units_col`)
    let m1_digits = number_to_boxed_digits(prob.multiplicand, d1, true);
    let m1_start = units_col - (d1 as f32) + 1.0;
    for (i, d) in m1_digits.into_iter().enumerate() {
        cells.push(GridCell {
            col: m1_start + (i as f32),
            row: 0.0,
            kind: CellKind::GivenDigit(d.unwrap_or(0)),
        });
    }

    // Fila 1: Signo '×' + Multiplicador
    let m2_digits = number_to_boxed_digits(prob.multiplier, d2, true);
    let m2_start = units_col - (d2 as f32) + 1.0;
    cells.push(GridCell {
        col: m2_start - 1.0,
        row: 1.0,
        kind: CellKind::Operator(OperatorSymbol::Mul),
    });
    for (i, d) in m2_digits.into_iter().enumerate() {
        cells.push(GridCell {
            col: m2_start + (i as f32),
            row: 1.0,
            kind: CellKind::GivenDigit(d.unwrap_or(0)),
        });
    }

    // Primera línea horizontal bajo el multiplicador
    let top_bar_left = (m1_start.min(m2_start) - 0.95).max(0.0);
    let bar_right = units_col + 0.92;
    rules.push(RuleLine {
        x1: top_bar_left,
        y1: 2.06,
        x2: bar_right,
        y2: 2.06,
    });

    if d2 > 1 {
        // Filas de productos parciales
        for (step_idx, pp) in prob.partial_products.iter().enumerate() {
            let row_y = 2.22 + (step_idx as f32);
            let end_col = units_col - (pp.shift as f32);
            let start_col = end_col - (pp.box_count as f32) + 1.0;
            let pp_digits = number_to_boxed_digits(pp.value, pp.box_count, false);

            if step_idx == prob.partial_products.len() - 1 {
                cells.push(GridCell {
                    col: (start_col - 1.0).max(0.0),
                    row: row_y,
                    kind: CellKind::Operator(OperatorSymbol::Add),
                });
            }

            for (i, d_opt) in pp_digits.into_iter().enumerate() {
                cells.push(GridCell {
                    col: start_col + (i as f32),
                    row: row_y,
                    kind: CellKind::AnswerDigit(d_opt),
                });
            }
        }

        // Segunda línea horizontal bajo los productos parciales
        let second_bar_y = 2.22 + (d2 as f32) + 0.06;
        rules.push(RuleLine {
            x1: 0.05,
            y1: second_bar_y,
            x2: bar_right,
            y2: second_bar_y,
        });

        // Fila final del producto total
        let final_row_y = 2.22 + (d2 as f32) + 0.22;
        let prod_digits = number_to_boxed_digits(prob.product, p_boxes, false);
        for (i, d_opt) in prod_digits.into_iter().enumerate() {
            cells.push(GridCell {
                col: 1.0 + (i as f32),
                row: final_row_y,
                kind: CellKind::AnswerDigit(d_opt),
            });
        }

        ProblemLayout {
            cells,
            rules,
            total_cols: units_col + 1.0,
            total_rows: final_row_y + 1.0,
        }
    } else {
        // Multiplicador de 1 sola cifra: fila directa de producto
        let final_row_y = 2.22;
        let prod_digits = number_to_boxed_digits(prob.product, p_boxes, false);
        for (i, d_opt) in prod_digits.into_iter().enumerate() {
            cells.push(GridCell {
                col: 1.0 + (i as f32),
                row: final_row_y,
                kind: CellKind::AnswerDigit(d_opt),
            });
        }

        ProblemLayout {
            cells,
            rules,
            total_cols: units_col + 1.0,
            total_rows: final_row_y + 1.0,
        }
    }
}

fn layout_addition(prob: &AdditionProblem) -> ProblemLayout {
    let mut cells = Vec::new();
    let mut rules = Vec::new();

    let p_boxes = prob.total_boxes;
    let units_col = p_boxes as f32;
    let n_addends = prob.addends.len();

    for (row_idx, &(val, width)) in prob.addends.iter().enumerate() {
        let row_y = row_idx as f32;
        let start_col = units_col - (width as f32) + 1.0;
        let digits = number_to_boxed_digits(val, width, true);

        if row_idx == n_addends - 1 {
            cells.push(GridCell {
                col: 0.0,
                row: row_y,
                kind: CellKind::Operator(OperatorSymbol::Add),
            });
        }

        for (i, d) in digits.into_iter().enumerate() {
            cells.push(GridCell {
                col: start_col + (i as f32),
                row: row_y,
                kind: CellKind::GivenDigit(d.unwrap_or(0)),
            });
        }
    }

    let bar_y = (n_addends as f32) + 0.06;
    rules.push(RuleLine {
        x1: 0.05,
        y1: bar_y,
        x2: units_col + 0.92,
        y2: bar_y,
    });

    let final_row_y = (n_addends as f32) + 0.22;
    let sum_digits = number_to_boxed_digits(prob.total, p_boxes, false);
    for (i, d_opt) in sum_digits.into_iter().enumerate() {
        cells.push(GridCell {
            col: 1.0 + (i as f32),
            row: final_row_y,
            kind: CellKind::AnswerDigit(d_opt),
        });
    }

    ProblemLayout {
        cells,
        rules,
        total_cols: units_col + 1.0,
        total_rows: final_row_y + 1.0,
    }
}

fn layout_subtraction(prob: &SubtractionProblem) -> ProblemLayout {
    let mut cells = Vec::new();
    let mut rules = Vec::new();

    let cols = prob.minuend_digits;
    let units_col = cols as f32;

    // Fila 0: Minuendo
    let min_digits = number_to_boxed_digits(prob.minuend, prob.minuend_digits, true);
    for (i, d) in min_digits.into_iter().enumerate() {
        cells.push(GridCell {
            col: 1.0 + (i as f32),
            row: 0.0,
            kind: CellKind::GivenDigit(d.unwrap_or(0)),
        });
    }

    // Fila 1: Signo '-' + Sustraendo alineado a la derecha
    cells.push(GridCell {
        col: 0.0,
        row: 1.0,
        kind: CellKind::Operator(OperatorSymbol::Sub),
    });
    let sub_start = units_col - (prob.subtrahend_digits as f32) + 1.0;
    let sub_digits = number_to_boxed_digits(prob.subtrahend, prob.subtrahend_digits, true);
    for (i, d) in sub_digits.into_iter().enumerate() {
        cells.push(GridCell {
            col: sub_start + (i as f32),
            row: 1.0,
            kind: CellKind::GivenDigit(d.unwrap_or(0)),
        });
    }

    // Barra horizontal
    rules.push(RuleLine {
        x1: 0.05,
        y1: 2.06,
        x2: units_col + 0.92,
        y2: 2.06,
    });

    // Fila 2: Diferencia
    let final_row_y = 2.22;
    let diff_digits = number_to_boxed_digits(prob.difference, prob.difference_boxes, false);
    for (i, d_opt) in diff_digits.into_iter().enumerate() {
        cells.push(GridCell {
            col: 1.0 + (i as f32),
            row: final_row_y,
            kind: CellKind::AnswerDigit(d_opt),
        });
    }

    ProblemLayout {
        cells,
        rules,
        total_cols: units_col + 1.0,
        total_rows: final_row_y + 1.0,
    }
}

fn layout_division(prob: &DivisionProblem) -> ProblemLayout {
    match prob.notation {
        DivisionNotation::Tradicional => layout_division_tradicional(prob),
        DivisionNotation::Galera => layout_division_galera(prob),
    }
}

/// Maquetación de división en caja tradicional hispana (`Dividendo | Divisor`, cociente debajo del divisor).
fn layout_division_tradicional(prob: &DivisionProblem) -> ProblemLayout {
    let mut cells = Vec::new();
    let mut rules = Vec::new();

    let n_dvd = prob.dividend_digits.len();
    let n_dsr = prob.divisor_digits.len();
    let n_quo = prob.quotient_digits.len();

    // 1. Dividendo en fila 0, columnas 0..n_dvd
    for (i, &d) in prob.dividend_digits.iter().enumerate() {
        cells.push(GridCell {
            col: i as f32,
            row: 0.0,
            kind: CellKind::GivenDigit(d),
        });
    }

    // Separación para la barra vertical de la caja
    let bar_x = (n_dvd as f32) + 0.08;
    let right_start_col = (n_dvd as f32) + 0.28;

    // 2. Divisor en fila 0 a la derecha de la caja
    for (i, &d) in prob.divisor_digits.iter().enumerate() {
        cells.push(GridCell {
            col: right_start_col + (i as f32),
            row: 0.0,
            kind: CellKind::GivenDigit(d),
        });
    }

    // Líneas de la caja tradicional (vertical a la izquierda del divisor + horizontal debajo del divisor)
    let right_cols = n_dsr.max(n_quo) as f32;
    let box_bottom_y = 1.05;
    rules.push(RuleLine {
        x1: bar_x,
        y1: -0.04,
        x2: bar_x,
        y2: box_bottom_y,
    });
    rules.push(RuleLine {
        x1: bar_x,
        y1: box_bottom_y,
        x2: right_start_col + right_cols - 0.08,
        y2: box_bottom_y,
    });

    // 3. Cociente en fila 1.22 debajo del divisor
    for (i, &(_, q_digit)) in prob.quotient_digits.iter().enumerate() {
        cells.push(GridCell {
            col: right_start_col + (i as f32),
            row: 1.22,
            kind: CellKind::AnswerDigit(Some(q_digit)),
        });
    }

    // 4. Restos parciales escalonados debajo del dividendo
    let mut max_row = 1.22f32;
    for (step_idx, step) in prob.steps.iter().enumerate() {
        let row_y = ((step_idx + 1) as f32) * 1.0;
        if row_y > max_row {
            max_row = row_y;
        }
        let len = step.digits.len();
        let start_col = (step.end_col + 1).saturating_sub(len);
        for (i, &d) in step.digits.iter().enumerate() {
            cells.push(GridCell {
                col: (start_col + i) as f32,
                row: row_y,
                kind: CellKind::AnswerDigit(Some(d)),
            });
        }
    }

    ProblemLayout {
        cells,
        rules,
        total_cols: right_start_col + right_cols,
        total_rows: max_row + 1.0,
    }
}

/// Maquetación de división en galera / casita (`Divisor ⟌ Dividendo`, cociente arriba del dividendo).
fn layout_division_galera(prob: &DivisionProblem) -> ProblemLayout {
    let mut cells = Vec::new();
    let mut rules = Vec::new();

    let n_dvd = prob.dividend_digits.len();
    let n_dsr = prob.divisor_digits.len();

    let wall_x = (n_dsr as f32) + 0.08;
    let dvd_start_col = (n_dsr as f32) + 0.28;
    let dvd_row_y = 1.22;

    // 1. Cociente en la fila 0.0 (arriba del techo de la galera, alineado con las columnas del dividendo)
    for &(dvd_col, q_digit) in &prob.quotient_digits {
        cells.push(GridCell {
            col: dvd_start_col + (dvd_col as f32),
            row: 0.0,
            kind: CellKind::AnswerDigit(Some(q_digit)),
        });
    }

    // 2. Techo y pared vertical de la galera (casita)
    let roof_y = 1.08;
    rules.push(RuleLine {
        x1: wall_x,
        y1: roof_y,
        x2: dvd_start_col + (n_dvd as f32) - 0.08,
        y2: roof_y,
    });
    rules.push(RuleLine {
        x1: wall_x,
        y1: roof_y,
        x2: wall_x,
        y2: dvd_row_y + 0.92,
    });

    // 3. Divisor en fila `dvd_row_y` a la izquierda de la pared
    for (i, &d) in prob.divisor_digits.iter().enumerate() {
        cells.push(GridCell {
            col: i as f32,
            row: dvd_row_y,
            kind: CellKind::GivenDigit(d),
        });
    }

    // 4. Dividendo en fila `dvd_row_y` a la derecha de la pared
    for (i, &d) in prob.dividend_digits.iter().enumerate() {
        cells.push(GridCell {
            col: dvd_start_col + (i as f32),
            row: dvd_row_y,
            kind: CellKind::GivenDigit(d),
        });
    }

    // 5. Restos parciales escalonados debajo del dividendo
    let mut max_row = dvd_row_y;
    for (step_idx, step) in prob.steps.iter().enumerate() {
        let row_y = dvd_row_y + ((step_idx + 1) as f32) * 1.0;
        if row_y > max_row {
            max_row = row_y;
        }
        let len = step.digits.len();
        let start_col = (step.end_col + 1).saturating_sub(len);
        for (i, &d) in step.digits.iter().enumerate() {
            cells.push(GridCell {
                col: dvd_start_col + ((start_col + i) as f32),
                row: row_y,
                kind: CellKind::AnswerDigit(Some(d)),
            });
        }
    }

    ProblemLayout {
        cells,
        rules,
        total_cols: dvd_start_col + (n_dvd as f32),
        total_rows: max_row + 1.0,
    }
}
