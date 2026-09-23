//! Pourquoi les panneaux se redessinent (DOCKS-1).
//!
//! # La question que la fiche 24 § 14.1 a laissée ouverte
//!
//! Le compteur `dock` a montré que les panneaux se **refont** sur dix des douze images les plus
//! lentes, et que leur composition n'était pas la cause. La fiche en a conclu que « c'est la
//! clé qui est trop large » — sans pouvoir dire laquelle de ses parties bougeait. Un chantier
//! lancé sur cette seule conclusion aurait deviné.
//!
//! Cette section range chaque image qui a refait un panneau selon la partie de la clé qui a
//! changé. Une souris posée dans un panneau le refait à bon droit ; une place ou une échelle
//! qui change pendant qu'on zoome n'a aucune raison de le faire. Les deux ne se corrigent pas
//! pareil, et aucune durée ne les sépare.

use super::barre;
use crate::chronique::Chronique;

impl Chronique {
    /// Pourquoi les panneaux se sont refaits, raison par raison, sur toutes les images.
    ///
    /// Muette quand aucun panneau ne s'est refait : une section vide n'apprend rien.
    pub(super) fn ecrire_les_panneaux(&self, t: &mut String) {
        let mut raisons: Vec<(&'static str, u64)> = self
            .panneaux_refaits()
            .filter(|(_, images)| *images > 0)
            .map(|(r, images)| (r.nom(), images))
            .collect();
        if raisons.is_empty() {
            return;
        }
        raisons.sort_by_key(|(_, images)| std::cmp::Reverse(*images));
        let total = self.rendues().max(1);
        t.push_str(
            "  Pourquoi les panneaux se redessinent -- une image peut en porter plusieurs\n\n",
        );
        for (nom, images) in raisons {
            let part = images as f64 / total as f64;
            t.push_str(&format!(
                "      {nom:<24} {images:>6} images  {:>5.1}%  {}\n",
                100.0 * part,
                barre(part)
            ));
        }
        t.push('\n');
    }
}
