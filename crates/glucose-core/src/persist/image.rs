//! Sérialisation des images de tableau et de leurs références d'actif.
//!
//! Voir l'INVARIANT PERSIST-1 dans [`super::document`] : la déstructuration exhaustive est ce
//! qui garantit qu'un champ ajouté à `BoardImage` casse la build tant qu'il n'est pas écrit.

use super::bytes::{Reader, Writer};
use super::document::{
    read_domain_assignment, read_temporal_anchor, write_domain_assignment, write_temporal_anchor,
};
use crate::error::{CoreError, CoreResult};
use crate::types::{AssetRef, BoardImage};

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
        locked,
        tags,
        slot_id,
        source_url,
        original_width,
        original_height,
        is_video,
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
    w.opt_text(fit.as_ref());
    w.seq(domains, write_domain_assignment);
    w.opt_text(mirror_of.as_ref());
    w.opt(temporal_anchor.as_ref(), write_temporal_anchor);
}

pub fn read_image(r: &mut Reader<'_>) -> CoreResult<BoardImage> {
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
        locked: r.flag()?,
        tags: r.seq(|rr| rr.text())?,
        slot_id: r.opt_text()?,
        source_url: r.opt_text()?,
        original_width: r.f64()?,
        original_height: r.f64()?,
        is_video: r.flag()?,
        fit: r.opt_text()?,
        domains: r.seq(read_domain_assignment)?,
        mirror_of: r.opt_text()?,
        temporal_anchor: r.opt(read_temporal_anchor)?,
    })
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
