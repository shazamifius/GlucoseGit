//! Git #1 (north star « indestructible ») — BUNDLE PORTABLE.
//! 100% Rust Standard Library (0 dépendance).

use crate::types::{AssetRef, Project};
use std::collections::HashMap;

pub const BUNDLE_FORMAT: &str = "glucose-bundle";
pub const BUNDLE_VERSION: u32 = 1;
pub const BUNDLE_DOC_NAME: &str = "project.glucose";
pub const BUNDLE_MANIFEST_NAME: &str = "bundle.json";
pub const BUNDLE_OBJECTS_DIR: &str = "objects";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencedAsset {
    pub name: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
}

/// Énumère TOUS les assets `asset:<name>` référencés par un projet (dédupliqués, triés). PUR : aucune I/O.
pub fn collect_referenced_assets(project: &Project) -> Vec<ReferencedAsset> {
    let mut by_name: HashMap<String, ReferencedAsset> = HashMap::new();

    for board in &project.boards {
        for img in &board.images {
            if let Some(AssetRef::Link {
                href,
                sha256,
                size_bytes,
            }) = &img.asset
            {
                if let Some(stripped) = href.strip_prefix("asset:") {
                    by_name
                        .entry(stripped.to_string())
                        .or_insert_with(|| ReferencedAsset {
                            name: stripped.to_string(),
                            sha256: sha256.clone(),
                            size_bytes: *size_bytes,
                        });
                }
            } else if let Some(ref src) = img.src {
                if let Some(stripped) = src.strip_prefix("asset:") {
                    by_name
                        .entry(stripped.to_string())
                        .or_insert_with(|| ReferencedAsset {
                            name: stripped.to_string(),
                            sha256: None,
                            size_bytes: None,
                        });
                }
            }
        }
    }

    let mut out: Vec<ReferencedAsset> = by_name.into_values().collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleManifest {
    pub format: String,
    pub version: u32,
    pub name: String,
    pub doc: String,
    pub assets: Vec<ReferencedAsset>,
    pub created_at: i64,
}

pub fn build_bundle_manifest(project: &Project, assets: Vec<ReferencedAsset>) -> BundleManifest {
    BundleManifest {
        format: BUNDLE_FORMAT.to_string(),
        version: BUNDLE_VERSION,
        name: project.name.clone(),
        doc: BUNDLE_DOC_NAME.to_string(),
        assets,
        created_at: project.created_at,
    }
}

pub fn asset_bytes_match(name: &str, bytes: &[u8], expected_sha: Option<&str>) -> bool {
    let full = sha256_hex(bytes);
    if let Some(expected) = expected_sha {
        if full != expected {
            return false;
        }
    }
    let stem = if let Some(idx) = name.rfind('.') {
        &name[..idx]
    } else {
        name
    };
    if stem.len() == 16 || stem.len() == 64 {
        full.starts_with(stem)
    } else {
        true
    }
}

pub fn mime_from_ext(ext: &str) -> &'static str {
    let clean = ext.trim_start_matches('.').to_lowercase();
    match clean.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

pub fn ext_from_mime(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        _ => "bin",
    }
}

pub fn base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    const TABLE: [i8; 256] = {
        let mut t = [-1i8; 256];
        let mut i = 0u8;
        while i < 26 {
            t[(b'A' + i) as usize] = i as i8;
            t[(b'a' + i) as usize] = (i + 26) as i8;
            i += 1;
        }
        let mut j = 0u8;
        while j < 10 {
            t[(b'0' + j) as usize] = (j + 52) as i8;
            j += 1;
        }
        t[b'+' as usize] = 62;
        t[b'/' as usize] = 63;
        t
    };

    let bytes: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    if !bytes.len().is_multiple_of(4) {
        return Err("Invalid base64 length");
    }

    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    let (quads, _) = bytes.as_chunks::<4>();
    for chunk in quads {
        let b0 = TABLE[chunk[0] as usize];
        let b1 = TABLE[chunk[1] as usize];
        let b2 = if chunk[2] == b'=' {
            0
        } else {
            TABLE[chunk[2] as usize]
        };
        let b3 = if chunk[3] == b'=' {
            0
        } else {
            TABLE[chunk[3] as usize]
        };

        if b0 < 0 || b1 < 0 || b2 < 0 || b3 < 0 {
            return Err("Invalid base64 character");
        }

        let triple = ((b0 as u32) << 18) | ((b1 as u32) << 12) | ((b2 as u32) << 6) | (b3 as u32);
        out.push(((triple >> 16) & 0xff) as u8);
        if chunk[2] != b'=' {
            out.push(((triple >> 8) & 0xff) as u8);
        }
        if chunk[3] != b'=' {
            out.push((triple & 0xff) as u8);
        }
    }
    Ok(out)
}

