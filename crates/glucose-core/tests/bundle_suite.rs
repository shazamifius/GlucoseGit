//! Tests du Bundle Portable, SHA-256 et Assets — 100% Rust Standard Library (std)
//! Ports de src/utils/bundle.test.ts, assetRef.test.ts et assets.test.ts

use glucose_core::bundle::{
    asset_bytes_match, build_bundle_manifest, collect_referenced_assets, data_url_to_bytes,
    ext_from_mime, mime_from_ext, resolve_asset_src, sha256_hex, BUNDLE_FORMAT, BUNDLE_VERSION,
};
use glucose_core::types::{AssetRef, BoardImage, Project};

fn mk_project(boards_images: Vec<Vec<BoardImage>>) -> Project {
    let mut p = Project::new("mon projet");
    p.boards.clear();
    for (i, imgs) in boards_images.into_iter().enumerate() {
        let mut b = glucose_core::types::Board::new(format!("b{}", i), format!("B{}", i));
        b.images = imgs;
        p.boards.push(b);
    }
    p
}

fn base_img(id: &str) -> BoardImage {
    BoardImage::new(id, 0.0, 0.0, 10.0, 10.0)
}

fn link_img(id: &str, name: &str, sha256: Option<&str>, size_bytes: Option<u64>) -> BoardImage {
    let mut img = base_img(id);
    img.asset = Some(AssetRef::Link {
        href: format!("asset:{}", name),
        sha256: sha256.map(|s| s.to_string()),
        size_bytes,
    });
    img
}

// ════════════════════════════════════════════════════════════════════
// collect_referenced_assets — Cœur pur
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_collecte_les_refs_asset_mode_link_avec_sha256_et_taille() {
    let full_sha = format!("aaa0000011112222{}", "0".repeat(48));
    let img = link_img("i1", "aaa0000011112222.png", Some(&full_sha), Some(1234));
    let p = mk_project(vec![vec![img]]);

    let assets = collect_referenced_assets(&p);
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].name, "aaa0000011112222.png");
    assert_eq!(assets[0].sha256.as_deref(), Some(full_sha.as_str()));
    assert_eq!(assets[0].size_bytes, Some(1234));
}

#[test]
fn test_collecte_aussi_le_champ_legacy_src_asset() {
    let mut img = base_img("i2");
    img.src = Some("asset:bbb1.jpg".into());
    let p = mk_project(vec![vec![img]]);

    let assets = collect_referenced_assets(&p);
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].name, "bbb1.jpg");
}

#[test]
fn test_deduplique_un_meme_asset_sur_plusieurs_images() {
    let p = mk_project(vec![
        vec![
            link_img("i1", "dup.png", None, None),
            link_img("i2", "dup.png", None, None),
        ],
        vec![link_img("i3", "dup.png", None, None)],
    ]);
    assert_eq!(collect_referenced_assets(&p).len(), 1);
}

#[test]
fn test_ignore_les_embed_et_liens_non_asset() {
    let mut embed = base_img("e1");
    embed.asset = Some(AssetRef::Embed {
        sha256: "deadbeef".into(),
        mime: "image/png".into(),
        size_bytes: None,
    });

    let mut web = base_img("w1");
    web.asset = Some(AssetRef::Link {
        href: "https://site/x.png".into(),
        sha256: None,
        size_bytes: None,
    });

    let mut file = base_img("f1");
    file.asset = Some(AssetRef::Link {
        href: "C:/Users/x/photo.png".into(),
        sha256: None,
        size_bytes: None,
    });

    let keep = link_img("k1", "keep.png", None, None);
    let p = mk_project(vec![vec![embed, web, file, keep]]);

    let assets = collect_referenced_assets(&p);
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].name, "keep.png");
}

#[test]
fn test_renvoie_un_ordre_deterministe_trie_par_nom() {
    let p = mk_project(vec![vec![
        link_img("i1", "zzz.png", None, None),
        link_img("i2", "aaa.png", None, None),
        link_img("i3", "mmm.png", None, None),
    ]]);
    let assets = collect_referenced_assets(&p);
    let names: Vec<&str> = assets.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, vec!["aaa.png", "mmm.png", "zzz.png"]);
}

#[test]
fn test_projet_sans_image_liste_vide() {
    let p = mk_project(vec![vec![]]);
    assert_eq!(collect_referenced_assets(&p).len(), 0);
}

