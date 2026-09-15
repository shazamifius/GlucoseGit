//! FONT-1 — la preuve que les polices embarquées couvrent ce que Glucose écrit (R-51).
//!
//! Ce test est le vrai livrable du changement de police : sans lui, la prochaine personne
//! qui remplace un fichier de fonte recrée le défaut sans que rien ne casse. Il lit la table
//! `cmap` de chaque police embarquée **avec son propre lecteur**, sur `std` seulement, pour
//! ne pas prouver la couverture avec le moteur qu'on cherche à contrôler ; puis il demande
//! la même chose à `fontdue`, qui est ce qui rastérise vraiment. Les deux doivent s'accorder.
//!
//! # Ce qu'on exige dépend de ce qu'un visage dessine
//!
//! Les quatre Inter portent **l'interface** : ses libellés, ses chevrons, ses flèches, ses
//! poignées. JetBrains Mono ne porte que du **contenu** — ce que l'utilisateur écrit entre
//! deux accents graves. Lui demander le `⠿` d'une poignée de dock serait exiger qu'une police
//! de code sache dessiner un widget, et cette exigence-là finirait par être contournée.
//!
//! D'où deux ensembles, et trois affirmations :
//!
//! 1. [`TEXT_SET`] — lettres, chiffres, ponctuation, tous les accents du français, ligatures,
//!    guillemets, tirets : **tout visage** doit le couvrir, y compris la chasse fixe, parce
//!    qu'un commentaire accentué dans un bloc de code est du texte comme un autre ;
//! 2. [`UI_SET`] — puces et flèches : les **visages d'interface** seulement ;
//! 3. **tous les caractères non-ASCII des chaînes et caractères littéraux du crate**,
//!    extraits des sources au moment du test — jamais devinés — pour les visages d'interface,
//!    puisque c'est en Inter que ces littéraux se dessinent. Une chaîne ajoutée demain avec un
//!    caractère que la police ignore fait échouer ce test, pas l'écran.
//!
//! Corollaire pour les tests : un caractère que l'on veut délibérément absent de la police
//! (pour éprouver le `.notdef`, par exemple) s'écrit `'\u{1F600}'`, pas en clair.

use super::{Face, Typography, FACE_BYTES};
use std::collections::BTreeSet;
use std::path::Path;

/// Ce que **tout** visage doit couvrir : le texte lui-même.
const TEXT_SET: &[(&str, &str)] = &[
    (
        "lettres latines et chiffres",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
    ),
    (
        "ponctuation ASCII",
        r##" !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##,
    ),
    ("accents français, minuscules", "àâäéèêëîïôöùûüÿç"),
    ("accents français, majuscules", "ÀÂÄÉÈÊËÎÏÔÖÙÛÜŸÇ"),
    ("ligatures", "œŒæÆ"),
    ("guillemets et apostrophes", "«»‹›‘’“”"),
    ("tirets et points de suspension", "–—…"),
    ("espace insécable, degré, euro", "\u{A0}°€"),
];

/// Ce que les visages d'**interface** doivent couvrir en plus : les ornements de la chrome.
const UI_SET: &[(&str, &str)] = &[("puces", "•●"), ("flèches", "←→↑↓")];

/// Le nom de fichier de chaque visage, pour que l'échec nomme le fichier à remplacer.
const FACE_FILES: [&str; 5] = [
    "Inter-Regular.ttf",
    "Inter-SemiBold.ttf",
    "Inter-Italic.otf",
    "Inter-SemiBoldItalic.otf",
    "JetBrainsMonoNL-Regular.ttf",
];

/// Les visages qui dessinent l'interface — tous sauf la chasse fixe, qui ne rend que du
/// contenu.
fn interface_faces() -> impl Iterator<Item = (Face, &'static str, &'static [u8])> {
    faces().filter(|(face, _, _)| *face != Face::Mono)
}

fn faces() -> impl Iterator<Item = (Face, &'static str, &'static [u8])> {
    Face::ALL
        .into_iter()
        .enumerate()
        .map(|(i, face)| (face, FACE_FILES[i], FACE_BYTES[i]))
}