pub fn data_url_to_bytes(data_url: &str) -> Result<(Vec<u8>, String), &'static str> {
    if !data_url.starts_with("data:") {
        return Err("non-data URL");
    }
    let rest = &data_url["data:".len()..];
    let (meta, encoded) = rest.split_once(',').ok_or("Malformed data URL")?;
    let mime = if let Some((m, _)) = meta.split_once(';') {
        m.to_string()
    } else if meta.is_empty() {
        "text/plain;charset=US-ASCII".to_string()
    } else {
        meta.to_string()
    };
    let bytes = base64_decode(encoded)?;
    Ok((bytes, mime))
}

pub fn resolve_asset_src(
    src: &str,
    assets_dir: Option<&str>,
    current_project_path: Option<&str>,
) -> String {
    if src.is_empty()
        || src.starts_with("data:")
        || src.starts_with("http://")
        || src.starts_with("https://")
    {
        return src.to_string();
    }
    if let Some(file_name) = src.strip_prefix("asset:") {
        let dir = assets_dir.unwrap_or("C:/assets");
        return format!("asset-src://{}/{}", dir.trim_end_matches('/'), file_name);
    }
    if src.starts_with('/') || (src.len() >= 2 && src.chars().nth(1) == Some(':')) {
        return format!("asset-src://{}", src);
    }
    if let Some(proj) = current_project_path {
        if let Some(parent) = std::path::Path::new(proj).parent() {
            let parent_str = parent.to_string_lossy().replace('\\', "/");
            return format!("asset-src://{}/{}", parent_str.trim_end_matches('/'), src);
        }
    }
    format!("asset-src://{}", src)
}

/// Implémentation SHA-256 standard (FIPS 180-4 / RFC 6234) en 100% Rust std pur (0 crate).
pub fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h0: u32 = 0x6a09e667;
    let mut h1: u32 = 0xbb67ae85;
    let mut h2: u32 = 0x3c6ef372;
    let mut h3: u32 = 0xa54ff53a;
    let mut h4: u32 = 0x510e527f;
    let mut h5: u32 = 0x9b05688c;
    let mut h6: u32 = 0x1f83d9ab;
    let mut h7: u32 = 0x5be0cd19;

    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while !(msg.len() + 8).is_multiple_of(64) {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    let (blocks, _) = msg.as_chunks::<64>();
    for chunk in blocks {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        let mut f = h5;
        let mut g = h6;
        let mut h = h7;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
        h5 = h5.wrapping_add(f);
        h6 = h6.wrapping_add(g);
        h7 = h7.wrapping_add(h);
    }

    let mut out = [0u8; 32];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out[20..24].copy_from_slice(&h5.to_be_bytes());
    out[24..28].copy_from_slice(&h6.to_be_bytes());
    out[28..32].copy_from_slice(&h7.to_be_bytes());
    out
}

pub fn sha256_hex(data: &[u8]) -> String {
    hex_of(&sha256(data))
}

/// Écriture hexadécimale minuscule d'une empreinte — le nom sous lequel un actif est rangé.
///
/// Table de chiffres plutôt que `write!` : formater dans une `String` ne peut pas échouer, mais
/// l'API de `write!` rend un `Result` qu'il faudrait `unwrap()`, ce que le standard § 6.2
/// interdit hors invariant prouvé. La table supprime la question.
pub fn hex_of(digest: &[u8; 32]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hello() {
        assert_eq!(
            sha256_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hex_of_writes_lowercase_and_pads_every_byte() {
        assert_eq!(hex_of(&[0u8; 32]), "0".repeat(64));
        assert_eq!(hex_of(&[0x0f; 32]), "0f".repeat(32));
        assert_eq!(hex_of(&sha256(b"hello")), sha256_hex(b"hello"));
    }

    #[test]
    fn test_collect_referenced_assets() {
        let mut proj = Project::new("test");
        let mut img1 = crate::types::BoardImage::new("i1", 0.0, 0.0, 100.0, 100.0);
        img1.asset = Some(AssetRef::Link {
            href: "asset:avatar.png".into(),
            sha256: Some("sha_avatar".into()),
            size_bytes: Some(1024),
        });

        let mut img2 = crate::types::BoardImage::new("i2", 0.0, 0.0, 100.0, 100.0);
        img2.src = Some("asset:avatar.png".into()); // Déduplication

        let mut img3 = crate::types::BoardImage::new("i3", 0.0, 0.0, 100.0, 100.0);
        img3.asset = Some(AssetRef::Link {
            href: "asset:background.jpg".into(),
            sha256: None,
            size_bytes: None,
        });

        proj.boards[0].images = vec![img1, img2, img3];

        let assets = collect_referenced_assets(&proj);
        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].name, "avatar.png");
        assert_eq!(assets[0].sha256.as_deref(), Some("sha_avatar"));
        assert_eq!(assets[1].name, "background.jpg");
    }
}
