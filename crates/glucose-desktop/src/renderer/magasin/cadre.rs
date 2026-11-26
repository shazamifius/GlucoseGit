//! **ABSENT-1 — le cadre d'un fichier introuvable** (fiche 176 ; la fiche 08 § 8 de Tauri : « liens
//! rompus : bordure pointillée rouge »). Un nœud lié (`AssetRef::Link` : une vidéo gardée à côté,
//! une photo d'un dossier du disque, une image d'un document de Tauri) montre un fichier qui vit
//! hors du document. Déplacé, renommé ou effacé, ce nœud n'était qu'un cadre gris « Image
//! [img-…] » : il dit désormais quel fichier manque, en rouge pointillé, et « Retrouver le
//! fichier… » le relie au chemin neuf.
//!
//! Le constat se fait **une fois, à l'échec** — le fichier existe-t-il sur cette machine ? —, pas à
//! chaque image : le cadre se dessine à cent images par seconde. Un fichier présent mais illisible
//! garde le cadre gris : il n'est pas introuvable.

use super::Magasin;
use glucose_core::types::{AssetRef, BoardImage};

/// **Ce que dit le cadre d'une image qui n'est pas là**, et s'il la dit introuvable — alors son
/// trait est rouge et pointillé, dans les deux voies.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cadre {
    pub texte: String,
    pub introuvable: bool,
}

impl Magasin {
    /// **Une image n'a rien donné** : un échec, qui ne se redemande pas ; et, si son fichier
    /// n'existe pas ici, un introuvable.
    pub(super) fn echoue(&mut self, src: String) {
        if !std::path::Path::new(&src).exists() {
            self.introuvables.insert(src.clone());
        }
        self.echecs.insert(src);
    }

    /// **Le cadre de ce nœud**, dont l'image n'est pas là : son fichier introuvable s'il est lié,
    /// sinon ce que dit l'attente (Q18) ou son identifiant.
    pub fn cadre(&self, img: &BoardImage) -> Cadre {
        let src = img.src.as_deref();
        let lie = matches!(img.asset, Some(AssetRef::Link { .. }));
        match src.filter(|s| lie && self.echecs.contains(*s) && self.introuvables.contains(*s)) {
            Some(chemin) => Cadre {
                texte: format!("Introuvable : {}", nom_du_fichier(chemin)),
                introuvable: true,
            },
            None => Cadre {
                texte: self.libelle(src, &img.id),
                introuvable: false,
            },
        }
    }
}

/// Le nom d'un fichier, quel que soit le système qui a écrit son chemin : un document partagé
/// porte les chemins de Windows jusqu'au Mac, et l'inverse.
fn nom_du_fichier(chemin: &str) -> &str {
    chemin.rsplit(['/', '\\']).next().unwrap_or(chemin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn liee(chemin: &str) -> BoardImage {
        let mut img = BoardImage::new("img-7", 0.0, 0.0, 100.0, 100.0);
        img.src = Some(chemin.into());
        img.asset = Some(AssetRef::Link {
            href: chemin.into(),
            sha256: None,
            size_bytes: None,
        });
        img
    }

    #[test]
    fn absent_1_le_cadre_dit_le_fichier_introuvable() {
        let dossier = std::env::temp_dir().join(format!("absent-1-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).expect("le dossier");
        let present = dossier.join("abime.mp4");
        std::fs::write(&present, b"pas une video").expect("le fichier");
        let parti = dossier.join("vacances à Nice.mp4");
        let (present, parti) = (present.to_string_lossy(), parti.to_string_lossy());

        let mut m = Magasin::nouveau();
        m.echoue(parti.to_string());
        m.echoue(present.to_string());
        let cadre = m.cadre(&liee(&parti));
        assert!(cadre.introuvable, "son fichier n'existe pas");
        assert_eq!(cadre.texte, "Introuvable : vacances à Nice.mp4");
        let illisible = m.cadre(&liee(&present));
        assert!(
            !illisible.introuvable,
            "présent mais illisible : pas introuvable"
        );
        assert_eq!(illisible.texte, "Image [img-7]");

        let mut scellee = liee(&parti);
        scellee.asset = None;
        assert!(
            !m.cadre(&scellee).introuvable,
            "un nœud scellé n'a pas de fichier à retrouver"
        );

        m.oublier(&parti);
        assert!(
            !m.cadre(&liee(&parti)).introuvable,
            "oublié, il se redemande"
        );
        std::fs::remove_dir_all(&dossier).ok();
    }

    #[test]
    fn absent_1_le_nom_se_lit_sous_tous_les_systemes() {
        assert_eq!(nom_du_fichier(r"C:\Users\Lou\clip.mov"), "clip.mov");
        assert_eq!(nom_du_fichier("/home/lou/clip.mov"), "clip.mov");
        assert_eq!(nom_du_fichier("clip.mov"), "clip.mov");
    }
}
