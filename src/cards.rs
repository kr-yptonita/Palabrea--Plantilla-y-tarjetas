use crate::lexicon::{are_antonyms, are_synonyms, Category, Rng, WordEntry, LEXICON};
use std::collections::HashSet;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CardKind {
    CategoryAndSyllables,
    CategoryAndLetters,
    SyllablesAndLetters,
    PhoneticAndOrthographic,
    SynonymsAndAntonyms,
}

#[derive(Clone, Debug)]
pub struct Card {
    pub prompt: String,
    pub kind: CardKind,
    pub sample_solutions: Vec<&'static str>,
}

fn number_word_es(n: usize) -> String {
    match n {
        1 => "una".to_string(),
        2 => "dos".to_string(),
        3 => "tres".to_string(),
        4 => "cuatro".to_string(),
        5 => "cinco".to_string(),
        6 => "seis".to_string(),
        7 => "siete".to_string(),
        8 => "ocho".to_string(),
        9 => "nueve".to_string(),
        10 => "diez".to_string(),
        _ => n.to_string(),
    }
}

/// Genera exactamente `count` tarjetas cuyas respuestas existen dentro de `placed_entries`.
pub fn generate_cards(
    placed_entries: &[&'static WordEntry],
    count: usize,
    rng: &mut Rng,
) -> Vec<Card> {
    if count == 0 {
        return Vec::new();
    }

    let mut by_cat_syl: Vec<Card> = Vec::new();
    let mut by_cat_let: Vec<Card> = Vec::new();
    let mut by_syl_let: Vec<Card> = Vec::new();
    let mut by_phonetic: Vec<Card> = Vec::new();
    let mut by_syn_ant: Vec<Card> = Vec::new();

    let verbs = ["Localiza", "Busca", "Encuentra"];

    // =========================================================================
    // 1. SINÓNIMOS Y ANTÓNIMOS (Basados en las palabras presentes en la hoja)
    // =========================================================================
    let mut placed_antonym_pairs: Vec<(&'static WordEntry, &'static WordEntry)> = Vec::new();
    let mut placed_synonym_pairs: Vec<(&'static WordEntry, &'static WordEntry)> = Vec::new();

    for (i, &a) in placed_entries.iter().enumerate() {
        for &b in &placed_entries[i + 1..] {
            if are_antonyms(a, b) {
                placed_antonym_pairs.push((a, b));
            }
            if are_synonyms(a, b) {
                placed_synonym_pairs.push((a, b));
            }
        }
    }
    rng.shuffle(&mut placed_antonym_pairs);
    rng.shuffle(&mut placed_synonym_pairs);

    // 1a. Parejas completas de antónimos en la hoja
    if let Some(&(a, b)) = placed_antonym_pairs.first() {
        by_syn_ant.push(Card {
            prompt: "Localiza dos palabras en la hoja que sean antónimos (significados opuestos)."
                .to_string(),
            kind: CardKind::SynonymsAndAntonyms,
            sample_solutions: vec![a.word, b.word],
        });
    }
    if placed_antonym_pairs.len() >= 2 {
        let (a1, b1) = placed_antonym_pairs[1];
        by_syn_ant.push(Card {
            prompt: format!(
                "Busca dos palabras opuestas entre sí (antónimos) que tengan {} y {} sílabas.",
                a1.syllables, b1.syllables
            ),
            kind: CardKind::SynonymsAndAntonyms,
            sample_solutions: vec![a1.word, b1.word],
        });
    }

    // 1b. Parejas completas de sinónimos en la hoja
    if let Some(&(a, b)) = placed_synonym_pairs.first() {
        by_syn_ant.push(Card {
            prompt: "Busca dos palabras en la hoja que sean sinónimos (significados parecidos)."
                .to_string(),
            kind: CardKind::SynonymsAndAntonyms,
            sample_solutions: vec![a.word, b.word],
        });
    }
    if placed_synonym_pairs.len() >= 2 {
        let (a1, b1) = placed_synonym_pairs[1];
        by_syn_ant.push(Card {
            prompt: format!(
                "Encuentra dos palabras que sean sinónimos {} en la hoja.",
                a1.category.related_phrase()
            ),
            kind: CardKind::SynonymsAndAntonyms,
            sample_solutions: vec![a1.word, b1.word],
        });
    }

    // 1c. Buscar en la hoja el antónimo de dos palabras concretas del diccionario (de distinto par)
    let mut single_antonym_clues: Vec<(&'static str, &'static WordEntry)> = Vec::new();
    let mut used_ant_groups: HashSet<u16> = HashSet::new();
    for &placed in placed_entries {
        if let Some((group_id, _)) = placed.antonym_pair {
            if used_ant_groups.insert(group_id) {
                if let Some(other) = LEXICON.iter().find(|cand| are_antonyms(placed, cand)) {
                    single_antonym_clues.push((other.word, placed));
                }
            }
        }
    }
    rng.shuffle(&mut single_antonym_clues);
    for chunk in single_antonym_clues.chunks(2) {
        if chunk.len() == 2 && chunk[0].1.word != chunk[1].1.word {
            let v = verbs[rng.gen_range(0, verbs.len())];
            by_syn_ant.push(Card {
                prompt: format!(
                    "{} en la hoja el antónimo de «{}» y el antónimo de «{}».",
                    v, chunk[0].0, chunk[1].0
                ),
                kind: CardKind::SynonymsAndAntonyms,
                sample_solutions: vec![chunk[0].1.word, chunk[1].1.word],
            });
        }
    }

    // 1d. Buscar en la hoja el sinónimo de dos palabras concretas del diccionario (de distinto grupo)
    let mut single_synonym_clues: Vec<(&'static str, &'static WordEntry)> = Vec::new();
    let mut used_syn_groups: HashSet<u16> = HashSet::new();
    for &placed in placed_entries {
        if let Some(group_id) = placed.synonym_group {
            if used_syn_groups.insert(group_id) {
                if let Some(other) = LEXICON.iter().find(|cand| are_synonyms(placed, cand)) {
                    single_synonym_clues.push((other.word, placed));
                }
            }
        }
    }
    rng.shuffle(&mut single_synonym_clues);
    for chunk in single_synonym_clues.chunks(2) {
        if chunk.len() == 2 && chunk[0].1.word != chunk[1].1.word {
            let v = verbs[rng.gen_range(0, verbs.len())];
            by_syn_ant.push(Card {
                prompt: format!(
                    "{} en la hoja un sinónimo de «{}» y un sinónimo de «{}».",
                    v, chunk[0].0, chunk[1].0
                ),
                kind: CardKind::SynonymsAndAntonyms,
                sample_solutions: vec![chunk[0].1.word, chunk[1].1.word],
            });
        }
    }

    // 1e. Combinación de un sinónimo y un antónimo
    if let (Some(&ant), Some(&syn)) = (single_antonym_clues.first(), single_synonym_clues.first()) {
        if ant.1.word != syn.1.word {
            by_syn_ant.push(Card {
                prompt: format!(
                    "Encuentra el antónimo de «{}» y el sinónimo de «{}».",
                    ant.0, syn.0
                ),
                kind: CardKind::SynonymsAndAntonyms,
                sample_solutions: vec![ant.1.word, syn.1.word],
            });
        }
    }

    // =========================================================================
    // 2. CATEGORÍA SEMÁNTICA + SÍLABAS
    // =========================================================================
    for &cat in Category::ALL {
        let cat_words: Vec<&'static WordEntry> = placed_entries
            .iter()
            .copied()
            .filter(|e| e.category == cat)
            .collect();

        if cat_words.len() >= 2 {
            // Dos palabras de la categoría con diferente número de sílabas
            if let Some(second) = cat_words
                .iter()
                .find(|w| w.syllables != cat_words[0].syllables)
            {
                let v = verbs[rng.gen_range(0, verbs.len())];
                by_cat_syl.push(Card {
                    prompt: format!(
                        "{} {} con diferente número de sílabas.",
                        v,
                        cat.plural_noun("dos")
                    ),
                    kind: CardKind::CategoryAndSyllables,
                    sample_solutions: vec![cat_words[0].word, second.word],
                });
            }
        }

        // Agrupar por número exacto de sílabas dentro de la categoría
        for syl in 1..=5u8 {
            let matching: Vec<&'static WordEntry> = cat_words
                .iter()
                .copied()
                .filter(|e| e.syllables == syl)
                .collect();
            if matching.len() >= 2 {
                let v = verbs[rng.gen_range(0, verbs.len())];
                let syl_txt = if syl == 1 {
                    "una sílaba".to_string()
                } else {
                    format!("{} sílabas", number_word_es(syl as usize))
                };
                by_cat_syl.push(Card {
                    prompt: format!(
                        "{} dos palabras con {} {}.",
                        v,
                        syl_txt,
                        cat.related_phrase()
                    ),
                    kind: CardKind::CategoryAndSyllables,
                    sample_solutions: vec![matching[0].word, matching[1].word],
                });
            }
        }

        // Combinación de dos conteos de sílabas distintos dentro de la misma categoría
        for s1 in 1..=4u8 {
            for s2 in (s1 + 1)..=5u8 {
                let w1 = cat_words.iter().copied().find(|e| e.syllables == s1);
                let w2 = cat_words.iter().copied().find(|e| e.syllables == s2);
                if let (Some(a), Some(b)) = (w1, w2) {
                    let s1_label = if s1 == 1 {
                        "1 sílaba".to_string()
                    } else {
                        format!("{} sílabas", s1)
                    };
                    by_cat_syl.push(Card {
                        prompt: format!(
                            "Encuentra {} con {} y {} con {} sílabas.",
                            cat.singular_noun(),
                            s1_label,
                            cat.another_pronoun(),
                            s2
                        ),
                        kind: CardKind::CategoryAndSyllables,
                        sample_solutions: vec![a.word, b.word],
                    });
                }
            }
        }
    }

    // =========================================================================
    // 3. CATEGORÍA SEMÁNTICA + NÚMERO DE LETRAS
    // =========================================================================
    for &cat in Category::ALL {
        let cat_words: Vec<&'static WordEntry> = placed_entries
            .iter()
            .copied()
            .filter(|e| e.category == cat)
            .collect();

        if cat_words.len() >= 2 {
            if let Some(second) = cat_words
                .iter()
                .find(|w| w.letters() != cat_words[0].letters())
            {
                let v = verbs[rng.gen_range(0, verbs.len())];
                by_cat_let.push(Card {
                    prompt: format!(
                        "{} {} con diferente número de letras.",
                        v,
                        cat.plural_noun("dos")
                    ),
                    kind: CardKind::CategoryAndLetters,
                    sample_solutions: vec![cat_words[0].word, second.word],
                });
            }
        }

        for len in 3..=9usize {
            let matching: Vec<&'static WordEntry> = cat_words
                .iter()
                .copied()
                .filter(|e| e.letters() == len)
                .collect();
            if matching.len() >= 2 {
                let v = verbs[rng.gen_range(0, verbs.len())];
                by_cat_let.push(Card {
                    prompt: format!(
                        "{} {} que tengan {} letras.",
                        v,
                        cat.plural_noun("dos"),
                        number_word_es(len)
                    ),
                    kind: CardKind::CategoryAndLetters,
                    sample_solutions: vec![matching[0].word, matching[1].word],
                });
            }
        }

        let long_cat: Vec<&'static WordEntry> = cat_words
            .iter()
            .copied()
            .filter(|e| e.letters() >= 7)
            .collect();
        if long_cat.len() >= 2 {
            by_cat_let.push(Card {
                prompt: format!(
                    "Localiza {} que tengan 7 o más letras.",
                    cat.plural_noun("dos")
                ),
                kind: CardKind::CategoryAndLetters,
                sample_solutions: vec![long_cat[0].word, long_cat[1].word],
            });
        }
    }

    // =========================================================================
    // 4. CONTEO DE LETRAS Y SÍLABAS GENERAL
    // =========================================================================
    for len in 3..=10usize {
        let matching: Vec<&'static WordEntry> = placed_entries
            .iter()
            .copied()
            .filter(|e| e.letters() == len)
            .collect();
        if matching.len() >= 2 {
            by_syl_let.push(Card {
                prompt: format!(
                    "Localiza dos palabras con {} letras.",
                    number_word_es(len)
                ),
                kind: CardKind::SyllablesAndLetters,
                sample_solutions: vec![matching[0].word, matching[1].word],
            });
        }
        if matching.len() >= 3 {
            by_syl_let.push(Card {
                prompt: format!(
                    "Busca tres palabras que tengan exactamente {} letras.",
                    number_word_es(len)
                ),
                kind: CardKind::SyllablesAndLetters,
                sample_solutions: vec![matching[0].word, matching[1].word, matching[2].word],
            });
        }
    }

    for syl in 1..=5u8 {
        let matching: Vec<&'static WordEntry> = placed_entries
            .iter()
            .copied()
            .filter(|e| e.syllables == syl)
            .collect();
        if matching.len() >= 3 {
            let syl_label = if syl == 1 {
                "una sola sílaba (monosílabas)".to_string()
            } else {
                format!("{} sílabas", number_word_es(syl as usize))
            };
            by_syl_let.push(Card {
                prompt: format!("Encuentra tres palabras con {}.", syl_label),
                kind: CardKind::SyllablesAndLetters,
                sample_solutions: vec![matching[0].word, matching[1].word, matching[2].word],
            });
        }
    }

    for syl in 2..=4u8 {
        for len in 4..=8usize {
            let matching: Vec<&'static WordEntry> = placed_entries
                .iter()
                .copied()
                .filter(|e| e.syllables == syl && e.letters() == len)
                .collect();
            if matching.len() >= 2 {
                by_syl_let.push(Card {
                    prompt: format!(
                        "Busca dos palabras con {} sílabas y {} letras.",
                        number_word_es(syl as usize),
                        number_word_es(len)
                    ),
                    kind: CardKind::SyllablesAndLetters,
                    sample_solutions: vec![matching[0].word, matching[1].word],
                });
            }
        }
    }

    // =========================================================================
    // 5. LETRA INICIAL, TERMINACIÓN, VOCALES Y TILDES
    // =========================================================================
    // 5a. Empiezan por vocal + sílabas
    for syl in 2..=4u8 {
        let matching: Vec<&'static WordEntry> = placed_entries
            .iter()
            .copied()
            .filter(|e| e.starts_with_vowel() && e.syllables == syl)
            .collect();
        if matching.len() >= 2 {
            by_phonetic.push(Card {
                prompt: format!(
                    "Localiza dos palabras que empiecen por vocal y tengan {} sílabas.",
                    number_word_es(syl as usize)
                ),
                kind: CardKind::PhoneticAndOrthographic,
                sample_solutions: vec![matching[0].word, matching[1].word],
            });
        }
    }

    // 5b. Empiezan por consonante específica
    for ch in ['c', 'p', 'm', 'b', 't', 's', 'g', 'l', 'f', 'd', 'r', 'v'] {
        let matching: Vec<&'static WordEntry> = placed_entries
            .iter()
            .copied()
            .filter(|e| e.first_letter_base() == ch)
            .collect();
        if matching.len() >= 2 {
            by_phonetic.push(Card {
                prompt: format!(
                    "Busca dos palabras que empiecen por la letra «{}».",
                    ch.to_ascii_uppercase()
                ),
                kind: CardKind::PhoneticAndOrthographic,
                sample_solutions: vec![matching[0].word, matching[1].word],
            });
        }
    }

    // 5c. Terminan en vocal específica o en consonante
    for end_v in ['a', 'e', 'o'] {
        let matching: Vec<&'static WordEntry> = placed_entries
            .iter()
            .copied()
            .filter(|e| e.last_letter_base() == end_v && e.syllables == 3)
            .collect();
        if matching.len() >= 2 {
            by_phonetic.push(Card {
                prompt: format!(
                    "Encuentra dos palabras de tres sílabas que terminen en la vocal «{}».",
                    end_v
                ),
                kind: CardKind::PhoneticAndOrthographic,
                sample_solutions: vec![matching[0].word, matching[1].word],
            });
        }
    }

    let end_consonant: Vec<&'static WordEntry> = placed_entries
        .iter()
        .copied()
        .filter(|e| !e.ends_with_vowel())
        .collect();
    if end_consonant.len() >= 2 {
        by_phonetic.push(Card {
            prompt: "Localiza dos palabras que terminen en consonante.".to_string(),
            kind: CardKind::PhoneticAndOrthographic,
            sample_solutions: vec![end_consonant[0].word, end_consonant[1].word],
        });
    }

    // 5d. Palabras con tilde
    let accented: Vec<&'static WordEntry> = placed_entries
        .iter()
        .copied()
        .filter(|e| e.has_accent())
        .collect();
    if accented.len() >= 2 {
        by_phonetic.push(Card {
            prompt: "Busca dos palabras en la hoja que lleven tilde (acento gráfico).".to_string(),
            kind: CardKind::PhoneticAndOrthographic,
            sample_solutions: vec![accented[0].word, accented[1].word],
        });
    }
    let accented_long: Vec<&'static WordEntry> = accented
        .iter()
        .copied()
        .filter(|e| e.letters() >= 6)
        .collect();
    if accented_long.len() >= 2 {
        by_phonetic.push(Card {
            prompt: "Encuentra dos palabras con tilde que tengan 6 o más letras.".to_string(),
            kind: CardKind::PhoneticAndOrthographic,
            sample_solutions: vec![accented_long[0].word, accented_long[1].word],
        });
    }

    // 5e. Cantidad exacta de vocales
    for v_count in 2..=5usize {
        let matching: Vec<&'static WordEntry> = placed_entries
            .iter()
            .copied()
            .filter(|e| e.vowels() == v_count)
            .collect();
        if matching.len() >= 2 {
            by_phonetic.push(Card {
                prompt: format!(
                    "Localiza dos palabras que tengan exactamente {} vocales.",
                    number_word_es(v_count)
                ),
                kind: CardKind::PhoneticAndOrthographic,
                sample_solutions: vec![matching[0].word, matching[1].word],
            });
        }
    }

    // Barajar cada familia de tarjetas
    rng.shuffle(&mut by_syn_ant);
    rng.shuffle(&mut by_cat_syl);
    rng.shuffle(&mut by_cat_let);
    rng.shuffle(&mut by_syl_let);
    rng.shuffle(&mut by_phonetic);

    // Intercalar equitativamente entre las 5 familias para que el mazo sea muy variado
    let mut pools = [
        by_syn_ant.into_iter(),
        by_cat_syl.into_iter(),
        by_cat_let.into_iter(),
        by_syl_let.into_iter(),
        by_phonetic.into_iter(),
    ];

    let mut result: Vec<Card> = Vec::with_capacity(count);
    let mut seen_prompts: HashSet<String> = HashSet::new();

    while result.len() < count {
        let mut added_in_round = false;
        for pool in &mut pools {
            if result.len() >= count {
                break;
            }
            while let Some(card) = pool.next() {
                if seen_prompts.insert(card.prompt.clone()) {
                    result.push(card);
                    added_in_round = true;
                    break;
                }
            }
        }
        if !added_in_round {
            break;
        }
    }

    // Si el usuario pide más tarjetas que el banco único (ej. >150), generar variaciones válidas adicionales
    let mut extra_idx = 0usize;
    while result.len() < count && placed_entries.len() >= 2 {
        let a = placed_entries[extra_idx % placed_entries.len()];
        let b = placed_entries[(extra_idx * 7 + 3) % placed_entries.len()];
        extra_idx += 1;
        if a.word == b.word {
            continue;
        }
        let prompt = format!(
            "Localiza {} de {} letras y {} de {} sílabas.",
            a.category.singular_noun(),
            a.letters(),
            b.category.singular_noun(),
            b.syllables
        );
        if seen_prompts.insert(prompt.clone()) {
            result.push(Card {
                prompt,
                kind: CardKind::CategoryAndSyllables,
                sample_solutions: vec![a.word, b.word],
            });
        } else if extra_idx > count * 4 {
            // Si ya agotamos combinaciones únicas, repetir ciclando desde el inicio
            let base = result[extra_idx % result.len()].clone();
            result.push(base);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wordcloud::generate_wordcloud_sheet;

    #[test]
    fn test_cards_all_have_valid_solutions_on_sheet() {
        let mut rng = Rng::from_seed(12345);
        let sheet = generate_wordcloud_sheet(841.89, 595.28, &mut rng);
        let placed_entries: Vec<&'static WordEntry> =
            sheet.placed_words.iter().map(|pw| pw.entry).collect();
        let placed_set: HashSet<&'static str> =
            placed_entries.iter().map(|e| e.word).collect();

        let cards = generate_cards(&placed_entries, 30, &mut rng);
        assert_eq!(cards.len(), 30);

        let mut has_syn_ant = false;
        for card in &cards {
            assert!(!card.prompt.is_empty());
            assert!(
                card.sample_solutions.len() >= 2,
                "La tarjeta '{}' tiene menos de 2 soluciones",
                card.prompt
            );
            for &sol in &card.sample_solutions {
                assert!(
                    placed_set.contains(sol),
                    "La solución '{}' para la tarjeta '{}' no está en la hoja generada",
                    sol,
                    card.prompt
                );
            }
            if card.kind == CardKind::SynonymsAndAntonyms {
                has_syn_ant = true;
            }
        }
        assert!(has_syn_ant, "El mazo debe incluir tarjetas de sinónimos y antónimos");
    }
}
