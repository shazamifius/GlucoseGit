//! **RECENTS-1 — les documents, à la Blender** (fiche 174 ; sa note du 25/11, `text-f5fn-348`).
//!
//! `Ctrl+O` ouvre la palette sur les documents ([`crate::commandes::palette::But::Ouvrir`]) : une
//! recherche, et dessous **tous** ses documents, dans cet ordre —
//!
//! 1. **les récents**, ceux que Glucose a ouverts ou enregistrés (la liste des connus du
//!    recensement, fiche 88), du plus récent au plus ancien ;
//! 2. parmi eux, **les introuvables en rouge** — supprimés, ou partis sur un autre disque ;
//! 3. puis **ceux qu'il n'a jamais ouverts**, trouvés sur ce PC ([`fouille`]), dans les dossiers
//!    de ses documents, et parmi ceux qu'il range lui-même — les plus récemment touchés d'abord.
//!
//! Un récent **déplacé ou renommé** se retrouve : son numéro ([`crate::plateforme::numero`]),
//! noté quand Glucose l'a connu, désigne le même fichier à sa nouvelle place, et la liste se
//! corrige d'elle-même. En bas, « Parcourir ce PC… » : l'ouverture d'avant, au cas où.

use crate::recensement::Connu;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub mod fouille;

/// **Ce qu'est un document de la liste.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sorte {
    /// Ouvert ou enregistré par Glucose.
    Recent,
    /// Connu, mais plus à sa place, et retrouvé nulle part : il se dit en rouge.
    Introuvable,
    /// Trouvé sur ce PC, jamais ouvert par Glucose.
    JamaisOuvert,
}

/// **Un document de la liste** : où il est, ce qu'il est, quand il a été touché.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub chemin: PathBuf,
    pub sorte: Sorte,
    pub touche: Option<SystemTime>,
}

/// Un document trouvé hors de la liste des connus : son chemin, et son numéro.
pub type Trouve = (PathBuf, Option<String>);

/// **Compose la liste** : les connus dans leur ordre — chacun à sa place, retrouvé ailleurs par
/// son numéro, ou introuvable —, puis les trouvés jamais ouverts, les plus récents d'abord.
///
/// Rend aussi les connus à récrire quand la liste a appris quelque chose : un numéro qui manquait
/// (une ligne d'avant RECENTS-1), un document retrouvé à sa nouvelle place.
pub fn composer(connus: Vec<Connu>, trouves: &[Trouve]) -> (Vec<Document>, Option<Vec<Connu>>) {
    let deja: HashSet<PathBuf> = connus.iter().map(|c| c.chemin.clone()).collect();
    let mut pris: HashSet<PathBuf> = HashSet::new();
    let mut appris = false;
    let mut documents = Vec::new();
    let mut corriges = Vec::with_capacity(connus.len());
    for mut connu in connus {
        let ailleurs = || {
            let numero = connu.numero.as_ref()?;
            trouves
                .iter()
                .find(|(p, n)| n.as_ref() == Some(numero) && !deja.contains(p) && !pris.contains(p))
                .map(|(p, _)| p.clone())
        };
        let (sorte, touche) = match touche(&connu.chemin) {
            Some(quand) => {
                // Le numéro d'aujourd'hui : une ligne d'avant n'en avait pas, et un fichier
                // remplacé par un autre logiciel (écrit à côté, puis renommé) en change.
                let actuel = crate::plateforme::numero::de(&connu.chemin);
                if actuel.is_some() && actuel != connu.numero {
                    connu.numero = actuel;
                    appris = true;
                }
                (Sorte::Recent, Some(quand))
            }
            None => match ailleurs() {
                Some(nouveau) => {
                    crate::dire!(
                        "[documents] retrouvé : {} → {}",
                        connu.chemin.display(),
                        nouveau.display()
                    );
                    pris.insert(nouveau.clone());
                    connu.chemin = nouveau;
                    appris = true;
                    (Sorte::Recent, touche(&connu.chemin))
                }
                None => (Sorte::Introuvable, None),
            },
        };
        documents.push(Document {
            chemin: connu.chemin.clone(),
            sorte,
            touche,
        });
        corriges.push(connu);
    }
    let mut vus = HashSet::new();
    let mut jamais: Vec<Document> = trouves
        .iter()
        .filter(|(p, _)| !deja.contains(p) && !pris.contains(p) && vus.insert(p.clone()))
        .filter_map(|(p, _)| {
            Some(Document {
                chemin: p.clone(),
                sorte: Sorte::JamaisOuvert,
                touche: Some(touche(p)?),
            })
        })
        .collect();
    jamais.sort_by_key(|d| Reverse(d.touche));
    documents.extend(jamais);
    (documents, appris.then_some(corriges))
}

