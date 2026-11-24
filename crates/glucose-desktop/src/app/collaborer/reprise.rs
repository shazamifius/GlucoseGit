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
//! # Revenir, jamais au numéro exact (REVENIR-2, fiche 148)
//!
//! Glucose retient, pour chaque document partagé (par son fichier) : **le code de la salle**,
//! **une borne basse** — un numéro de la salle que le fichier contient à coup sûr, sur le disque —, et
//! s'il y est encore (**dedans**). Au lancement, un document dedans **revient** dans sa salle : il la
//! reçoit depuis sa base, la rebâtit à la borne (l'ancêtre commun) et maintenant, puis fusionne
//! (`Store::revenir_dans_la_salle`, fiche 147 § 6). Une borne trop basse ne perd rien : au pire, une
//! version d'ici dite en conflit. Un arrêt net revient donc aussi, et rien n'attend plus, à la
//! fermeture, le retour de mes gestes : partis ou non, ils sont dans le fichier.
//!
//! **« Quitter » ne l'oublie plus** (fiche 146 § 4, le 1) : le document garde son code, dehors ; son
//! prochain « Partager » revient dans la même salle au lieu d'en ouvrir une seconde.
//!
//! Sous **une marque neuve** : les noms de mes lettres d'avant ne se mêlent pas aux nouvelles.
//!
//! **Dans le dossier de Glucose, lisible de lui seul — jamais dans le `.glucose`** : le code
//! « écrire » est un secret ; un fichier envoyé à quelqu'un ne l'emporte pas.

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::salle::code::Code;
use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError};

/// Le fichier, dans le dossier de Glucose.
pub const FICHIER: &str = "canevas-partages.json";

/// **Ce que Glucose retient d'un document partagé.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Souvenir {
    /// Le code de la salle — celui qui écrit, s'il l'a.
    pub code: String,
    /// Un numéro de la salle que le fichier contient à coup sûr : jamais plus haut.
    pub retenu: u64,
    /// Encore dans la salle : le lancement l'y fait revenir. « Quitter » le met dehors.
    pub dedans: bool,
}

/// Tous les souvenirs, par fichier de document. Absent ou illisible : aucun.
fn tous(fichier: &Path) -> Vec<(String, Json)> {
    match std::fs::read_to_string(fichier).map(|t| Json::lire(&t)) {
        Ok(Ok(Json::Objet(o))) => o,
        _ => Vec::new(),
    }
}

/// Le souvenir de ce document. Celui d'avant REVENIR-2 (`confirme`, `null` après un arrêt net) se
/// lit encore : il était dedans, et sa borne vaut au moins zéro.
pub fn lire(fichier: &Path, document: &str) -> Option<Souvenir> {
    let (_, s) = tous(fichier).into_iter().find(|(d, _)| d == document)?;
    let nombre = |cle| s.get(cle).and_then(Json::as_f64).map(|n| n as u64);
    Some(Souvenir {
        code: s.get("code")?.as_str()?.to_string(),
        retenu: nombre("retenu").or_else(|| nombre("confirme")).unwrap_or(0),
        dedans: s.get("dedans").and_then(Json::as_bool).unwrap_or(true),
    })
}

/// Retient ce souvenir pour ce document — ou l'oublie (`None`).
pub fn retenir(fichier: &Path, document: &str, souvenir: Option<&Souvenir>) -> Result<(), String> {
    let mut tous = tous(fichier);
    tous.retain(|(d, _)| d != document);
    if let Some(s) = souvenir {
        // Jamais « confirme » : un Glucose d'avant y lirait un numéro exact.
        let valeur = Json::objet([
            ("code", Json::texte(s.code.clone())),
            ("retenu", Json::Nombre(s.retenu as f64)),
            ("dedans", Json::Bool(s.dedans)),
        ]);
        tous.push((document.to_string(), valeur));
    }
    crate::ia::cles::ecrire_secret(fichier, Json::Objet(tous).ecrire().as_bytes())
}

/// **La borne du partage, tenue sur le disque** : la dernière retenue, et la synchronisation en vol
/// avec le numéro qu'elle rendra sûr. Une seule à la fois : le disque donne son propre rythme.
#[derive(Default)]
pub struct Borne {
    pub retenue: u64,
    en_vol: Option<(Receiver<Result<(), String>>, u64)>,
}

impl Borne {
    /// Partant de ce numéro, sûr.
    pub fn depuis(retenue: u64) -> Self {
        Self {
            retenue,
            en_vol: None,
        }
    }
}

impl GlucoseApp {
    /// Le fichier du document ouvert, s'il en a un : la clé de son souvenir.
    /// Le document dont cette fenêtre tient le partage — **seulement si elle l'écrit** (fiche 144
    /// § 2) : une seconde fenêtre qui le lit sans le verrou (il est tenu ailleurs) ne reprend pas
    /// sa salle, et ne retient rien — elle retiendrait un numéro que le fichier ne contient pas.
    pub(super) fn document_du_canevas(&self) -> Option<String> {
        let document = self.disque.objets.document()?;
        let scribe = (self.disque.ecriture.as_ref())
            .is_some_and(|e| crate::persist::verrou::meme_fichier(&e.chemin, &document));
        scribe.then(|| document.display().to_string())
    }

