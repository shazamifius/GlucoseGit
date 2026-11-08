//! **Ce que les autres fichiers disent des domaines**, branché sur le panneau (fiche 88) : suivre le
//! recensement, poser ici un domaine connu d'ailleurs, répondre à la question d'un homonyme, aller à
//! la place d'un nœud dans un autre fichier.

use crate::app::GlucoseApp;
use crate::dock::domains::{Ailleurs, Connus, LigneOu, Ou};
use crate::dock::TabId;
use crate::persist::close::Ensuite;
use glucose_core::error::CoreError;
use glucose_core::recensement::{homonymes, recenser, DocumentRecense, DomaineConnu};
use glucose_core::store::{lignee_de, DomainPatch};
use std::path::PathBuf;

impl GlucoseApp {
    /// **Démarre la lecture des fichiers connus** (au vrai lancement seulement : une épreuve n'ouvre
    /// aucun fichier d'une personne).
    pub(crate) fn demarrer_le_recensement(&mut self, reveil: crate::plateforme::Reveil) {
        let Some(dossier) = self.disque.brouillons.parent().map(PathBuf::from) else {
            return;
        };
        let rangs = vec![
            self.disque.brouillons.clone(),
            self.disque.documents.clone(),
        ];
        self.disque.recenseur.demarrer(&dossier, rangs, reveil);
    }

    /// **Le fichier où ce document s'écrit**, sous lequel le recensement le connaît : son fichier
    /// nommé, ou son brouillon. L'identité du document ne convient pas : enregistrer ne la change pas
    /// (fiche 80), et il se compterait deux fois, sous son nom et sous son fichier. Sans fichier
    /// encore, son identité : il n'est nulle part ailleurs.
    pub(crate) fn fichier_de_ce_document(&self) -> String {
        self.disque.ecriture.as_ref().map_or_else(
            || self.store.identite.clone(),
            |e| e.chemin.display().to_string(),
        )
    }

    /// **Ce document, sans ses nœuds** : le panneau n'a besoin que de son catalogue, et ne parcourt
    /// jamais les nœuds (DOM-UI-3).
    fn ce_catalogue(&self) -> DocumentRecense {
        DocumentRecense {
            chemin: self.fichier_de_ce_document(),
            catalogue: self.store.project.domains.clone(),
            noeuds: Vec::new(),
        }
    }

    pub(super) fn vue_du_catalogue(&self) -> Vec<DomaineConnu> {
        self.disque.recenseur.etat.vue(Some(&self.ce_catalogue()))
    }

    /// **Relève ce qui est arrivé**, à chaque image : un `try_recv`, et rien tant que rien ne change.
    pub(crate) fn suivre_le_recensement(&mut self) {
        let ouvert = self.disque.ecriture.as_ref().map(|e| e.chemin.clone());
        self.disque.recenseur.suivre_le_fichier_ouvert(ouvert);
        let arrive = self.disque.recenseur.relever();
        if arrive {
            self.accorder_les_domaines();
        }
        let ici = self.fichier_de_ce_document();
        let a_jour = self
            .disque
            .recenseur
            .vu
            .as_ref()
            .is_some_and(|(d, v)| *d == ici && *v == self.store.version);
        if arrive || (!a_jour && self.dock_manager.is_open(TabId::Domains)) {
            self.disque.recenseur.vu = Some((ici, self.store.version));
            self.dock_manager.domains.connus = self.domaines_connus();
            if arrive {
                self.mark_dirty();
            }
        }
        self.aller_a_la_destination();
    }

    /// **Ce que le panneau montre de ce qui vit ailleurs** (DOM-UI-4).
    pub(crate) fn domaines_connus(&self) -> Connus {
        let vue = self.vue_du_catalogue();
        let ici = &self.store.project.domains;
        let fichier = self.fichier_de_ce_document();
        let mut connus = Connus::default();
        for c in &vue {
            let autres = c.fichiers.iter().filter(|f| f.chemin != fichier);
            let (noeuds, fichiers) = autres.fold((0, 0), |(n, f), x| (n + x.noeuds, f + 1));
            match ici.iter().find(|d| lignee_de(d) == c.lignee) {
                Some(d) => {
                    if homonymes(&vue, &c.definition.name).len() > 1 {
                        connus.homonymes.push(d.id.clone());
                    }
                    if fichiers > 0 {
                        connus.aussi.push((d.id.clone(), noeuds, fichiers));
                    }
                }
                // Un homonyme de ce document ne fait pas une rangée à lui : la sienne pose la question.
                None if self.store.trouver_un_domaine(&c.definition.name).is_none() => {
                    connus.ailleurs.push(Ailleurs {
                        lignee: c.lignee.clone(),
                        definition: c.definition.clone(),
                        noeuds,
                        fichiers,
                    });
                }
                None => {}
            }
        }
        connus
    }

