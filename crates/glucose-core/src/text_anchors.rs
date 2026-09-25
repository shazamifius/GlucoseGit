//! **Ancrer une sélection de texte par sa place, et la retrouver** — 0 dépendance (FLECHE-4).
//!
//! # Le défaut de Tauri, que son utilisateur a relevé
//!
//! Deux fois « bonjours » dans une carte : sélectionner le **premier** allumait les deux, et
//! le survol n'en faisait briller qu'un. Une chaîne ne désigne pas *quelle* occurrence. Une
//! ancre désigne donc des **caractères précis** : une plage `[start, end)` de la source de la
//! carte, et, pour la retrouver si le texte a changé autrement que sous nos yeux, sa citation
//! et trente-deux caractères de contexte de chaque côté — le modèle du W3C Web Annotation
//! (`TextPositionSelector` et `TextQuoteSelector`), celui d'Hypothesis.
//!
//! # Une seule unité : l'octet de la source
//!
//! Les positions sont des **octets** de la source UTF-8 de la carte : l'unité de sa mise en
//! page, de son éditeur et de son curseur. Le portage de Tauri en mêlait deux — la création
//! comptait des caractères, la résolution découpait des octets —, et son contexte de « 32
//! octets avant » tombait au milieu d'une lettre accentuée : un plantage sur le premier texte
//! français venu. Rien ici ne découpe une chaîne ailleurs qu'à une frontière de caractère ;
//! une position qui n'en est pas une — celles de Tauri comptent des unités UTF-16 d'un texte
//! rendu — est simplement fausse, et l'ancre se retrouve par sa citation.

use crate::types::{TextAnchor, TextSelection};

/// Le séparateur de l'ancien encodage de Tauri, plusieurs citations dans une chaîne.
pub const LEGACY_SEP: &str = " ‖ ";
/// Le contexte gardé de chaque côté d'une citation, en caractères.
pub const CONTEXT_LEN: usize = 32;

/// Une plage résolue, en octets de la source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedRange {
    pub start: usize,
    pub end: usize,
}

