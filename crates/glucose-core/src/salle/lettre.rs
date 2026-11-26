//! **La lettre** (fiche 117) : ce qu'un participant dit à la salle, en clair, avant d'être scellé —
//! qui parle (sa marque, son nom, et la main : lui, ou un de ses agents), puis **la base** du
//! canevas, **un geste**, **son curseur**, ou **ce qu'il a choisi**.
//!
//! Écrite dans la forme du disque (`persist::bytes`), préfixée du schéma de document de l'émetteur :
//! un geste et une base se relisent au schéma où ils ont été écrits, comme l'histoire d'un fichier.
//! Un schéma plus récent que le sien se refuse en le disant — l'autre a une version plus neuve.

use crate::error::{CoreError, CoreResult};
use crate::persist::bytes::{Reader, Writer};
use crate::persist::histoire::geste::{ecrire_partage, lire_partage};
use crate::persist::{decode_document_v, DOCUMENT_VERSION};
use crate::salle::morceaux;
use crate::store::journal::partage::GestePartage;
use crate::store::journal::Main;
use crate::types::Project;

const BASE: u8 = 1;
const GESTE: u8 = 2;
const CURSEUR: u8 = 3;
const OBJET: u8 = 4;
const DEMANDE: u8 = 5;
const SELECTION: u8 = 6;

/// **Ce que dit la lettre.**
#[derive(Debug, Clone, PartialEq)]
pub enum Corps {
    /// Le canevas au moment où il a été partagé : le premier message de toute salle.
    Base(Box<Project>),
    Geste(GestePartage),
    /// Où est sa souris : le tableau qu'il regarde, et le point du monde.
    Curseur {
        tableau: String,
        x: f64,
        y: f64,
    },
    /// **Les octets d'une photo** (B1) : ce qu'un geste pose et que les autres n'ont pas — passés par
    /// la salle sans y être gardés.
    Objet {
        cle: String,
        octets: Vec<u8>,
    },
    /// **Les photos qui manquent ici**, par leur clé : qui les a les renvoie. `bonjour` : celui qui
    /// demande vient d'entrer — il a peut-être ce qui manque aux autres, qui redemandent.
    Demande {
        cles: Vec<String>,
        bonjour: bool,
    },
    /// **Ce qu'il a choisi en dernier** (REGARD-1, fiche 177 ; sa note du 24/11 dans
    /// `relais-rep-mcp` : *« les derniers éléments sélectionnés par les utilisateurs, même à
    /// distance, pour savoir sur quoi ils se sont focalisés »*) : le tableau, les nœuds dans
    /// l'ordre du choix — ceux qui tiennent dans un curseur ([`Lettre::tenir_dans_un_curseur`]),
    /// sur `en_tout` —, et depuis combien de secondes. Elle passe comme un curseur, ni gardée ni
    /// numérotée : une version qui ne la connaît pas l'ignore sans bruit (la beta.6 jette tout
    /// curseur qu'elle ne sait pas lire).
    Selection {
        tableau: String,
        ids: Vec<String>,
        en_tout: usize,
        age: u64,
    },
}

/// **Une lettre** : qui, et quoi.
#[derive(Debug, Clone, PartialEq)]
pub struct Lettre {
    /// Sa marque (ID-2) : ce qui le distingue des autres, dans ses noms de nœuds et ses messages.
    pub marque: String,
    /// Le nom qu'il se donne.
    pub nom: String,
    /// Sa couleur : un rang dans la palette des participants (les dix de Glucose Tauri).
    pub couleur: u8,
    /// **La main** : `Humain` pour lui, `Agent` pour une IA qui travaille dans son Glucose. Chez
    /// celui qui reçoit, l'humain devient un participant ([`Lettre::main_chez_moi`]).
    pub main: Main,
    pub corps: Corps,
}

