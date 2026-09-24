//! **Ce que l'œil verra après `Ctrl+B`**, sur de VRAIES images, rendu par le vrai code
//! (BORDURES-4).
//!
//! # Pourquoi ce banc
//!
//! `bench_bordures` dit ce que la détection **retire**. Il ne pouvait pas dire ce que l'écran
//! **montre** — et c'est là que le liseré de la forêt de l'utilisateur vivait pour moitié : le
//! fichier recadré était propre à droite, et sa capture y montrait pourtant une colonne claire.
//! Le filtre lisait la colonne que `Ctrl+B` venait de retirer.
//!
//! Ce banc détecte, puis rend le recadrage par la pyramide et le report de l'application, à
//! quatre zooms, **avec** la fenêtre de lecture et **sans** elle. Pour chaque bord, il écrit la
//! luminosité du rang montré contre celle du rang d'à côté : un liseré est un bord qui s'en
//! écarte. Il ne choisit rien — c'est en le lisant, et en regardant les images, qu'on juge.
//!
//! ```text
//!     cargo run -p glucose-desktop --release --example apercu_recadrage -- <fichiers...>
//!     GLUCOSE_APERCU=<dossier> ...   écrit aussi chaque rendu en PNG
//! ```

use glucose_core::bordures;
use glucose_core::occlusion::Boite;
use glucose_core::report::{self, Pixel, Vue, VueMut};
use glucose_core::types::Recadrage;
use glucose_desktop::renderer::photo::Pyramide;
use tiny_skia::Pixmap;

/// Les zooms essayés : un niveau réduit de la pyramide, l'échelle de la capture de
/// l'utilisateur, la taille native, et un zoom proche où un demi-texel fait plusieurs pixels.
const ZOOMS: [f64; 4] = [0.45, 0.613, 1.0, 2.5];

fn main() {
    let sortie = std::env::var("GLUCOSE_APERCU").ok();
    for f in std::env::args().skip(1) {
        let Some(source) = charger(&f) else {
            println!("{f} : illisible");
            continue;
        };
        let (l, h) = (source.width(), source.height());
        let (texels, _) = source.data().as_chunks::<4>();
        let crop =
            Vue::nouvelle(texels, l, h).map_or(Recadrage::ENTIER, |v| bordures::detecter(&v));
        let (g, ht, d, b) = crop.marges();
        let px = |part: f64, dim: u32| (part * f64::from(dim)).round();
        println!(
            "\n{f}  ({l} x {h})  retire : gauche {}  haut {}  droite {}  bas {}",
            px(g, l),
            px(ht, h),
            px(d, l),
            px(b, h)
        );
        let pyramide = Pyramide::nouvelle(source);
        for zoom in ZOOMS {
            for borne in [true, false] {
                let (rendu, (cl, ch)) = rendre(&pyramide, crop, zoom, borne);
                println!(
                    "  zoom {zoom:5.3}  {}  {}",
                    if borne { "fenetre " } else { "sans    " },
                    bords(&rendu, cl, ch)
                );
                if let (Some(dossier), true) = (&sortie, borne) {
                    let nom = std::path::Path::new(&f)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let chemin = format!("{dossier}/{nom}-zoom{zoom}.png");
                    if let Err(e) = rendu.save_png(&chemin) {
                        println!("  {chemin} : {e}");
                    }
                }
            }
        }
    }
}

/// Un fichier image, en pixels prémultipliés comme le reste du projet les voit.
fn charger(chemin: &str) -> Option<Pixmap> {
    let img = image::ImageReader::open(chemin)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    let mut p = Pixmap::new(img.width(), img.height())?;
    for (dst, src) in p
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(img.pixels())
    {
        let [r, g, b, a] = src.0;
        let m = |c: u8| ((u16::from(c) * u16::from(a) + 127) / 255) as u8;
        *dst = [m(r), m(g), m(b), a];
    }
    Some(p)
}

