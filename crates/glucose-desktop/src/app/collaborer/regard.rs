//! **REGARD-1 — sur quoi il se concentre** (fiche 177 ; sa note du 24/11 dans `relais-rep-mcp` :
//! *« un système qui voit aussi les derniers éléments sélectionnés par les utilisateurs, même à
//! distance, pour savoir sur quoi ils se sont focalisés »*).
//!
//! Chaque appareil retient **sa dernière sélection non vide** — le tableau, les nœuds, l'heure — et
//! la dit aux autres du canevas partagé, comme son curseur : ni gardée ni numérotée par la salle.
//! C'est la présence de tldraw (`selectedShapeIds` à côté du curseur) et celle de Yjs : un état
//! entier, redit à qui arrive — ici à chaque bonjour, le sien comme celui d'un autre. Une IA la lit
//! dans `glucose_quoi_de_neuf` : où il en est, sans relire le tableau (ses jetons, sa priorité du
//! 24/11).

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use crate::boite_noire::sondes::maintenant_ms;
use crate::salle::lien::Ordre;
use glucose_core::salle::lettre::{Corps, Lettre};
use glucose_core::store::journal::Main;

/// **Ce qu'un appareil a choisi en dernier** : le tableau, les nœuds dans l'ordre du choix (ceux
/// qu'un curseur a portés, sur `en_tout`), et quand, en millisecondes depuis 1970.
#[derive(Debug, Clone, PartialEq)]
pub struct Regard {
    pub tableau: String,
    pub ids: Vec<String>,
    pub en_tout: usize,
    pub quand_ms: u64,
}

/// **Ce que ce Glucose a vu choisir ici.**
#[derive(Debug, Default)]
pub struct Regards {
    /// Sa dernière sélection non vide.
    pub ici: Option<Regard>,
    /// La marque des deux choix au dernier relevé : une sélection écrite change sa marque
    /// (CHOIX-1), la comparer ne relit rien.
    marques: (u64, u64),
    /// La sélection était vide au dernier relevé : la même, choisie de nouveau, est un regard neuf.
    vide: bool,
}

impl GlucoseApp {
    /// **À chaque tour de la boucle** : une sélection neuve et non vide devient son regard, et part
    /// aux autres. La même réécrite — un geste d'IA remet la sélection telle qu'elle était — n'en
    /// est pas un : son heure ne bouge pas.
    pub(crate) fn suivre_le_regard(&mut self) {
        let s = &self.store;
        let marques = (
            s.selected_image_ids.marque(),
            s.selected_annotation_ids.marque(),
        );
        let regards = &mut self.lancement.agents.regards;
        if marques == regards.marques {
            return;
        }
        regards.marques = marques;
        let ids: Vec<String> = (s.selected_image_ids.iter())
            .chain(s.selected_annotation_ids.iter())
            .cloned()
            .collect();
        let vide = std::mem::replace(&mut regards.vide, ids.is_empty());
        let tableau = &s.project.active_board_id;
        let meme = (regards.ici.as_ref()).is_some_and(|r| r.ids == ids && r.tableau == *tableau);
        if ids.is_empty() || (meme && !vide) {
            return;
        }
        regards.ici = Some(Regard {
            tableau: tableau.clone(),
            en_tout: ids.len(),
            ids,
            quand_ms: maintenant_ms(),
        });
        self.dire_mon_regard();
    }

    /// **Mon regard part** aux autres du canevas partagé, avec son âge : quand il change, et à
    /// chaque bonjour.
    pub(crate) fn dire_mon_regard(&self) {
        let agents = &self.lancement.agents;
        let (Some(r), Some(cp)) = (&agents.regards.ici, &agents.partage) else {
            return;
        };
        if cp.entree.attend() {
            return;
        }
        let lettre = Lettre {
            marque: cp.marque.clone(),
            nom: cp.nom.clone(),
            couleur: cp.couleur,
            main: Main::Humain,
            corps: Corps::Selection {
                tableau: r.tableau.clone(),
                ids: r.ids.clone(),
                en_tout: r.en_tout,
                age: maintenant_ms().saturating_sub(r.quand_ms) / 1000,
            },
        };
        let lettre = lettre.tenir_dans_un_curseur();
        cp.lien.envoyer(Ordre::Curseur(lettre.ecrire()));
    }

    /// **Le regard d'un autre**, daté chez soi de l'âge qu'il dit : aucune horloge à accorder.
    pub(super) fn un_regard(&mut self, marque: &str, corps: Corps) {
        let Corps::Selection {
            tableau,
            ids,
            en_tout,
            age,
        } = corps
        else {
            return;
        };
        let Some(p) =
            (self.lancement.agents.partage.as_mut()).and_then(|cp| cp.pairs.get_mut(marque))
        else {
            return;
        };
        p.regard = Some(Regard {
            tableau,
            ids,
            en_tout,
            quand_ms: maintenant_ms().saturating_sub(age.saturating_mul(1000)),
        });
    }

    /// **Sur quoi il se concentre**, pour une IA : la dernière sélection de chaque appareil — ce
    /// Glucose (« ici »), puis chacun du canevas partagé, par son nom —, la plus récente d'abord.
    pub(crate) fn regards_en_json(&self) -> Vec<Json> {
        let agents = &self.lancement.agents;
        let ici = (agents.regards.ici.as_ref()).map(|r| ("ici", r));
        let pairs = (agents.partage.iter())
            .flat_map(|cp| cp.pairs.values())
            .filter_map(|p| p.regard.as_ref().map(|r| (p.nom.as_str(), r)));
        let mut tous: Vec<_> = ici.into_iter().chain(pairs).collect();
        tous.sort_by_key(|(_, r)| std::cmp::Reverse(r.quand_ms));
        tous.into_iter()
            .map(|(appareil, r)| {
                let quand = i64::try_from(r.quand_ms).ok();
                let quand = quand.and_then(crate::plateforme::heure::heure_locale);
                let mut champs = vec![
                    ("appareil", Json::from(appareil)),
                    ("tableau", Json::from(r.tableau.as_str())),
                    (
                        "ids",
                        Json::Liste(r.ids.iter().map(|i| Json::from(i.as_str())).collect()),
                    ),
                    (
                        "quand",
                        Json::texte(quand.map_or_else(String::new, |h| h.to_string())),
                    ),
                ];
                if r.en_tout > r.ids.len() {
                    champs.push(("en_tout", Json::Nombre(r.en_tout as f64)));
                }
                Json::objet(champs)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
