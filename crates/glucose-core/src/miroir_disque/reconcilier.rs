//! **DISQUE-2 — réconcilier un niveau** (fiche 164 § 4 et § 9) : ce que le tableau d'un miroir
//! montre, ce que le disque vient de rendre, et ce que le miroir avait vu la fois d'avant donnent
//! ce qui change — des gestes ordinaires, que le magasin applique en un seul.
//!
//! * **Un fichier nouveau** entre ; **un fichier changé** (sa taille ou sa date) se relit ;
//!   **un fichier renommé** garde son nœud, ses flèches et ses domaines — reconnu, sur tous les
//!   systèmes, par une paire unique (disparu, apparu) de même taille, même date, même extension ;
//!   **un fichier disparu** emporte son nœud, sauf si ce nœud porte quelque chose de sa main.
//! * **Un nœud qu'il retire ne revient pas** : un fichier que le miroir avait vu et qu'aucun nœud ne
//!   montre plus a été refusé — tant que le fichier ne change pas. Un rendu refait sous le même nom
//!   est une autre image, et revient.
//! * **Un seul sens** (la réponse proposée dans `relais-q-dossier-disque`) : un texte qu'il a récrit
//!   n'est plus celui que le fichier donnait — le miroir le reconnaît à l'empreinte de ce qu'il y
//!   avait posé, et ne le touche plus jamais.
//!
//! **Ce qu'un miroir a vu** ([`Vus`]) n'est pas dans le document : le chemin d'un fichier n'a de sens
//! que sur cette machine. L'application le garde à côté ; perdu (une autre machine, un « Enregistrer
//! sous »), les nœuds qu'il avait retirés reviennent une fois — le défaut se voit, il ne cache rien.

use super::Entree;
use crate::types::{Annotation, Board, BoardImage, CanvasFolder};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// **Ce que le miroir a vu d'un fichier**, à sa dernière lecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vu {
    pub taille: u64,
    pub date: i64,
    /// Pour un texte, l'empreinte du Markdown que le miroir en a fait ([`empreinte`]) : un nœud qui
    /// ne la porte plus a été récrit de sa main.
    pub empreinte: Option<u64>,
}

/// Ce qu'un miroir a vu, par chemin.
pub type Vus = HashMap<String, Vu>;

/// **Un nœud du tableau**, par sa sorte et son identifiant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Noeud {
    Photo(String),
    /// Un texte ou une tuile.
    Note(String),
    Dossier(String),
}

/// **Un nœud qui suit son fichier** : renommé, ou changé.
#[derive(Debug, Clone, PartialEq)]
pub struct Suivi {
    pub noeud: Noeud,
    pub entree: Entree,
    /// Un texte que sa main n'a pas récrit : son contenu se relit.
    pub relire: bool,
}

/// **Ce qu'une lecture change à un miroir.**
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Plan {
    /// Les fichiers qui entrent, dans l'ordre du niveau.
    pub nouveaux: Vec<Entree>,
    pub suivis: Vec<Suivi>,
    /// Les nœuds dont le fichier a disparu, et qui ne portent rien de sa main.
    pub partis: Vec<Noeud>,
    /// Ce que le miroir aura vu, pour la lecture suivante.
    pub vus: Vus,
}

impl Plan {
    /// Rien ne change au tableau (ce que le miroir a vu peut, lui, avoir changé).
    pub fn est_vide(&self) -> bool {
        self.nouveaux.is_empty() && self.suivis.is_empty() && self.partis.is_empty()
    }
}

/// **L'empreinte d'un texte** : les huit premiers octets de son SHA-256 — stable d'une version de
/// Glucose à l'autre, ce que le hachage de la bibliothèque standard ne promet pas.
pub fn empreinte(texte: &str) -> u64 {
    let h = crate::hash::sha256(texte.as_bytes());
    u64::from_le_bytes([h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]])
}

/// **Ce fichier est-il de ce niveau ?** Son dossier est la racine du miroir.
pub fn vient_de(chemin: &str, racine: &str) -> bool {
    Path::new(chemin).parent() == Some(Path::new(racine))
}

