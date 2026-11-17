//! **Son nom et sa couleur dans un canevas partagé** — comme Glucose Tauri (COLLAB-1,
//! `src/multiplayer/localUser.ts`) : tirés au sort la première fois (« Renard Bleu »), **gardés**
//! d'une session à l'autre, et le nom **se change**. La couleur est l'une des dix de Tauri, choisies
//! pour rester lisibles en pastille de quelques pixels sur fond sombre.

use std::path::Path;

/// Le fichier, dans le dossier de Glucose.
pub const FICHIER: &str = "identite.txt";

/// La palette de Tauri : ciel, émeraude, orange, violet, rose, ambre, turquoise, indigo, rouge,
/// citron.
pub const COULEURS: [[u8; 3]; 10] = [
    [0x38, 0xbd, 0xf8],
    [0x34, 0xd3, 0x99],
    [0xfb, 0x92, 0x3c],
    [0xc0, 0x84, 0xfc],
    [0xf4, 0x72, 0xb6],
    [0xfb, 0xbf, 0x24],
    [0x2d, 0xd4, 0xbf],
    [0x81, 0x8c, 0xf8],
    [0xf8, 0x71, 0x71],
    [0xa3, 0xe6, 0x35],
];

const ADJECTIFS: [&str; 10] = [
    "Bleu",
    "Rouge",
    "Vert",
    "Jaune",
    "Orange",
    "Violet",
    "Rose",
    "Cyan",
    "Indigo",
    "Émeraude",
];
const ANIMAUX: [&str; 10] = [
    "Renard",
    "Ours",
    "Aigle",
    "Chat",
    "Chien",
    "Lapin",
    "Loup",
    "Cerf",
    "Hibou",
    "Écureuil",
];

/// Au-delà, l'étiquette d'un curseur devient une banderole qui cache le canevas des autres (Tauri).
pub const NOM_MAX: usize = 24;

/// **Qui l'on est** pour les autres participants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identite {
    pub nom: String,
    /// Le rang de sa couleur dans [`COULEURS`].
    pub couleur: usize,
}

impl Identite {
    /// **Tirée au sort** de trois octets du hasard.
    pub fn tiree(hasard: [u8; 3]) -> Self {
        let [a, b, c] = hasard.map(usize::from);
        Self {
            nom: format!("{} {}", ANIMAUX[a % 10], ADJECTIFS[b % 10]),
            couleur: c % COULEURS.len(),
        }
    }

    /// **Lue** dans son fichier : le nom, puis le rang de la couleur ; `None` s'il n'y en a pas.
    pub fn lire(texte: &str) -> Option<Self> {
        let mut lignes = texte.lines();
        let nom = nom_propre(lignes.next()?)?;
        let couleur = lignes.next()?.trim().parse::<usize>().ok()?;
        (couleur < COULEURS.len()).then_some(Self { nom, couleur })
    }

    pub fn ecrire(&self) -> String {
        format!("{}\n{}\n", self.nom, self.couleur)
    }

    /// **La sienne** : lue, ou tirée et gardée — un nom tiré qui changerait à chaque lancement,
    /// c'est ce que Tauri avait corrigé.
    pub fn la_sienne(chemin: &Path) -> Self {
        if let Some(i) = std::fs::read_to_string(chemin)
            .ok()
            .and_then(|t| Self::lire(&t))
        {
            return i;
        }
        let tiree = Self::tiree(crate::plateforme::hasard::octets().unwrap_or([0, 0, 0]));
        tiree.garder(chemin);
        tiree
    }

    /// L'écrit dans son fichier.
    pub fn garder(&self, chemin: &Path) {
        let dossier = chemin.parent().unwrap_or(Path::new("."));
        let ecrit = std::fs::create_dir_all(dossier).and_then(|()| {
            crate::persist::atomic::ecrire_d_un_bloc(chemin, self.ecrire().as_bytes())
        });
        if let Err(e) = ecrit {
            crate::dire_erreur!("[Glucose] identité : {e}");
        }
    }
}

/// **Un nom utilisable** : une seule ligne, sans blancs en trop, borné ; rien s'il est vide.
pub fn nom_propre(brut: &str) -> Option<String> {
    let plat = brut.split_whitespace().collect::<Vec<_>>().join(" ");
    let nom: String = plat.chars().take(NOM_MAX).collect();
    (!nom.is_empty()).then_some(nom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identite_tiree_lue_et_gardee_a_l_identique() {
        let i = Identite::tiree([0, 1, 13]);
        assert_eq!(i.nom, "Renard Rouge");
        assert_eq!(i.couleur, 3);
        assert_eq!(Identite::lire(&i.ecrire()), Some(i));
        assert_eq!(
            Identite::lire("Moi\n10\n"),
            None,
            "une couleur hors palette"
        );
        assert_eq!(Identite::lire("\n2\n"), None, "un nom vide");
    }

    #[test]
    fn un_nom_tient_sur_une_ligne_et_se_borne() {
        assert_eq!(nom_propre("  Ana \n  Lys\t").as_deref(), Some("Ana Lys"));
        assert_eq!(nom_propre(" \n "), None);
        assert_eq!(nom_propre(&"x".repeat(40)).map(|n| n.len()), Some(NOM_MAX));
    }
}
