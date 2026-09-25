//! **L'écriture du document** : ce que le fil qui dessine confie au scribe, et quand.
//!
//! # Chaque geste s'écrit
//!
//! Le journal du noyau garde dans sa file de sortie chaque transaction appliquée au document
//! (JRN-5). À chaque image, [`Ecriture::consigner`] la vide : chaque transaction devient un
//! **geste** confié au scribe. Enregistrer n'est plus une action : c'est ce qui arrive,
//! toujours, pendant qu'on travaille — comme Glucose Tauri, qui réécrivait son fichier une
//! seconde et demie après chaque changement, mais au coût du geste et non du document.
//!
//! # Quand écrire un instantané — sans seuil
//!
//! Rouvrir un document rejoue ce qui suit son dernier instantané. Un instantané s'écrit donc
//! quand les gestes écrits depuis **pèsent autant que lui** : rejouer ne coûte alors jamais
//! plus que relire un instantané, et l'histoire ne grossit que d'un facteur borné. C'est la
//! règle qu'Automerge applique à ses morceaux incrémentaux ; aucun nombre n'est choisi.
//!
//! # Les images à sceller
//!
//! Une image qui entre dans le document porte une clé qui est, jusqu'ici, un chemin de fichier
//! — celui qu'on a déposé, ou celui que le dossier temporaire a reçu. Tant que ses octets ne
//! sont pas dans le document, elle est confiée au scribe ([`super::scribe`]). Une image dont
//! le fichier n'existe pas encore — une image collée, que l'atelier est en train d'écrire —
//! attend son tour, et se reconfie dès qu'il paraît.
//!
//! # Le texte en cours de frappe
//!
//! Ce qu'une carte en édition contient se confie au scribe à chaque changement
//! ([`Ecriture::saisir`]) ; il le garde à côté du document jusqu'à ce que la saisie devienne
//! un geste. Rouvrir le document le retrouve ([`Ecriture::reprendre`]).

use super::objets::{Objets, Source};
use super::scribe::{chemin_de_saisie, Depart, Octets, Ordre, Scribe};
use glucose_core::persist::histoire::{self, nature, Genre, Geste, Jalon, Ouvert, Saisie, Vue};
use glucose_core::store::journal::Transaction;
use glucose_core::types::{BoardImage, Project};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Le document en train de s'écrire.
pub struct Ecriture {
    scribe: Scribe,
    /// Le fichier qui reçoit l'histoire : celui de l'utilisateur, ou un brouillon.
    pub chemin: PathBuf,
    /// Vrai pour un document qui n'a pas encore de nom : son fichier est un brouillon, dans le
    /// dossier de l'application, et « Enregistrer » lui en donnera un.
    pub brouillon: bool,
    /// Octets de gestes écrits depuis le dernier instantané, et taille de celui-ci.
    depuis: u64,
    taille: u64,
    /// Qui écrit : un identifiant par lancement.
    auteur: u64,
    /// Les clés confiées au scribe et pas encore revenues scellées.
    confiees: HashSet<String>,
    /// Les clés dont le fichier n'existait pas encore quand on a voulu les sceller.
    en_attente: Vec<String>,
    /// Le texte en cours de frappe confié au scribe — ce que son fichier garde. `None` : il
    /// n'y en a pas.
    saisie: Option<Saisie>,
    /// Combien de gestes le fichier porte, et après combien le dernier jalon a été posé : un
    /// `Ctrl+S` sans rien de nouveau depuis n'en pose pas un de plus.
    gestes: usize,
    dernier_jalon: Option<usize>,
}

impl Ecriture {
    /// Continue l'histoire d'un document qu'on vient d'ouvrir. Échoue si le fichier ne
    /// s'ouvre pas en écriture — lecture seule, disque retiré, document tenu ailleurs.
    ///
    /// Rend aussi le texte qu'un arrêt a laissé en cours de frappe dans ce document, s'il
    /// vaut encore : aucun geste ne l'a suivi. Celui qui ne vaut plus s'effacera.
    pub fn reprendre(
        ouvert: &Ouvert,
        chemin: PathBuf,
        objets: &Arc<Objets>,
        saisies: &Path,
    ) -> Result<(Self, Option<Saisie>), String> {
        let depart = Depart {
            chemin: chemin.clone(),
            base: None,
            fin: ouvert.fin,
            chaine: ouvert.chaine,
            version: ouvert.version_du_conteneur,
            objets: ouvert.objets.iter().map(|(e, t)| (*e, *t)).collect(),
            dernier_geste: ouvert.dernier_geste,
            saisies: saisies.to_path_buf(),
        };
        let fichier = chemin_de_saisie(saisies, &chemin);
        let lue = std::fs::read(&fichier)
            .ok()
            .map(|o| histoire::saisie::lire(&o));
        let mut e = Self::avec(
            Scribe::commencer(depart, Arc::clone(objets))?,
            chemin,
            false,
            ouvert.octets_depuis_l_instantane,
            ouvert.taille_de_l_instantane,
        );
        e.gestes = ouvert.gestes.len();
        e.dernier_jalon = ouvert.jalons.last().map(|(apres, _)| *apres);
        // Un fichier présent, lisible ou non, est à effacer dès qu'aucune saisie ne l'occupe.
        e.saisie = lue
            .as_ref()
            .map(|l| l.as_ref().map(|(_, s)| s.clone()).unwrap_or_default());
        let rendue = lue
            .flatten()
            .filter(|(point, _)| *point == ouvert.dernier_geste)
            .map(|(_, s)| s);
        Ok((e, rendue))
    }