// ── Lecteur de `cmap`, sur std ──────────────────────────────────────────────

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    data.get(at..at + 2)
        .map(|b| u16::from_be_bytes([b[0], b[1]]))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    data.get(at..at + 4)
        .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

/// Une table du répertoire TrueType/OpenType, par étiquette.
fn table<'a>(font: &'a [u8], tag: &[u8; 4]) -> Option<&'a [u8]> {
    let count = u16_at(font, 4)? as usize;
    (0..count).map(|i| 12 + 16 * i).find_map(|record| {
        if font.get(record..record + 4)? != tag {
            return None;
        }
        let offset = u32_at(font, record + 8)? as usize;
        let length = u32_at(font, record + 12)? as usize;
        font.get(offset..offset + length)
    })
}

/// Indice de glyphe de `code` dans une sous-table de format 4 (plan multilingue de base).
fn glyph_in_format4(sub: &[u8], code: u32) -> Option<u16> {
    let code = u16::try_from(code).ok()?;
    let seg_x2 = u16_at(sub, 6)? as usize;
    let (ends, starts) = (14, 16 + seg_x2);
    let (deltas, range_offsets) = (16 + 2 * seg_x2, 16 + 3 * seg_x2);
    for i in 0..seg_x2 / 2 {
        if u16_at(sub, ends + 2 * i)? < code {
            continue;
        }
        let start = u16_at(sub, starts + 2 * i)?;
        if start > code {
            return Some(0);
        }
        let delta = u16_at(sub, deltas + 2 * i)?;
        let range_offset = u16_at(sub, range_offsets + 2 * i)? as usize;
        if range_offset == 0 {
            return Some(code.wrapping_add(delta));
        }
        let at = range_offsets + 2 * i + range_offset + 2 * (code - start) as usize;
        let glyph = u16_at(sub, at)?;
        return Some(if glyph == 0 {
            0
        } else {
            glyph.wrapping_add(delta)
        });
    }
    Some(0)
}

/// Indice de glyphe de `code` dans une sous-table de format 12 (groupes 32 bits).
fn glyph_in_format12(sub: &[u8], code: u32) -> Option<u32> {
    let groups = u32_at(sub, 12)? as usize;
    for g in 0..groups {
        let at = 16 + 12 * g;
        let (start, end) = (u32_at(sub, at)?, u32_at(sub, at + 4)?);
        if (start..=end).contains(&code) {
            return Some(u32_at(sub, at + 8)? + (code - start));
        }
    }
    Some(0)
}

/// Indice de glyphe de `ch`, lu dans la table `cmap` de `font` ; 0 si aucune sous-table
/// ne le connaît. Toutes les sous-tables sont consultées : la première qui répond gagne.
fn glyph_index(font: &[u8], ch: char) -> u32 {
    let Some(cmap) = table(font, b"cmap") else {
        return 0;
    };
    let count = u16_at(cmap, 2).unwrap_or(0) as usize;
    (0..count)
        .filter_map(|i| {
            let offset = u32_at(cmap, 8 + 8 * i)? as usize;
            let sub = cmap.get(offset..)?;
            match u16_at(sub, 0)? {
                4 => glyph_in_format4(sub, ch as u32).map(u32::from),
                12 => glyph_in_format12(sub, ch as u32),
                _ => None,
            }
        })
        .find(|&glyph| glyph != 0)
        .unwrap_or(0)
}

// ── Extraction des littéraux non-ASCII des sources ──────────────────────────

/// Consomme une chaîne brute `r#*"…"#*` à partir du `r` en `i` ; rend l'indice suivant.
fn skip_raw_string(c: &[char], i: usize, found: &mut BTreeSet<char>) -> Option<usize> {
    let mut j = i + 1;
    let mut hashes = 0usize;
    while c.get(j) == Some(&'#') {
        hashes += 1;
        j += 1;
    }
    if c.get(j) != Some(&'"') {
        return None;
    }
    j += 1;
    while j < c.len() {
        if c[j] == '"'
            && c[j + 1..]
                .iter()
                .take(hashes)
                .filter(|&&h| h == '#')
                .count()
                == hashes
        {
            return Some(j + 1 + hashes);
        }
        if !c[j].is_ascii() {
            found.insert(c[j]);
        }
        j += 1;
    }
    Some(j)
}

