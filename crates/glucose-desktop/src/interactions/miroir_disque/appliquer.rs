//! **Du plan aux nœuds** (DISQUE-2) : le noyau a dit ce qui entre, ce qui suit son fichier et ce qui
//! part ; ici, chaque nœud neuf naît comme au dépôt, et chaque nœud suivi prend sa nouvelle version
//! — son chemin, sa taille, son texte relu. La hauteur d'un texte se mesure à la typographie : c'est
//! pourquoi cela se fait ici, et non dans le noyau.

use super::{source_en_attente, Entree, Sorte};
use crate::app::GlucoseApp;
use glucose_core::miroir_disque::reconcilier::{Noeud, Plan, Suivi};
use glucose_core::store::miroir_disque::{Niveau, Reconciliation};
use glucose_core::types::{Annotation, AssetRef, BoardImage, CanvasFolder, FolderSortMode};
use std::collections::HashMap;
use std::path::Path;

impl GlucoseApp {
    /// **La réconciliation de ce plan**, sur ce tableau : `textes`, le Markdown de chaque texte relu.
    pub(super) fn reconciliation(
        &mut self,
        tableau: &str,
        plan: &Plan,
        textes: &HashMap<String, String>,
        tri: Option<FolderSortMode>,
    ) -> Reconciliation {
        let mut r = Reconciliation {
            partis: plan.partis.clone(),
            ..Reconciliation::default()
        };
        for e in &plan.nouveaux {
            self.naitre_du_disque(e, textes, tri, &mut r.neufs);
        }
        for s in &plan.suivis {
            self.suivre_son_fichier(tableau, s, textes, &mut r.suivis);
        }
        r
    }

    /// **Un fichier qui entre** : son nœud naît comme au dépôt — un sous-dossier à lire quand on y
    /// entrera, une photo liée, un texte lu, et le reste en tuile qui le révèle.
    fn naitre_du_disque(
        &mut self,
        e: &Entree,
        textes: &HashMap<String, String>,
        tri: Option<FolderSortMode>,
        niveau: &mut Niveau,
    ) {
        let chemin = Path::new(&e.chemin);
        match e.sorte {
            Sorte::Dossier => {
                let mut f = CanvasFolder::new(self.store.generate_id("folder"), &*e.nom, "");
                f.mirror_source = Some(source_en_attente(&e.chemin, tri));
                niveau.dossiers.push(f);
            }
            Sorte::Photo { largeur, hauteur } => {
                let origine = (f64::from(largeur), f64::from(hauteur));
                let (w, h) = crate::interactions::clipboard::taille_posee(origine);
                let mut img = BoardImage::new(self.store.generate_id("img"), 0.0, 0.0, w, h);
                (img.original_width, img.original_height) = origine;
                lier(&mut img, e);
                niveau.photos.push(img);
            }
            Sorte::Lisible if textes.contains_key(&e.chemin) => {
                let markdown = textes[&e.chemin].clone();
                let carte = self.carte_de_markdown(chemin, markdown, (0.0, 0.0));
                niveau.textes.push(carte);
            }
            Sorte::Lisible | Sorte::Autre => {
                niveau.tuiles.push(self.tuile(chemin, &e.nom, (0.0, 0.0)));
            }
        }
    }

    /// **Un nœud qui suit son fichier** : sa nouvelle version, entière.
    fn suivre_son_fichier(
        &self,
        tableau: &str,
        s: &Suivi,
        textes: &HashMap<String, String>,
        niveau: &mut Niveau,
    ) {
        let Some(b) = self.store.board(tableau) else {
            return;
        };
        let e = &s.entree;
        match &s.noeud {
            Noeud::Photo(id) => {
                if let Some(img) = b.image(id) {
                    niveau.photos.push(photo_suivie(img, e));
                }
            }
            Noeud::Note(id) => {
                if let Some(a) = b.annotation(id) {
                    let relu = s.relire.then(|| textes.get(&e.chemin)).flatten();
                    let a = self.note_suivie(a, e, relu);
                    match a {
                        Annotation::Text { .. } => niveau.textes.push(a),
                        _ => niveau.tuiles.push(a),
                    }
                }
            }
            Noeud::Dossier(id) => {
                if let Some(f) = b.dossier(id) {
                    niveau.dossiers.push(dossier_suivi(f, e));
                }
            }
        }
    }

