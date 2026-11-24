//! **Collaborer** (CLIENT-1, fiche 117) : le bouton ouvre la question — partager ce canevas,
//! rejoindre par son code (REJOINDRE-1), la porte aux IA publiques —, et à chaque tour de la boucle, ce qui
//! arrive de la salle entre au magasin, ce que le magasin fait partir va au fil.
//!
//! Le magasin tient la loi (`store::participation` : en attente, rejouer l'ordre de la salle, défaire
//! les siens) ; le fil, le réseau et le chiffre (`salle::lien`). Ici, rien que les relier — et le
//! dire : qui est là, ce qu'un code permet, quand deux gestes se sont croisés.

pub mod api;
pub mod identite;
pub mod nommer;
pub mod photos;
mod rejoindre;
pub mod reprise;
mod revenir;

use crate::app::GlucoseApp;
use crate::salle::lien::{self, Lien, Nouvelle, Ordre};
use crate::ui::question::{Question, Reponse, Suite};
use glucose_core::salle::code::Code;
use glucose_core::salle::lettre::{Corps, Lettre};
use glucose_core::store::journal::partage::GestePartage;
use glucose_core::store::journal::Main;
use glucose_core::store::{Arrivee, Envoi};
use glucose_core::types::Project;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const PARTAGER: Reponse = Reponse::Choix(0);
const REJOINDRE: Reponse = Reponse::Choix(1);
const PORTE: Reponse = Reponse::Choix(2);
const COPIER_ECRIRE: Reponse = Reponse::Choix(3);
const COPIER_LIRE: Reponse = Reponse::Choix(4);
const QUITTER: Reponse = Reponse::Choix(5);
const NOMMER: Reponse = Reponse::Choix(6);
/// Un curseur part au plus toutes les tant : une dizaine par seconde, comme la page de la fiche 113.
const CURSEUR: Duration = Duration::from_millis(100);
/// Un participant dont rien n'est venu depuis ne montre plus son curseur.
const SILENCE: Duration = Duration::from_secs(30);

/// **Où en est l'entrée dans la salle.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entree {
    /// On l'a ouverte : la base est partie la première.
    Partage,
    /// On la rejoint : rien ne s'applique avant la base.
    AttendLaBase,
    /// On y revient (REVENIR-2) : la salle arrive depuis sa base, puis la fusion.
    Revient,
    /// On y est.
    Dedans,
}

impl Entree {
    /// Pas encore dedans : la salle arrive, rien n'en part.
    pub fn attend(self) -> bool {
        matches!(self, Entree::AttendLaBase | Entree::Revient)
    }
}

/// **Un autre participant**, tel qu'on le voit.
#[derive(Debug, Clone)]
pub struct Pair {
    pub nom: String,
    pub couleur: u8,
    /// Le tableau qu'il regarde et le point du monde où est sa souris.
    pub curseur: Option<(String, f64, f64)>,
    pub vu: Instant,
}

/// **Le canevas partagé de ce Glucose.**
pub struct CanevasPartage {
    pub lien: Lien,
    pub code: Code,
    pub marque: String,
    pub nom: String,
    pub couleur: u8,
    pub entree: Entree,
    /// Les autres, par leur marque.
    pub pairs: BTreeMap<String, Pair>,
    curseur_parti: Option<(Instant, String, f64, f64)>,
    /// Le facteur des photos, et ce qu'il faut demander au prochain tour (B1).
    pub photos: photos::Photos,
    /// Le numéro que le fichier contient à coup sûr (REVENIR-2).
    pub borne: reprise::Borne,
    /// Ce que la salle a donné, pendant qu'on y revient.
    retour: Option<Box<revenir::Retour>>,
}

