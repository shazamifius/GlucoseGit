//! **SCELLE-1 — les morceaux scellés** (fiche 117) : une lettre devient un ou plusieurs gestes de la
//! salle, chacun scellé à part (ChaCha20-Poly1305, `chiffre`).
//!
//! # Pourquoi des morceaux, et pas seulement pour la base
//!
//! Une valeur de la salle tient deux mégaoctets (`GESTE_MAX` du relais : 1,5 million de signes
//! base64). Sa carte de Wikipédia en pèse 2,1 — mais **un geste aussi peut la dépasser** : retirer
//! un tableau entier porte tout ce qu'il contenait (`Edit::Board`, JRN-1). Une seule loi pour tout
//! message : découper en morceaux d'un mébioctet, numérotés `rang / total`.
//!
//! # Ce que le sceau couvre
//!
//! Les données associées de chaque morceau sont **l'identité de la salle et son nom local**
//! (`<id>_<rang>`) : la salle ne peut ni le lire, ni le changer, ni le faire passer pour un autre
//! morceau, une autre lettre ou une autre salle. Le rang et le total voyagent **dans** le scellé.
//! Elle peut encore retenir, répéter ou réordonner — le client compte les numéros (une répétition
//! s'ignore, un trou se redemande), comme Secsync l'admet de son serveur aveugle.
//!
//! # Le nonce
//!
//! Quatre-vingt-seize bits du hasard du système, tirés par l'appelant pour chaque morceau : deux
//! messages ne partagent un nonce qu'avec une chance de l'ordre de N²/2⁹⁷ — un milliard de messages
//! sous la même clé laissent une chance sur cent milliards.

use crate::chiffre::{self, CLE, NONCE};
use crate::encodage::{base64, base64_ecrire};
use crate::persist::bytes::{Reader, Writer};
use std::collections::HashMap;

/// Un morceau, en octets clairs : un mébioctet, scellé et en base64 sous les 1,5 million de signes.
pub const MORCEAU: usize = 1 << 20;
/// Une lettre ne dépasse pas soixante-quatre morceaux : soixante-quatre mébioctets.
pub const MORCEAUX_MAX: usize = 64;
/// **Le plus gros curseur que la salle passe**, en signes base64 (`outils/porte/src/messages.js` :
/// `m.o.length <= 4096`) ; au-delà, elle le refuse.
pub const CURSEUR_MAX: usize = 4096;
/// **Ce qu'un curseur porte en clair** : scellé, nonce et sceau compris, il tient en base64 sous
/// [`CURSEUR_MAX`].
pub const CURSEUR_CLAIR: usize = CURSEUR_MAX / 4 * 3 - NONCE - chiffre::SCEAU;
/// Le nom des données associées d'un curseur, qui ne se garde ni ne se numérote.
const CURSEUR: &str = "curseur";

pub(super) fn associees(salle: &str, local: &str) -> Vec<u8> {
    let mut a = Vec::with_capacity(salle.len() + 1 + local.len());
    a.extend_from_slice(salle.as_bytes());
    a.push(0);
    a.extend_from_slice(local.as_bytes());
    a
}

pub(super) fn sceller_un(cle: &[u8; CLE], nonce: [u8; NONCE], ad: &[u8], clair: &[u8]) -> String {
    let mut o = nonce.to_vec();
    o.extend(chiffre::sceller(cle, &nonce, ad, clair));
    base64_ecrire(&o)
}

fn ouvrir_un(cle: &[u8; CLE], ad: &[u8], o: &str) -> Option<Vec<u8>> {
    let octets = base64(o)?;
    let (nonce, scelle) = octets.split_at_checked(NONCE)?;
    chiffre::ouvrir(cle, nonce.try_into().ok()?, ad, scelle)
}

/// **Scelle une lettre** en morceaux : `(nom local, scellé en base64)`, dans l'ordre où ils partent.
/// `id` est le nom de la lettre (`<marque>-<compteur>`), unique pour ce participant.
pub fn sceller(
    cle: &[u8; CLE],
    salle: &str,
    id: &str,
    clair: &[u8],
    nonce: &mut dyn FnMut() -> [u8; NONCE],
) -> Vec<(String, String)> {
    let parts: Vec<&[u8]> = if clair.is_empty() {
        vec![&[]]
    } else {
        clair.chunks(MORCEAU).collect()
    };
    let total = parts.len();
    parts
        .into_iter()
        .enumerate()
        .map(|(rang, part)| {
            let local = format!("{id}_{rang}");
            let mut w = Writer::with_capacity(part.len() + 8);
            w.count(rang);
            w.count(total);
            w.raw(part);
            let o = sceller_un(cle, nonce(), &associees(salle, &local), &w.into_bytes());
            (local, o)
        })
        .collect()
}

/// Scelle un curseur : un seul morceau, ni gardé ni numéroté par la salle.
pub fn sceller_curseur(cle: &[u8; CLE], salle: &str, clair: &[u8], nonce: [u8; NONCE]) -> String {
    sceller_un(cle, nonce, &associees(salle, CURSEUR), clair)
}

/// Ouvre un curseur, ou rien.
pub fn ouvrir_curseur(cle: &[u8; CLE], salle: &str, o: &str) -> Option<Vec<u8>> {
    ouvrir_un(cle, &associees(salle, CURSEUR), o)
}

/// **Les lettres en train d'arriver**, morceau après morceau — plusieurs à la fois : les morceaux de
/// deux participants s'entrelacent dans l'ordre de la salle. Ce qui est reçu se garde à travers une
/// reconnexion : la salle ne renvoie que ce qui suit le dernier numéro vu, et le début d'une lettre
/// est peut-être avant.
#[derive(Debug, Default)]
pub struct Reassemblage {
    partiels: HashMap<String, Vec<Option<Vec<u8>>>>,
}

impl Reassemblage {
    /// **Reçoit un morceau** numéroté par la salle. Rend la lettre entière quand son dernier morceau
    /// manquant arrive : `(son nom, ses octets clairs)` — elle prend place, dans l'ordre de la salle,
    /// au numéro de ce morceau-là. Un morceau qui ne s'ouvre pas, ou qui ment sur son nom, est refusé.
    pub fn recevoir(
        &mut self,
        cle: &[u8; CLE],
        salle: &str,
        local: &str,
        o: &str,
    ) -> Result<Option<(String, Vec<u8>)>, String> {
        let clair = ouvrir_un(cle, &associees(salle, local), o)
            .ok_or("un morceau qui ne s'ouvre pas : autre clé, ou abîmé")?;
        let mut r = Reader::new(&clair);
        let lu = (|| Ok::<_, crate::error::CoreError>((r.count()?, r.count()?)))();
        let (rang, total) = lu.map_err(|_| "un morceau sans rang".to_string())?;
        let (id, suffixe) = local.rsplit_once('_').ok_or("un nom local sans rang")?;
        if suffixe != rang.to_string() || rang >= total || total > MORCEAUX_MAX {
            return Err(format!("un morceau qui ment sur son rang ({local})"));
        }
        let octets = r.take(r.remaining()).map_err(|e| e.to_string())?.to_vec();
        let partiel = self
            .partiels
            .entry(id.to_string())
            .or_insert_with(|| vec![None; total]);
        if partiel.len() != total {
            return Err(format!("deux totaux pour la lettre {id}"));
        }
        partiel[rang] = Some(octets);
        if partiel.iter().any(Option::is_none) {
            return Ok(None);
        }
        let entiere = self.partiels.remove(id).unwrap_or_default();
        Ok(Some((
            id.to_string(),
            entiere.into_iter().flatten().flatten().collect(),
        )))
    }
}