    /// Commence un fichier neuf, dont la base est l'état `depart` du document.
    pub fn nouvelle(
        chemin: PathBuf,
        depart: &Project,
        brouillon: bool,
        objets: &Arc<Objets>,
        instant: i64,
        saisies: &Path,
    ) -> Result<Self, String> {
        let base =
            glucose_core::persist::encode(depart, &glucose_core::types::AssetStore::new(), instant);
        let taille = glucose_core::persist::encode_document(depart).len() as u64;
        let d = Depart::neuf(chemin.clone(), base, saisies.to_path_buf())?;
        Ok(Self::avec(
            Scribe::commencer(d, Arc::clone(objets))?,
            chemin,
            brouillon,
            0,
            taille,
        ))
    }

    fn avec(scribe: Scribe, chemin: PathBuf, brouillon: bool, depuis: u64, taille: u64) -> Self {
        Self {
            scribe,
            chemin,
            brouillon,
            depuis,
            taille,
            auteur: auteur_de_ce_lancement(),
            confiees: HashSet::new(),
            en_attente: Vec::new(),
            saisie: None,
            gestes: 0,
            dernier_jalon: None,
        }
    }

    /// Le texte de la carte en édition — `(annotation, texte)` sur ce tableau — ou `None`
    /// quand aucune ne l'est. Ne dérange le scribe que s'il a changé.
    pub fn saisir(&mut self, tableau: &str, en_cours: Option<(&str, &str)>) {
        let pareille = match (&self.saisie, en_cours) {
            (None, None) => true,
            (Some(s), Some((annotation, texte))) => {
                s.texte == texte && s.annotation == annotation && s.tableau == tableau
            }
            _ => false,
        };
        if pareille {
            return;
        }
        self.saisie = en_cours.map(|(annotation, texte)| Saisie {
            document: String::new(),
            tableau: tableau.to_string(),
            annotation: annotation.to_string(),
            texte: texte.to_string(),
        });
        self.scribe.envoyer(Ordre::Saisie(self.saisie.clone()));
    }

    /// Écrit les transactions appliquées depuis la dernière fois, scelle les images qui
    /// entrent, et pose un instantané quand l'histoire le demande.
    pub fn consigner(
        &mut self,
        transactions: Vec<Transaction>,
        projet: &Project,
        objets: &Objets,
        instant: i64,
    ) {
        for transaction in transactions {
            for img in transaction.images_posees() {
                self.sceller(img, objets);
            }
            let contenu = histoire::contenu_geste(&Geste {
                instant,
                auteur: self.auteur,
                transaction,
            });
            self.depuis += contenu.len() as u64;
            self.gestes += 1;
            self.envoyer(nature::GESTE, contenu);
        }
        self.reessayer(objets);
        if self.depuis >= self.taille {
            self.instantane(projet);
        }
    }

    /// Des images attendent-elles que leur fichier paraisse ?
    pub fn a_du_travail(&self) -> bool {
        !self.en_attente.is_empty()
    }

    /// Pose un instantané de l'état courant, tout de suite.
    pub fn instantane(&mut self, projet: &Project) {
        let contenu = histoire::contenu_instantane(projet);
        self.taille = contenu.len() as u64;
        self.depuis = 0;
        self.envoyer(nature::INSTANTANE, contenu);
    }

    fn envoyer(&self, nature: u8, contenu: Vec<u8>) {
        self.scribe.envoyer(Ordre::Entree { nature, contenu });
    }

