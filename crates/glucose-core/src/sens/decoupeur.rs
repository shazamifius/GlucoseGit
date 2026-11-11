//! **Le découpeur** (SENS-1, fiche 106) : un texte, puis les rangs de ses morceaux de mots — le
//! WordPiece de BERT multilingue, sans majuscules ni accents, réécrit à l'identique de la référence
//! (le `BertNormalizer`, le `BertPreTokenizer` et le `WordPiece` de Hugging Face) pour que le noyau
//! reste sans dépendance. Une épreuve le compare aux rangs de la référence, texte piège par texte
//! piège.
//!
//! Dans l'ordre de la référence :
//! 1. **nettoyer** : les caractères de contrôle, de mise en forme et d'usage privé partent, tout blanc
//!    devient une espace, chaque idéogramme chinois est isolé ;
//! 2. **retirer les accents** : la décomposition canonique, sans ses marques ;
//! 3. **les minuscules**, caractère par caractère (un sigma final reste « σ », comme la référence) ;
//! 4. **les mots** : séparés par les blancs, chaque signe de ponctuation à part ;
//! 5. **les morceaux** : le plus long que le vocabulaire connaît d'abord, puis les suites ; un mot
//!    qu'on ne sait pas finir, ou de plus de cent caractères, devient « [UNK] ».

use super::unicode::{MARQUES, MISE_EN_FORME, PONCTUATION, SANS_ACCENT};

/// Ce que le découpeur demande au vocabulaire.
pub(super) trait Vocabulaire {
    /// Le rang d'un morceau qui commence un mot.
    fn debut(&self, morceau: &str) -> Option<u32>;
    /// Le rang d'un morceau qui en continue un (« ##morceau »).
    fn suite(&self, morceau: &str) -> Option<u32>;
    /// Le rang de « [UNK] ».
    fn inconnu(&self) -> u32;
}

/// Au-delà, un mot est « [UNK] » sans être cherché : la `max_input_chars_per_word` de la référence.
const CARACTERES_PAR_MOT: usize = 100;

/// **Les rangs des morceaux de ce texte.**
pub(super) fn decouper(texte: &str, vocabulaire: &impl Vocabulaire) -> Vec<u32> {
    let normal = normaliser(texte);
    let mut rangs = Vec::new();
    for mot in mots(&normal) {
        morceaux(mot, vocabulaire, &mut rangs);
    }
    rangs
}

/// Les étapes 1 à 3 : nettoyé, sans accents, en minuscules.
fn normaliser(texte: &str) -> String {
    let mut propre = String::with_capacity(texte.len());
    for c in texte.chars() {
        if c == '\0' || c == '\u{FFFD}' || est_controle(c) {
            continue;
        }
        if est_blanc(c) {
            propre.push(' ');
        } else if est_ideogramme(c) {
            propre.extend([' ', c, ' ']);
        } else {
            propre.push(c);
        }
    }
    let mut sans_accent = String::with_capacity(propre.len());
    for c in propre.chars() {
        retirer_les_marques(c, &mut sans_accent);
    }
    let mut minuscules = String::with_capacity(sans_accent.len());
    for c in sans_accent.chars() {
        minuscules.extend(c.to_lowercase());
    }
    minuscules
}

/// L'étape 4 : les mots, chaque signe de ponctuation à part.
fn mots(texte: &str) -> Vec<&str> {
    let mut mots = Vec::new();
    for bloc in texte.split(est_blanc).filter(|b| !b.is_empty()) {
        let mut debut = 0;
        for (i, c) in bloc.char_indices() {
            if est_ponctuation(c) {
                if debut < i {
                    mots.push(&bloc[debut..i]);
                }
                mots.push(&bloc[i..i + c.len_utf8()]);
                debut = i + c.len_utf8();
            }
        }
        if debut < bloc.len() {
            mots.push(&bloc[debut..]);
        }
    }
    mots
}