impl CanevasPartage {
    /// **Un partage qui commence** : le fil, le code, qui l'on est, où l'on en est — et le facteur
    /// des photos, qui lit le registre et confie ses lettres au fil.
    fn neuf(
        lien: Lien,
        code: Code,
        qui: photos::Qui,
        entree: Entree,
        objets: std::sync::Arc<crate::persist::objets::Objets>,
    ) -> Self {
        let photos = photos::Photos::lancer(objets, lien.expediteur(), qui.clone());
        let (marque, nom, couleur) = qui;
        Self {
            lien,
            code,
            marque,
            nom,
            couleur,
            entree,
            pairs: BTreeMap::new(),
            curseur_parti: None,
            photos,
            borne: reprise::Borne::default(),
            retour: None,
        }
    }

    /// Les autres qu'on a vus récemment, par leur nom.
    pub fn presents(&self) -> Vec<&Pair> {
        self.pairs
            .values()
            .filter(|p| p.vu.elapsed() < SILENCE)
            .collect()
    }
}

const PAS_DE_RESEAU: &str = "Ce Glucose n'a pas de réseau ici.";
const SALLE_VIDE: &str = "Ce code ne mène encore à aucun canevas : vérifie qu'il est entier, ou \
                          demande-en un autre. Glucose attend qu'il arrive.";

/// **Ce qui n'a pas tenu**, en une phrase : deux gestes croisés sur un même nœud, ou les gestes d'un
/// lecteur, défaits.
fn ce_qui_n_a_pas_tenu(croises: bool, defaits: usize) -> Option<String> {
    match (croises, defaits) {
        (_, d) if d > 0 => Some("Ton code ne fait que regarder : tes gestes ne partent pas".into()),
        (true, _) => Some("Deux gestes ont touché le même nœud : le plus récent l'emporte".into()),
        _ => None,
    }
}

/// **Les curseurs qui se montrent** : ceux des autres qui regardent ce tableau-ci, de leur couleur.
fn curseurs_sur(pairs: &[&Pair], tableau: &str) -> Vec<crate::params::CurseurDuPair> {
    pairs
        .iter()
        .filter_map(|p| {
            let (t, x, y) = p.curseur.as_ref()?;
            (t == tableau).then(|| crate::params::CurseurDuPair {
                monde: (*x, *y),
                couleur: identite::COULEURS[usize::from(p.couleur) % identite::COULEURS.len()],
                nom: p.nom.clone(),
            })
        })
        .collect()
}

/// **Une marque neuve** (ID-2) : une lettre, puis trois lettres ou chiffres, du hasard du système.
fn nouvelle_marque() -> Result<String, String> {
    const SIGNES: &[u8; 36] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let h: [u8; 4] = crate::plateforme::hasard::octets()?;
    let premiere = char::from(SIGNES[usize::from(h[0]) % 26]);
    let suite = h[1..]
        .iter()
        .map(|o| char::from(SIGNES[usize::from(*o) % 36]));
    Ok(std::iter::once(premiere).chain(suite).collect())
}

impl GlucoseApp {
    /// **« Collaborer »** : seul, partager ou rejoindre ; partagé, les codes, qui est là, quitter.
    pub(crate) fn montrer_collaborer(&mut self) {
        self.montrer_collaborer_avec(None);
    }

    /// La question, avec ce qui vient d'avoir lieu en tête — un code copié, un partage qui n'a pas
    /// pu se faire : ce qu'on a fait se dit là où on l'a fait.
    fn montrer_collaborer_avec(&mut self, note: Option<String>) {
        let porte = ("La porte aux IA publiques…".to_string(), PORTE);
        let nommer = ("Ton nom et ta couleur…".to_string(), NOMMER);
        let question = match &self.lancement.agents.partage {
            None => Question {
                titre: "Collaborer".into(),
                texte: "Partager ce canevas : d'autres y travaillent avec toi, chacun dans son \
                        Glucose ; tu leur donnes un code. Rejoindre : avec le code qu'on t'a \
                        donné.\n\nLa porte aux IA publiques : Claude.ai, depuis le nuage."
                    .into(),
                choix: vec![
                    ("Partager ce canevas".into(), PARTAGER),
                    ("Rejoindre…".into(), REJOINDRE),
                    nommer,
                    porte,
                    ("L'API publique…".into(), crate::app::collaborer::api::MENU),
                    ("Fermer".into(), Reponse::Annuler),
                ],
                ..Default::default()
            },
            Some(cp) => {
                let mut choix = Vec::new();
                if cp.code.peut_ecrire() {
                    choix.push(("Copier le code pour y travailler".into(), COPIER_ECRIRE));
                }
                choix.push(("Copier le code pour regarder".into(), COPIER_LIRE));
                choix.push(("Quitter le partage".into(), QUITTER));
                choix.push(nommer);
                choix.push(porte);
                choix.push(("L'API publique…".into(), crate::app::collaborer::api::MENU));
                choix.push(("Fermer".into(), Reponse::Annuler));
                Question {
                    titre: "Canevas partagé".into(),
                    texte: self.etat_du_partage(cp),
                    choix,
                    ..Default::default()
                }
            }
        };
        let mut question = question;
        if let Some(n) = note {
            question.texte = format!("{n}\n\n{}", question.texte);
        }
        self.demander(question, Suite::Collaborer);
    }

