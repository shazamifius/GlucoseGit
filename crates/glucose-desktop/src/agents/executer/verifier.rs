//! **`glucose_verifier`** (VERIFIER-1, fiche 184) : le contrôleur du tableau, pour une IA qui ne le
//! voit pas — ses chevauchements, ses appartenances contredites, ses flèches qui traversent
//! ([`glucose_core::store::Store::verifier_le_tableau`]). Chaque nœud en cause dit qui l'a écrit
//! (`qui`, la provenance de MCP-AUTEUR) : ce qu'une IA a posé, elle sait que c'est à elle de le
//! ranger ; ce que l'humain a posé, elle ne le déplace pas sans lui.

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::store::agents::Constat;

impl GlucoseApp {
    pub(super) fn agent_verifier(&mut self, board: &str) -> Result<Json, String> {
        let constat = (self.store.verifier_le_tableau(board)).map_err(|r| r.to_string())?;
        let ecrits = self.store.journal.provenances.get(board);
        // Chaque nœud en cause, une fois, dans l'ordre où le constat le nomme.
        let mut en_cause: Vec<&str> = (constat.chevauchements.iter())
            .flat_map(|(a, b)| [a.as_str(), b.as_str()])
            .chain(constat.appartenances.iter().map(|a| a.id.as_str()))
            .chain(constat.traversees.iter().map(|(f, _)| f.as_str()))
            .collect();
        let mut vus = std::collections::HashSet::new();
        en_cause.retain(|id| vus.insert(*id));
        let qui = (en_cause.iter())
            .filter_map(|id| {
                let p = ecrits?.get(*id)?;
                let mains = p.par.iter().map(|m| Json::from(m.as_str())).collect();
                Some((id.to_string(), Json::Liste(mains)))
            })
            .collect();
        let mut dit = constat_en_json(&constat);
        if let Json::Objet(champs) = &mut dit {
            champs.push(("qui".to_string(), Json::Objet(qui)));
        }
        Ok(dit)
    }
}

fn constat_en_json(c: &Constat) -> Json {
    let id = |s: &Option<String>| s.as_deref().map_or(Json::Nul, Json::from);
    let chevauchements = (c.chevauchements.iter())
        .map(|(a, b)| Json::objet([("a", Json::from(a.as_str())), ("b", Json::from(b.as_str()))]))
        .collect();
    let appartenances = (c.appartenances.iter())
        .map(|a| {
            Json::objet([
                ("id", Json::from(a.id.as_str())),
                ("a_l_oeil", id(&a.a_l_oeil)),
                ("pour_l_ia", id(&a.pour_l_ia)),
            ])
        })
        .collect();
    let traversees = (c.traversees.iter())
        .map(|(f, ids)| {
            let ids = ids.iter().map(|i| Json::from(i.as_str())).collect();
            Json::objet([
                ("fleche", Json::from(f.as_str())),
                ("traverse", Json::Liste(ids)),
            ])
        })
        .collect();
    Json::objet([
        ("propre", Json::Bool(c.est_propre())),
        ("chevauchements", Json::Liste(chevauchements)),
        ("appartenances", Json::Liste(appartenances)),
        ("traversees", Json::Liste(traversees)),
    ])
}
