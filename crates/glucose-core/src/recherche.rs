//! **Chercher dans un document** (`Ctrl+F`, fiche 89) : ce que l'humain cherche, ce qu'un agent
//! cherche, la même loi.
//!
//! # PLI-1 — on cherche sans majuscules ni accents
//!
//! « ete » trouve « Été », « oeuvre » trouve « Œuvre » : [`pli`] ramène chaque lettre latine à sa
//! lettre de base, et retire les accents écrits à part (U+0300 à U+036F). Le noyau n'a aucune
//! dépendance, donc aucune table Unicode : la table est écrite ici, pour le latin (le Latin-1 et le
//! Latin étendu A). Une écriture qui n'est pas latine se cherche telle quelle, en minuscules.
//!
//! **Chercher n'est pas nommer** : un nom de domaine garde ses accents (« pêche » n'est pas
//! « péché », DOM-4), parce qu'il désigne ; une recherche les ignore, parce qu'elle trouve.
//!
//! # TROUVE-1 — chaque mot, n'importe où ; le début d'un mot d'abord
//!
//! Un nœud répond si **chacun** des mots cherchés est dans son texte. Il passe devant les autres
//! quand ses mots commencent par ceux qu'on a écrits (« eau » trouve « l'eau » avant « bureau ») ; à
//! égalité, l'ordre du document. Ce qu'on montre de lui est **la phrase** où le premier mot paraît —
//! l'unité du texte lui-même, comme pour un agent (fiche 79).
//!
//! # Le coût
//!
//! Le document se plie une fois par version ([`Index`]) ; chercher ne fait plus que comparer, et la
//! phrase d'un résultat ne se calcule que pour les rangées qu'on montre ([`passage_du`]). Mesuré
//! (`examples/bench_recherche`) : voir la fiche 89. La cible des 10⁷ demandera un index de trigrammes
//! (la méthode de Google Code Search), sans changer ce que [`Index::chercher`] rend.

use crate::store::agents::Sorte;
use crate::types::{Annotation, BoardImage, Project};

/// **La forme d'un texte qu'on compare** pour chercher (PLI-1).
pub fn pli(texte: &str) -> String {
    let mut out = String::with_capacity(texte.len());
    for c in texte.chars() {
        // La lettre ASCII, presque toutes : sa minuscule est immédiate (mesuré : fiche 89).
        if c.is_ascii() {
            out.push(c.to_ascii_lowercase());
            continue;
        }
        if ('\u{300}'..='\u{36f}').contains(&c) {
            continue;
        }
        match base_latine(c) {
            Some(base) => out.push_str(base),
            None => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// La lettre de base d'une lettre latine accentuée, majuscule ou minuscule.
fn base_latine(c: char) -> Option<&'static str> {
    let b = match c as u32 {
        0xC0..=0xC5 | 0xE0..=0xE5 | 0x100..=0x105 => "a",
        0xC6 | 0xE6 => "ae",
        0xC7 | 0xE7 | 0x106..=0x10D => "c",
        0xD0 | 0xF0 | 0x10E..=0x111 => "d",
        0xC8..=0xCB | 0xE8..=0xEB | 0x112..=0x11B => "e",
        0x11C..=0x123 => "g",
        0x124..=0x127 => "h",
        0xCC..=0xCF | 0xEC..=0xEF | 0x128..=0x131 => "i",
        0x132..=0x133 => "ij",
        0x134..=0x135 => "j",
        0x136..=0x138 => "k",
        0x139..=0x142 => "l",
        0xD1 | 0xF1 | 0x143..=0x14B => "n",
        0xD2..=0xD6 | 0xD8 | 0xF2..=0xF6 | 0xF8 | 0x14C..=0x151 => "o",
        0x152..=0x153 => "oe",
        0x154..=0x159 => "r",
        0xDF => "ss",
        0x15A..=0x161 | 0x17F => "s",
        0x162..=0x167 => "t",
        0xDE | 0xFE => "th",
        0xD9..=0xDC | 0xF9..=0xFC | 0x168..=0x173 => "u",
        0x174..=0x175 => "w",
        0xDD | 0xFD | 0xFF | 0x176..=0x178 => "y",
        0x179..=0x17E => "z",
        _ => return None,
    };
    Some(b)
}

/// Les phrases d'un texte : coupées après `.`, `!` ou `?` suivis d'un blanc, et à chaque ligne.
pub fn phrases(texte: &str) -> Vec<&str> {
    let mut v = Vec::new();
    let mut debut = 0;
    let mut precedent: Option<char> = None;
    for (i, c) in texte.char_indices() {
        let fin_de_phrase = c.is_whitespace() && matches!(precedent, Some('.' | '!' | '?'));
        if c == '\n' || fin_de_phrase {
            v.push(&texte[debut..i]);
            debut = i + c.len_utf8();
        }
        precedent = Some(c);
    }
    v.push(&texte[debut..]);
    v.into_iter()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect()
}

/// La première phrase d'un texte.
pub fn premiere_phrase(texte: &str) -> Option<&str> {
    phrases(texte).into_iter().next()
}

/// La première phrase qui contient `cherche`, sans majuscules ni accents (PLI-1).
pub fn passage(texte: &str, cherche: &str) -> Option<String> {
    let c = pli(cherche.trim());
    if c.is_empty() {
        return None;
    }
    phrases(texte)
        .into_iter()
        .find(|p| pli(p).contains(&c))
        .map(str::to_string)
}

/// **Ce qu'on a trouvé** : un tableau, un dossier ou un nœud.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    Tableau,
    Dossier,
    Noeud(Sorte),
}

impl Genre {
    pub fn nom(self) -> &'static str {
        match self {
            Self::Tableau => "tableau",
            Self::Dossier => "dossier",
            Self::Noeud(s) => s.nom(),
        }
    }
}