    fn etat_du_partage(&self, cp: &CanevasPartage) -> String {
        use crate::agents::publique::lien::EtatDuLien;
        let lien = match cp.lien.etat() {
            EtatDuLien::Relie if cp.entree.attend() => "Relié ; le canevas arrive…".into(),
            EtatDuLien::Relie => "Relié.".to_string(),
            EtatDuLien::EnRoute => "Il se relie…".to_string(),
            EtatDuLien::Hors(r) => format!("Pas relié ({r}) : il réessaie."),
        };
        let noms: Vec<&str> = cp.presents().iter().map(|p| p.nom.as_str()).collect();
        let avec = if noms.is_empty() {
            "Personne d'autre pour l'instant.".to_string()
        } else {
            format!("Avec : {}.", noms.join(", "))
        };
        let pouvoir = if cp.code.peut_ecrire() {
            ""
        } else {
            "\n\nTon code ne fait que regarder : tes gestes ne partent pas."
        };
        format!("{lien} Tu es « {} ».\n\n{avec}{pouvoir}", cp.nom)
    }

    /// Sa réponse à la question de Collaborer.
    pub(crate) fn repondre_a_collaborer(&mut self, reponse: Reponse) {
        match reponse {
            // La première fois, son nom et sa couleur d'abord (IDENTITE-2).
            PARTAGER if !self.d_abord_son_nom(nommer::Puis::Partager) => self.partager_ce_canevas(),
            REJOINDRE if !self.d_abord_son_nom(nommer::Puis::Rejoindre) => self.demander_le_code(),
            NOMMER => self.demander_son_nom(None, None),
            PORTE => self.montrer_la_porte_publique(),
            crate::app::collaborer::api::MENU => self.montrer_l_api_publique(None),
            COPIER_ECRIRE | COPIER_LIRE => {
                let texte = self.lancement.agents.partage.as_ref().and_then(|cp| {
                    if reponse == COPIER_ECRIRE {
                        cp.code.texte_ecrire()
                    } else {
                        Some(cp.code.texte_lire())
                    }
                });
                if let Some(t) = texte {
                    let note = self.copier_un_code(t);
                    self.montrer_collaborer_avec(Some(note));
                }
            }
            QUITTER => {
                self.laisser_le_canevas();
                self.quitter_le_canevas_partage();
            }
            _ => {}
        }
    }

    /// Copie un code ; rend ce qui s'est passé, pour la question — le code lui-même s'il n'a pas pu
    /// se copier, à recopier à la main.
    fn copier_un_code(&self, code: String) -> String {
        let copie =
            crate::interactions::presse_papiers::ouvrir().and_then(|mut a| a.ecrire(code.clone()));
        match copie {
            Ok(()) => "Code copié : donne-le à qui doit venir.".into(),
            Err(e) => format!("Le code n'a pas pu se copier ({e}) ; le voici :\n{code}"),
        }
    }

    /// Le fil vers la salle de ce code, depuis ce numéro.
    fn relier(&self, code: Code, depuis: u64) -> Option<Lien> {
        let boite = self.lancement.agents.boite.clone()?;
        let racine = self
            .lancement
            .agents
            .relais
            .clone()
            .unwrap_or_else(crate::agents::publique::relais);
        Some(lien::lancer(&racine, code, depuis, boite))
    }

