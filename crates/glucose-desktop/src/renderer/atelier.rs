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
//! # Ce que ce module ne fait pas
//!
//! Il ne décide pas quoi garder : le cache et sa borne appartiennent à l'appelant. Il ne
//! décide pas non plus ce qu'on dessine en attendant. Il répond à une seule question — « ces
//! octets, décodés, les voici » — et il y répond quand il peut.

use super::photo::Pyramide;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use tiny_skia::Pixmap;

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
type Decodee = (String, Option<Pyramide>, std::time::Duration);

/// Les fils qui décodent, et ce qu'ils ont en chantier.
pub struct Atelier {
    demandes: Sender<String>,
    prets: Receiver<Decodee>,
    /// Ce qui est parti au décodage et n'est pas revenu. Sert à ne pas redemander à chaque
    /// image la même photo — sans quoi une seule image en cours de décodage saturerait la
    /// file en quelques dixièmes de seconde.
    en_cours: HashSet<String>,
}

impl Default for Atelier {
    fn default() -> Self {
        Self::nouveau()
    }
}

impl Atelier {
    /// Ouvre l'atelier et lance ses ouvriers.
    pub fn nouveau() -> Self {
        let (demandes, file) = channel::<String>();
        let (retour, prets) = channel::<Decodee>();
        let file = Arc::new(Mutex::new(file));

        for _ in 0..ouvriers() {
            let file = Arc::clone(&file);
            let retour = retour.clone();
            // Un ouvrier vit tant que la file existe. Quand l'atelier est détruit, le `Sender`
            // tombe, `recv` rend une erreur et la boucle se termine d'elle-même : il n'y a ni
            // drapeau d'arrêt à lever, ni fil à attendre.
            std::thread::spawn(move || loop {
                let Ok(src) = file.lock().map_err(|_| ()).and_then(|f| f.recv().map_err(|_| ()))
                else {
                    return;
                };
                // Le temps que ce fichier a coûté est ce que sa reconstruction coûterait :
                // c'est exactement son utilité dans un cache, et elle se mesure ici plutôt
                // que de s'estimer ailleurs (ADAPT-1).
                let debut = std::time::Instant::now();
                let image = decoder(&src).map(Pyramide::nouvelle);
                if retour.send((src, image, debut.elapsed())).is_err() {
                    return;
                }
            });
        }

        Self {
            demandes,
            prets,
            en_cours: HashSet::new(),
        }
    }

    /// Demande le décodage de ce fichier, s'il n'est pas déjà en chantier.
    ///
    /// Rend `true` si la demande est partie — ce qui n'arrive qu'une fois par fichier tant
    /// qu'il n'est pas revenu.
    pub fn demander(&mut self, src: &str) -> bool {
        if self.en_cours.contains(src) {
            return false;
        }
        if self.demandes.send(src.to_string()).is_err() {
            return false;
        }
        self.en_cours.insert(src.to_string());
        true
    }

    /// Récolte tout ce qui est prêt, sans jamais attendre.
    ///
    /// C'est le seul point de contact entre les fils de fond et le rendu, et il ne bloque
    /// pas : ce qui n'est pas fini sera récolté à l'image suivante.
    pub fn recolter(&mut self) -> Vec<Decodee> {
        let mut moisson = Vec::new();
        while let Ok((src, image, cout)) = self.prets.try_recv() {
            self.en_cours.remove(&src);
            moisson.push((src, image, cout));
        }
        moisson
    }

    /// Combien d'images sont encore en chantier.
    ///
    /// C'est ce qui dit à la boucle d'événements de repasser bientôt : tant qu'il reste du
    /// travail, l'application a une raison de se réveiller même si l'utilisateur ne fait rien.
    pub fn en_travail(&self) -> usize {
        self.en_cours.len()
    }
}

/// Combien d'ouvriers, sur cette machine.
///
/// Tous les cœurs sauf un : celui qui reste tient la cadence, et il ne doit jamais se
/// retrouver en concurrence avec le décodage pour son propre temps. Au moins un ouvrier, même
/// sur une machine qui n'annonce qu'un cœur — sinon l'atelier ne décoderait jamais rien.
fn ouvriers() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1))
        .unwrap_or(1)
        .max(1)
}

/// Lit un fichier image et le rend prêt à poser : décodé, et prémultiplié.
///
/// La prémultiplication est faite ici, sur le fil de fond, parce qu'elle coûte autant que le
/// décodage sur une grande photo — la laisser au fil de rendu aurait déplacé le problème d'un
/// mètre.
fn decoder(src: &str) -> Option<Pixmap> {
    let chemin = std::path::Path::new(src);
    if !chemin.exists() {
        return None;
    }
    let rgba = image::open(chemin).ok()?.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut pixmap = Pixmap::new(w, h)?;
    let source = rgba.into_raw();
    let (source, _) = source.as_chunks::<4>();
    let (destination, _) = pixmap.data_mut().as_chunks_mut::<4>();

    for (px, out) in source.iter().zip(destination.iter_mut()) {
        let a = u32::from(px[3]);
        // Prémultiplication en entiers, arrondie au plus proche : `t = c·a + 128`, puis
        // `(t + (t >> 8)) >> 8`. Le décalage remplace la division par 255 exactement sur toute
        // la plage — c'est vérifié canal par canal, et `a = 255` redonne `c` sans écart.
        //
        // La version flottante qui était ici tronquait : un canal à 255 sur un pixel opaque
        // ressortait à 254. Invisible sur une photo, visible sur un aplat près d'une bordure.
        // Ma première réécriture se trompait dans l'autre sens — 128 devenait 129 — et c'est
        // le test qui l'a dit, pas la relecture.
        let premultiplie = |c: u8| {
            let t = u32::from(c) * a + 128;
            ((t + (t >> 8)) >> 8) as u8
        };
        *out = [
            premultiplie(px[0]),
            premultiplie(px[1]),
            premultiplie(px[2]),
            px[3],
        ];
    }
    Some(pixmap)
}

#[cfg(test)]
mod tests;