/// **Un résultat** : où il est, et ce qu'il est. Sa phrase se demande à part ([`passage_du`]) :
/// on ne la calcule que pour les rangées qu'on montre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trouve {
    pub tableau: String,
    /// Le nœud ou le dossier ; rien pour un tableau trouvé par son nom.
    pub id: Option<String>,
    pub genre: Genre,
    /// Combien des mots cherchés commencent un mot du texte : plus il y en a, mieux il répond.
    pub debuts: usize,
}

/// Les mots d'une recherche, pliés.
fn mots_de(requete: &str) -> Vec<String> {
    pli(requete)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// **Combien des mots commencent un mot de ce texte** — ou rien, si l'un d'eux n'y est pas
/// (TROUVE-1).
fn repond(texte_plie: &str, mots: &[String]) -> Option<usize> {
    let mut debuts = 0;
    for m in mots {
        let mut present = false;
        let mut au_debut = false;
        for (i, _) in texte_plie.match_indices(m.as_str()) {
            present = true;
            if texte_plie[..i]
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_alphanumeric())
            {
                au_debut = true;
                break;
            }
        }
        if !present {
            return None;
        }
        debuts += usize::from(au_debut);
    }
    Some(debuts)
}

/// La sorte d'une annotation, et le texte qu'elle porte — emprunté, jamais recopié.
fn texte_d_une_annotation(a: &Annotation) -> (Sorte, &str) {
    match a {
        Annotation::Text { text, .. } => (Sorte::Texte, text),
        Annotation::Sticky { text, .. } => (Sorte::PostIt, text),
        Annotation::Membrane { text, .. } => (Sorte::Membrane, text.as_deref().unwrap_or("")),
        Annotation::Arrow { text, .. } => (Sorte::Fleche, text.as_deref().unwrap_or("")),
    }
}

/// Ce qu'une photo donne à chercher : ses étiquettes, puis l'adresse d'où elle vient.
fn texte_d_une_photo(i: &BoardImage) -> String {
    let mut t = i.tags.join(" ");
    if let Some(a) = &i.source_url {
        t.push('\n');
        t.push_str(a);
    }
    t
}

/// Une entrée de l'index : où, quoi, et son texte plié.
#[derive(Debug, Clone)]
struct Entree {
    tableau: usize,
    id: Option<String>,
    genre: Genre,
    plie: String,
}

