//! **Une photo d'iPhone montrée avant d'être décodée, par sa vignette** (HEIC-3, fiche 161 § 8).
//!
//! Une photo d'iPhone se décode en 0,43 s sur un fil ici, autour d'une seconde sur une petite
//! machine : trois à cinq fois un JPEG de même taille. Mais l'iPhone range dans chaque photo sa
//! **vignette** (l'élément `thmb`, 320 × 240), qui se décode en 2 ms. À la première vue d'une
//! photo, elle en devient l'**aperçu**, par le chemin de ceux du disque (ETAGES-4) : une pyramide
//! partielle, ses niveaux plus grands que la vignette perdus — l'écran qui les veut fait décoder
//! l'image entière. Rien de neuf : la photo paraît tout de suite, en moins bien, puis nette.
//!
//! # Ses couleurs
//!
//! `heifer::decode_thumbnail` rend la vignette dans les couleurs du fichier — le Display P3 de
//! l'iPhone —, sans la convertir en sRGB comme l'image entière : elle paraîtrait terne, puis la
//! photo s'aviverait d'un coup. Cette conversion n'est pas publique ; `heifer` la fait donc sur
//! une copie du fichier dont l'**image première** (`pitm`) est la vignette — le chemin de l'image
//! entière, au bit près ; la copie, un mégaoctet, coûte moins que la vignette.

use crate::renderer::apercu::Apercu;
use crate::renderer::photo::{dimensions_au_rang, Pyramide};
use heifer::heifer_isobmff::HeifFile;
use std::ops::Range;

/// **L'aperçu de cette photo par sa vignette** — ou rien : pas de vignette, une vignette qui
/// n'est pas l'image, un fichier illisible. L'image entière se décode alors, comme avant.
pub(crate) fn apercu(octets: &[u8]) -> Option<Apercu> {
    let natives = super::dimensions(octets)?;
    let vignette = decoder_la_vignette(octets)?;
    let (rang, (l, h)) = rang_de_la_vignette(natives, (vignette.width, vignette.height))?;
    let tampon = image::RgbaImage::from_raw(vignette.width, vignette.height, vignette.data)?;
    let reduite = image::imageops::resize(&tampon, l, h, image::imageops::FilterType::Triangle);
    let petite = Pyramide::depuis_rgba(l, h, reduite.as_raw())?.apercu(0)?;
    Some(Apercu {
        natives,
        rang,
        ..petite
    })
}

/// **Le premier niveau de l'image que la vignette couvre sans s'agrandir**, et ses dimensions —
/// ou rien si la vignette n'en couvre aucun sous l'image entière (elle ne servirait à rien), ou
/// si elle n'a pas les proportions de l'image : son petit côté doit tomber, à un pixel près, où
/// son grand côté le place. Au-delà, ce n'est pas la même image (un recadrage, un quart de tour
/// que la vignette n'a pas).
pub(super) fn rang_de_la_vignette(
    natives: (u32, u32),
    (vl, vh): (u32, u32),
) -> Option<(usize, (u32, u32))> {
    let (nl, nh) = (u64::from(natives.0), u64::from(natives.1));
    let ecart = (u64::from(vl) * nh).abs_diff(u64::from(vh) * nl);
    if ecart > nl.max(nh) {
        return None;
    }
    (1..)
        .map_while(|rang| Some((rang, dimensions_au_rang(natives, rang)?)))
        .find(|&(_, (l, h))| l <= vl && h <= vh)
}

/// **La vignette, en sRGB sur huit bits** comme l'image entière : la plus petite, s'il y en a
/// plusieurs (comme `heifer::decode_thumbnail`), décodée en image première d'une copie.
fn decoder_la_vignette(octets: &[u8]) -> Option<heifer::Image<u8>> {
    let fichier = HeifFile::parse(octets).ok()?;
    let surface = |id| {
        let taille = fichier.image_size(id).ok().flatten();
        taille.map_or(u64::MAX, |(l, h)| u64::from(l) * u64::from(h))
    };
    let vignette = fichier
        .referencing_items(fichier.primary_id, b"thmb")
        .into_iter()
        .min_by_key(|&id| surface(id))?;
    let mut copie = octets.to_vec();
    designer_l_image_premiere(&mut copie, vignette)?;
    let un_fil = heifer::Options::new().with_max_threads(1);
    heifer::decode_srgb_rgba8(&copie, &un_fil).ok()
}

/// **Fait de cet élément l'image première du fichier** : son numéro dans la boîte `pitm`, que
/// la boîte `meta` contient (ISO/IEC 14496-12 § 8.11.4) — sur deux octets, ou quatre en version 1.
pub(super) fn designer_l_image_premiere(octets: &mut [u8], id: u32) -> Option<()> {
    let meta = boite(octets, 0..octets.len(), b"meta")?;
    // `meta` est une boîte pleine : sa version et ses drapeaux précèdent ses enfants.
    let pitm = boite(octets, meta.start + 4..meta.end, b"pitm")?;
    let champ = pitm.start + 4;
    match octets.get(pitm.start)? {
        0 => octets
            .get_mut(champ..champ + 2)?
            .copy_from_slice(&u16::try_from(id).ok()?.to_be_bytes()),
        1 => octets
            .get_mut(champ..champ + 4)?
            .copy_from_slice(&id.to_be_bytes()),
        _ => return None,
    }
    Some(())
}

/// **Le contenu de la première boîte de cette sorte** dans cette étendue, après son en-tête ; une
/// taille qui sort de l'étendue rend rien, jamais une lecture au-delà.
fn boite(octets: &[u8], etendue: Range<usize>, sorte: &[u8; 4]) -> Option<Range<usize>> {
    let mot = |i: usize| -> Option<u32> {
        Some(u32::from_be_bytes(octets.get(i..i + 4)?.try_into().ok()?))
    };
    let mut i = etendue.start;
    while i + 8 <= etendue.end {
        let (entete, taille) = match mot(i)? {
            // Jusqu'à la fin de l'étendue.
            0 => (8, etendue.end - i),
            // La taille sur huit octets, après la sorte.
            1 => (
                16,
                usize::try_from((u64::from(mot(i + 8)?) << 32) | u64::from(mot(i + 12)?)).ok()?,
            ),
            t => (8, usize::try_from(t).ok()?),
        };
        let fin = i.checked_add(taille)?;
        if taille < entete || fin > etendue.end {
            return None;
        }
        if octets.get(i + 4..i + 8)? == sorte {
            return Some(i + entete..fin);
        }
        i = fin;
    }
    None
}
