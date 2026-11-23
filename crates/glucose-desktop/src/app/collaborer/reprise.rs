//! **PARTAGE-SURVIT — un canevas partagé survit au relancement de Glucose** (ACCÈS-H24 point 3,
//! fiche 131).
//!
//! Sa demande du 22/11 : *« que je puisse me connecter à distance […] en lecture + écriture pour
//! interagir avec elle »* — de loin, il entre dans son canevas par le code « écrire ». La salle
//! garde tout (la base et chaque geste numéroté) : on y entre même quand son Glucose est fermé.
//! Mais son Glucose relancé (son service, une mise à jour, le PC redémarré) **ne participait
//! plus** : ce qu'on faisait de loin n'arrivait plus dans son document, et ce que le relais y
//! écrivait ne partait plus.
//!
//! # Reprendre au numéro exact, ou ne pas reprendre
//!
//! Glucose retient, pour chaque document partagé (par son fichier) : **le code de la salle**, et,
//! à la fermeture propre du document — tout écrit et synchronisé —, **le numéro du dernier geste
//! de la salle que le fichier contient**. Au lancement, rouvert, le document rejoint la salle à ce
//! numéro, et rattrape ce que les autres ont fait entre-temps.
//!
//! Le numéro doit être **exact** : un déplacement voyage comme un vecteur (`Deplacement::delta`) —
//! le rejouer deux fois déplacerait deux fois. Il n'est donc retenu qu'à la fermeture du document
//! (pas à la fin du programme : la boucle peut encore appliquer un geste après, qui ne s'écrirait
//! nulle part), et seulement si aucun de mes gestes n'attend encore son numéro (il serait dans le
//! fichier sans être dans la salle). Pendant le partage, le souvenir dit « en cours » : un arrêt
//! net le laisse ainsi, et le lancement suivant **ne reprend pas tout seul** — il le dit.
//!
//! Sous **une marque neuve** : les noms de mes lettres d'avant ne se mêlent pas aux nouvelles.
//!
//! **Dans le dossier de Glucose, lisible de lui seul — jamais dans le `.glucose`** : le code
//! « écrire » est un secret ; un fichier envoyé à quelqu'un ne l'emporte pas.

use super::{identite, nouvelle_marque, CanevasPartage, Entree};
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::salle::code::Code;
use std::path::Path;

/// Le fichier, dans le dossier de Glucose.
pub const FICHIER: &str = "canevas-partages.json";

/// **Ce que Glucose retient d'un document partagé.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Souvenir {
    /// Le code de la salle — celui qui écrit, s'il l'a.
    pub code: String,
    /// Le numéro du dernier geste de la salle que le fichier contient, retenu à sa fermeture
    /// propre ; `None` pendant le partage — un arrêt net le laisse ainsi.
    pub confirme: Option<u64>,
}

/// Tous les souvenirs, par fichier de document. Absent ou illisible : aucun.
fn tous(fichier: &Path) -> Vec<(String, Json)> {
    match std::fs::read_to_string(fichier).map(|t| Json::lire(&t)) {
        Ok(Ok(Json::Objet(o))) => o,
        _ => Vec::new(),
    }
}

/// Le souvenir de ce document.
pub fn lire(fichier: &Path, document: &str) -> Option<Souvenir> {
    let (_, s) = tous(fichier).into_iter().find(|(d, _)| d == document)?;
    Some(Souvenir {
        code: s.get("code")?.as_str()?.to_string(),
        confirme: s.get("confirme").and_then(Json::as_f64).map(|n| n as u64),
    })
}

/// Retient ce souvenir pour ce document — ou l'oublie (`None`).
pub fn retenir(fichier: &Path, document: &str, souvenir: Option<&Souvenir>) -> Result<(), String> {
    let mut tous = tous(fichier);
    tous.retain(|(d, _)| d != document);
    if let Some(s) = souvenir {
        let confirme = s.confirme.map_or(Json::Nul, |n| Json::Nombre(n as f64));
        let valeur = Json::objet([
            ("code", Json::texte(s.code.clone())),
            ("confirme", confirme),
        ]);
        tous.push((document.to_string(), valeur));
    }
    crate::ia::cles::ecrire_secret(fichier, Json::Objet(tous).ecrire().as_bytes())
}

impl GlucoseApp {
    /// Le fichier du document ouvert, s'il en a un : la clé de son souvenir.
    fn document_du_canevas(&self) -> Option<String> {
        self.disque
            .objets
            .document()
            .map(|d| d.display().to_string())
    }