/// Un nœud né d'un fichier de ce niveau, tel que le tableau le montre.
enum Ne<'a> {
    Photo(&'a BoardImage),
    Note(&'a Annotation),
    Dossier(&'a CanvasFolder),
}

impl Ne<'_> {
    fn noeud(&self) -> Noeud {
        match self {
            Ne::Photo(i) => Noeud::Photo(i.id.clone()),
            Ne::Note(a) => Noeud::Note(a.id().to_string()),
            Ne::Dossier(f) => Noeud::Dossier(f.id.clone()),
        }
    }

    fn id(&self) -> &str {
        match self {
            Ne::Photo(i) => &i.id,
            Ne::Note(a) => a.id(),
            Ne::Dossier(f) => &f.id,
        }
    }
}

/// **Les nœuds nés du disque**, par chemin : ceux dont le fichier est de ce niveau, ou que le miroir
/// avait vus (un dossier renommé garde le temps d'une lecture les chemins d'avant). Le reste — ses
/// textes, ses flèches, une photo qu'il a collée là — est à lui, et la loi n'y touche jamais. Deux
/// nœuds du même fichier : le premier le suit, l'autre est à lui.
fn nes_du_disque<'a>(b: &'a Board, racine: &str, avant: Option<&Vus>) -> HashMap<&'a str, Ne<'a>> {
    let du_disque = |c: &str| vient_de(c, racine) || avant.is_some_and(|v| v.contains_key(c));
    let mut nes = HashMap::new();
    let photos = b
        .images
        .iter()
        .filter_map(|i| Some((i.src.as_deref()?, Ne::Photo(i))));
    let notes = b.annotations.iter().filter_map(|a| match a {
        Annotation::Text { source_file, .. } | Annotation::Sticky { source_file, .. } => {
            Some((source_file.as_deref()?, Ne::Note(a)))
        }
        _ => None,
    });
    let dossiers = b.folders.iter().filter_map(|f| {
        let s = f.mirror_source.as_ref()?;
        Some((s.root_path.as_str(), Ne::Dossier(f)))
    });
    for (chemin, ne) in photos.chain(notes).chain(dossiers) {
        if du_disque(chemin) {
            nes.entry(chemin).or_insert(ne);
        }
    }
    nes
}

/// La lecture en cours : ce qui a été lu, ce qui était vu, et le plan qui se fait.
struct Lecture<'a> {
    tableau: &'a Board,
    avant: Option<&'a Vus>,
    /// L'empreinte de chaque texte relu cette fois.
    frais: &'a HashMap<String, u64>,
    plan: Plan,
}

/// **Réconcilie un niveau** : `lus`, ce que le disque vient de rendre (dans l'ordre du tri) ;
/// `avant`, ce que le miroir avait vu (aucun : la mémoire est perdue, ou c'est la première fois) ;
/// `frais`, l'empreinte du Markdown de chaque texte relu — l'application ne relit que ceux qui ont
/// changé, ou qu'elle ne connaissait pas.
pub fn reconcilier(
    tableau: &Board,
    racine: &str,
    avant: Option<&Vus>,
    lus: &[Entree],
    frais: &HashMap<String, u64>,
) -> Plan {
    let mut nes = nes_du_disque(tableau, racine, avant);
    let mut l = Lecture {
        tableau,
        avant,
        frais,
        plan: Plan::default(),
    };
    let mut sans_noeud = Vec::new();
    for e in lus {
        match nes.remove(e.chemin.as_str()) {
            Some(ne) => l.revoir(&ne, e),
            None => sans_noeud.push(e),
        }
    }
    let mut disparus: Vec<(&str, Ne)> = nes.into_iter().collect();
    disparus.sort_by(|a, b| a.0.cmp(b.0));
    let renommes = l.renommages(&disparus, &sans_noeud);
    for e in sans_noeud {
        match renommes.get(e.chemin.as_str()) {
            Some(&k) => l.renommer(&disparus[k], e),
            None => l.accueillir(e),
        }
    }
    let suivis: HashSet<&str> = renommes.values().map(|&k| disparus[k].0).collect();
    for (chemin, ne) in disparus.iter().filter(|(c, _)| !suivis.contains(c)) {
        l.perdre(chemin, ne);
    }
    l.plan
}

