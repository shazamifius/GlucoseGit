//! Le décodage des images, hors de l'image (DECODE-1).
//!
//! # Ce que ce module répare
//!
//! Le décodage avait lieu **dans la boucle de rendu**. Une photo de téléphone de douze
//! mégapixels coûte 351 ms à ouvrir, convertir et prémultiplier : à 400 images par seconde,
//! c'est **cent quarante images perdues** pour une seule photo. Vu de l'utilisateur, importer
//! une image fige l'application — et aucune optimisation du dessin n'y pouvait quoi que ce
//! soit, puisque le dessin n'avait pas encore commencé.
//!
//! C'est exactement le cas que la charte décrit : une action lourde a le droit de durer, mais
//! **le rendu ne l'attend jamais**. Le décodage part donc sur des fils de fond, et l'image qui
//! n'est pas encore prête se dessine comme ce qu'elle est — une image en chemin.
//!
//! # Pourquoi aucun nombre d'ouvriers n'est choisi
//!
//! Un ouvrier suffirait à ne plus bloquer le rendu, mais un lot de trente photos déposées
//! ensemble se décoderait alors l'une après l'autre. Le nombre d'ouvriers est donc celui que
//! la machine annonce — [`std::thread::available_parallelism`] — et il n'apparaît nulle part
//! comme constante. Un cœur de moins gardé pour le fil de rendu, parce que c'est lui qui tient
//! la cadence et qu'il ne doit jamais attendre son tour.
//!
//! # Trois gestes, et leur ordre est la priorité (ETAGES-1)
//!
//! Depuis la mémoire par étages, l'atelier ne fait plus que décoder. Il **reprend** au système
//! les niveaux que l'écran redemande, et il lui **offre** ceux qui ne servent plus — deux
//! gestes de trois à cinq millisecondes pour dix mégaoctets, trop chers pour le fil qui
//! dessine. Un ouvrier libre prend d'abord une reprise (l'écran l'attend, et elle coûte sept
//! fois moins qu'un décodage), puis un décodage, puis une offre (rien ne l'attend). Aucune
//! priorité chiffrée : trois files, lues dans cet ordre.
//!
//! # Ce que ce module ne fait pas
//!
//! Il ne décide pas quoi garder : le cache et sa borne appartiennent à l'appelant. Il ne
//! décide pas non plus ce qu'on dessine en attendant. Il répond à une seule question — « ces
//! octets, décodés, les voici » — et il y répond quand il peut.

use super::apercu::{self, Apercu};
use super::photo::{Pyramide, Retour, Transit};
use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Ce qu'un ouvrier rend : le chemin demandé, et la **pyramide** s'il a su lire le fichier.
///
/// # Pourquoi une pyramide et non une image
///
/// Rendre le seul niveau natif laissait au fil de rendu tout ce qui s'en déduit : constater
/// l'opacité (un parcours de quarante-huit mégaoctets) et construire les réductions. Mesuré
/// dans cet état : **73 ms sur la récolte**, en pleine image, pour une seule photo. Le
/// décodage était bien parti sur un fil de fond, et le travail était resté.
///
/// L'invariant DECODE-1 porte donc sur tout ce qui dérive du fichier, pas sur le seul appel à
/// la bibliothèque de décodage. Ce qu'un ouvrier rend est **prêt à poser**, sans reste.
///
/// `None` n'est pas une erreur à signaler : un fichier absent ou illisible est un cas normal
/// du document, et l'appelant en fait un cache négatif.
pub type Decodee = (String, Option<Pyramide>, Duration);

/// **Un niveau qui voyage** : de quelle image, de quelle pyramide, quel rang — et ce qu'il
/// porte, à l'aller comme au retour.
///
/// La **génération** dit quelle pyramide l'a envoyé : une image redécodée entre-temps en a
/// une neuve, et ce qui revient pour l'ancienne ne doit pas s'y poser.
pub struct Deplacement<T> {
    pub src: String,
    pub generation: u64,
    pub rang: usize,
    pub charge: T,
}

/// Ce qu'un ouvrier rend.
pub enum Fait {
    Decodee(Decodee),
    Deplace(Deplacement<Retour>),
}