    /// **La question d'un nom déjà pris** (DOM-4, sa règle), ou rien s'il est libre. `sauf` est le
    /// domaine qu'on renomme : changer sa casse n'est pas prendre un nom.
    pub(super) fn question_du_nom(&self, nom: &str, sauf: Option<&str>) -> Option<String> {
        let vue = self.vue_du_catalogue();
        let sa_lignee = sauf.and_then(|id| self.store.domain(id)).map(lignee_de);
        let h = homonymes(&vue, nom);
        let pris = h.iter().find(|c| Some(&c.lignee) != sa_lignee.as_ref())?;
        let question = CoreError::NomDeDomainePris(pris.definition.name.clone());
        Some(format!(
            "{question} ({} nœud(s), {} fichier(s))",
            pris.noeuds(),
            pris.fichiers.len()
        ))
    }

    /// **Un domaine d'un autre fichier, posé sur la sélection** : sa définition entre au catalogue de
    /// ce document, pour que ce fichier reste entier quand il part ailleurs.
    pub(super) fn adopter_le_domaine(&mut self, lignee: &str, weight: f64) {
        let connus = &self.dock_manager.domains.connus;
        let Some(definition) = connus
            .ailleurs
            .iter()
            .find(|a| a.lignee == lignee)
            .map(|a| a.definition.clone())
        else {
            return;
        };
        self.poser_sur_la_selection(
            |s| s.adopter_un_domaine(&definition).map_err(|e| e.to_string()),
            weight,
        );
    }

    /// **« Même sujet »** : ce domaine rejoint son homonyme, en un seul geste.
    pub(super) fn reunir_le_domaine(&mut self, id: &str) {
        let Some(ici) = self.store.domain(id).cloned() else {
            return;
        };
        let vue = self.vue_du_catalogue();
        let lignee = lignee_de(&ici);
        let Some(autre) = homonymes(&vue, &ici.name)
            .into_iter()
            .find(|c| c.lignee != lignee)
            .map(|c| c.definition.clone())
        else {
            return;
        };
        let dit = match self.store.reunir_un_domaine(id, &autre) {
            Ok(n) => format!(
                "« {} » réuni : c'est le même sujet ({n} nœud(s) ici)",
                autre.name
            ),
            Err(err) => err.to_string(),
        };
        self.ui.show_toast(dit);
    }

    /// **Un nom ou une couleur changés dans un autre fichier** : ce document l'apprend ici, par la
    /// date la plus récente (RECENSE-1). C'est un geste comme un autre, annulable. Aucun message : un
    /// domaine est un seul sujet d'un fichier à l'autre, et son nouveau nom est celui qu'on lui a
    /// donné ailleurs — le panneau le montre.
    fn accorder_les_domaines(&mut self) {
        if self.store.in_live_edit() || self.disque.voyage.is_some() {
            return;
        }
        let vue = self.vue_du_catalogue();
        for d in self.store.project.domains.clone() {
            let lignee = lignee_de(&d);
            let Some(c) = vue.iter().find(|c| c.lignee == lignee) else {
                continue;
            };
            // La vue contient ce document : la définition qu'elle rend n'est jamais plus ancienne que
            // la nôtre (RECENSE-1). Si elle dit la même chose, il n'y a rien à apprendre.
            let r = &c.definition;
            if (&r.name, &r.color, &r.icon) == (&d.name, &d.color, &d.icon) {
                continue;
            }
            let patch = DomainPatch::new()
                .with_name(&r.name)
                .with_color(&r.color)
                .with_icon(&r.icon)
                .at(r.defini_a);
            // Refusé quand ce nom est déjà celui d'un autre domaine d'ici : la vue a alors deux sujets
            // sous un nom, et le panneau pose la question (DOM-UI-4). Rien ne se tait.
            let _ = self.store.try_update_domain(&d.id, patch);
        }
    }

