//! **Les photos d'un participant à l'autre** (B1, fiches 113 et 117 § 7 bis).
//!
//! Un geste ne porte que la place d'une photo et sa clé ; ses octets vivent dans le registre de
//! celui qui l'a posée. Ils passent par la salle **sans y être gardés** (l'offre gratuite de
//! Cloudflare tient 5 Go pour tous les canevas) :
//!
//! * **ce que je pose part avec mon geste** — les octets de chaque photo qu'il pose, juste après
//!   lui ;
//! * **ce qui manque ici se demande quand quelqu'un entre** : moi, à chaque « à jour » de la salle
//!   (une entrée, un retour du réseau) — c'est mon *bonjour*, qui part même s'il ne manque rien ;
//!   les autres, au bonjour d'un nouveau venu, qui a peut-être ce qui leur manque. Aucune minuterie :
//!   seule une entrée peut apporter des octets qui n'étaient pas là ;
//! * **qui l'a la renvoie** — s'il écrit : la salle refuse les octets d'un lecteur. À l'arrivée, les
//!   octets entrent dans le registre, le scribe les scelle dans mon document, et le magasin oublie
//!   l'échec de la photo — même celui que rendra un décodage parti avant eux : le geste arrive
//!   avant ses octets, l'atelier a déjà cherché.
//!
//! **Le facteur** est un fil à part : il lit les octets hors de l'image (une tranche sur le disque,
//! quelques mégaoctets copiés), et attend ceux qu'un ouvrier a promis — une photo collée part avec
//! son geste avant d'être encodée. **Il ne lit que le registre** : jamais un chemin qu'une clé
//! nommerait, sans quoi n'importe qui demanderait `~/.ssh/…` et le recevrait.

use crate::app::GlucoseApp;
use crate::persist::objets::{Objets, Source};
use crate::salle::lien::Passe;
use glucose_core::salle::lettre::{Corps, Lettre};
use glucose_core::store::journal::Main;
use glucose_core::types::Project;
use std::sync::{mpsc, Arc};

/// **Ce que le partage retient des photos.**
#[derive(Debug)]
pub struct Photos {
    /// La boîte du facteur.
    facteur: mpsc::Sender<Course>,
    /// Ce qu'il faut demander au prochain tour.
    a_demander: Demander,
}

/// Ce qu'on demande au prochain tour, du moins au plus : un bonjour vaut une relance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Demander {
    Rien,
    /// Quelqu'un est entré : ce qui manque se redemande, s'il manque quelque chose.
    Relancer,
    /// J'entre : je le dis, avec ce qui me manque.
    Bonjour,
}

/// **Une course du facteur.**
#[derive(Debug)]
enum Course {
    /// Les octets de cette photo, s'ils sont au registre — attendus s'ils sont promis.
    Photo(String),
    Demande {
        cles: Vec<String>,
        bonjour: bool,
    },
    /// **Son nom ou sa couleur ont changé** (IDENTITE-2) : ses lettres suivantes les portent.
    Signer(Qui),
}

/// Qui signe les lettres du facteur : marque, nom, couleur.
pub(super) type Qui = (String, String, u8);

impl Photos {
    /// Le facteur part, avec le registre où lire et le fil où confier ses lettres ; il s'arrête
    /// avec le partage (sa boîte se ferme) ou avec le fil.
    pub(super) fn lancer(
        objets: Arc<Objets>,
        expediteur: mpsc::SyncSender<Passe>,
        qui: Qui,
    ) -> Self {
        let (facteur, courses) = mpsc::channel();
        let lance = std::thread::Builder::new()
            .name("glucose-facteur".into())
            .spawn(move || porter(&objets, &expediteur, qui, &courses));
        if let Err(e) = lance {
            crate::dire_erreur!("[Glucose] le facteur des photos ne part pas : {e}");
        }
        Self {
            facteur,
            a_demander: Demander::Rien,
        }
    }

    /// Les octets de cette photo partiront dès qu'ils seront là.
    pub(super) fn porter(&self, cle: String) {
        let _ = self.facteur.send(Course::Photo(cle));
    }

    /// Ses lettres suivantes porteront ce nom et cette couleur (IDENTITE-2).
    pub(super) fn signer(&self, qui: Qui) {
        let _ = self.facteur.send(Course::Signer(qui));
    }