/// **L'index d'un document** : chaque texte qu'on peut chercher, plié une fois (PLI-1). Il se refait
/// quand le document change ; chercher ne fait plus que comparer — ce que l'on écrit lettre après
/// lettre ne replie pas tout le document à chaque lettre.
#[derive(Debug, Clone, Default)]
pub struct Index {
    tableaux: Vec<String>,
    entrees: Vec<Entree>,
}

impl Index {
    /// **Plie le document** : les noms des tableaux et des dossiers, le texte des nœuds (textes,
    /// post-it, titres de membranes, ce que disent les flèches), les étiquettes et l'adresse des
    /// photos.
    pub fn du(project: &Project) -> Self {
        let mut index = Index::default();
        for (rang, tableau) in project.boards.iter().enumerate() {
            index.tableaux.push(tableau.id.clone());
            let mut garder = |id: Option<&str>, genre, texte: &str| {
                if !texte.trim().is_empty() {
                    index.entrees.push(Entree {
                        tableau: rang,
                        id: id.map(str::to_string),
                        genre,
                        plie: pli(texte),
                    });
                }
            };
            garder(None, Genre::Tableau, &tableau.name);
            for i in &tableau.images {
                garder(
                    Some(&i.id),
                    Genre::Noeud(Sorte::Photo),
                    &texte_d_une_photo(i),
                );
            }
            for a in &tableau.annotations {
                let (sorte, texte) = texte_d_une_annotation(a);
                garder(Some(a.id()), Genre::Noeud(sorte), texte);
            }
            for d in &tableau.folders {
                garder(Some(&d.id), Genre::Dossier, &d.name);
            }
        }
        index
    }

    /// **Ce que cette recherche trouve**, les meilleurs d'abord (TROUVE-1). Une recherche vide ne
    /// trouve rien.
    pub fn chercher(&self, requete: &str) -> Vec<Trouve> {
        let mots = mots_de(requete);
        if mots.is_empty() {
            return Vec::new();
        }
        let mut trouves: Vec<Trouve> = self
            .entrees
            .iter()
            .filter_map(|e| {
                let debuts = repond(&e.plie, &mots)?;
                Some(Trouve {
                    tableau: self.tableaux[e.tableau].clone(),
                    id: e.id.clone(),
                    genre: e.genre,
                    debuts,
                })
            })
            .collect();
        // Un tri stable : à égalité, l'ordre du document.
        trouves.sort_by_key(|t| std::cmp::Reverse(t.debuts));
        trouves
    }
}

/// **Chercher dans tout le document**, une fois : l'index se fait et se jette. Ce qu'on cherche
/// lettre après lettre garde le sien ([`Index`]).
pub fn chercher(project: &Project, requete: &str) -> Vec<Trouve> {
    Index::du(project).chercher(requete)
}

/// **La phrase qui montre ce résultat** : celle où paraît le premier mot cherché, sinon la première.
pub fn passage_du(project: &Project, t: &Trouve, requete: &str) -> String {
    let Some(tableau) = project.boards.iter().find(|b| b.id == t.tableau) else {
        return String::new();
    };
    let texte = match (&t.id, t.genre) {
        (None, _) | (Some(_), Genre::Tableau) => tableau.name.clone(),
        (Some(id), Genre::Dossier) => tableau
            .folders
            .iter()
            .find(|d| &d.id == id)
            .map(|d| d.name.clone())
            .unwrap_or_default(),
        (Some(id), Genre::Noeud(Sorte::Photo)) => tableau
            .images
            .iter()
            .find(|i| &i.id == id)
            .map(texte_d_une_photo)
            .unwrap_or_default(),
        (Some(id), Genre::Noeud(_)) => tableau
            .annotations
            .iter()
            .find(|a| a.id() == id)
            .map(|a| texte_d_une_annotation(a).1.to_string())
            .unwrap_or_default(),
    };
    mots_de(requete)
        .first()
        .and_then(|m| passage(&texte, m))
        .or_else(|| premiere_phrase(&texte).map(str::to_string))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