    /// **« Où ? »** : chaque place où vit ce domaine, du nœud le plus lourd au plus léger.
    pub(super) fn ouvrir_le_ou(&mut self, lignee: &str) {
        let fichier = self.fichier_de_ce_document();
        let courant = recenser(&fichier, &self.store.project);
        let vue = self.disque.recenseur.etat.vue(Some(&courant));
        let Some(nom) = vue
            .iter()
            .find(|c| c.lignee == lignee)
            .map(|c| c.definition.name.clone())
        else {
            return;
        };
        let lignes = self
            .disque
            .recenseur
            .etat
            .noeuds_du_domaine(Some(&courant), lignee)
            .iter()
            .map(|t| LigneOu {
                chemin: t.chemin.to_string(),
                tableau: t.tableau.to_string(),
                noeud: t.decrit.id.clone(),
                texte: t
                    .decrit
                    .texte
                    .as_deref()
                    .and_then(crate::agents::lecture::premiere_phrase)
                    .map_or_else(|| t.decrit.sorte.nom().to_string(), str::to_string),
                ici: t.chemin == fichier,
            })
            .collect();
        let ui = &mut self.dock_manager.domains;
        ui.ou = Some(Ou { nom, lignes });
        ui.decalage = 0;
        ui.rename = None;
        ui.pending_delete = None;
    }

    /// **Aller à une place** : ici, la caméra vole ; dans un autre fichier, on le quitte comme tout
    /// document (rien ne se perd), et la caméra vole à l'arrivée.
    pub(super) fn aller_a_la_place(&mut self, rang: usize) {
        let ligne = self
            .dock_manager
            .domains
            .ou
            .as_ref()
            .and_then(|o| o.lignes.get(rang))
            .cloned();
        let Some(ligne) = ligne else {
            return;
        };
        if ligne.ici {
            self.voler_vers_la_place(&ligne);
            return;
        }
        let chemin = PathBuf::from(&ligne.chemin);
        self.disque.recenseur.destination = Some(ligne);
        self.laisser_puis(Ensuite::Ouvrir(chemin));
    }

    /// Le document visé est ouvert : la caméra part. Il ne s'est pas ouvert et plus rien ne l'attend
    /// (un refus, un fichier disparu) : la destination s'oublie.
    fn aller_a_la_destination(&mut self) {
        let Some(chemin) = self
            .disque
            .recenseur
            .destination
            .as_ref()
            .map(|d| d.chemin.clone())
        else {
            return;
        };
        if chemin == self.fichier_de_ce_document() {
            if let Some(ligne) = self.disque.recenseur.destination.take() {
                self.voler_vers_la_place(&ligne);
            }
        } else if self.ui.question.is_none() {
            self.disque.recenseur.destination = None;
        }
    }

    fn voler_vers_la_place(&mut self, ligne: &LigneOu) {
        if self.store.project.active_board_id != ligne.tableau {
            self.store.set_active_board_id(&ligne.tableau);
        }
        self.mark_dirty();
        self.voler_vers_le_noeud(&ligne.noeud);
    }

    /// **La molette sur le panneau Domaines** fait défiler ses rangées, d'une par cran. Rend vrai si
    /// elle l'a prise. Elle s'arrête quand la dernière rangée est montrée : la mise en page dit ce qui
    /// reste caché, et c'est la seule loi (DOM-UI-2).
    pub(crate) fn molette_sur_les_domaines(&mut self, dy: f64) -> bool {
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        let ecran = self.screen_frame(largeur, hauteur);
        let (x, y) = (self.mouse_pos.0 as f32, self.mouse_pos.1 as f32);
        let Some((TabId::Domains, cadre)) =
            crate::dock::panneau_sous(&self.dock_manager, ecran, crate::params::Pointer { x, y })
        else {
            return false;
        };
        let ui = &self.dock_manager.domains;
        let layout = crate::dock::domains::layout_domains_panel(cadre, &self.store, ui);
        let premier = match &layout.ou {
            Some((premier, _)) => *premier,
            None => layout.rows.first().map_or(0, |r| r.index),
        };
        // Ce qui est montré commence à `premier` ; ce qui reste caché en dessous, c'est `hidden`
        // moins ce qui est caché au-dessus.
        let en_dessous = layout.hidden.saturating_sub(premier);
        self.dock_manager.domains.decalage = if dy > 0.0 {
            premier.saturating_sub(1)
        } else if en_dessous > 0 {
            premier + 1
        } else {
            premier
        };
        self.mark_dirty();
        true
    }
}

#[cfg(test)]
mod tests;