enum Travail {
    /// Une image à décoder — par son aperçu d'abord, s'il y a un dossier où le chercher —,
    /// et le registre qui dit où sont ses octets.
    Decoder(
        String,
        Option<std::sync::Arc<std::path::Path>>,
        Option<Arc<crate::persist::objets::Objets>>,
    ),
    Deplacer(Deplacement<Transit>),
    /// La vue d'ensemble d'une image à garder sur le disque (ETAGES-4) : la source, le
    /// dossier, l'aperçu.
    Ecrire(String, std::sync::Arc<std::path::Path>, Apercu),
    /// Des pixels déjà là — une image collée — à poser en pyramide **et** à écrire en fichier
    /// sous ce chemin (COLLER-1).
    Adopter(String, Vec<u8>, (u32, u32)),
}

/// Les trois files, lues dans l'ordre de leur urgence.
#[derive(Default)]
struct Files {
    reprises: VecDeque<Travail>,
    decodages: VecDeque<Travail>,
    offres: VecDeque<Travail>,
    fermee: bool,
}

impl Files {
    fn prochain(&mut self) -> Option<Travail> {
        self.reprises
            .pop_front()
            .or_else(|| self.decodages.pop_front())
            .or_else(|| self.offres.pop_front())
    }
}

#[derive(Default)]
struct Commandes {
    files: Mutex<Files>,
    reveil: Condvar,
}

/// Les fils qui décodent, reprennent et offrent, et ce qu'ils ont en chantier.
pub struct Atelier {
    commandes: Arc<Commandes>,
    prets: Receiver<Fait>,
    /// Ce qui est parti au décodage et n'est pas revenu. Sert à ne pas redemander à chaque
    /// image la même photo — sans quoi une seule image en cours de décodage saturerait la
    /// file en quelques dixièmes de seconde.
    en_cours: HashSet<String>,
    /// Les reprises parties et pas revenues : comme un décodage, elles apporteront quelque
    /// chose à montrer.
    reprises: usize,
    /// Les offres parties et pas revenues : elles n'apportent rien à montrer, et ne
    /// réveillent donc personne — mais un témoin qui veut un état fini les attend.
    offres: usize,
    /// Où chercher et garder les aperçus, si l'application le veut (ETAGES-4). Sans lui,
    /// l'atelier ne touche jamais au disque que pour lire les sources — ce que veulent les
    /// épreuves, qui ne doivent rien écrire chez l'utilisateur.
    apercus: Option<std::sync::Arc<std::path::Path>>,
    /// Les écritures parties : comme les offres, un état fini les attend.
    ecritures: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    /// Où sont les octets de chaque image (HISTOIRE-1). Sans lui — les épreuves —, une clé
    /// se lit comme un chemin de fichier, ce qu'elle était avant le registre.
    objets: Option<Arc<crate::persist::objets::Objets>>,
}

impl Default for Atelier {
    fn default() -> Self {
        Self::nouveau()
    }
}

impl Drop for Atelier {
    /// Les ouvriers se terminent d'eux-mêmes : la file se ferme, et chacun le voit en se
    /// réveillant. Ce qui restait à faire tombe avec elle — des niveaux à offrir ou à
    /// reprendre, dont la mémoire est rendue au système.
    fn drop(&mut self) {
        if let Ok(mut files) = self.commandes.files.lock() {
            files.fermee = true;
        }
        self.commandes.reveil.notify_all();
    }
}

impl Atelier {
    /// Ouvre l'atelier et lance ses ouvriers.
    pub fn nouveau() -> Self {
        let commandes = Arc::new(Commandes::default());
        let (retour, prets) = channel::<Fait>();
        let ecritures: Arc<std::sync::atomic::AtomicUsize> = Default::default();
        for _ in 0..ouvriers() {
            let (commandes, retour, ecritures) = (
                Arc::clone(&commandes),
                retour.clone(),
                Arc::clone(&ecritures),
            );
            std::thread::spawn(move || ouvrier(&commandes, &retour, &ecritures));
        }
        Self {
            commandes,
            prets,
            en_cours: HashSet::new(),
            reprises: 0,
            offres: 0,
            apercus: None,
            ecritures,
            objets: None,
        }
    }

    /// Demande le décodage de ce fichier, s'il n'est pas déjà en chantier.
    ///
    /// Rend `true` si la demande est partie — ce qui n'arrive qu'une fois par fichier tant
    /// qu'il n'est pas revenu.
    pub fn demander(&mut self, src: &str) -> bool {
        self.decoder_par(src, self.apercus.clone())
    }

    /// Demande le décodage **entier** de ce fichier, sans passer par son aperçu : un niveau
    /// que l'écran veut n'y est pas (ETAGES-4).
    pub fn redecoder(&mut self, src: &str) -> bool {
        self.decoder_par(src, None)
    }