// ════════════════════════════════════════════════════════════════════
// build_bundle_manifest
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_produit_un_manifeste_au_bon_format() {
    let p = mk_project(vec![vec![link_img("i1", "a.png", None, None)]]);
    let assets = collect_referenced_assets(&p);
    let m = build_bundle_manifest(&p, assets);

    assert_eq!(m.format, BUNDLE_FORMAT);
    assert_eq!(m.version, BUNDLE_VERSION);
    assert_eq!(m.name, "mon projet");
    assert_eq!(m.doc, "project.glucose");
    assert_eq!(m.assets.len(), 1);
}

// ════════════════════════════════════════════════════════════════════
// asset_bytes_match
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_asset_bytes_match_vrai_quand_le_nom_encode_le_hash() {
    let bytes = [1, 2, 3, 4, 5];
    let full = sha256_hex(&bytes);
    let name = format!("{}.png", &full[..16]);
    assert!(asset_bytes_match(&name, &bytes, None));
    assert!(asset_bytes_match(&name, &bytes, Some(&full)));
}

#[test]
fn test_asset_bytes_match_faux_si_stem_ne_correspond_pas() {
    let bytes = [9, 9, 9];
    assert!(!asset_bytes_match("ffff000011112222.png", &bytes, None));
}

#[test]
fn test_asset_bytes_match_faux_si_sha_attendu_differe() {
    let bytes = [1, 2, 3, 4, 5];
    let full = sha256_hex(&bytes);
    let name = format!("{}.png", &full[..16]);
    let wrong_sha = "0".repeat(64);
    assert!(!asset_bytes_match(&name, &bytes, Some(&wrong_sha)));
}

// ════════════════════════════════════════════════════════════════════
// sha256_hex & data_url_to_bytes
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_sha256_hex_produit_64_chars_stables() {
    let data = [1, 2, 3, 4, 5];
    let h1 = sha256_hex(&data);
    assert_eq!(h1.len(), 64);
    assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    let h2 = sha256_hex(&data);
    assert_eq!(h1, h2);

    let diff = sha256_hex(&[0]);
    assert_ne!(h1, diff);
}

#[test]
fn test_data_url_to_bytes_decode_1x1_transparent_png() {
    let data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";
    let res = data_url_to_bytes(data_url);
    assert!(res.is_ok());
    let (bytes, mime) = res.unwrap();
    assert_eq!(mime, "image/png");
    assert!(bytes.len() > 50);
    // Magic bytes PNG: 89 50 4E 47 0D 0A 1A 0A
    assert_eq!(bytes[0], 0x89);
    assert_eq!(bytes[1], 0x50);
    assert_eq!(bytes[2], 0x4e);
    assert_eq!(bytes[3], 0x47);
}

#[test]
fn test_data_url_to_bytes_rejette_non_data_urls() {
    let res = data_url_to_bytes("https://x.com/a.png");
    assert!(res.is_err());
}

#[test]
fn test_mime_from_ext_et_inverse() {
    assert_eq!(mime_from_ext("png"), "image/png");
    assert_eq!(mime_from_ext(".jpg"), "image/jpeg");
    assert_eq!(mime_from_ext("JPEG"), "image/jpeg");
    assert_eq!(mime_from_ext("mp4"), "video/mp4");
    assert_eq!(mime_from_ext("unknownext"), "application/octet-stream");

    assert_eq!(ext_from_mime("image/jpeg"), "jpg");
    assert_eq!(ext_from_mime("image/png"), "png");
    assert_eq!(ext_from_mime("application/octet-stream"), "bin");
}

// ════════════════════════════════════════════════════════════════════
// resolve_asset_src
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_resolve_asset_src_routage() {
    let data = "data:image/png;base64,AAAABBBBCCCC";
    assert_eq!(resolve_asset_src(data, None, None), data);

    assert_eq!(
        resolve_asset_src("https://exemple.com/y.png", None, None),
        "https://exemple.com/y.png"
    );
    assert_eq!(
        resolve_asset_src("http://exemple.com/y.png", None, None),
        "http://exemple.com/y.png"
    );

    assert_eq!(
        resolve_asset_src("asset:abc.jpg", Some("C:/assets"), None),
        "asset-src://C:/assets/abc.jpg"
    );

    assert_eq!(
        resolve_asset_src("C:/photos/x.png", None, None),
        "asset-src://C:/photos/x.png"
    );

    assert_eq!(
        resolve_asset_src("images/x.png", None, Some("C:/proj/notes.glucose")),
        "asset-src://C:/proj/images/x.png"
    );

    assert_eq!(resolve_asset_src("", None, None), "");
}
