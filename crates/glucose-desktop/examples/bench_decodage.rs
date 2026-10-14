//! **Le pic de mémoire d'un décodage**, chemin d'avant contre décodage sur place (MEMOIRE-3,
//! fiche 66).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_decodage
//! ```
//!
//! # La question
//!
//! L'atelier décodait une photo en trois copies pleine taille vivantes en même temps : l'image
//! décodée, sa copie en RGBA, la pyramide. L'arithmétique donne un pic d'environ 9,3 octets par
//! pixel, et 5,3 en décodant dans la page du niveau natif. Une arithmétique n'est pas une
//! mesure : ce banc décode une photo de cinquante mégapixels — celle d'un moto g54 — **chaque
//! chemin dans son propre processus**, et lit le pic que Windows a compté (`PeakWorkingSetSize`).
//! Un seul processus mélangerait les deux pics et ce que l'allocateur garde de l'un pour l'autre.
//!
//! # Ce qu'il a mesuré (14/10, son PC)
//!
//! ```text
//!     JPEG de 50 Mpx         chemin d'avant   sur place   pyramide
//!     ordinaire                  445 Mo        254 Mo      254 Mo
//!     progressif 4:2:0           445 Mo        292 Mo
//!     progressif 4:4:4           445 Mo        437 Mo
//! ```
//!
//! Les progressifs se donnent en argument (l'encodeur d'`image` n'en écrit pas) : par exemple
//! ceux de Pillow, `progressive=True`.

#[cfg(windows)]
fn main() {
    windows_seulement::main();
}

#[cfg(not(windows))]
fn main() {
    println!("ce banc lit les compteurs de Windows ; il ne mesure rien ailleurs");
}

#[cfg(windows)]
mod windows_seulement {
    use glucose_desktop::renderer::photo::Pyramide;
    use image::{ImageDecoder, ImageEncoder};
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;

    /// Une photo de téléphone : 8 160 × 6 120 pixels, un dégradé bruité qui ne se compresse pas
    /// en rien.
    const COTES: (u32, u32) = (8_160, 6_120);

    fn pic_mo() -> f64 {
        let mut c = PROCESS_MEMORY_COUNTERS {
            cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        // Sûr : la structure est locale et dimensionnée par son propre champ.
        let _ = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
        c.PeakWorkingSetSize as f64 / (1024.0 * 1024.0)
    }

    fn photo() -> std::path::PathBuf {
        let chemin = std::env::temp_dir().join("glucose-bench-decodage-50mpx.jpg");
        if !chemin.exists() {
            let (l, h) = COTES;
            let mut brut = vec![0u8; (l * h * 3) as usize];
            for (i, px) in brut.as_chunks_mut::<3>().0.iter_mut().enumerate() {
                let (x, y) = (i as u32 % l, i as u32 / l);
                let bruit = (i as u32).wrapping_mul(2_654_435_761) >> 27;
                *px = [
                    (x / 32 % 256) as u8 ^ bruit as u8,
                    (y / 24 % 256) as u8,
                    ((x + y) / 50 % 256) as u8,
                ];
            }
            let mut sortie = std::fs::File::create(&chemin).expect("la photo s'écrit");
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut sortie, 90)
                .write_image(&brut, l, h, image::ExtendedColorType::Rgb8)
                .expect("la photo s'encode");
        }
        chemin
    }

    fn decoder(chemin: &std::path::Path, sur_place: bool) {
        let octets = std::fs::read(chemin).expect("la photo se lit");
        let avant = pic_mo();
        let pyramide = if sur_place {
            let d = image::ImageReader::new(std::io::Cursor::new(&octets[..]))
                .with_guessed_format()
                .expect("format")
                .into_decoder()
                .expect("décodeur");
            assert!(Pyramide::octets_par_pixel(d.color_type()).is_some());
            Pyramide::depuis_le_decodeur(d)
        } else {
            let rgba = image::load_from_memory(&octets)
                .expect("décodée")
                .to_rgba8();
            let (l, h) = rgba.dimensions();
            Pyramide::depuis_rgba(l, h, rgba.as_raw())
        }
        .expect("la pyramide");
        let pyramide_mo = pyramide.octets() as f64 / (1024.0 * 1024.0);
        println!(
            "{:<22} pic {:>7.0} Mo   (avant le décodage {:>4.0} Mo, pyramide {:>4.0} Mo, fichier {:.0} Mo)",
            if sur_place { "décodage sur place" } else { "chemin d'avant" },
            pic_mo() - avant,
            avant,
            pyramide_mo,
            octets.len() as f64 / (1024.0 * 1024.0),
        );
    }

    /// Sans argument : la photo du banc, puis chaque JPEG donné en plus — un progressif, par
    /// exemple, que l'encodeur d'`image` ne sait pas écrire —, chaque chemin dans son processus.
    pub fn main() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        match args.first().map(String::as_str) {
            Some(mode @ ("avant" | "sur-place")) => {
                decoder(std::path::Path::new(&args[1]), mode == "sur-place");
            }
            _ => {
                let photos = std::iter::once(photo()).chain(args.iter().map(Into::into));
                let moi = std::env::current_exe().expect("ce banc");
                for chemin in photos {
                    println!("{}", chemin.display());
                    for mode in ["avant", "sur-place"] {
                        let sortie = std::process::Command::new(&moi)
                            .arg(mode)
                            .arg(&chemin)
                            .output()
                            .expect("un processus par chemin");
                        print!("{}", String::from_utf8_lossy(&sortie.stdout));
                    }
                }
            }
        }
    }
}