    /// **Ce que le document partagé laisse en se fermant** : son code, et le numéro exact que son
    /// fichier contient — sauf si un de mes gestes attend encore le sien.
    pub(crate) fn souvenir_du_canevas(&self) -> Option<(String, Souvenir)> {
        let cp = self.lancement.agents.partage.as_ref()?;
        let p = self.store.partage.as_ref()?;
        let code = cp
            .code
            .texte_ecrire()
            .unwrap_or_else(|| cp.code.texte_lire());
        let confirme = (p.en_attente() == 0).then_some(p.confirme);
        Some((self.document_du_canevas()?, Souvenir { code, confirme }))
    }

    /// Le retient, une fois le document écrit et synchronisé.
    pub(crate) fn retenir_le_canevas(&self, souvenir: Option<(String, Souvenir)>) {
        if let Some((document, s)) = souvenir {
            if let Err(e) = retenir(&self.disque.canevas, &document, Some(&s)) {
                crate::dire_erreur!("[Glucose] canevas partagé : pas retenu ({e})");
            }
        }
    }

    /// **Le partage commence, ou reprend** : son souvenir dit « en cours ».
    pub(crate) fn retenir_le_canevas_en_cours(&self) {
        let (Some(cp), Some(document)) = (
            self.lancement.agents.partage.as_ref(),
            self.document_du_canevas(),
        ) else {
            return;
        };
        let code = cp
            .code
            .texte_ecrire()
            .unwrap_or_else(|| cp.code.texte_lire());
        let souvenir = Souvenir {
            code,
            confirme: None,
        };
        if let Err(e) = retenir(&self.disque.canevas, &document, Some(&souvenir)) {
            crate::dire_erreur!("[Glucose] canevas partagé : pas retenu ({e})");
        }
    }

    /// **« Quitter le partage »** : le document l'oublie.
    pub(crate) fn oublier_le_canevas(&self) {
        if let Some(document) = self.document_du_canevas() {
            let _ = retenir(&self.disque.canevas, &document, None);
        }
    }

    /// **Au lancement** : le document rouvert était-il partagé ? Il rejoint sa salle au numéro que
    /// son fichier contient — ou le dit, s'il s'était arrêté net.
    pub(crate) fn reprendre_le_canevas_partage(&mut self) {
        let Some(document) = self.document_du_canevas() else {
            return;
        };
        let Some(souvenir) = lire(&self.disque.canevas, &document) else {
            return;
        };
        let code = Code::lire_le_texte(&souvenir.code);
        match (code, souvenir.confirme) {
            (Some(code), Some(n)) => self.rejoindre_au_numero(code, n),
            (Some(_), None) => {
                let _ = retenir(&self.disque.canevas, &document, None);
                self.dire_ce_qui_arrive_au_canevas(ARRETE_NET.into());
            }
            (None, _) => {
                let _ = retenir(&self.disque.canevas, &document, None);
            }
        }
    }

    /// Rejoint la salle de ce code au numéro `n` : ce document en contient tout jusque-là.
    fn rejoindre_au_numero(&mut self, code: Code, n: u64) {
        let Ok(marque) = nouvelle_marque() else {
            return;
        };
        let Some(lien) = self.relier(code.clone(), n) else {
            return;
        };
        if !self.store.rejoindre(&marque, code.peut_ecrire(), n) {
            return;
        }
        let moi = identite::Identite::la_sienne(&self.disque.identite);
        let qui = (marque, moi.nom, moi.couleur as u8);
        let objets = std::sync::Arc::clone(&self.disque.objets);
        self.lancement.agents.partage = Some(CanevasPartage::neuf(
            lien,
            code,
            qui,
            Entree::Dedans,
            objets,
        ));
        self.ui.porte_ouverte = true;
        self.retenir_le_canevas_en_cours();
        crate::dire!("[Glucose] canevas partagé : repris au geste n° {n} de sa salle");
    }
}

/// Ce que dit un document partagé qui s'était arrêté net.
const ARRETE_NET: &str = "Ce document était partagé, mais Glucose s'est arrêté net : il ne sait \
                          plus exactement où il en était dans la salle, et ne la rejoint pas tout \
                          seul. Collaborer : la partager de nouveau, ou la rejoindre par son code";

#[cfg(test)]
mod tests;
