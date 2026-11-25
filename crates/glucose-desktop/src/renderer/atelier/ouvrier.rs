//! **La vie d'un ouvrier de l'atelier** : prendre le plus urgent, le faire, le rendre — et ce
//! qu'il sait faire d'une image (la lire, l'encoder). Sorti d'`atelier.rs`, qui touchait son
//! plafond de 600 lignes (fiche 159).

use super::{admission, apercu, Commandes, Deplacement, Fait, Partis, Travail};
use crate::renderer::photo::Pyramide;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::Sender;
use std::time::Instant;

/// La vie d'un ouvrier : prendre le plus urgent, le faire, le rendre — jusqu'à la fermeture.
pub(super) fn ouvrier(commandes: &Commandes, retour: &Sender<Fait>, partis: &Partis) {
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
                let empreinte = objets.as_deref().and_then(|o| o.empreinte(&src));
                let pyramide = apercus
                    .and_then(|d| apercu::lire(&apercu::chemin(&d, &src, empreinte)?))
                    .and_then(Pyramide::depuis_apercu)
                    .or_else(|| decoder(&src, objets.as_deref(), admission::de_la_machine()));
                let gif = est_anime(&src, objets.as_deref());
                Fait::Decodee((src, pyramide, debut.elapsed(), gif))
            }
            Travail::Adopter(src, rgba, (l, h), parole) => {
                let debut = Instant::now();
                // Encodé en mémoire et **promis** : le scribe l'attend, aucun fichier ne passe
                // entre eux. Un fichier écrit ici, puis relu, s'est scellé vide le 28/09
                // (COLLER-2) ; une promesse ne se lit qu'entière.
                let tenue = encoder_en_png(&rgba, (l, h)).map(|png| parole.tenir(png));
                let pyramide = tenue.and_then(|()| Pyramide::depuis_rgba(l, h, &rgba));
                Fait::Decodee((src, pyramide, debut.elapsed(), false))
            }
            Travail::Ecrire(src, dossier, a, empreinte) => {
                // Un aperçu qui ne s'écrit pas n'est qu'un aperçu de moins : la source reste.
                // Un aperçu déjà là et qui se lit n'est pas réécrit : son nom dit la source
                // telle qu'elle est. Un aperçu abîmé, lui, se refait — sans quoi il resterait
                // muet pour toujours.
                let chemin = apercu::chemin(&dossier, &src, empreinte)
                    .filter(|c| !c.exists() || apercu::lire(c).is_none());
                if let Some(chemin) = chemin {
                    let _ = apercu::ecrire(&chemin, &a);
                }
                partis.ecritures.fetch_sub(1, Relaxed);
                continue;
            }
            Travail::Jeter(ce_qui_part, octets) => {
                drop(ce_qui_part);
                partis.octets.fetch_sub(octets, Relaxed);
                partis.jets.fetch_sub(1, Relaxed);
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

/// Lit une image et la rend prête à poser : décodée, prémultipliée, et réduite.
///
/// Les octets viennent du registre des objets — le document pour une image scellée, le
/// fichier d'origine sinon (HISTOIRE-1). Le format se reconnaît à la **signature** des
/// octets, jamais à une extension : une clé du document n'en a pas.
///
/// **Deux règles de la mémoire** (MEMOIRE-3, fiche 66). Le décodage n'entre qu'admis : ce que
/// les décodages en vol tiendront ne dépasse jamais la moitié de ce que la machine laisse — son
/// pic se lit dans l'en-tête, avant de décoder ([`admission`]). Et il écrit directement dans la
/// page du niveau natif ([`Pyramide::depuis_le_decodeur`]) : ni l'image décodée à part, ni sa
/// copie en RGBA, deux copies pleine taille de moins. Les couleurs qui ne s'étalent pas sur
/// place — seize bits, flottants — gardent le chemin d'avant, qui donne les mêmes octets.
pub(super) fn decoder(
    src: &str,
    objets: Option<&crate::persist::objets::Objets>,
    porte: &admission::Admission,
) -> Option<Pyramide> {
    use image::ImageDecoder;
    let octets = match objets {
        Some(o) => o.lire(src)?,
        None => std::fs::read(src).ok()?,
    };
    let decodeur = image::ImageReader::new(std::io::Cursor::new(&octets[..]))
        .with_guessed_format()
        .ok()?
        .into_decoder()
        .ok()?;
    let (l, h) = decodeur.dimensions();
    porte.pendant(admission::pic(octets.len() as u64, (l, h)), || {
        if Pyramide::octets_par_pixel(decodeur.color_type()).is_some() {
            return Pyramide::depuis_le_decodeur(decodeur);
        }
        let rgba = image::DynamicImage::from_decoder(decodeur).ok()?.to_rgba8();
        Pyramide::depuis_rgba(l, h, rgba.as_raw())
    })
}

/// Les pixels d'une image, encodés en PNG, en mémoire.
fn encoder_en_png(rgba: &[u8], (l, h): (u32, u32)) -> Option<Vec<u8>> {
    use image::ImageEncoder;
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(rgba, l, h, image::ExtendedColorType::Rgba8)
        .ok()?;
    Some(png)
}

/// **Cette image bouge-t-elle ?** Un GIF, un WebP ou un PNG animés, à leur signature : son début
/// seulement, autant qu'il en faut pour le dire (fiche 159 § 9) — une image rouverte vient de son
/// aperçu et ne passe jamais par le décodeur (GIF-2).
pub(super) fn est_anime(src: &str, objets: Option<&crate::persist::objets::Objets>) -> bool {
    use std::io::Read;
    let lire = |n: usize| match objets {
        Some(o) => o.debut(src, n),
        None => std::fs::File::open(src).ok().and_then(|f| {
            let mut debut = Vec::new();
            f.take(n as u64).read_to_end(&mut debut).ok()?;
            Some(debut)
        }),
    };
    let mut n = 64;
    loop {
        let Some(debut) = lire(n) else {
            return false;
        };
        match crate::renderer::gif::signature::animee(&debut) {
            Ok(anime) => return anime,
            // Le fichier finit avant : il ne le dira pas.
            Err(_) if debut.len() < n => return false,
            Err(besoin) => n = besoin.max(n.saturating_mul(2)),
        }
    }
}