/// Longueur d'un caractère littéral commençant au `'` en `i`, ou 1 si c'est une durée de vie.
///
/// Reconnaître `'"'` et `'\''` ici est ce qui empêche le scanner de prendre la citation
/// qu'ils contiennent pour l'ouverture d'une chaîne.
fn char_literal_len(c: &[char], i: usize, found: &mut BTreeSet<char>) -> usize {
    match (c.get(i + 1), c.get(i + 2)) {
        (Some('\\'), _) => {
            let close = c
                .get(i + 3..)
                .and_then(|rest| rest.iter().position(|&x| x == '\''));
            close.map_or(c.len() - i, |p| p + 4)
        }
        (Some(&ch), Some('\'')) => {
            if !ch.is_ascii() {
                found.insert(ch);
            }
            3
        }
        _ => 1,
    }
}

/// Collecte les caractères non-ASCII des chaînes et caractères littéraux de `src`.
///
/// Les commentaires sont sautés — ils sont écrits pour le lecteur, pas pour l'écran — et
/// les échappements `\u{…}` restent invisibles par construction, puisqu'ils sont ASCII.
fn scan_source(src: &str, found: &mut BTreeSet<char>) {
    let c: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < c.len() {
        let next = c.get(i + 1).copied();
        if c[i] == '/' && next == Some('/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
        } else if c[i] == '/' && next == Some('*') {
            i += 2;
            while i < c.len() && !(c[i] == '*' && c.get(i + 1) == Some(&'/')) {
                i += 1;
            }
            i += 2;
        } else if c[i] == 'r' && matches!(next, Some('"' | '#')) {
            match skip_raw_string(&c, i, found) {
                Some(end) => i = end,
                None => i += 1,
            }
        } else if c[i] == '"' {
            i += 1;
            while i < c.len() && c[i] != '"' {
                if c[i] == '\\' {
                    i += 1;
                } else if !c[i].is_ascii() {
                    found.insert(c[i]);
                }
                i += 1;
            }
            i += 1;
        } else if c[i] == '\'' {
            i += char_literal_len(&c, i, found);
        } else {
            i += 1;
        }
    }
}

/// Parcourt récursivement `dir` et scanne chaque fichier `.rs`.
fn scan_directory(dir: &Path, found: &mut BTreeSet<char>) {
    let entries = std::fs::read_dir(dir).expect("le dossier des sources du crate existe");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_directory(&path, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let src = std::fs::read_to_string(&path).expect("source lisible en UTF-8");
            scan_source(&src, found);
        }
    }
}

// ── Les tests ───────────────────────────────────────────────────────────────

/// Les caractères de `chars` qu'une police ignore, selon le lecteur maison **et** fontdue.
fn missing_from(font: &[u8], chars: impl IntoIterator<Item = char>) -> Vec<char> {
    let loaded =
        fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).expect("police valide");
    chars
        .into_iter()
        .filter(|&ch| glyph_index(font, ch) == 0 || loaded.lookup_glyph_index(ch) == 0)
        .collect()
}

#[test]
fn test_font_1_every_face_covers_the_text_it_may_have_to_draw() {
    for (_, file, font) in faces() {
        for (family, chars) in TEXT_SET {
            let missing = missing_from(font, chars.chars());
            assert!(
                missing.is_empty(),
                "{file} : {family} — absents : {missing:?}"
            );
        }
    }
}

/// Les ornements de la chrome ne sont exigés que des visages qui la dessinent.
#[test]
fn test_font_1_interface_faces_cover_the_chrome_ornaments() {
    for (_, file, font) in interface_faces() {
        for (family, chars) in UI_SET {
            let missing = missing_from(font, chars.chars());
            assert!(
                missing.is_empty(),
                "{file} : {family} — absents : {missing:?}"
            );
        }
    }
}

