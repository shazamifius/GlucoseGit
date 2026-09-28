//! ENVOI-1 : ce qui part par les tampons qui restent arrive **exactement** où on l'envoie — un
//! rectangle au milieu d'une texture dont les rangées ne tombent pas sur 256 octets, lu dans une
//! source plus large que lui — et rien d'autre ne bouge.
//!
//! Sur une machine sans carte utilisable, l'épreuve se saute : l'absence de matériel n'est pas
//! un défaut du code.

use super::Envoi;
use crate::present::banc_gpu;

/// Une texture qu'on peut relire.
fn texture(peripherique: &wgpu::Device, (l, h): (u32, u32)) -> wgpu::Texture {
    peripherique.create_texture(&wgpu::TextureDescriptor {
        label: Some("envoi: épreuve"),
        size: wgpu::Extent3d {
            width: l,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// Des octets qui disent d'où ils viennent : chaque texel porte sa colonne, sa rangée et une
/// marque.
fn motif((l, h): (u32, u32), marque: u8) -> Vec<u8> {
    (0..h)
        .flat_map(|y| (0..l).flat_map(move |x| [x as u8, y as u8, marque, 255]))
        .collect()
}

#[test]
fn test_envoi_1_un_rectangle_arrive_ou_on_l_envoie_et_rien_d_autre_ne_bouge() {
    let Some((peripherique, file)) = banc_gpu::carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    // 37 texels de large : 148 octets par rangée, loin des 256 qu'une copie exige.
    let (l, h) = (37, 23);
    let cible = texture(&peripherique, (l, h));
    let mut envoi = Envoi::nouveau(&peripherique, &file);
    let fond = motif((l, h), 7);
    envoi.texture(&cible, (0, 0), (l, h), (&fond, l as usize * 4));
    envoi.soumettre();

    // Un rectangle de 12 × 8, pris au coin (2, 1) d'une source de 50 texels de large, posé en
    // (5, 3) : la source a son propre pas de rangée, et l'origine porte le décalage.
    let (sl, sh) = (50, 20);
    let source = motif((sl, sh), 200);
    let depart = (sl as usize + 2) * 4;
    envoi.texture(
        &cible,
        (5, 3),
        (12, 8),
        (&source[depart..], sl as usize * 4),
    );
    envoi.soumettre();

    let lu = banc_gpu::relire(&peripherique, &file, &cible, (l, h)).expect("relue");
    for y in 0..h {
        for x in 0..l {
            let i = ((y * l + x) * 4) as usize;
            let dedans = (5..17).contains(&x) && (3..11).contains(&y);
            let attendu = if dedans {
                [(x - 5 + 2) as u8, (y - 3 + 1) as u8, 200, 255]
            } else {
                [x as u8, y as u8, 7, 255]
            };
            assert_eq!(lu.data()[i..i + 4], attendu, "le texel ({x}, {y})");
        }
    }
}