    fn decoder_par(&mut self, src: &str, apercus: Option<std::sync::Arc<std::path::Path>>) -> bool {
        if self.en_cours.contains(src) {
            return false;
        }
        if !self.confier(Travail::Decoder(
            src.to_string(),
            apercus,
            self.objets.clone(),
        )) {
            return false;
        }
        self.en_cours.insert(src.to_string());
        true
    }

    /// **Adopte des pixels déjà là** — une image collée — sous ce chemin : un ouvrier en fait
    /// la pyramide, écrit le fichier, puis la rend comme un décodage (COLLER-1).
    ///
    /// Le fichier d'abord, la pyramide ensuite : une image qu'on relirait avant qu'il existe
    /// passerait pour illisible, et ne se redemanderait jamais.
    pub fn adopter(&mut self, src: &str, rgba: Vec<u8>, dimensions: (u32, u32)) -> bool {
        if self.en_cours.contains(src)
            || !self.confier(Travail::Adopter(src.to_string(), rgba, dimensions))
        {
            return false;
        }
        self.en_cours.insert(src.to_string());
        true
    }

    /// **Lit les octets des images par ce registre** : le document pour une image scellée,
    /// le fichier d'origine pour une image qui ne l'est pas encore (HISTOIRE-1).
    pub fn brancher_les_objets(&mut self, objets: Arc<crate::persist::objets::Objets>) {
        self.objets = Some(objets);
    }

    /// **Garde les aperçus dans ce dossier** : les décodages y cherchent d'abord, et les
    /// vues d'ensemble s'y écrivent.
    pub fn brancher_les_apercus(&mut self, dossier: std::path::PathBuf) {
        self.apercus = Some(dossier.into());
    }

    /// Les aperçus sont-ils gardés ?
    pub fn garde_les_apercus(&self) -> bool {
        self.apercus.is_some()
    }