/// La frontière de caractère la plus proche **en dessous** de `i` (ou `i` lui-même).
fn frontiere_avant(texte: &str, i: usize) -> usize {
    let mut i = i.min(texte.len());
    while !texte.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// La frontière de caractère la plus proche **au-dessus** de `i` (ou `i` lui-même).
fn frontiere_apres(texte: &str, i: usize) -> usize {
    let mut i = i.min(texte.len());
    while !texte.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Les `n` derniers caractères avant l'octet `i`.
fn avant(texte: &str, i: usize, n: usize) -> &str {
    if n == 0 {
        return "";
    }
    let debut = texte[..i]
        .char_indices()
        .rev()
        .nth(n - 1)
        .map_or(0, |(b, _)| b);
    &texte[debut..i]
}

/// Les `n` premiers caractères après l'octet `i`.
fn apres(texte: &str, i: usize, n: usize) -> &str {
    let reste = &texte[i..];
    let fin = reste.char_indices().nth(n).map_or(reste.len(), |(b, _)| b);
    &reste[..fin]
}

/// **Une ancre pour la plage `[start, end)` de `plain`**, en octets.
///
/// La plage est ramenée aux frontières de caractère qui l'englobent, puis rognée des blancs de
/// ses bords (une sélection à la souris déborde souvent d'une espace). `None` si elle est vide.
pub fn create_anchor(plain: &str, start: usize, end: usize) -> Option<TextAnchor> {
    let (mut s, mut e) = (start.min(end), start.max(end));
    s = frontiere_avant(plain, s);
    e = frontiere_apres(plain, e);
    let citation = &plain[s..e];
    let debut_utile = citation.len() - citation.trim_start().len();
    let fin_utile = citation.trim_end().len();
    if fin_utile <= debut_utile {
        return None;
    }
    let (s, e) = (s + debut_utile, s + fin_utile);
    Some(TextAnchor {
        start: s as i64,
        end: e as i64,
        quote: plain[s..e].to_string(),
        prefix: Some(avant(plain, s, CONTEXT_LEN).to_string()),
        suffix: Some(apres(plain, e, CONTEXT_LEN).to_string()),
    })
}

/// Ajoute une ancre, sauf si elle chevauche une ancre déjà là. Deux occurrences d'un même mot
/// sont deux ancres : c'est exactement ce que le défaut de Tauri empêchait.
pub fn add_anchor(mut anchors: Vec<TextAnchor>, anchor: TextAnchor) -> Vec<TextAnchor> {
    let chevauche = anchors
        .iter()
        .any(|a| a.start < anchor.end && anchor.start < a.end);
    if chevauche {
        return anchors;
    }
    anchors.push(anchor);
    anchors.sort_by_key(|a| a.start);
    anchors
}

/// Ramène toute forme gardée à une liste d'ancres. L'ancien encodage de Tauri, « a ‖ b », donne
/// des ancres sans position (`start: -1`), retrouvées par leur citation.
pub fn normalize_text_sel(sel: Option<&TextSelection>) -> Vec<TextAnchor> {
    match sel {
        None => Vec::new(),
        Some(TextSelection::Anchors(list)) => list.clone(),
        Some(TextSelection::Legacy(s)) => s
            .split(LEGACY_SEP)
            .map(str::trim)
            .filter(|q| !q.is_empty())
            .map(|quote| TextAnchor {
                start: -1,
                end: -1,
                quote: quote.to_string(),
                prefix: None,
                suffix: None,
            })
            .collect(),
    }
}

/// La sélection gardée porte-t-elle au moins une ancre ?
pub fn has_text_selection(sel: Option<&TextSelection>) -> bool {
    !normalize_text_sel(sel).is_empty()
}

/// Les débuts, en octets, des occurrences de `citation` dans `texte` — exactes d'abord ; sans
/// aucune, sans tenir compte de la casse. La comparaison sans casse se fait caractère à
/// caractère sur le texte d'origine : passer tout en minuscules changerait des longueurs
/// (« İ » en compte deux), et les positions ne désigneraient plus le texte.
fn occurrences(texte: &str, citation: &str) -> Vec<(usize, usize)> {
    if citation.is_empty() {
        return Vec::new();
    }
    // À chaque frontière de caractère, et non par `match_indices` : deux occurrences qui se
    // chevauchent sont deux occurrences, et l'ancre peut viser la seconde.
    let exactes: Vec<(usize, usize)> = texte
        .char_indices()
        .filter(|(i, _)| texte[*i..].starts_with(citation))
        .map(|(i, _)| (i, i + citation.len()))
        .collect();
    if !exactes.is_empty() {
        return exactes;
    }
    texte
        .char_indices()
        .filter_map(|(i, _)| {
            let mut fin = i;
            let mut restant = texte[i..].chars();
            for voulu in citation.chars() {
                let c = restant.next()?;
                if !c.to_lowercase().eq(voulu.to_lowercase()) {
                    return None;
                }
                fin += c.len_utf8();
            }
            Some((i, fin))
        })
        .collect()
}

/// Combien de caractères communs à la fin de `a` et de `b`.
fn suffixe_commun(a: &str, b: &str) -> usize {
    a.chars()
        .rev()
        .zip(b.chars().rev())
        .take_while(|(x, y)| x == y)
        .count()
}

/// Combien de caractères communs au début de `a` et de `b`.
fn prefixe_commun(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

/// Retrouve l'occurrence visée : chaque candidate est notée sur la ressemblance de son
/// contexte, puis départagée par la distance à la position d'origine.
fn reancrer(plain: &str, ancre: &TextAnchor) -> Option<ResolvedRange> {
    let prefixe = ancre.prefix.as_deref().unwrap_or("");
    let suffixe = ancre.suffix.as_deref().unwrap_or("");
    let origine = usize::try_from(ancre.start).unwrap_or(0);
    occurrences(plain, &ancre.quote)
        .into_iter()
        .map(|(debut, fin)| {
            let score = suffixe_commun(avant(plain, debut, CONTEXT_LEN), prefixe)
                + prefixe_commun(apres(plain, fin, CONTEXT_LEN), suffixe);
            (debut, fin, score, debut.abs_diff(origine))
        })
        // Le meilleur contexte, puis la plus proche de la position d'origine, puis la
        // première : un ordre total, donc une réponse unique.
        .min_by(|a, b| b.2.cmp(&a.2).then(a.3.cmp(&b.3)).then(a.0.cmp(&b.0)))
        .map(|(start, end, _, _)| ResolvedRange { start, end })
}

/// **Résout des ancres contre la source d'une carte** : les plages qu'elles désignent, triées
/// et fondues. Une ancre dont la citation a disparu est abandonnée, sans surlignage fantôme.
///
/// # Une position n'est pas une identité
///
/// Tauri — et ce module, dans sa première version — prenait l'ancre à sa position dès que la
/// citation s'y trouvait. Mais un texte ajouté devant peut y amener **une autre** occurrence :
/// seize caractères insérés avant « bonjours / test / test / bonjours », et la position du
/// second tombe sur le premier. C'est le défaut qu'il a vu. Chaque occurrence est donc notée
/// sur son contexte, et la position d'origine ne fait que départager deux ex æquo : une ancre
/// intacte garde sa place, puisque son contexte y est entier et sa distance nulle.
pub fn resolve_anchors(plain: &str, anchors: &[TextAnchor]) -> Vec<ResolvedRange> {
    let mut plages: Vec<ResolvedRange> =
        anchors.iter().filter_map(|a| reancrer(plain, a)).collect();
    plages.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));
    let mut fondues: Vec<ResolvedRange> = Vec::with_capacity(plages.len());
    for r in plages {
        match fondues.last_mut() {
            Some(derniere) if r.start <= derniere.end => derniere.end = derniere.end.max(r.end),
            _ => fondues.push(r),
        }
    }
    fondues
}

/// **Fait traverser une édition à des ancres** (FLECHE-4) : `ancien` est devenu `nouveau` sous
/// les doigts de l'auteur, et chaque ancre suit ses caractères comme un curseur suit sa place.
///
/// L'édition se lit comme **un** remplacement : ce qui a changé est ce qui reste une fois ôtés
/// le plus long début et la plus longue fin communs. Une frappe, un collage, une suppression en
/// sont un. Chaque bout d'ancre se reporte alors :
///
/// * avant le remplacement, il ne bouge pas — taper juste **après** un mot ancré ne l'allonge
///   pas ;
/// * après, il glisse de la différence de longueur — taper juste **avant** ne l'allonge pas
///   non plus ;
/// * dedans, le début vient au début du remplacement, la fin à sa fin : réécrire un mot ancré
///   garde l'ancre sur ce qu'on a écrit à sa place.
///
/// Une ancre réduite à rien — son texte effacé — disparaît. Les autres reprennent leur citation
/// et leur contexte dans le texte nouveau : elles se retrouveraient même si le texte changeait
/// ensuite ailleurs que sous nos yeux.
pub fn suivre(ancres: &[TextAnchor], ancien: &str, nouveau: &str) -> Vec<TextAnchor> {
    let debut = ancien
        .char_indices()
        .zip(nouveau.chars())
        .find(|((_, a), b)| a != b)
        .map_or(ancien.len().min(nouveau.len()), |((i, _), _)| i);
    let fin = ancien[debut..]
        .chars()
        .rev()
        .zip(nouveau[debut..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum::<usize>();
    let (remplace, remplacant) = (ancien.len() - fin, nouveau.len() - fin);
    let reporter = |x: usize, est_la_fin: bool| {
        if x <= debut {
            x
        } else if x >= remplace {
            x - remplace + remplacant
        } else if est_la_fin {
            remplacant
        } else {
            debut
        }
    };
    ancres
        .iter()
        .filter_map(|a| {
            let r = reancrer(ancien, a)?;
            create_anchor(nouveau, reporter(r.start, false), reporter(r.end, true))
        })
        .collect()
}

/// Résout directement depuis la forme gardée.
pub fn resolve_text_sel(plain: &str, sel: Option<&TextSelection>) -> Vec<ResolvedRange> {
    resolve_anchors(plain, &normalize_text_sel(sel))
}

#[cfg(test)]
mod tests;