#[test]
fn test_font_1_every_non_ascii_literal_of_the_crate_has_a_glyph() {
    let mut found = BTreeSet::new();
    scan_directory(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut found,
    );
    // Un scanner qui ne trouverait rien prouverait qu'il est cassé, pas que tout va bien.
    assert!(
        found.contains(&'é'),
        "le scanner n'a pas vu la carte d'accueil de app.rs"
    );
    println!(
        "[FONT-1] {} caractères non-ASCII dans les littéraux du crate : {}",
        found.len(),
        found.iter().collect::<String>()
    );
    for (_, file, font) in interface_faces() {
        let missing = missing_from(font, found.iter().copied());
        assert!(
            missing.is_empty(),
            "{file} ne couvre pas ces caractères présents dans les chaînes du crate : {missing:?}"
        );
    }
}

#[test]
fn test_font_1_the_cmap_reader_agrees_with_fontdue_and_rejects_the_absent() {
    // Un lecteur qui répondrait « présent » à tout passerait les deux tests précédents.
    let regular = FACE_BYTES[0];
    let loaded = fontdue::Font::from_bytes(regular, fontdue::FontSettings::default())
        .expect("police valide");
    for ch in ['\u{1F600}', '\u{4E2D}', '\u{FE0F}', '\u{2304}'] {
        assert_eq!(glyph_index(regular, ch), 0, "{ch:?} devrait manquer");
        assert_eq!(
            loaded.lookup_glyph_index(ch),
            0,
            "{ch:?} devrait manquer pour fontdue"
        );
    }
    for ch in ['A', 'é', '→', '\u{A0}'] {
        let ours = glyph_index(regular, ch);
        assert_eq!(
            ours,
            u32::from(loaded.lookup_glyph_index(ch)),
            "indice de {ch:?}"
        );
        assert_ne!(ours, 0);
    }
}

#[test]
fn test_font_1_accents_are_drawn_not_stripped() {
    // Avant : « é » devenait « e » par une table de repli, et « É » un `.notdef`.
    let typo = Typography::new();
    let glyph = |ch: char| typo.get_glyph(ch, 32.0, Face::Regular);
    assert_ne!(
        glyph('é').bitmap,
        glyph('e').bitmap,
        "l'accent aigu doit changer le glyphe"
    );
    assert!(
        glyph('É').metrics.height > glyph('E').metrics.height,
        "l'accent dépasse la capitale"
    );
    assert!(
        glyph('ç').metrics.height > glyph('c').metrics.height,
        "la cédille descend sous la ligne"
    );
    assert!(
        glyph('œ').metrics.advance_width > glyph('o').metrics.advance_width,
        "la ligature est large"
    );
    let (with, _) = typo.measure_text("Éditer — déjà prêt, à bientôt, cœur", 14.0, Face::Regular);
    let (without, _) =
        typo.measure_text("Editer - deja pret, a bientot, coeur", 14.0, Face::Regular);
    assert!(
        with > without,
        "le texte accentué ne se mesure plus comme sa version amputée"
    );
}

#[test]
fn test_font_1_the_scanner_reads_literals_and_skips_comments() {
    // `'"'` et `'\''` sont des caractères, pas des ouvertures de chaîne : le commentaire
    // qui les suit doit rester invisible.
    let snippet = "// commentaire à ignorer\n/* bloc é ignoré */\nlet a = \"à la « une »\";\n\
                   let r = r#\"ç\"#;\nlet c = 'ô';\nlet q = '\"';\nlet e = '\\'';\n\
                   // è après les citations\nlet l: &'a str = \"\\u{e9}\";\n";
    let mut found = BTreeSet::new();
    scan_source(snippet, &mut found);
    let expected: BTreeSet<char> = "à«»çô".chars().collect();
    assert_eq!(found, expected);
}

