//! **Lire par niveaux, et renommer** (`glucose_lire`, `glucose_renommer`) — sortis d'`executer.rs`
//! à la fiche 133, quand les dossiers y sont entrés.
//!
//! # MCP-DOSSIER — ce qu'un humain voit d'un dossier, une IA le lit (fiche 133)
//!
//! Sa note du 23/11 : *« lorsque tu auras réparé le MCP, on pourra renommer archive »* ; celle
//! d'une IA qui s'en servait, le 22/11 : *« les dossiers sont invisibles pour une IA »*, *« aucun
//! outil ne permet de renommer un dossier ou un onglet »*, *« quand tu entres dans le dossier, la
//! réponse dit encore Canvas Principal ; elle renvoie une liste vide sans aucune erreur »*.
//! Désormais : un dossier se lit comme un nœud (son nom, sa boîte, ce qu'il **ouvre**, combien il
//! **contient** — le chiffre de son badge) ; la lecture dit **où** est le tableau lu (le fil que
//! l'humain voit en haut de l'écran) ; un identifiant demandé qui n'existe pas est **dit
//! introuvable**, jamais tu ; et un dossier ou un onglet se renomme.

use super::decrit::{boite_en_json, decrit_en_json, parente_en_json};
use super::Args;
use crate::agents::json::Json;
use crate::agents::lecture::{choisir, Demande, Niveau};
use crate::app::GlucoseApp;
use glucose_core::store::agents::{Decrit, Sorte};

/// Ce qui sépare deux étapes du chemin d'un tableau : le fil de l'écran.
const FIL: &str = " › ";

impl GlucoseApp {
    /// **Lire par niveaux** (`lecture`) : le sommaire par défaut, le texte entier des nœuds
    /// choisis, ou la phrase qui contient ce qu'on cherche.
    pub(super) fn agent_lire(&self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let decrits = self
            .store
            .decrire_pour_un_agent(board)
            .map_err(|r| r.to_string())?;
        let ids = ids_demandes(a)?;
        let domaine = self.domaine_a_lire(a)?;
        let demande = Demande {
            domaine: domaine.as_deref(),
            dans: a.texte("dans"),
            ids,
            cherche: a.texte("cherche"),
            niveau: Niveau::lire(a.texte("niveau"))?,
        };
        let noeuds = self.noeuds_en_json(board, &decrits, &demande);
        let mut champs = vec![
            // Pour une IA du nuage, aucun chemin, ni ici ni là (C1, fiches 116 et 122 : deux
            // essais réels l'ont montré, chacun pour un champ).
            ("document", Json::from(self.document_vu())),
            ("fichier", Json::from(self.fichier_vu())),
            ("tableau", Json::from(board)),
            (
                "ou",
                Json::from(self.store.chemin_du_tableau(board).join(FIL)),
            ),
            ("visible", boite_en_json(self.ce_qu_il_voit())),
            ("selection", self.ce_qu_il_designe(board)),
            ("noeuds", Json::Liste(noeuds)),
        ];
        let absents = introuvables(&decrits, demande.ids.as_deref().unwrap_or_default());
        if !absents.is_empty() {
            champs.push(("introuvables", Json::Liste(absents)));
        }
        champs.push(("tableaux", self.tableaux_en_json()));
        champs.push(("domaines", self.catalogue_pour_un_agent()));
        Ok(Json::objet(champs))
    }

    /// Les nœuds choisis, chacun comme l'agent le lit — et ce que l'humain voit sans lire : la
    /// parenté d'un texte, ce qu'une membrane porte (au plan), le badge d'un dossier.
    fn noeuds_en_json(&self, board: &str, decrits: &[Decrit], demande: &Demande<'_>) -> Vec<Json> {
        let plan = demande.niveau == Niveau::Plan;
        let avis = self.renderer.sens_des_textes.avis_du_tableau(board);
        choisir(decrits, demande)
            .into_iter()
            .map(|(d, rendu)| {
                let mut j = decrit_en_json(d, &rendu);
                parente_en_json(avis.and_then(|a| a.get(&d.id)), &mut j);
                // Le plan dit ce que chaque membrane porte, sans le porter (fiche 90) ; un dossier
                // dit toujours le chiffre de son badge (MCP-DOSSIER-1).
                let contient = match (d.sorte, d.ouvre.as_deref()) {
                    (Sorte::Membrane, _) if plan => Some(
                        decrits
                            .iter()
                            .filter(|x| x.membrane.as_deref() == Some(d.id.as_str()))
                            .count(),
                    ),
                    (Sorte::Dossier, Some(ouvre)) => Some(self.store.folder_child_count(ouvre)),
                    _ => None,
                };
                if let (Some(n), Json::Objet(champs)) = (contient, &mut j) {
                    champs.push(("contient".to_string(), Json::Nombre(n as f64)));
                }
                j
            })
            .collect()
    }

    /// Les onglets du document, et celui que l'humain regarde.
    fn tableaux_en_json(&self) -> Json {
        let tableaux = self.store.onglets().map(|(id, nom, actif)| {
            Json::objet([
                ("id", Json::from(id)),
                ("nom", Json::from(nom)),
                ("regarde", Json::Bool(actif)),
            ])
        });
        Json::Liste(tableaux.collect())
    }

    /// **MCP-DOSSIER-2 — renommer un dossier ou un onglet** : un geste, que `Ctrl+Z` défait. Un
    /// dossier et le tableau qu'il ouvre portent le même nom (RENOMMER-2).
    pub(super) fn agent_renommer(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let id = a.requis("id")?;
        let nom = a.requis("nom")?;
        let sorte = self
            .store
            .geste_d_agent(|s| s.renommer_pour_un_agent(board, id, nom))
            .map_err(|r| r.to_string())?;
        Ok(Json::objet([
            ("id", Json::from(id)),
            ("sorte", Json::from(sorte)),
            ("nom", Json::from(nom.trim())),
        ]))
    }
}

/// Les identifiants que l'agent demande à lire en entier.
fn ids_demandes<'a>(a: &Args<'a>) -> Result<Option<Vec<&'a str>>, String> {
    let Some(l) = a.0.get("ids") else {
        return Ok(None);
    };
    let liste = l
        .as_liste()
        .ok_or("l'argument « ids » est une liste d'identifiants")?;
    Ok(Some(liste.iter().filter_map(Json::as_str).collect()))
}

/// **Ce qu'on a demandé et qui n'est pas sur ce tableau** : dit, jamais tu — une liste qui se
/// tait laisse croire qu'un nœud n'existe plus, quand il est ailleurs (une IA, le 22/11).
fn introuvables(decrits: &[Decrit], ids: &[&str]) -> Vec<Json> {
    ids.iter()
        .filter(|id| !decrits.iter().any(|d| d.id == **id))
        .map(|id| Json::from(*id))
        .collect()
}
