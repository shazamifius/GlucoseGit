//! Sérialisation des images de tableau et de leurs références d'actif.
//!
//! Voir l'INVARIANT PERSIST-1 dans [`super::document`] : la déstructuration exhaustive est ce
//! qui garantit qu'un champ ajouté à `BoardImage` casse la build tant qu'il n'est pas écrit.

use super::bytes::{Reader, Writer};
use super::document::{
    read_domain_assignment, read_temporal_anchor, write_domain_assignment, write_temporal_anchor,
};
use crate::error::{CoreError, CoreResult};
use crate::types::{AssetRef, BoardImage, ChoixVideo, Recadrage};

pub fn write_image(w: &mut Writer, img: &BoardImage) {
    let BoardImage {
        id,
        membrane_id,
        asset,
        src,
        x,
        y,
        width,
        height,
        rotation,
        miroir,
        locked,
        tags,
        slot_id,
        source_url,
        original_width,
        original_height,
        is_video,
        video,
        affiche,
        crop,
        fit,
        domains,
        mirror_of,
        temporal_anchor,
    } = img;

    w.text(id);
    w.opt_text(membrane_id.as_ref());
    w.opt(asset.as_ref(), write_asset_ref);
    w.opt_text(src.as_ref());
    w.f64(*x);
    w.f64(*y);
    w.f64(*width);
    w.f64(*height);
    w.f64(*rotation);
    w.flag(*locked);
    w.seq(tags, |ww, tag| ww.text(tag));
    w.opt_text(slot_id.as_ref());
    w.opt_text(source_url.as_ref());
    w.f64(*original_width);
    w.f64(*original_height);
    w.flag(*is_video);
    // RECADRAGE-1, schema de document v2. Quatre fractions toujours ecrites : un recadrage
    // entier en vaut quatre nuls, ce qui est le meme etat et se lit sans branche.
    let (g, h, d, b) = crop.marges();
    for part in [g, h, d, b] {
        w.f64(part);
    }
    w.opt_text(fit.as_ref());
    w.seq(domains, write_domain_assignment);
    w.opt_text(mirror_of.as_ref());
    w.opt(temporal_anchor.as_ref(), write_temporal_anchor);
    // MIROIR-1, schéma v6 : à la fin, pour qu'un lecteur v5 n'ait rien d'autre à sauter.
    w.flag(*miroir);
    // VIDEO-2, schéma v7 : écrits seulement là ; une vidéo qui a des choix l'exige.
    if w.schema() >= crate::persist::VIDEO_DEPUIS {
        w.flag(video.lecture_auto);
        w.flag(video.a_cote);
    } else if !video.est_neutre() {
        w.exiger(crate::persist::VIDEO_DEPUIS);
    }
    // Fiche 195, schéma v8 : l'affiche d'une vidéo à côté.
    if w.schema() >= crate::persist::AFFICHE_DEPUIS {
        w.opt_text(affiche.as_ref());
    } else if affiche.is_some() {
        w.exiger(crate::persist::AFFICHE_DEPUIS);
    }
}

/// **Lit une image du schema `version`.**
///
/// C'est la premiere migration chainee que le § 8 de [`super`] annonce, et elle est de la
/// forme la plus simple qui soit : un champ apparu en v2 se lit en v2 et vaut son neutre en
/// v1. Le lecteur ne DEVINE rien -- un champ optionnel deduit de ce qui reste d'octets serait
/// un format qui se relit a l'envers.
pub fn read_image(r: &mut Reader<'_>, version: u16) -> CoreResult<BoardImage> {
    let mut image = read_image_v5(r, version)?;
    image.miroir = version >= crate::persist::MIROIR_DEPUIS && r.flag()?;
    if version >= crate::persist::VIDEO_DEPUIS {
        image.video = ChoixVideo {
            lecture_auto: r.flag()?,
            a_cote: r.flag()?,
        };
    }
    if version >= crate::persist::AFFICHE_DEPUIS {
        image.affiche = r.opt_text()?;
    }
    Ok(image)
}

/// Les champs d'une image jusqu'au schéma v5, dans l'ordre où ils s'écrivent.
fn read_image_v5(r: &mut Reader<'_>, version: u16) -> CoreResult<BoardImage> {
    Ok(BoardImage {
        id: r.text()?,
        membrane_id: r.opt_text()?,
        asset: r.opt(read_asset_ref)?,
        src: r.opt_text()?,
        x: r.f64()?,
        y: r.f64()?,
        width: r.f64()?,
        height: r.f64()?,
        rotation: r.f64()?,
        // Lu à la fin de la notice (schéma v6), posé ici après coup.
        miroir: false,
        locked: r.flag()?,
        tags: r.seq(|rr| rr.text())?,
        slot_id: r.opt_text()?,
        source_url: r.opt_text()?,
        original_width: r.f64()?,
        original_height: r.f64()?,
        is_video: r.flag()?,
        // Lus à la fin de la notice (schéma v7), posés après coup.
        video: ChoixVideo::default(),
        affiche: None,
        crop: read_crop(r, version)?,
        fit: r.opt_text()?,
        domains: r.seq(read_domain_assignment)?,
        mirror_of: r.opt_text()?,
        temporal_anchor: r.opt(read_temporal_anchor)?,
    })
}

/// Le recadrage d'une image, apparu au schema v2 (RECADRAGE-1).
///
/// Un document v1 n'en porte aucun octet, et ses images sont entieres -- ce qu'elles etaient
/// avant que le geste n'existe. La reconstruction passe par le meme constructeur que tout le
/// reste, donc un fichier trafique ne peut pas introduire un recadrage qui ne laisse rien.
fn read_crop(r: &mut Reader<'_>, version: u16) -> CoreResult<Recadrage> {
    if version < crate::persist::RECADRAGE_DEPUIS {
        return Ok(Recadrage::ENTIER);
    }
    let (gauche, haut, droite, bas) = (r.f64()?, r.f64()?, r.f64()?, r.f64()?);
    Ok(Recadrage::depuis_les_marges(gauche, haut, droite, bas))
}

const ASSET_EMBED: u8 = 0;
const ASSET_LINK: u8 = 1;

fn write_asset_ref(w: &mut Writer, asset: &AssetRef) {
    match asset {
        AssetRef::Embed {
            sha256,
            mime,
            size_bytes,
        } => {
            w.u8(ASSET_EMBED);
            w.text(sha256);
            w.text(mime);
            w.opt_u64(*size_bytes);
        }
        AssetRef::Link {
            href,
            sha256,
            size_bytes,
        } => {
            w.u8(ASSET_LINK);
            w.text(href);
            w.opt_text(sha256.as_ref());
            w.opt_u64(*size_bytes);
        }
    }
}

fn read_asset_ref(r: &mut Reader<'_>) -> CoreResult<AssetRef> {
    match r.u8()? {
        ASSET_EMBED => Ok(AssetRef::Embed {
            sha256: r.text()?,
            mime: r.text()?,
            size_bytes: r.opt_u64()?,
        }),
        ASSET_LINK => Ok(AssetRef::Link {
            href: r.text()?,
            sha256: r.opt_text()?,
            size_bytes: r.opt_u64()?,
        }),
        other => Err(CoreError::DeserializationError(format!(
            "type de référence d'actif inconnu ({other}) : ce document vient d'une version plus récente de Glucose, mets-la à jour"
        ))),
    }
}