    /// **Écrit la vue d'ensemble de cette image**, sur un fil et sans rien attendre.
    pub fn ecrire_l_apercu(&mut self, src: &str, apercu: Apercu) {
        let Some(dossier) = self.apercus.clone() else {
            return;
        };
        if self.confier(Travail::Ecrire(src.to_string(), dossier, apercu)) {
            self.ecritures
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// **Confie un niveau à un ouvrier**, qui l'offrira ou le reprendra.
    pub fn deplacer(&mut self, deplacement: Deplacement<Transit>) {
        let reprise = matches!(deplacement.charge, Transit::AReprendre(_));
        if self.confier(Travail::Deplacer(deplacement)) {
            match reprise {
                true => self.reprises += 1,
                false => self.offres += 1,
            }
        }
    }

    fn confier(&self, travail: Travail) -> bool {
        let Ok(mut files) = self.commandes.files.lock() else {
            return false;
        };
        match &travail {
            Travail::Decoder(..) | Travail::Adopter(..) => files.decodages.push_back(travail),
            Travail::Deplacer(d) => match d.charge {
                Transit::AReprendre(_) => files.reprises.push_back(travail),
                Transit::AOffrir(_) => files.offres.push_back(travail),
            },
            Travail::Ecrire(..) => files.offres.push_back(travail),
        }
        drop(files);
        self.commandes.reveil.notify_one();
        true
    }

    /// Récolte tout ce qui est prêt, sans jamais attendre.
    ///
    /// C'est le seul point de contact entre les fils de fond et le rendu, et il ne bloque
    /// pas : ce qui n'est pas fini sera récolté à l'image suivante.
    pub fn recolter(&mut self) -> Vec<Fait> {
        let mut moisson = Vec::new();
        while let Ok(fait) = self.prets.try_recv() {
            match &fait {
                Fait::Decodee((src, ..)) => {
                    self.en_cours.remove(src);
                }
                Fait::Deplace(d) => match d.charge {
                    Retour::Repris(_) => self.reprises = self.reprises.saturating_sub(1),
                    Retour::Offerts(_) => self.offres = self.offres.saturating_sub(1),
                },
            }
            moisson.push(fait);
        }
        moisson
    }

    /// Combien de décodages et de reprises sont encore en chantier.
    ///
    /// C'est ce qui dit à la boucle d'événements de repasser bientôt : tant qu'il reste du
    /// travail qui apportera quelque chose à montrer, l'application a une raison de se
    /// réveiller même si l'utilisateur ne fait rien. Une offre n'en apporte pas.
    pub fn en_travail(&self) -> usize {
        self.en_cours.len() + self.reprises
    }

    /// Tout ce qui est parti et pas revenu, offres et écritures comprises.
    pub fn en_route(&self) -> usize {
        self.en_travail() + self.offres + self.ecritures.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// La vie d'un ouvrier : prendre le plus urgent, le faire, le rendre — jusqu'à la fermeture.
fn ouvrier(
    commandes: &Commandes,
    retour: &Sender<Fait>,
    ecritures: &std::sync::atomic::AtomicUsize,
) {
    // Le cœur « laissé au fil qui dessine » ne lui est réservé par rien : il faut le lui
    // céder (CEDER-1).
    crate::plateforme::priorite::ceder_au_fil_qui_dessine();
    while let Some(travail) = attendre(commandes) {
        let fait = match travail {
            Travail::Decoder(src, apercus, objets) => {
                // Le temps que ce fichier a coûté est ce que sa reconstruction coûterait :
                // c'est exactement son utilité dans un cache, et elle se mesure ici plutôt
                // que de s'estimer ailleurs (ADAPT-1). Un aperçu coûte peu : l'image qui en
                // naît se rend la première si la mémoire manque, et c'est juste.
                let debut = Instant::now();
                let pyramide = apercus
                    .and_then(|dossier| apercu::lire(&apercu::chemin(&dossier, &src)?))
                    .and_then(Pyramide::depuis_apercu)
                    .or_else(|| decoder(&src, objets.as_deref()));
                Fait::Decodee((src, pyramide, debut.elapsed()))
            }
            Travail::Adopter(src, rgba, (l, h)) => {
                let debut = Instant::now();
                let ecrit =
                    image::save_buffer(&src, &rgba, l, h, image::ExtendedColorType::Rgba8).is_ok();
                let pyramide = ecrit.then(|| Pyramide::depuis_rgba(l, h, &rgba)).flatten();
                Fait::Decodee((src, pyramide, debut.elapsed()))
            }
            Travail::Ecrire(src, dossier, a) => {
                // Un aperçu qui ne s'écrit pas n'est qu'un aperçu de moins : la source reste.
                // Un aperçu déjà là n'est pas réécrit : son nom dit la source telle qu'elle est.
                if let Some(chemin) = apercu::chemin(&dossier, &src).filter(|c| !c.exists()) {
                    let _ = apercu::ecrire(&chemin, &a);
                }
                ecritures.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                continue;
            }
            Travail::Deplacer(d) => Fait::Deplace(Deplacement {
                src: d.src,
                generation: d.generation,
                rang: d.rang,
                charge: d.charge.accomplir(),
            }),
        };
        if retour.send(fait).is_err() {
            return;
        }
    }
}

/// Le prochain travail, en dormant tant qu'il n'y en a pas ; `None` à la fermeture.
fn attendre(commandes: &Commandes) -> Option<Travail> {
    let mut files = commandes.files.lock().ok()?;
    loop {
        if files.fermee {
            return None;
        }
        if let Some(travail) = files.prochain() {
            return Some(travail);
        }
        files = commandes.reveil.wait(files).ok()?;
    }
}

/// Combien d'ouvriers, sur cette machine.
///
/// Tous les cœurs sauf un : celui qui reste tient la cadence, et il ne doit jamais se
/// retrouver en concurrence avec le décodage pour son propre temps — ce que seul l'ordre de
/// passage garantit ([`crate::plateforme::priorite`]). Au moins un ouvrier, même
/// sur une machine qui n'annonce qu'un cœur — sinon l'atelier ne décoderait jamais rien.
fn ouvriers() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1))
        .unwrap_or(1)
        .max(1)
}

/// Lit une image et la rend prête à poser : décodée, prémultipliée, et réduite.
///
/// Les octets viennent du registre des objets — le document pour une image scellée, le
/// fichier d'origine sinon (HISTOIRE-1). Le format se reconnaît à la **signature** des
/// octets, jamais à une extension : une clé du document n'en a pas.
///
/// Les pixels vont directement dans les pages de la pyramide ([`Pyramide::depuis_rgba`]) : le
/// `Pixmap` intermédiaire qu'il y avait ici coûtait une copie de plus par image — dix
/// mégaoctets pour une épingle.
fn decoder(src: &str, objets: Option<&crate::persist::objets::Objets>) -> Option<Pyramide> {
    let octets = match objets {
        Some(o) => o.lire(src)?,
        None => std::fs::read(src).ok()?,
    };
    let rgba = image::load_from_memory(&octets).ok()?.to_rgba8();
    let (w, h) = rgba.dimensions();
    Pyramide::depuis_rgba(w, h, rgba.as_raw())
}

#[cfg(test)]
mod tests;