    /// Un texte ou une tuile qui suit son fichier : son chemin ; le nom d'une tuile qu'il n'a pas
    /// renommée ; le texte relu, à sa hauteur, s'il ne l'a pas récrit (le noyau l'a dit).
    fn note_suivie(&self, a: &Annotation, e: &Entree, relu: Option<&String>) -> Annotation {
        let largeur = a.rect().map_or(0.0, |r| r.width);
        let mut a = a.clone();
        match &mut a {
            Annotation::Text {
                text,
                height,
                source_file,
                ..
            } => {
                if let Some(markdown) = relu {
                    *height = Some(crate::renderer::card::text_card_fit_height(
                        &self.renderer.typography,
                        &self.renderer.math,
                        markdown,
                        largeur,
                    ));
                    text.clone_from(markdown);
                }
                *source_file = Some(e.chemin.clone());
            }
            Annotation::Sticky {
                text, source_file, ..
            } => {
                let ancien = source_file.as_deref().and_then(nom_du_fichier);
                if ancien == Some(text.as_str()) {
                    text.clone_from(&e.nom);
                }
                *source_file = Some(e.chemin.clone());
            }
            _ => {}
        }
        a
    }

    /// **Les photos dont le fichier a changé sous le même chemin** : ce que le magasin en sait
    /// s'oubliera, une fois le geste fait. Une photo renommée n'en a pas besoin : son chemin est une
    /// autre clé.
    pub(super) fn photos_qui_changent(&self, tableau: &str, plan: &Plan) -> Vec<String> {
        let meme_chemin = |s: &Suivi| match &s.noeud {
            Noeud::Photo(id) => {
                let src = self.store.image(tableau, id)?.src.clone()?;
                (src == s.entree.chemin).then_some(src)
            }
            _ => None,
        };
        plan.suivis.iter().filter_map(meme_chemin).collect()
    }

    pub(super) fn oublier_les_pixels(&mut self, srcs: &[String]) {
        for src in srcs {
            self.renderer.magasin.oublier(src);
        }
    }
}

/// Une photo liée à ce fichier : sa clé est son chemin, comme une photo déposée ; son `asset` dit
/// qu'elle reste où elle est, et sa taille.
fn lier(img: &mut BoardImage, e: &Entree) {
    img.src = Some(e.chemin.clone());
    img.asset = Some(AssetRef::Link {
        href: e.chemin.clone(),
        sha256: None,
        size_bytes: Some(e.taille),
    });
}

/// **Une photo qui suit son fichier** : son chemin et sa taille ; d'autres dimensions (un rendu refait
/// à une autre résolution), sa largeur reste et sa hauteur suit la forme nouvelle.
fn photo_suivie(img: &BoardImage, e: &Entree) -> BoardImage {
    let mut img = img.clone();
    lier(&mut img, e);
    if let Sorte::Photo { largeur, hauteur } = e.sorte {
        let origine = (f64::from(largeur), f64::from(hauteur));
        if origine != (img.original_width, img.original_height) && largeur > 0 {
            img.height = img.width * origine.1 / origine.0;
            (img.original_width, img.original_height) = origine;
        }
    }
    img
}

/// Un sous-dossier renommé : son chemin, et son nom s'il ne l'a pas changé.
fn dossier_suivi(f: &CanvasFolder, e: &Entree) -> CanvasFolder {
    let mut f = f.clone();
    if let Some(s) = f.mirror_source.as_mut() {
        if nom_du_fichier(&s.root_path) == Some(f.name.as_str()) {
            f.name.clone_from(&e.nom);
        }
        s.root_path.clone_from(&e.chemin);
    }
    f
}

fn nom_du_fichier(chemin: &str) -> Option<&str> {
    Path::new(chemin).file_name()?.to_str()
}