    /// Le code du partage en cours : celui qui écrit, s'il l'a.
    fn code_du_partage(&self) -> Option<String> {
        let cp = self.lancement.agents.partage.as_ref()?;
        Some(
            cp.code
                .texte_ecrire()
                .unwrap_or_else(|| cp.code.texte_lire()),
        )
    }

    /// **Ce que le document partagé laisse en se fermant**, tout consigné : son code, et le dernier
    /// numéro de la salle qu'il contient — retenu une fois le fichier synchronisé.
    pub(crate) fn souvenir_du_canevas(&self) -> Option<(String, Souvenir)> {
        let retenu = self.store.partage.as_ref()?.confirme;
        let code = self.code_du_partage()?;
        let s = Souvenir {
            code,
            retenu,
            dedans: true,
        };
        Some((self.document_du_canevas()?, s))
    }

    /// Le retient, une fois le document écrit et synchronisé.
    pub(crate) fn retenir_le_canevas(&self, souvenir: Option<(String, Souvenir)>) {
        if let Some((document, s)) = souvenir {
            if let Err(e) = retenir(&self.disque.canevas, &document, Some(&s)) {
                crate::dire_erreur!("[Glucose] canevas partagé : pas retenu ({e})");
            }
        }
    }

    /// **Le partage commence, revient, ou sa borne monte** : son souvenir dit « dedans », à la borne
    /// que le disque a rendue sûre.
    pub(crate) fn retenir_le_canevas_en_cours(&self) {
        let (Some(cp), Some(code)) = (
            self.lancement.agents.partage.as_ref(),
            self.code_du_partage(),
        ) else {
            return;
        };
        let s = Souvenir {
            code,
            retenu: cp.borne.retenue,
            dedans: true,
        };
        self.retenir_le_canevas(self.document_du_canevas().map(|d| (d, s)));
    }

    /// **« Quitter le partage »** : le document garde son code, dehors — tout écrit et synchronisé
    /// d'abord, pour que la borne soit la plus haute possible.
    pub(crate) fn laisser_le_canevas(&mut self) {
        self.consigner();
        let sur = (self.disque.ecriture.as_ref()).is_some_and(|e| e.synchroniser().is_ok());
        let Some(document) = self.document_du_canevas() else {
            return;
        };
        let souvenir = if sur {
            self.souvenir_du_canevas().map(|(_, s)| s)
        } else {
            lire(&self.disque.canevas, &document)
        };
        if let Some(s) = souvenir {
            let dehors = Souvenir { dedans: false, ..s };
            self.retenir_le_canevas(Some((document, dehors)));
        }
    }

    /// **À chaque tour du partage, la borne monte** : quand la salle a confirmé plus loin que la
    /// borne, ce qui attend s'écrit, une synchronisation part sans qu'on l'attende ; revenue, le
    /// numéro qu'elle couvrait est sûr et se retient.
    pub(crate) fn tenir_la_borne(&mut self) {
        let Some(confirme) = self.store.partage.as_ref().map(|p| p.confirme) else {
            return;
        };
        let Some(cp) = self.lancement.agents.partage.as_mut() else {
            return;
        };
        if let Some((reponse, n)) = &cp.borne.en_vol {
            match reponse.try_recv() {
                Err(TryRecvError::Empty) => return,
                Ok(Ok(())) => {
                    cp.borne.retenue = cp.borne.retenue.max(*n);
                    cp.borne.en_vol = None;
                    self.retenir_le_canevas_en_cours();
                    return;
                }
                Ok(Err(_)) | Err(TryRecvError::Disconnected) => cp.borne.en_vol = None,
            }
        }
        if confirme <= cp.borne.retenue {
            return;
        }
        self.consigner();
        let reponse = self
            .disque
            .ecriture
            .as_ref()
            .map(|e| e.synchroniser_plus_tard());
        if let (Some(r), Some(cp)) = (reponse, self.lancement.agents.partage.as_mut()) {
            cp.borne.en_vol = Some((r, confirme));
        }
    }

    /// **Au lancement** : le document rouvert était-il dans sa salle ? Il y revient.
    pub(crate) fn reprendre_le_canevas_partage(&mut self) {
        let Some(document) = self.document_du_canevas() else {
            return;
        };
        let Some(souvenir) = lire(&self.disque.canevas, &document) else {
            return;
        };
        match Code::lire_le_texte(&souvenir.code) {
            Some(code) if souvenir.dedans => {
                self.revenir_par_le_code(code, souvenir.retenu);
            }
            Some(_) => {}
            None => {
                let _ = retenir(&self.disque.canevas, &document, None);
            }
        }
    }
}

#[cfg(test)]
mod tests;