/// L'étape 5 : le plus long morceau connu d'abord.
fn morceaux(mot: &str, vocabulaire: &impl Vocabulaire, rangs: &mut Vec<u32>) {
    let bornes: Vec<usize> = mot
        .char_indices()
        .map(|(i, _)| i)
        .chain([mot.len()])
        .collect();
    let n = bornes.len() - 1;
    if n > CARACTERES_PAR_MOT {
        rangs.push(vocabulaire.inconnu());
        return;
    }
    let premier = rangs.len();
    let mut debut = 0;
    while debut < n {
        let trouve = (debut + 1..=n).rev().find_map(|fin| {
            let morceau = &mot[bornes[debut]..bornes[fin]];
            let rang = if debut == 0 {
                vocabulaire.debut(morceau)
            } else {
                vocabulaire.suite(morceau)
            };
            rang.map(|r| (r, fin))
        });
        let Some((rang, fin)) = trouve else {
            rangs.truncate(premier);
            rangs.push(vocabulaire.inconnu());
            return;
        };
        rangs.push(rang);
        debut = fin;
    }
}

/// Un caractère qui part au nettoyage : contrôle, mise en forme, usage privé (la catégorie « Other »
/// de la référence, hors tabulation et fins de ligne).
fn est_controle(c: char) -> bool {
    if matches!(c, '\t' | '\n' | '\r') {
        return false;
    }
    let o = c as u32;
    c.is_control()
        || dans(MISE_EN_FORME, o)
        || (0xE000..=0xF8FF).contains(&o)
        || (0xF0000..=0xFFFFD).contains(&o)
        || (0x10_0000..=0x10_FFFD).contains(&o)
}

/// Un blanc, au sens de la référence.
fn est_blanc(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r') || c.is_whitespace()
}

/// Un idéogramme chinois (les blocs CJK de la référence) : isolé, chacun est un mot.
fn est_ideogramme(c: char) -> bool {
    let o = c as u32;
    [
        (0x4E00, 0x9FFF),
        (0x3400, 0x4DBF),
        (0x2_0000, 0x2_A6DF),
        (0x2_A700, 0x2_B73F),
        (0x2_B740, 0x2_B81F),
        (0x2_B820, 0x2_CEAF),
        (0xF900, 0xFAFF),
        (0x2_F800, 0x2_FA1F),
    ]
    .iter()
    .any(|&(a, b)| (a..=b).contains(&o))
}

/// Un signe de ponctuation : l'ASCII qui n'est ni lettre, ni chiffre, ni blanc ; et la ponctuation
/// d'Unicode.
fn est_ponctuation(c: char) -> bool {
    c.is_ascii_punctuation() || (!c.is_ascii() && dans(PONCTUATION, c as u32))
}

/// Ajoute `c` sans ses marques : sa décomposition canonique, privée de ses marques sans chasse.
fn retirer_les_marques(c: char, dans_la_suite: &mut String) {
    let o = c as u32;
    if dans(MARQUES, o) {
        return;
    }
    if let Some(jamos) = decomposer_un_hangul(o) {
        dans_la_suite.extend(jamos.into_iter().flatten());
        return;
    }
    match SANS_ACCENT.binary_search_by_key(&o, |&(k, _)| k) {
        Ok(i) => dans_la_suite.push_str(SANS_ACCENT[i].1),
        Err(_) => dans_la_suite.push(c),
    }
}

/// **La décomposition d'une syllabe coréenne**, par la formule d'Unicode (§ 3.12) : la consonne
/// initiale, la voyelle, et la finale s'il y en a une. Rien pour un autre caractère.
fn decomposer_un_hangul(o: u32) -> Option<[Option<char>; 3]> {
    const S: u32 = 0xAC00;
    const L: u32 = 0x1100;
    const V: u32 = 0x1161;
    const T: u32 = 0x11A7;
    const NV: u32 = 21;
    const NT: u32 = 28;
    let rang = o.checked_sub(S).filter(|&r| r < 19 * NV * NT)?;
    let (l, v, t) = (
        L + rang / (NV * NT),
        V + (rang % (NV * NT)) / NT,
        T + rang % NT,
    );
    Some([
        char::from_u32(l),
        char::from_u32(v),
        (t != T).then(|| char::from_u32(t)).flatten(),
    ])
}

/// Le code est-il dans l'une de ces plages, triées ?
fn dans(plages: &[(u32, u32)], o: u32) -> bool {
    let i = plages.partition_point(|&(_, fin)| fin < o);
    plages.get(i).is_some_and(|&(debut, _)| debut <= o)
}