    /// **Partage ce canevas** : un code neuf, la base qui part la première, le code copié.
    fn partager_ce_canevas(&mut self) {
        if self.revenir_dans_sa_salle() {
            return;
        }
        let prepare = nouvelle_marque()
            .and_then(|m| crate::plateforme::hasard::octets::<32>().map(|h| (m, Code::neuf(h))));
        let (marque, code) = match prepare {
            Ok(p) => p,
            Err(e) => return self.montrer_collaborer_avec(Some(format!("Partager : {e}."))),
        };
        let Some(lien) = self.relier(code.clone(), 0) else {
            return self.montrer_collaborer_avec(Some(PAS_DE_RESEAU.into()));
        };
        self.terminer_les_gestes_en_cours();
        self.consigner();
        self.store.partager_ce_canevas(&marque);
        let moi = identite::Identite::la_sienne(&self.disque.identite);
        let qui = (marque, moi.nom, moi.couleur as u8);
        let objets = std::sync::Arc::clone(&self.disque.objets);
        self.lancement.agents.partage = Some(CanevasPartage::neuf(
            lien,
            code.clone(),
            qui,
            Entree::Partage,
            objets,
        ));
        self.ui.porte_ouverte = true;
        self.retenir_le_canevas_en_cours();
        let note = code.texte_ecrire().map(|t| self.copier_un_code(t));
        self.montrer_collaborer_avec(note);
    }

    /// Le document laissé : un canevas vide attend la base, le fil part.
    pub(crate) fn commencer_a_rejoindre(&mut self, code: Code) {
        let marque = match nouvelle_marque() {
            Ok(m) => m,
            Err(e) => return self.montrer_collaborer_avec(Some(format!("Rejoindre : {e}."))),
        };
        let Some(lien) = self.relier(code.clone(), 0) else {
            return self.montrer_collaborer_avec(Some(PAS_DE_RESEAU.into()));
        };
        self.quitter_le_canevas_partage();
        self.adopter_un_document_vierge();
        let moi = identite::Identite::la_sienne(&self.disque.identite);
        let qui = (marque, moi.nom, moi.couleur as u8);
        let objets = std::sync::Arc::clone(&self.disque.objets);
        self.lancement.agents.partage = Some(CanevasPartage::neuf(
            lien,
            code,
            qui,
            Entree::AttendLaBase,
            objets,
        ));
        self.ui.porte_ouverte = true;
        self.montrer_collaborer();
    }

    /// **Quitte le partage** : le canevas reste ici, à soi seul ; les autres continuent sans soi.
    pub(crate) fn quitter_le_canevas_partage(&mut self) {
        if self.lancement.agents.partage.take().is_some() {
            self.store.quitter_le_partage();
            self.ui.porte_ouverte = self.lancement.agents.publique.is_some();
        }
    }

    /// **À chaque tour de la boucle** : ce qui arrive entre, ce qui part s'en va.
    pub(crate) fn suivre_le_canevas_partage(&mut self) {
        let Some(cp) = self.lancement.agents.partage.as_ref() else {
            return;
        };
        // Dans le passé, rien n'entre : ce qui arrive attend le retour au présent, dans le fil.
        if self.disque.voyage.is_some() {
            return;
        }
        // Un autre document s'est ouvert (le chargement quitte le partage) : le fil s'arrête aussi.
        if !cp.entree.attend() && self.store.partage.is_none() {
            self.lancement.agents.partage = None;
            self.ui.porte_ouverte = self.lancement.agents.publique.is_some();
            return;
        }
        let Some(cp) = self.lancement.agents.partage.as_ref() else {
            return;
        };
        for nouvelle in cp.lien.relever() {
            self.une_nouvelle(nouvelle);
        }
        if self.lancement.agents.partage.is_none() {
            return;
        }
        let version = self.store.version;
        let croises = !self.store.appliquer_les_arrivees().is_empty();
        let defaits = self.faire_partir();
        self.demander_ce_qui_manque();
        self.tenir_la_borne();
        if let Some(phrase) = ce_qui_n_a_pas_tenu(croises, defaits) {
            self.dire_ce_qui_arrive_au_canevas(phrase);
        }
        if self.store.version != version {
            self.mark_dirty();
        }
    }