impl Lecture<'_> {
    fn vu(&self, chemin: &str) -> Option<&Vu> {
        self.avant?.get(chemin)
    }

    /// Le fichier a-t-il changé depuis la lecture d'avant ? Sans mémoire, une photo le dit par la
    /// taille qu'elle porte ; un texte, par son contenu ([`Self::texte_suivi`]).
    fn a_change(&self, ne: &Ne, e: &Entree) -> bool {
        match (self.vu(&e.chemin), ne) {
            (Some(v), _) => (v.taille, v.date) != (e.taille, e.date),
            (None, Ne::Photo(i)) => taille_portee(i) != Some(e.taille),
            (None, _) => false,
        }
    }

    /// **Un texte que sa main n'a pas récrit** : il porte encore l'empreinte de ce que le miroir y
    /// avait posé — ou, sans mémoire, celle de ce que le fichier donne aujourd'hui.
    fn texte_suivi(&self, a: &Annotation, ancien: &str, nouveau: &str) -> bool {
        let Annotation::Text { text, .. } = a else {
            return false;
        };
        let pose = self
            .vu(ancien)
            .map_or(self.frais.get(nouveau).copied(), |v| v.empreinte);
        pose == Some(empreinte(text))
    }

    /// Un fichier qui a son nœud : il le suit s'il a changé.
    fn revoir(&mut self, ne: &Ne, e: &Entree) {
        let mut empreinte_vue = self.vu(&e.chemin).and_then(|v| v.empreinte);
        let relire = match ne {
            Ne::Note(a) if self.frais.contains_key(&e.chemin) => {
                let suivi = self.texte_suivi(a, &e.chemin, &e.chemin);
                let frais = self.frais[&e.chemin];
                if suivi || empreinte_vue.is_none() {
                    empreinte_vue = Some(frais);
                }
                suivi && !matches!(a, Annotation::Text { text, .. } if empreinte(text) == frais)
            }
            _ => false,
        };
        if relire || (matches!(ne, Ne::Photo(_)) && self.a_change(ne, e)) {
            self.suivre(ne, e, relire);
        }
        self.voir(e, empreinte_vue);
    }

    fn suivre(&mut self, ne: &Ne, e: &Entree, relire: bool) {
        self.plan.suivis.push(Suivi {
            noeud: ne.noeud(),
            entree: e.clone(),
            relire,
        });
    }

    fn voir(&mut self, e: &Entree, empreinte: Option<u64>) {
        let vu = Vu {
            taille: e.taille,
            date: e.date,
            empreinte,
        };
        self.plan.vus.insert(e.chemin.clone(), vu);
    }

    /// **Les renommages** : chaque fichier apparu qu'une seule paire (taille, date, extension) relie
    /// à un seul nœud disparu — l'apparu par son chemin, le disparu par son rang.
    fn renommages(&self, disparus: &[(&str, Ne)], apparus: &[&Entree]) -> HashMap<String, usize> {
        let cle_vue = |c: &str| {
            let v = self.vu(c)?;
            Some((v.taille, v.date, crate::file_kind::extension(c)))
        };
        let mut d: HashMap<_, Vec<usize>> = HashMap::new();
        for (k, (c, _)) in disparus.iter().enumerate() {
            if let Some(cle) = cle_vue(c) {
                d.entry(cle).or_default().push(k);
            }
        }
        let mut a: HashMap<_, Vec<&Entree>> = HashMap::new();
        for e in apparus.iter().filter(|e| self.vu(&e.chemin).is_none()) {
            let cle = (e.taille, e.date, crate::file_kind::extension(&e.chemin));
            a.entry(cle).or_default().push(e);
        }
        a.into_iter()
            .filter_map(|(cle, es)| match (es.as_slice(), d.get(&cle)?.as_slice()) {
                ([e], [k]) => Some((e.chemin.clone(), *k)),
                _ => None,
            })
            .collect()
    }

    /// Un fichier renommé : son nœud le suit ; un texte que sa main n'a pas récrit se relit (son
    /// titre est le nom du fichier).
    fn renommer(&mut self, (ancien, ne): &(&str, Ne), e: &Entree) {
        let relire = matches!(ne, Ne::Note(a) if self.texte_suivi(a, ancien, &e.chemin))
            && self.frais.contains_key(&e.chemin);
        let empreinte = if relire {
            self.frais.get(&e.chemin).copied()
        } else {
            self.vu(ancien).and_then(|v| v.empreinte)
        };
        self.suivre(ne, e, relire);
        self.voir(e, empreinte);
    }

    /// Un fichier sans nœud : nouveau, ou refusé — vu à l'identique la fois d'avant, son nœud a été
    /// retiré, et il ne revient pas.
    fn accueillir(&mut self, e: &Entree) {
        let refuse = self.vu(&e.chemin).copied();
        match refuse {
            Some(v) if (v.taille, v.date) == (e.taille, e.date) => {
                self.plan.vus.insert(e.chemin.clone(), v);
            }
            _ => {
                self.voir(e, self.frais.get(&e.chemin).copied());
                self.plan.nouveaux.push(e.clone());
            }
        }
    }

    /// Un fichier disparu : son nœud part, sauf s'il porte quelque chose de sa main — il reste, et
    /// le miroir s'en souvient (le fichier revenu, il le retrouvera).
    fn perdre(&mut self, chemin: &str, ne: &Ne) {
        if self.porte_sa_main(chemin, ne) {
            if let Some(&v) = self.vu(chemin) {
                self.plan.vus.insert(chemin.to_string(), v);
            }
        } else {
            self.plan.partis.push(ne.noeud());
        }
    }

    /// **Ce que sa main a ajouté ne se perd jamais à cause du disque** : une flèche qui le touche, un
    /// domaine, une membrane, un texte récrit ou une tuile renommée — et un sous-dossier déjà lu, dont
    /// le contenu peut porter son travail.
    fn porte_sa_main(&self, chemin: &str, ne: &Ne) -> bool {
        let id = ne.id();
        let fleche = self.tableau.annotations.iter().any(|a| match a {
            Annotation::Arrow {
                source_id,
                target_id,
                ..
            } => source_id.as_deref() == Some(id) || target_id.as_deref() == Some(id),
            _ => false,
        });
        fleche
            || match ne {
                Ne::Photo(i) => !i.domains.is_empty() || i.membrane_id.is_some(),
                Ne::Dossier(f) => f.mirror_source.as_ref().is_none_or(|s| !s.pending_scan),
                Ne::Note(a) => {
                    a.membrane_id().is_some()
                        || !a.domains().is_empty()
                        || match a {
                            Annotation::Sticky { text, .. } => Some(text.as_str()) != nom(chemin),
                            _ => !self.texte_suivi(a, chemin, chemin),
                        }
                }
            }
    }
}