/// Le recadrage rendu comme l'application le pose : la source entière sur une boîte plus
/// grande, la boîte du nœud pour clip, au niveau de pyramide qu'elle choisirait. Rend l'image
/// et la taille de la boîte, posée à une phase non entière dans une marge transparente.
fn rendre(pyramide: &Pyramide, crop: Recadrage, zoom: f64, borne: bool) -> (Pixmap, (u32, u32)) {
    let natives = pyramide.dimensions_natives();
    let (bl, bh) = (
        f64::from(natives.0) * crop.largeur_visible() * zoom,
        f64::from(natives.1) * crop.hauteur_visible() * zoom,
    );
    let boite = (10.3, 10.6, bl, bh);
    let (x, y, vw, vh) = crop.source_pour(boite);
    let niveau = pyramide
        .niveau_pour(vw as f32)
        .expect("une pyramide neuve tient tous ses niveaux");
    let fenetre = crop.texels_lisibles(natives, pyramide.facteur_pour(vw as f32));
    let (dl, dh) = (bl.ceil() as u32 + 21, bh.ceil() as u32 + 22);
    let mut dest = Pixmap::new(dl, dh).expect("un rendu");
    let (texels, _) = niveau.data().as_chunks::<4>();
    let vue = Vue::nouvelle(texels, niveau.width(), niveau.height()).expect("une vue");
    let vue = if borne {
        vue.avec_fenetre(fenetre)
    } else {
        vue
    };
    let (pixels, _) = dest.data_mut().as_chunks_mut::<4>();
    let mut cible = VueMut::nouvelle(pixels, dl, dh).expect("une cible");
    let pose = report::Pose {
        x: x as f32,
        y: y as f32,
        largeur: vw as f32,
        hauteur: vh as f32,
    };
    let clip = Boite::nouvelle(boite.0 as f32, boite.1 as f32, bl as f32, bh as f32);
    report::reporter(
        &mut cible,
        &vue,
        pose,
        clip,
        report::Melange::Remplacer,
        report::Filtre::Lisse,
    );
    (dest, (dl, dh))
}

/// Pour chaque bord, la luminosité du rang montré, puis celle du rang d'à côté.
fn bords(rendu: &Pixmap, l: u32, h: u32) -> String {
    let (px, _) = rendu.data().as_chunks::<4>();
    let lum = |p: &Pixel| (u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2])) as f64 / 3.0;
    // Les rangs et colonnes réellement écrits : ceux dont un pixel n'est pas transparent.
    let ecrit = |x: u32, y: u32| px[(y * l + x) as usize][3] != 0;
    let lignes: Vec<u32> = (0..h).filter(|&y| (0..l).any(|x| ecrit(x, y))).collect();
    let colonnes: Vec<u32> = (0..l).filter(|&x| (0..h).any(|y| ecrit(x, y))).collect();
    let (Some(&y0), Some(&y1), Some(&x0), Some(&x1)) = (
        lignes.first(),
        lignes.last(),
        colonnes.first(),
        colonnes.last(),
    ) else {
        return "rien d'ecrit".into();
    };
    let ligne = |y: u32| {
        (x0..=x1)
            .map(|x| lum(&px[(y * l + x) as usize]))
            .sum::<f64>()
            / f64::from(x1 - x0 + 1)
    };
    let colonne = |x: u32| {
        (y0..=y1)
            .map(|y| lum(&px[(y * l + x) as usize]))
            .sum::<f64>()
            / f64::from(y1 - y0 + 1)
    };
    format!(
        "haut {:5.1} ({:5.1})  bas {:5.1} ({:5.1})  gauche {:5.1} ({:5.1})  droite {:5.1} ({:5.1})",
        ligne(y0),
        ligne(y0 + 1),
        ligne(y1),
        ligne(y1 - 1),
        colonne(x0),
        colonne(x0 + 1),
        colonne(x1),
        colonne(x1 - 1)
    )
}
