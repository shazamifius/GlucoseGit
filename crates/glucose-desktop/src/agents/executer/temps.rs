//! **La Time Machine, pour un agent** (MAIN-1, NOTE-1 — fiche 108) : relire l'histoire du document
//! — chaque geste, qui l'a fait, ce qu'il a changé, ce qu'on en a écrit — et attacher un écrit à
//! un geste.
//!
//! Sa parole du 12/11/2026 : *« on UTILISE la Time Machine : c'est la vraie invention de Glucose
//! […] tout le système IA se basera là-dessus »*, et *« pourquoi pas avoir parfois plus d'info que
//! juste un simple geste, mais vraiment des écrits de ce qu'une IA comprend de ce geste, et
//! pourquoi c'est important »*. L'écrit ne touche pas le geste : il s'y attache, comme une note de
//! git sur un commit.

use super::Args;
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::persist::histoire::ouvrir::Repere;
use glucose_core::persist::histoire::{self, Ouvert};
use std::io::{Read, Seek, SeekFrom};

/// Combien de gestes l'histoire rend par défaut : les plus récents.
const DERNIERS: usize = 50;

impl GlucoseApp {
    /// **Attache un écrit à un geste** : le dernier que cet agent a fait, ou celui de rang
    /// `geste`. Les gestes en attente s'écrivent d'abord : l'écrit les suit dans l'histoire.
    pub(super) fn agent_noter(&mut self, a: &Args<'_>) -> Result<Json, String> {
        let texte = a.requis("texte")?;
        let geste = match a.nombre("geste") {
            Some(g) if g >= 0.0 && g.fract() == 0.0 => Some(g as usize),
            Some(g) => return Err(format!("« {g} » n'est pas le rang d'un geste")),
            None => None,
        };
        self.consigner();
        let main = self.store.journal.main.clone();
        let ecriture = self
            .disque
            .ecriture
            .as_mut()
            .ok_or("ce document n'a pas encore d'histoire : aucun geste n'y a été fait")?;
        let rang = ecriture.noter(&main, (geste, texte), crate::persist::now_millis())?;
        Ok(Json::objet([("geste", Json::Nombre(rang as f64))]))
    }

    /// **Relit l'histoire du document** : ses gestes les plus récents (rang, date, auteur, ce
    /// qu'ils ont fait, les écrits qui s'y attachent), ses jalons, et combien elle en compte.
    pub(super) fn agent_histoire(&mut self, a: &Args<'_>) -> Result<Json, String> {
        let nombre = a.nombre("nombre").map_or(DERNIERS, |n| n.max(1.0) as usize);
        let o = self
            .lire_l_histoire()
            .ok_or("ce document n'a pas encore d'histoire : aucun geste n'y a été fait")?;
        let chemin = self
            .disque
            .ecriture
            .as_ref()
            .map(|e| e.chemin.clone())
            .ok_or("ce document n'a pas de fichier")?;
        let mut fichier = std::fs::File::open(&chemin).map_err(|e| e.to_string())?;
        let depuis = o.gestes.len().saturating_sub(nombre);
        let gestes = (depuis..o.gestes.len())
            .map(|rang| geste_en_json(&o, rang, &mut fichier))
            .collect();
        let jalons = o
            .jalons
            .iter()
            .map(|(apres, j)| {
                Json::objet([
                    ("apres", Json::Nombre(*apres as f64)),
                    ("libelle", Json::texte(j.libelle.clone())),
                ])
            })
            .collect();
        Ok(Json::objet([
            ("total", Json::Nombre(o.gestes.len() as f64)),
            ("gestes", Json::Liste(gestes)),
            ("jalons", Json::Liste(jalons)),
        ]))
    }
}

/// **Qui est cet auteur**, en mots : « humain », ou « agent » et son nom.
fn auteur_en_json(o: &Ouvert, id: u64) -> Json {
    match o.auteurs.get(&id).filter(|a| a.agent) {
        Some(a) => Json::texte(format!("agent {}", a.nom)),
        None => Json::texte("humain"),
    }
}

/// Un geste de l'histoire : son rang, sa date, son auteur, ce qu'il a fait — relu dans le
/// fichier —, et les écrits qui s'y attachent.
fn geste_en_json(o: &Ouvert, rang: usize, fichier: &mut std::fs::File) -> Json {
    let repere: Repere = o.gestes[rang];
    let date = crate::plateforme::heure::heure_locale(repere.instant)
        .map_or_else(String::new, |h| h.to_string());
    let fait = relire(fichier, repere)
        .map(|g| g.transaction.en_mots())
        .unwrap_or_default();
    let ecrits = o
        .notes
        .iter()
        .filter(|n| n.geste == rang as u64)
        .map(|n| {
            Json::objet([
                ("auteur", auteur_en_json(o, n.auteur)),
                ("texte", Json::texte(n.texte.clone())),
            ])
        })
        .collect();
    Json::objet([
        ("rang", Json::Nombre(rang as f64)),
        ("date", Json::texte(date)),
        ("auteur", auteur_en_json(o, repere.auteur)),
        (
            "fait",
            Json::Liste(fait.into_iter().map(Json::texte).collect()),
        ),
        ("ecrits", Json::Liste(ecrits)),
    ])
}

/// Le geste repéré, relu dans le fichier.
fn relire(fichier: &mut std::fs::File, repere: Repere) -> Option<histoire::Geste> {
    let mut contenu = vec![0u8; repere.tranche.longueur as usize];
    fichier.seek(SeekFrom::Start(repere.tranche.offset)).ok()?;
    fichier.read_exact(&mut contenu).ok()?;
    histoire::lire_geste(&contenu).ok()
}