/// La taille qu'une photo liée porte dans le document.
fn taille_portee(i: &BoardImage) -> Option<u64> {
    match i.asset.as_ref()? {
        crate::types::AssetRef::Link { size_bytes, .. } => *size_bytes,
        _ => None,
    }
}

/// Le nom d'un fichier, sans son dossier.
fn nom(chemin: &str) -> Option<&str> {
    Path::new(chemin).file_name()?.to_str()
}

/// **Écrit ce qu'un miroir a vu**, une ligne par fichier, rangées par chemin : `taille`, `date`,
/// l'empreinte en hexadécimal (ou `-`), puis le chemin, séparés par des tabulations — le chemin
/// dernier, qui peut en contenir. Un chemin qui contient un saut de ligne ne s'écrit pas : à la
/// lecture suivante, son fichier sera inconnu, et un nœud qu'on avait retiré reviendrait une fois.
pub fn ecrire_les_vus(vus: &Vus) -> String {
    let mut lignes: Vec<(&String, &Vu)> = vus.iter().filter(|(c, _)| !c.contains('\n')).collect();
    lignes.sort_by(|a, b| a.0.cmp(b.0));
    let mut texte = String::new();
    for (chemin, v) in lignes {
        let e = v
            .empreinte
            .map_or_else(|| "-".to_string(), |e| format!("{e:016x}"));
        texte.push_str(&format!("{}\t{}\t{e}\t{chemin}\n", v.taille, v.date));
    }
    texte
}

/// **Relit ce qu'un miroir a vu** ; une ligne qui ne se comprend pas est passée.
pub fn lire_les_vus(texte: &str) -> Vus {
    let ligne = |l: &str| {
        let mut champs = l.splitn(4, '\t');
        let taille = champs.next()?.parse().ok()?;
        let date = champs.next()?.parse().ok()?;
        let empreinte = match champs.next()? {
            "-" => None,
            hex => Some(u64::from_str_radix(hex, 16).ok()?),
        };
        let chemin = champs.next().filter(|c| !c.is_empty())?;
        Some((
            chemin.to_string(),
            Vu {
                taille,
                date,
                empreinte,
            },
        ))
    };
    texte.lines().filter_map(ligne).collect()
}

#[cfg(test)]
mod tests;