impl crate::app::GlucoseApp {
    /// **Ce que le disque a fait depuis le dernier tour** : les copies reçues scellées se retirent
    /// (RECUE-1) ; la fouille a trouvé du neuf — ou fini —, et la palette des documents se
    /// redessine pour le montrer : c'est en se dessinant qu'elle se recompose.
    pub(crate) fn suivre_le_disque(&mut self) {
        use crate::commandes::palette::But;
        use crate::ui::question::Suite;
        self.retirer_les_copies_scellees();
        if let Some((_, Suite::Palette(etat))) = &self.ui.question {
            if etat.but == But::Ouvrir && etat.fouillee != self.disque.fouille.version() {
                self.mark_dirty();
            }
        }
    }
}

/// Quand ce fichier a été touché ; `None` s'il n'est plus là.
fn touche(chemin: &Path) -> Option<SystemTime> {
    let m = std::fs::metadata(chemin).ok()?;
    m.is_file()
        .then(|| m.modified().unwrap_or(std::time::UNIX_EPOCH))
}

/// **Les documents de ces dossiers**, chacun avec son numéro : ceux que Glucose range, ceux qui
/// voisinent ses documents connus.
pub fn dans_ces_dossiers<'a>(dossiers: impl IntoIterator<Item = &'a Path>) -> Vec<Trouve> {
    dossiers
        .into_iter()
        .collect::<std::collections::BTreeSet<&Path>>()
        .into_iter()
        .filter_map(|d| std::fs::read_dir(d).ok())
        .flat_map(|entrees| entrees.flatten().map(|e| e.path()))
        .filter(|p| fouille::est_un_document(p))
        .map(|p| {
            let numero = crate::plateforme::numero::de(&p);
            (p, numero)
        })
        .collect()
}

/// **Le dossier d'un document**, tel qu'on le lit : sous le dossier personnel, `~/…`.
pub fn dossier_court(chemin: &Path, personnel: Option<&Path>) -> String {
    chemin.parent().map_or_else(String::new, |d| {
        match personnel.and_then(|p| d.strip_prefix(p).ok()) {
            Some(reste) if reste.as_os_str().is_empty() => "~".to_string(),
            Some(reste) => format!("~/{}", reste.display()),
            None => d.display().to_string(),
        }
    })
}

/// **Ce qu'une rangée dit à droite du nom** : son dossier, raccourci par `~`, et depuis quand
/// il n'a pas été touché — ou ce qui lui arrive.
pub fn precision(document: &Document, personnel: Option<&Path>, maintenant: SystemTime) -> String {
    let dossier = dossier_court(&document.chemin, personnel);
    let depuis = |quand: SystemTime| {
        let ms = |t: SystemTime| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_millis() as i64)
        };
        crate::dock::temps::il_y_a(ms(maintenant), ms(quand))
    };
    match (document.sorte, document.touche) {
        (Sorte::Introuvable, _) => format!("introuvable · {dossier}"),
        (Sorte::JamaisOuvert, _) => format!("jamais ouvert · {dossier}"),
        (Sorte::Recent, Some(quand)) => format!("{dossier} · {}", depuis(quand)),
        (Sorte::Recent, None) => dossier,
    }
}

#[cfg(test)]
mod tests;
