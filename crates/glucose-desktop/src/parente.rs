//! **PARENTE-2 — ce que l'IA dit des textes du tableau** (fiche 106) : un fil lit le sens de chaque
//! texte avec la petite IA du noyau ([`glucose_core::sens`]), en tire les familles et la sûreté
//! ([`glucose_core::sens::familles`]), et rend à chaque texte son **avis** — la teinte de sa
//! famille, sur un cercle dont l'origine est le document lui-même, et la sûreté de l'IA. Le rendu
//! en fait des couleurs ([`crate::renderer::sens`]).
//!
//! **PARENTE-4** (fiche 107) : chaque flèche entre deux textes, ou entre un texte et une membrane
//! (ce qu'elle porte, et son nom), reçoit aussi un avis — la sûreté que ses deux bouts parlent de
//! la même chose ([`liens`]). Le rendu en fait la place d'un lien quand ils abondent
//! ([`crate::renderer::arrow::presence::part`]).
//!
//! # Quand
//!
//! Seulement quand les **textes** du tableau, ou les liens qui les relient, changent : un texte
//! écrit, ajouté, retiré, une flèche tendue ou défaite, un autre tableau. La version du document n'avance qu'à un vrai geste, et une empreinte de ses textes dit
//! si ce geste les a touchés — déplacer un nœud ne relance rien. Le rendu n'attend jamais : l'avis
//! d'avant reste peint (au lancement, le gris) jusqu'à ce que le fil rende le neuf, et une seule
//! image le montre.
//!
//! # Combien
//!
//! Le sens d'un texte coûte une lecture de table par morceau de mot, et se garde tant que son texte
//! ne change pas. La parenté compare chaque texte à tous les autres : `n²` ressemblances, six octets
//! chacune au plus fort du calcul. Elle suit la règle de la mémoire de Glucose (ADAPT-1) — **elle
//! ne prend jamais plus que ce qu'elle laisse** : sur un document trop grand pour ce que la machine
//! a de libre, elle ne se calcule pas, les halos restent gris, et la chronique le dit.
//!
//! **Sans fil** (une épreuve, un banc), le calcul se fait sur place, avant l'image : déterministe.

use crate::plateforme::Reveil;
use glucose_core::sens::{self, familles};
use glucose_core::store::Store;
use liens::Liens;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::mpsc::{self, Receiver, Sender};

/// **L'avis de l'IA sur un nœud** — un texte : sa famille (son numéro et sa teinte, en radians ;
/// rien s'il est seul), la sûreté qu'il a un parent, de 0 à 1, et ce parent — le texte qui lui
/// ressemble le plus, d'où la sûreté se tire (B3, fiche 110) ; une flèche entre deux textes :
/// aucune famille, et la sûreté que ses deux textes parlent de la même chose (PARENTE-4).
#[derive(Debug, Clone, PartialEq)]
pub struct AvisDUnNoeud {
    pub famille: Option<usize>,
    pub teinte: Option<f64>,
    pub surete: f32,
    pub parent: Option<String>,
}

/// L'avis de l'IA sur les textes d'un tableau et leurs liens, par identifiant. Un texte sans mot
/// n'en a pas, ni une flèche dont un bout n'est ni un texte lu, ni une membrane (PARENTE-4).
pub type Avis = HashMap<String, AvisDUnNoeud>;

/// Les textes d'un tableau, dans l'ordre du document : identifiant et texte.
type Textes = Vec<(String, String)>;

/// Ce qu'on demande au fil : le tableau, ses textes et ses liens.
type Demande = (String, Textes, Liens);

/// Ce qu'il rend : le tableau, et l'avis sur ses textes.
pub type AvisDuTableau = (String, Avis);

struct Fil {
    demandes: Sender<Demande>,
    reponses: Receiver<AvisDuTableau>,
}

/// **La parenté des textes** : ce qu'on a demandé, et le fil qui la calcule.
#[derive(Default)]
pub struct Parente {
    fil: Option<Fil>,
    /// Le tableau et la version du document au dernier regard.
    vu: Option<(String, u64)>,
    /// L'empreinte des textes de la dernière demande.
    empreinte: Option<u64>,
    /// Sans fil, le calcul se fait ici.
    sur_place: Calcul,
}

impl Parente {
    /// **Démarre le fil**, au vrai lancement : `reveil` réveille la boucle quand un avis est prêt.
    pub fn demarrer(&mut self, reveil: Reveil) {
        let (demandes, recues) = mpsc::channel::<Demande>();
        let (envoi, reponses) = mpsc::channel();
        let lance = std::thread::Builder::new()
            .name("parente".into())
            .spawn(move || {
                let mut calcul = Calcul::default();
                while let Ok(mut demande) = recues.recv() {
                    // Seule la dernière demande compte : celles d'avant sont déjà périmées.
                    while let Ok(plus_neuve) = recues.try_recv() {
                        demande = plus_neuve;
                    }
                    let (tableau, textes, liens) = demande;
                    if envoi
                        .send((tableau, calcul.calculer(textes, liens)))
                        .is_err()
                    {
                        return;
                    }
                    reveil();
                }
            });
        if lance.is_ok() {
            self.fil = Some(Fil { demandes, reponses });
        }
    }