    /// Une nouvelle du fil.
    fn une_nouvelle(&mut self, nouvelle: Nouvelle) {
        match nouvelle {
            Nouvelle::Lettre { n, id, lettre } => self.une_lettre(n, id, *lettre),
            Nouvelle::Curseur(lettre) => self.un_curseur(*lettre),
            Nouvelle::Passe(lettre) => self.une_lettre_qui_passe(*lettre),
            Nouvelle::AJour(n) => self.salle_a_jour(n),
            // Un lecteur ne fait rien partir, et une lettre se découpe : un refus dit un défaut.
            Nouvelle::Refus(pourquoi) => {
                crate::dire_erreur!("[Glucose] la salle refuse : {pourquoi}")
            }
            Nouvelle::Illisible(e) => crate::dire_erreur!("[Glucose] canevas partagé : {e}"),
        }
    }

    /// **La salle a tout donné** — j'entre, ou le réseau revient : mon bonjour partira, avec ce qui me
    /// manque (B1). Si l'on attend sa base et qu'elle était vide, ce code ne mène encore
    /// à aucun canevas — un code coupé, ou un partage pas encore fini. On le dit là où l'on a agi ; on
    /// continue d'attendre, la base peut encore venir.
    fn salle_a_jour(&mut self, n: u64) {
        self.mark_dirty();
        let Some(cp) = self.lancement.agents.partage.as_mut() else {
            return;
        };
        cp.photos.dire_bonjour();
        if cp.entree == Entree::Revient {
            return self.revenir_quand_la_salle_est_la(n);
        }
        let attend = cp.entree == Entree::AttendLaBase;
        // Jamais par-dessus une autre question (une IA qui demande la porte, un travail à garder).
        let libre = matches!(self.ui.question, None | Some((_, Suite::Collaborer)));
        if attend && n == 0 && libre {
            self.montrer_collaborer_avec(Some(SALLE_VIDE.into()));
        }
    }

    fn un_pair(&mut self, lettre: &Lettre) {
        let Some(cp) = self.lancement.agents.partage.as_mut() else {
            return;
        };
        if lettre.marque == cp.marque {
            return;
        }
        let pair = cp.pairs.entry(lettre.marque.clone()).or_insert(Pair {
            nom: lettre.nom.clone(),
            couleur: lettre.couleur,
            curseur: None,
            vu: Instant::now(),
        });
        pair.nom.clone_from(&lettre.nom);
        pair.couleur = lettre.couleur;
        pair.vu = Instant::now();
    }

    /// **Une lettre entière** : la base qu'on attendait, un geste, ou le retour de sa propre base.
    fn une_lettre(&mut self, n: u64, id: String, lettre: Lettre) {
        self.un_pair(&lettre);
        let main = lettre.main_chez_moi();
        let Some(entree) = self.lancement.agents.partage.as_ref().map(|cp| cp.entree) else {
            return;
        };
        match (entree, lettre.corps) {
            (Entree::AttendLaBase, Corps::Base(base)) => self.entrer_par_la_base(n, *base),
            (Entree::AttendLaBase, _) => {}
            (Entree::Revient, corps) => self.recueillir(n, corps),
            (_, Corps::Geste(geste)) => self.store.recevoir(Arrivee { n, id, main, geste }),
            // Sa propre base revient numérotée : elle se confirme, comme un geste.
            (_, Corps::Base(_)) => self.store.recevoir(Arrivee {
                n,
                id,
                main,
                geste: GestePartage::default(),
            }),
            // Un curseur, une photo, une demande ne viennent jamais numérotés.
            (_, Corps::Curseur { .. } | Corps::Objet { .. } | Corps::Demande { .. }) => {}
        }
    }

