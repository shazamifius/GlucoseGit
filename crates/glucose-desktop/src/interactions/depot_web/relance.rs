//! **Remplacer un lien par son image** (DEPOT-WEB-6) : le repli d'un dépôt se rattrape.
//!
//! # Ce qui s'est passé
//!
//! Le 07/10, six épingles glissées depuis Pinterest sont devenues six liens. Son document le
//! date : trois nés dans la même seconde, puis un toutes les seize secondes — le délai de chaque
//! étape du téléchargement, épuisé l'un après l'autre. Une heure plus tard, les six se
//! rapatriaient en une seconde chacune, en pleine résolution : le réseau de cet instant-là ne
//! répondait pas. Mais le lien posé était une **impasse** : rien ne permettait de réessayer, sinon
//! retourner sur Pinterest et refaire chaque geste.
//!
//! # Ce qu'on fait
//!
//! Un nœud qui n'est **qu'un** lien — ce que le dépôt pose quand l'image ne vient pas
//! ([`moisson::adresse_du_lien`]) — porte au clic droit « Remplacer par l'image ». La recherche
//! repart sur un fil, avec son marqueur au coin du lien ; ce qu'elle rapporte prend la place du
//! lien **en un seul geste** — un `Ctrl+Z` rend le lien — et se tait, comme un collage : l'image
//! à sa place est le compte-rendu. Si elle ne vient toujours pas, le lien reste, et le message
//! dit pourquoi.
//!
//! Le chemin est celui de tout dépôt — l'annonce, le marqueur, la livraison par numéro — et il
//! ne dépend d'aucun pont : un lien de Windows se rattrape aussi sous Linux.

use crate::app::GlucoseApp;
use crate::interactions::drop::Lot;
use crate::params::Arrivage;
use crate::plateforme::moisson::{self, Depot, Moisson};
use crate::plateforme::rapatrier::{self, Recherche};
use glucose_core::store::Store;
use glucose_core::types::Annotation;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};

/// **Les relances en vol** : leur canal de retour, et le lien que chacune remplacera.
pub struct Relances {
    envoi: Sender<Depot>,
    recu: Receiver<Depot>,
    /// Le numéro d'une relance → le tableau et le lien qu'elle remplacera.
    liens: HashMap<u64, (String, String)>,
    /// La recherche elle-même : le réseau en vrai ; en épreuve, celle que l'épreuve fournit —
    /// et sans elle, un refus (aucune épreuve n'atteint le réseau, comme DIAL-3 pour les boîtes).
    pub(crate) recherche: Recherche,
}

impl Default for Relances {
    fn default() -> Self {
        let (envoi, recu) = channel();
        Self {
            envoi,
            recu,
            liens: HashMap::new(),
            recherche: if cfg!(test) {
                refusee_en_epreuve
            } else {
                rapatrier::chercher_sur_le_reseau
            },
        }
    }
}

impl Relances {
    /// Ce que les relances ont rapporté depuis le dernier passage, sans attendre.
    pub(super) fn recoltees(&self) -> Vec<Depot> {
        self.recu.try_iter().collect()
    }
}

/// La recherche d'une épreuve qui n'en a pas fourni : elle refuse, et l'épreuve le verra.
fn refusee_en_epreuve(_: &[String]) -> Result<Moisson, String> {
    Err("aucune epreuve n'atteint le reseau".into())
}

/// Un lien choisi, prêt à remplacer : son tableau, son identifiant, son adresse, son coin.
pub struct LienChoisi {
    pub board: String,
    pub id: String,
    pub adresse: String,
    pub coin: (f64, f64),
}

/// **Les nœuds choisis qui ne sont qu'un lien**, dans l'ordre de la sélection. C'est ce que le
/// menu demande pour proposer l'entrée, et ce que l'entrée remplace.
pub fn liens_choisis(store: &Store) -> Vec<LienChoisi> {
    let board = &store.project.active_board_id;
    store
        .selected_annotation_ids
        .iter()
        .filter_map(|id| match store.project.annotation(board, id)? {
            Annotation::Text { x, y, text, .. } => {
                moisson::adresse_du_lien(text).map(|adresse| LienChoisi {
                    board: board.clone(),
                    id: id.clone(),
                    adresse,
                    coin: (*x, *y),
                })
            }
            _ => None,
        })
        .collect()
}

/// Ce lien est-il toujours là, et toujours un lien seul ? Pendant la seconde de la recherche,
/// l'utilisateur a pu l'effacer ou y écrire : ses mots ne se remplacent pas.
fn lien_intact(store: &Store, board: &str, id: &str) -> bool {
    matches!(
        store.project.annotation(board, id),
        Some(Annotation::Text { text, .. }) if moisson::adresse_du_lien(text).is_some()
    )
}

impl GlucoseApp {
    /// **« Remplacer par l'image »** : chaque lien choisi repart chercher son image, son
    /// marqueur posé à son coin.
    pub(crate) fn remplacer_les_liens_par_leur_image(&mut self) {
        let reveil = self.window.clone().map(|fenetre| {
            let reveil: crate::plateforme::Reveil =
                std::sync::Arc::new(move || fenetre.request_redraw());
            reveil
        });
        for lien in liens_choisis(&self.store) {
            let numero = rapatrier::numero_suivant();
            let hote = crate::plateforme::sources::decouper(&lien.adresse)
                .map(|a| a.hote)
                .unwrap_or_default();
            self.depot.en_chemin.push(Arrivage {
                numero,
                monde: lien.coin,
                hote,
            });
            let relances = &mut self.depot.relances;
            relances.liens.insert(numero, (lien.board, lien.id));
            rapatrier::relancer(
                lien.adresse,
                numero,
                relances.recherche,
                (relances.envoi.clone(), reveil.clone()),
            );
        }
        self.mark_dirty();
    }

    /// **La livraison d'une relance**, si ce numéro en est une : le lien laisse sa place à
    /// l'image en un seul geste, ou reste, et le message dit pourquoi. Rend `false` si ce
    /// numéro n'est pas une relance — c'est alors un dépôt ordinaire.
    pub(super) fn livrer_une_relance(
        &mut self,
        numero: Option<u64>,
        moisson: &Moisson,
        coin: Option<(f64, f64)>,
    ) -> bool {
        let Some((board, id)) = numero.and_then(|n| self.depot.relances.liens.remove(&n)) else {
            return false;
        };
        if !lien_intact(&self.store, &board, &id) {
            return true;
        }
        if moisson.recus.is_empty() {
            let raison = moisson.echec.as_deref().unwrap_or("rien n'est venu");
            self.dire_le_depot(format!(
                "Image toujours introuvable ({raison}) : le lien reste"
            ));
            return true;
        }
        // Un seul geste : le lien part, l'image arrive à son coin. Une image qui se voit à sa
        // place se tait, comme un collage. Si rien n'a pu se poser, le geste s'abandonne — le
        // lien ne part jamais seul.
        let coin = coin.unwrap_or((0.0, 0.0));
        self.store.begin_live_edit();
        self.store.remove_annotations(&board, &[id.as_str()]);
        let lot = Lot::de(&[], moisson.recus.clone(), &[]);
        if self.poser_le_lot(&board, lot, coin) > 0 {
            self.store.end_live_edit();
        } else {
            self.store.cancel_live_edit();
            self.dire_le_depot("L'image venue ne se lit pas : le lien reste".to_string());
        }
        self.mark_dirty();
        true
    }
}

#[cfg(test)]
mod tests;