    /// **Ce qui est neuf**, avant chaque image : l'avis que le fil vient de rendre — ou, sans fil,
    /// celui qu'on vient de calculer —, rien tant que les textes ne changent pas.
    pub fn suivre(&mut self, store: &Store) -> Option<AvisDuTableau> {
        let mut neuf = self.fil.as_ref().and_then(|f| f.reponses.try_iter().last());
        if let Some((tableau, textes, liens)) = self.textes_changes(store) {
            match &self.fil {
                Some(fil) => {
                    // Un fil parti ne rend plus rien : l'avis d'avant reste peint.
                    let _ = fil.demandes.send((tableau, textes, liens));
                }
                None => neuf = Some((tableau, self.sur_place.calculer(textes, liens))),
            }
        }
        neuf
    }

    /// Le tableau actif, ses textes et ses liens, s'ils ont changé depuis la dernière demande —
    /// ce qu'une membrane touchée par une flèche porte compris.
    fn textes_changes(&mut self, store: &Store) -> Option<Demande> {
        let tableau = store.active_board()?;
        let deja_vu = self
            .vu
            .as_ref()
            .is_some_and(|(id, v)| *id == tableau.id && *v == store.version);
        if deja_vu {
            return None;
        }
        self.vu = Some((tableau.id.clone(), store.version));
        let mut h = DefaultHasher::new();
        tableau.id.hash(&mut h);
        for texte in sens::textes_du_tableau(tableau) {
            texte.hash(&mut h);
        }
        let liens = Liens::du_tableau(tableau, &store.journal.des_agents);
        liens.hash(&mut h);
        let empreinte = Some(h.finish());
        if self.empreinte == empreinte {
            return None;
        }
        self.empreinte = empreinte;
        let textes = sens::textes_du_tableau(tableau);
        let textes = textes.map(|(i, t)| (i.into(), t.into())).collect();
        Some((tableau.id.clone(), textes, liens))
    }
}

/// **Le calcul**, et ce qu'il garde d'une demande à l'autre.
#[derive(Default)]
struct Calcul {
    /// Le sens de chaque texte déjà lu, par son texte : il ne se relit pas tant qu'il ne change
    /// pas. Ce qu'une demande ne porte plus s'oublie.
    sens: HashMap<String, Option<Vec<f32>>>,
    /// La mémoire a-t-elle déjà manqué ? La chronique ne le dit qu'une fois.
    a_manque: bool,
}

impl Calcul {
    fn calculer(&mut self, textes: Textes, liens: Liens) -> Avis {
        let Some(modele) = sens::embarque() else {
            return Avis::new();
        };
        let mut gardes = HashMap::with_capacity(textes.len());
        let mut lire = |texte: &str| {
            let lu = gardes
                .get(texte)
                .cloned()
                .or_else(|| self.sens.remove(texte))
                .unwrap_or_else(|| modele.plonger(texte));
            gardes.insert(texte.to_string(), lu.clone());
            lu
        };
        let (mut ids, mut vecteurs) = (Vec::new(), Vec::new());
        for (id, texte) in textes {
            if let Some(v) = lire(&texte) {
                ids.push(id);
                vecteurs.push(v);
            }
        }
        for nom in liens.noms() {
            lire(nom);
        }
        self.sens = gardes;
        if !self.la_memoire_suffit(vecteurs.len()) {
            return Avis::new();
        }
        let mut parente = familles::parente(&vecteurs);
        parente.orienter(0.0);
        let fleches = liens.juger((&ids, &vecteurs), |nom| {
            self.sens.get(nom).cloned().flatten()
        });
        let parents: Vec<Option<String>> = (parente.parent.iter())
            .map(|p| p.map(|j| ids[j].clone()))
            .collect();
        let textes = ids.into_iter().zip(parents).enumerate();
        let textes = textes.map(|(k, (id, parent))| {
            let famille = parente.famille[k];
            let avis = AvisDUnNoeud {
                famille,
                teinte: famille.map(|f| parente.teintes[f]),
                surete: parente.surete[k],
                parent,
            };
            (id, avis)
        });
        let fleches = fleches.into_iter().map(|(id, surete)| {
            let avis = AvisDUnNoeud {
                famille: None,
                teinte: None,
                surete,
                parent: None,
            };
            (id, avis)
        });
        textes.chain(fleches).collect()
    }

    /// **La règle de la mémoire** : au plus fort du calcul, la matrice des ressemblances (`n²`
    /// flottants) et la liste de leurs paires (`n²/2`) — la parenté ne prend jamais plus que ce
    /// qu'elle laisse ([`crate::memoire::Memoire::part_pour_un_cache`]). Une machine qui ne dit pas
    /// sa mémoire n'a pas de borne, comme pour le magasin des images : aucune n'est privée de la
    /// parenté sur un doute.
    fn la_memoire_suffit(&mut self, n: usize) -> bool {
        let besoin = besoin(n);
        let libre = crate::memoire::Memoire::du_systeme().map(|m| m.part_pour_un_cache());
        if suffit(besoin, libre) {
            return true;
        }
        if !self.a_manque {
            crate::dire!(
                "[Glucose] la parenté de {n} textes demanderait {} Mo, plus que ce que la machine \
                 laisse : leurs halos restent gris",
                besoin >> 20
            );
        }
        self.a_manque = true;
        false
    }
}

/// Ce que la parenté de `n` textes occupe au plus fort de son calcul, en octets : `n²` flottants
/// pour la matrice, `n²/2` pour la liste des paires.
fn besoin(n: usize) -> u64 {
    6 * (n as u64) * (n as u64)
}

/// La règle : jamais plus que la part d'un cache ; sans borne connue, pas de borne.
fn suffit(besoin: u64, libre: Option<u64>) -> bool {
    libre.is_none_or(|l| besoin <= l)
}

mod liens;
pub(crate) mod rectifications;

#[cfg(test)]
mod tests;
