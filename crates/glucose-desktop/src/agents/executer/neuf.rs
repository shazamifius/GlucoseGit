//! **Quoi de neuf** (MCP-NEUF, fiche 144 ; sa liste du 24/11, `relais-rep-mcp` point 2 : *« ce qui
//! a changé depuis ma dernière lecture, qui l'a fait (toi, ton autre appareil, une IA), où et
//! quand »*). La Time Machine rend des gestes ; une IA qui revient veut des **nœuds** : chacun une
//! fois, posé, changé ou retiré, par qui, quand, où, et sa première phrase — de quoi choisir ce
//! qu'elle relit, sans relire le tableau (ses jetons, sa priorité du 24/11).
//!
//! Depuis quand : le rang qu'elle donne (`depuis`, le `jusqu_a` de sa lecture d'avant) ; sans lui,
//! **depuis son dernier geste** — ce que les autres ont fait depuis qu'elle a agi ; une IA qui n'a
//! jamais agi lit les derniers gestes, comme l'histoire.

use super::temps::{auteur_en_json, relire, DERNIERS};
use super::Args;
use crate::agents::json::Json;
use crate::agents::lecture::premiere_phrase;
use crate::app::GlucoseApp;
use glucose_core::persist::histoire::{Auteur, Ouvert};
use glucose_core::store::agents::Decrit;
use glucose_core::store::journal::Effet;
use std::collections::HashMap;

/// Un nœud que les gestes lus ont touché.
struct Neuf {
    board: String,
    id: String,
    sorte: &'static str,
    /// Il existait avant le premier de ces gestes ; il existe après le dernier.
    avant: bool,
    apres: bool,
    par: Vec<Json>,
    dernier: usize,
    /// Son texte avant le premier de ces gestes, et après le dernier.
    textes: [Option<String>; 2],
}

impl GlucoseApp {
    /// **Ce qui a changé** depuis le rang `depuis` (ou le dernier geste de cet agent) : les nœuds,
    /// les plus récents d'abord, puis le reste en mots, et `jusqu_a`, d'où repartir la fois suivante.
    pub(super) fn agent_quoi_de_neuf(&mut self, a: &Args<'_>) -> Result<Json, String> {
        let o = self
            .lire_l_histoire()
            .ok_or("ce document n'a pas encore d'histoire : aucun geste n'y a été fait")?;
        let chemin = (self.disque.ecriture.as_ref())
            .map(|e| e.chemin.clone())
            .ok_or("ce document n'a pas de fichier")?;
        let mut fichier = std::fs::File::open(&chemin).map_err(|e| e.to_string())?;
        let depuis = match a.nombre("depuis") {
            Some(n) if n >= 0.0 && n.fract() == 0.0 => (n as usize).min(o.gestes.len()),
            Some(n) => return Err(format!("« {n} » n'est pas le rang d'un geste")),
            None => self.apres_mon_dernier_geste(&o),
        };
        let (mut neufs, mut autres) = (Vec::<Neuf>::new(), Vec::<(String, usize)>::new());
        for rang in depuis..o.gestes.len() {
            let repere = o.gestes[rang];
            let Some(geste) = relire(&mut fichier, repere) else {
                continue;
            };
            let qui = auteur_en_json(&o, repere.auteur);
            for effet in geste.transaction.effets() {
                match effet {
                    Effet::Noeud(t) => match neufs.iter_mut().find(|n| n.id == t.id) {
                        Some(n) => {
                            n.apres = t.apres;
                            n.dernier = rang;
                            n.textes[1] = t.textes[1].clone();
                            if !n.par.contains(&qui) {
                                n.par.push(qui.clone());
                            }
                        }
                        None => neufs.push(Neuf {
                            board: t.board,
                            id: t.id,
                            sorte: t.sorte,
                            avant: t.avant,
                            apres: t.apres,
                            par: vec![qui.clone()],
                            dernier: rang,
                            textes: t.textes,
                        }),
                    },
                    Effet::Autre(mots) => match autres.iter_mut().find(|(m, _)| *m == mots) {
                        Some((_, fois)) => *fois += 1,
                        None => autres.push((mots, 1)),
                    },
                }
            }
        }
        // Venu et reparti entre deux lectures : rien n'a changé pour qui revient.
        neufs.retain(|n| n.avant || n.apres);
        neufs.sort_by_key(|n| std::cmp::Reverse(n.dernier));
        let noeuds = self.neufs_en_json(&o, neufs);
        let autres = (autres.into_iter())
            .map(|(m, fois)| {
                Json::texte(if fois > 1 {
                    format!("{m} (×{fois})")
                } else {
                    m
                })
            })
            .collect();
        let mut champs = vec![
            ("depuis", Json::Nombre(depuis as f64)),
            ("jusqu_a", Json::Nombre(o.gestes.len() as f64)),
            ("noeuds", Json::Liste(noeuds)),
            ("autres", Json::Liste(autres)),
        ];
        // Sur quoi il se concentre (REGARD-1, fiche 177), seulement s'il a choisi quelque chose.
        let regards = self.regards_en_json();
        if !regards.is_empty() {
            champs.push(("regards", Json::Liste(regards)));
        }
        Ok(Json::objet(champs))
    }