/// **FONT-2** — les cinq visages s'accordent, et c'est mesuré.
///
/// Deux exigences, pour deux raisons différentes :
///
/// * les quatre Inter partagent **montante, descente et interligne**, parce qu'un mot en
///   italique au milieu d'une phrase doit s'asseoir sur la même ligne de base que ses
///   voisins ; deux versions d'Inter mélangées (3.19 et 4.x n'ont pas les mêmes métriques
///   verticales) le feraient sauter d'un pixel, et rien ne le dirait ;
/// * la chasse fixe partage **hauteur d'œil et hauteur de capitale** avec elles à taille
///   égale. C'est ce qui permet à un `` `code` `` de garder le corps du texte au lieu du
///   `0.875em` que les feuilles de style du Web appliquent pour rattraper une monospace
///   système trop grande : le facteur n'a pas été réglé, il a disparu.
#[test]
fn test_every_face_shares_the_metrics_of_its_family() {
    let typo = Typography::new();
    const SIZE: f32 = 14.0;
    let ligne = |face| {
        typo.font(face)
            .horizontal_line_metrics(SIZE)
            .expect("métriques de ligne")
    };
    let reference = ligne(Face::Regular);
    for (face, file, _) in interface_faces() {
        let m = ligne(face);
        assert_eq!(
            (m.ascent, m.descent, m.new_line_size),
            (reference.ascent, reference.descent, reference.new_line_size),
            "{file} ne partage pas les métriques verticales d'Inter-Regular"
        );
    }

    let oeil = |face| typo.font(face).metrics('x', SIZE).height;
    let capitale = |face| typo.font(face).metrics('H', SIZE).height;
    assert_eq!(
        (oeil(Face::Mono), capitale(Face::Mono)),
        (oeil(Face::Regular), capitale(Face::Regular)),
        "la chasse fixe ne s'accorde plus à Inter : le code en ligne aurait besoin d'un facteur"
    );

    // Et elle est bien à chasse fixe — sans quoi un bloc de code n'alignerait rien.
    let avance = |ch| typo.get_glyph(ch, SIZE, Face::Mono).metrics.advance_width;
    for ch in ['i', 'm', 'W', '.', 'é'] {
        assert_eq!(avance(ch), avance('x'), "{ch:?} n'a pas la chasse commune");
    }
}

/// **MEASURE-1** — le chemin rapide de la mesure donne exactement ce que le tracé avance.
///
/// [`Typography::advance`] lit les tables de la police ; le tracé, lui, avance de ce que le
/// glyphe rastérisé porte dans ses métriques. Les deux viennent de la même source, mais rien
/// ne l'impose : le jour où ils divergeraient, le curseur se poserait à côté du texte et la
/// coupe des lignes mentirait — sans qu'aucun autre test ne le voie.
#[test]
fn test_measuring_agrees_with_drawing() {
    let typo = Typography::new();
    for face in Face::ALL {
        for size in [8.0, 14.0, 14.3, 32.0] {
            for ch in "aMé€ i.W~`*_".chars() {
                assert_eq!(
                    typo.advance(ch, size, face),
                    typo.get_glyph(ch, size, face).metrics.advance_width,
                    "{ch:?} à {size} en {face:?}"
                );
            }
        }
    }
    // La borne de taille s'applique des deux côtés, sans quoi une taille aberrante mesurerait
    // autrement qu'elle ne se dessine.
    for size in [f32::NAN, 0.0, 1e9] {
        assert_eq!(
            typo.advance('a', size, Face::Regular),
            typo.get_glyph('a', size, Face::Regular)
                .metrics
                .advance_width,
            "taille aberrante : {size}"
        );
    }
}

/// Les caractères qu'au moins un visage d'**interface** ne sait pas dessiner.
///
/// Exposée pour que d'autres modules puissent fonder une décision de rendu sur ce que les
/// polices couvrent réellement, plutôt que sur une supposition : c'est ainsi que les sigles
/// de prédicat ont été tracés au lieu d'être écrits (PRED-1).
pub(crate) fn missing_from_interface(chars: &str) -> Vec<char> {
    let mut manquants: BTreeSet<char> = BTreeSet::new();
    for (_, _, bytes) in interface_faces() {
        manquants.extend(missing_from(bytes, chars.chars()));
    }
    manquants.into_iter().collect()
}