    /// Confie au scribe les octets d'une image qui n'est pas encore dans le document.
    fn sceller(&mut self, img: &BoardImage, objets: &Objets) {
        let Some(cle) = img.src.as_deref() else {
            return;
        };
        if self.confiees.contains(cle) || objets.est_scellee(cle) {
            return;
        }
        let chemin = match objets.source(cle) {
            Some(Source::Fichier(p)) => p,
            _ => PathBuf::from(cle),
        };
        if !chemin.is_file() {
            if !self.en_attente.iter().any(|c| c == cle) {
                self.en_attente.push(cle.to_string());
            }
            return;
        }
        self.confiees.insert(cle.to_string());
        self.scribe.envoyer(Ordre::Sceller {
            cle: cle.to_string(),
            octets: Octets::Chemin(chemin),
        });
    }

    /// Les images dont le fichier est apparu depuis.
    fn reessayer(&mut self, objets: &Objets) {
        if self.en_attente.is_empty() {
            return;
        }
        let attente = std::mem::take(&mut self.en_attente);
        for cle in attente {
            let mut img = BoardImage::new("", 0.0, 0.0, 0.0, 0.0);
            img.src = Some(cle);
            self.sceller(&img, objets);
        }
    }

    /// Scelle des octets qui ne viennent d'aucun fichier — une image d'un vieux document
    /// Tauri, portée en base64 par le document lui-même.
    pub fn sceller_des_octets(&mut self, cle: &str, octets: Vec<u8>) {
        self.confiees.insert(cle.to_string());
        self.scribe.envoyer(Ordre::Sceller {
            cle: cle.to_string(),
            octets: Octets::Memoire(octets),
        });
    }

    /// Scelle chaque image du projet qui ne l'est pas encore : un document ouvert dont
    /// certaines images vivaient encore hors de lui, un document importé.
    pub fn sceller_tout(&mut self, projet: &Project, objets: &Objets) {
        for img in projet.toutes_les_images() {
            self.sceller(img, objets);
        }
    }

    /// Pose un jalon, écrit la vue, et attend que tout soit sur le disque.
    ///
    /// Un `Ctrl+S` qui suit un jalon sans qu'aucun geste les sépare n'en pose pas un second :
    /// quatre « Enregistré » au même geste ne disent rien de plus qu'un seul. Il attend
    /// seulement que tout soit sur le disque.
    pub fn jalon(
        &mut self,
        projet: &Project,
        genre: Genre,
        libelle: &str,
        instant: i64,
    ) -> Result<(), String> {
        if genre == Genre::Enregistrement && self.dernier_jalon == Some(self.gestes) {
            return self.scribe.synchroniser();
        }
        self.dernier_jalon = Some(self.gestes);
        self.vue(projet);
        let contenu = histoire::contenu_jalon(&Jalon {
            instant,
            genre,
            libelle: libelle.to_string(),
        });
        self.envoyer(nature::JALON, contenu);
        self.scribe.synchroniser()
    }

    /// Écrit où l'on regarde : tableau actif, caméras, signets.
    pub fn vue(&self, projet: &Project) {
        self.envoyer(nature::VUE, histoire::contenu_vue(&Vue::de(projet)));
    }

    /// Continue dans `vers`, copie exacte du fichier courant. Un brouillon est oublié.
    pub fn deplacer(&mut self, vers: PathBuf) -> Result<(), String> {
        self.scribe.deplacer(vers.clone(), self.brouillon)?;
        self.chemin = vers;
        self.brouillon = false;
        Ok(())
    }

    pub fn synchroniser(&self) -> Result<(), String> {
        self.scribe.synchroniser()
    }

    /// L'erreur d'écriture survenue depuis la dernière fois, que l'utilisateur doit lire.
    pub fn prendre_l_erreur(&self) -> Option<String> {
        self.scribe.prendre_l_erreur()
    }

    /// Oublie ce document : attend le scribe, puis efface le brouillon s'il en est un — et
    /// le texte qu'on y tapait.
    pub fn abandonner(self) {
        let (chemin, brouillon) = (self.chemin.clone(), self.brouillon);
        self.scribe.envoyer(Ordre::Saisie(None));
        drop(self);
        if brouillon {
            let _ = std::fs::remove_file(chemin);
        }
    }
}

/// Un identifiant par lancement : l'instant et le processus, mêlés. Il ne distingue
/// aujourd'hui qu'une main de l'autre dans une même histoire ; le co-working en demandera un
/// par installation (fiche 36, phase 5).
fn auteur_de_ce_lancement() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut x = nanos ^ (u64::from(std::process::id()) << 32) ^ 0x9E37_79B9_7F4A_7C15;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^ (x >> 33)
}

/// Un nom de brouillon neuf, dans ce dossier.
pub fn nouveau_brouillon(dossier: &Path, instant: i64) -> PathBuf {
    dossier.join(format!(
        "{instant}-{}.{}",
        std::process::id(),
        glucose_core::persist::FILE_EXTENSION
    ))
}