    /// **La base est là** : elle devient le document, et l'on participe — ce qui suit s'y rejoue.
    fn entrer_par_la_base(&mut self, n: u64, base: Project) {
        self.adopter_ce_projet(base);
        let Some(cp) = self.lancement.agents.partage.as_mut() else {
            return;
        };
        if self.store.rejoindre(&cp.marque, cp.code.peut_ecrire(), n) {
            cp.entree = Entree::Dedans;
            self.retenir_le_canevas_en_cours();
        }
    }

    fn un_curseur(&mut self, lettre: Lettre) {
        self.un_pair(&lettre);
        let Corps::Curseur { tableau, x, y } = lettre.corps else {
            return;
        };
        if let Some(p) = self
            .lancement
            .agents
            .partage
            .as_mut()
            .and_then(|cp| cp.pairs.get_mut(&lettre.marque))
        {
            p.curseur = Some((tableau, x, y));
            self.mark_dirty();
        }
    }

    /// **Ce qui part** : les gestes du magasin, écrits et confiés au fil ; son curseur, s'il a bougé.
    /// Rend combien de gestes d'un lecteur se sont défaits.
    fn faire_partir(&mut self) -> usize {
        let sortants = self.store.prendre_les_sortants();
        let defaits = self
            .store
            .partage
            .as_mut()
            .map_or(0, |p| std::mem::take(&mut p.defaits_au_lecteur));
        let curseur = self.curseur_a_partir();
        let Some(cp) = self.lancement.agents.partage.as_mut() else {
            return defaits;
        };
        for s in sortants {
            let corps = match s.envoi {
                Envoi::Base(p) => Corps::Base(p),
                Envoi::Geste(g) => {
                    // Les photos que mes gestes posent partent juste après eux (B1).
                    let photos: Vec<String> = g
                        .images_posees()
                        .iter()
                        .filter_map(|i| i.src.clone())
                        .collect();
                    for cle in photos {
                        cp.photos.porter(cle);
                    }
                    Corps::Geste(g)
                }
            };
            let lettre = Lettre {
                marque: cp.marque.clone(),
                nom: cp.nom.clone(),
                couleur: cp.couleur,
                main: s.main,
                corps,
            };
            cp.lien.envoyer(Ordre::Lettre {
                id: s.id,
                clair: lettre.ecrire(),
            });
        }
        if let Some((tableau, x, y)) = curseur {
            let lettre = Lettre {
                marque: cp.marque.clone(),
                nom: cp.nom.clone(),
                couleur: cp.couleur,
                main: Main::Humain,
                corps: Corps::Curseur {
                    tableau: tableau.clone(),
                    x,
                    y,
                },
            };
            cp.lien.envoyer(Ordre::Curseur(lettre.ecrire()));
            cp.curseur_parti = Some((Instant::now(), tableau, x, y));
        }
        defaits
    }

    /// **Les curseurs des autres**, sur le tableau qu'on regarde, de ceux qu'on a vus récemment.
    pub(crate) fn curseurs_des_pairs(&self) -> Vec<crate::params::CurseurDuPair> {
        self.lancement
            .agents
            .partage
            .as_ref()
            .map(|cp| curseurs_sur(&cp.presents(), &self.store.project.active_board_id))
            .unwrap_or_default()
    }

    /// Sa souris dans le monde, si elle a bougé depuis le dernier curseur parti et qu'il est temps.
    fn curseur_a_partir(&self) -> Option<(String, f64, f64)> {
        let cp = self.lancement.agents.partage.as_ref()?;
        if cp.entree.attend() || !self.curseur_vu {
            return None;
        }
        let vp = self.store.viewport();
        let (x, y) = crate::canvas::screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        let tableau = self.store.project.active_board_id.clone();
        match &cp.curseur_parti {
            Some((_, t, px, py)) if *t == tableau && (*px, *py) == (x, y) => None,
            Some((quand, ..)) if quand.elapsed() < CURSEUR => None,
            _ => Some((tableau, x, y)),
        }
    }
}

#[cfg(test)]
mod tests;