    /// Le rang qui suit le dernier geste de cet agent ; sans geste de lui, les derniers gestes.
    fn apres_mon_dernier_geste(&self, o: &Ouvert) -> usize {
        let moi = Auteur::d_une_main(&self.store.journal.main).map(|a| a.id);
        (o.gestes.iter())
            .rposition(|r| Some(r.auteur) == moi)
            .map_or(o.gestes.len().saturating_sub(DERNIERS), |k| k + 1)
    }

    /// Chaque nœud en JSON : ce qui lui est arrivé, par qui, quand, et — s'il est encore là — sa
    /// membrane et sa première phrase, lues sur son tableau tel qu'il est.
    fn neufs_en_json(&self, o: &Ouvert, neufs: Vec<Neuf>) -> Vec<Json> {
        let mut tableaux: HashMap<String, Vec<Decrit>> = HashMap::new();
        let mut rendus = Vec::with_capacity(neufs.len());
        for n in neufs {
            let fait = match (n.avant, n.apres) {
                (false, _) => "pose",
                (true, false) => "retire",
                (true, true) => "change",
            };
            let quand = crate::plateforme::heure::heure_locale(o.gestes[n.dernier].instant)
                .map_or_else(String::new, |h| h.to_string());
            let mut champs = vec![
                ("id".to_string(), Json::texte(n.id.clone())),
                ("sorte".to_string(), Json::texte(n.sorte)),
                ("fait".to_string(), Json::texte(fait)),
                ("par".to_string(), Json::Liste(n.par)),
                ("quand".to_string(), Json::texte(quand)),
                ("tableau".to_string(), Json::texte(n.board.clone())),
            ];
            let decrits = (tableaux.entry(n.board.clone())).or_insert_with(|| {
                (self.store.decrire_pour_un_agent(&n.board)).unwrap_or_default()
            });
            if let Some(d) = decrits.iter().find(|d| d.id == n.id).filter(|_| n.apres) {
                if let Some(m) = &d.membrane {
                    champs.push(("membrane".to_string(), Json::texte(m.clone())));
                }
                if let Some(p) = d.texte.as_deref().and_then(premiere_phrase) {
                    champs.push(("debut".to_string(), Json::texte(p)));
                }
            }
            if let ([Some(avant), Some(apres)], true) = (&n.textes, n.avant && n.apres) {
                if avant != apres {
                    champs.push(("changements".to_string(), changements(avant, apres)));
                }
            }
            rendus.push(Json::Objet(champs));
        }
        rendus
    }
}

/// **Ce qui a changé dans son texte** (MCP-CHANGE, fiche 180 ; `relais-rep-mcp`, point 6) :
/// chaque changement à sa ligne, ce qu'elle disait et ce qu'elle dit — sa réponse sous une
/// question se lit seule. Plus long que le texte, il se relit entier.
fn changements(avant: &str, apres: &str) -> Json {
    let Some(changes) = crate::agents::difference::changements(avant, apres) else {
        return Json::texte("récrit : relis-le entier (glucose_lire, ids)");
    };
    let en_json = |c: crate::agents::difference::Changement| {
        let mut champs = vec![("ligne".to_string(), Json::Nombre(c.ligne as f64))];
        champs.extend(c.avant.map(|t| ("avant".to_string(), Json::texte(t))));
        champs.extend(c.apres.map(|t| ("apres".to_string(), Json::texte(t))));
        Json::Objet(champs)
    };
    Json::Liste(changes.into_iter().map(en_json).collect())
}