    /// **La salle est à jour** — j'entre, ou le réseau revient : ce qui était en route a pu se
    /// perdre. Mon bonjour partira dès que je serai dedans.
    pub(super) fn dire_bonjour(&mut self) {
        self.a_demander = Demander::Bonjour;
    }
}

/// **Le travail du facteur**, une course après l'autre : la mémoire ne tient qu'une photo à la fois,
/// même pour un lot de deux cents.
fn porter(
    objets: &Objets,
    expediteur: &mpsc::SyncSender<Passe>,
    mut qui: Qui,
    courses: &mpsc::Receiver<Course>,
) {
    for (n, course) in courses.iter().enumerate() {
        let (corps, photo) = match course {
            Course::Photo(cle) => {
                // Le registre seul : `lire` irait sinon chercher un fichier du même nom.
                if objets.source(&cle).is_none() {
                    continue;
                }
                let Some(octets) = objets.lire_en_attendant(&cle) else {
                    continue;
                };
                (Corps::Objet { cle, octets }, true)
            }
            Course::Demande { cles, bonjour } => (Corps::Demande { cles, bonjour }, false),
            Course::Signer(nouveau) => {
                qui = nouveau;
                continue;
            }
        };
        let lettre = Lettre {
            marque: qui.0.clone(),
            nom: qui.1.clone(),
            couleur: qui.2,
            main: Main::Humain,
            corps,
        };
        let passe = Passe {
            id: format!("{}-p{n}", qui.0),
            clair: lettre.ecrire(),
            photo,
        };
        if expediteur.send(passe).is_err() {
            return;
        }
    }
}

/// **Une photo manque ici** si rien ne dit où lire ses octets : ni le registre, ni un fichier.
pub(super) fn manque(objets: &Objets, cle: &str) -> bool {
    !cle.is_empty() && objets.source(cle).is_none() && !std::path::Path::new(cle).is_file()
}

/// Les clés des photos du document dont les octets manquent ici, chacune une fois.
pub(super) fn cles_qui_manquent(projet: &Project, objets: &Objets) -> Vec<String> {
    let mut cles: Vec<String> = projet
        .toutes_les_images()
        .filter_map(|i| i.src.clone())
        .filter(|c| manque(objets, c))
        .collect();
    cles.sort();
    cles.dedup();
    cles
}

impl GlucoseApp {
    /// **Ce qui manque ici se demande**, si quelqu'un vient d'entrer — sauf à qui attend la base :
    /// il n'a rien à demander, son bonjour attend avec lui. Celui qui a ouvert le partage dit
    /// bonjour comme les autres : son réseau aussi revient.
    pub(super) fn demander_ce_qui_manque(&mut self) {
        let Some(cp) = self.lancement.agents.partage.as_mut() else {
            return;
        };
        if cp.photos.a_demander == Demander::Rien || cp.entree.attend() {
            return;
        }
        let bonjour = cp.photos.a_demander == Demander::Bonjour;
        cp.photos.a_demander = Demander::Rien;
        let cles = cles_qui_manquent(&self.store.project, &self.disque.objets);
        if bonjour || !cles.is_empty() {
            let _ = cp.photos.facteur.send(Course::Demande { cles, bonjour });
        }
    }

    /// **Une lettre qui passe** : des octets qu'on attendait, ou une demande — à laquelle on répond
    /// si l'on écrit ; et si c'est un bonjour, on redemande ce qui nous manque.
    pub(super) fn une_lettre_qui_passe(&mut self, lettre: Lettre) {
        match lettre.corps {
            Corps::Objet { cle, octets } => self.recevoir_la_photo(cle, octets),
            Corps::Demande { cles, bonjour } => {
                let Some(cp) = self.lancement.agents.partage.as_mut() else {
                    return;
                };
                if cp.code.peut_ecrire() {
                    for cle in cles {
                        cp.photos.porter(cle);
                    }
                }
                if bonjour {
                    cp.photos.a_demander = cp.photos.a_demander.max(Demander::Relancer);
                }
            }
            _ => {}
        }
    }

    /// **Les octets d'une photo sont là** : au registre, scellés dans le document par le scribe,
    /// et l'échec oublié, pour que l'atelier la décode enfin.
    pub(super) fn recevoir_la_photo(&mut self, cle: String, octets: Vec<u8>) {
        if !manque(&self.disque.objets, &cle) {
            return;
        }
        self.disque
            .objets
            .poser(&cle, Source::Memoire(Arc::new(octets)));
        self.sceller_ce_qui_attend();
        self.renderer.magasin.octets_arrives(&cle);
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