impl Lettre {
    /// **Écrit la lettre**, au plus petit schéma qui la dit.
    pub fn ecrire(&self) -> Vec<u8> {
        // Au plus petit schéma qui la dit : une beta.6 lit encore ce qui ne parle pas de ses
        // champs neufs (VIDEO-2, fiche 170).
        crate::persist::au_plus_petit_schema(|w| {
            w.u16(w.schema());
            w.text(&self.marque);
            w.text(&self.nom);
            w.u8(self.couleur);
            match &self.main {
                Main::Agent(nom) => {
                    w.u8(1);
                    w.text(nom);
                }
                // Un participant ne renvoie jamais un geste reçu : seule sa main ou ses agents parlent.
                Main::Humain | Main::Participant { .. } => w.u8(0),
            }
            match &self.corps {
                Corps::Base(p) => {
                    w.u8(BASE);
                    crate::persist::ecrire_le_document(w, p);
                }
                Corps::Geste(g) => {
                    w.u8(GESTE);
                    ecrire_partage(w, g);
                }
                Corps::Curseur { tableau, x, y } => {
                    w.u8(CURSEUR);
                    w.text(tableau);
                    w.f64(*x);
                    w.f64(*y);
                }
                Corps::Objet { cle, octets } => {
                    w.u8(OBJET);
                    w.text(cle);
                    w.blob(octets);
                }
                Corps::Demande { cles, bonjour } => {
                    w.u8(DEMANDE);
                    w.flag(*bonjour);
                    w.seq(cles, |w, c| w.text(c));
                }
                Corps::Selection {
                    tableau,
                    ids,
                    en_tout,
                    age,
                } => {
                    w.u8(SELECTION);
                    w.text(tableau);
                    w.seq(ids, |w, id| w.text(id));
                    w.count(*en_tout);
                    w.uvarint(*age);
                }
            }
        })
        .1
    }

    /// **Relit une lettre** écrite par n'importe quelle version jusqu'à celle-ci.
    pub fn lire(octets: &[u8]) -> CoreResult<Self> {
        let mut r = Reader::new(octets);
        let schema = r.u16()?;
        if schema > DOCUMENT_VERSION {
            return Err(CoreError::DeserializationError(format!(
                "une lettre du schéma {schema}, plus récent que celui-ci ({DOCUMENT_VERSION}) : \
                 l'autre a une version de Glucose plus neuve"
            )));
        }
        let marque = r.text()?;
        let nom = r.text()?;
        let couleur = r.u8()?;
        let main = match r.u8()? {
            0 => Main::Humain,
            1 => Main::Agent(r.text()?),
            m => {
                return Err(CoreError::DeserializationError(format!(
                    "une main inconnue ({m})"
                )))
            }
        };
        let corps = match r.u8()? {
            BASE => {
                let reste = r.take(r.remaining())?;
                Corps::Base(Box::new(decode_document_v(reste, schema)?))
            }
            GESTE => Corps::Geste(lire_partage(&mut r, schema)?),
            CURSEUR => Corps::Curseur {
                tableau: r.text()?,
                x: r.f64()?,
                y: r.f64()?,
            },
            OBJET => Corps::Objet {
                cle: r.text()?,
                octets: r.blob()?,
            },
            DEMANDE => {
                let bonjour = r.flag()?;
                Corps::Demande {
                    bonjour,
                    cles: r.seq(|r| r.text())?,
                }
            }
            SELECTION => Corps::Selection {
                tableau: r.text()?,
                ids: r.seq(|r| r.text())?,
                en_tout: r.count()?,
                age: r.uvarint()?,
            },
            s => {
                return Err(CoreError::DeserializationError(format!(
                    "une lettre inconnue ({s})"
                )))
            }
        };
        r.finish()?;
        Ok(Self {
            marque,
            nom,
            couleur,
            main,
            corps,
        })
    }

    /// **Une sélection qui tient dans un curseur** : la salle n'en passe qu'un morceau
    /// ([`morceaux::CURSEUR_CLAIR`]). Les derniers choisis s'en vont jusqu'à ce qu'elle tienne ;
    /// `en_tout` garde leur nombre. Toute autre lettre reste telle quelle.
    pub fn tenir_dans_un_curseur(mut self) -> Self {
        let trop = self.ecrire().len().saturating_sub(morceaux::CURSEUR_CLAIR);
        if let Corps::Selection { ids, .. } = &mut self.corps {
            // Chaque nœud retiré rend son nom et sa longueur ; le compte de la suite ne fait que
            // rétrécir, `en_tout` ne bouge pas.
            let mut rendu = 0;
            while rendu < trop {
                let Some(id) = ids.pop() else { break };
                let mut w = Writer::new();
                w.text(&id);
                rendu += w.into_bytes().len();
            }
        }
        self
    }

    /// **La main qui signe ce geste chez celui qui le reçoit** : un autre humain y est un
    /// participant, jamais « l'humain » de ce Glucose ; une IA reste une IA, sous son nom.
    pub fn main_chez_moi(&self) -> Main {
        match &self.main {
            Main::Agent(nom) => Main::Agent(nom.clone()),
            Main::Humain | Main::Participant { .. } => Main::Participant {
                marque: self.marque.clone(),
                nom: self.nom.clone(),
            },
        }
    }
}
