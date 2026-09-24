//! **Qui occupe la mémoire du processus**, étage par étage (fiche 32).
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_memoire -- dossier/d/images
//! ```
//!
//! # La question
//!
//! Sa longue session du 23/09 finissait à **1 550 Mo** de mémoire de travail. Ses 45 épingles
//! décodées en pyramides en pèsent 235, les 32 images de son document 131 : les images
//! n'expliquent pas le quart du total. Construire une mémoire par étages sans savoir qui
//! occupe le reste serait déplacer ce qu'on ne voit pas.
//!
//! Ce banc refait le chemin de l'application, une marche à la fois, et relit ce que le système
//! compte après chacune : la mémoire de travail (ce que le gestionnaire des tâches montre), la
//! mémoire privée engagée, et ce que la carte graphique dit porter pour ce processus — dans sa
//! propre mémoire, et dans la mémoire partagée du processeur.
//!
//! Une marche absorbe tout ce qui la précède : chaque ligne dit donc ce qui a **changé**, et
//! non un total qu'on attribuerait à la dernière chose faite.

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
    use glucose_desktop::present::scene_gpu::SceneGpu;
    use glucose_desktop::renderer::magasin::Magasin;
    use windows::core::Interface;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIAdapter1, IDXGIAdapter3, IDXGIFactory1,
        DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL,
        DXGI_QUERY_VIDEO_MEMORY_INFO,
    };
    use windows::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows::Win32::System::Threading::GetCurrentProcess;

    const MO: f64 = 1_048_576.0;

    /// Ce que le système compte, à un instant.
    #[derive(Clone, Copy, Default)]
    struct Releve {
        travail: u64,
        privee: u64,
        carte_locale: u64,
        carte_partagee: u64,
    }

    fn processus() -> (u64, u64) {
        let mut c = PROCESS_MEMORY_COUNTERS_EX::default();
        let taille = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        // Sûr : la structure étendue commence par la structure de base, et sa taille est
        // déclarée ; l'appel n'écrit que des entiers.
        let ok = unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                (&raw mut c).cast::<PROCESS_MEMORY_COUNTERS>(),
                taille,
            )
        };
        if ok.is_err() {
            return (0, 0);
        }
        (c.WorkingSetSize as u64, c.PrivateUsage as u64)
    }

    fn adaptateur_dxgi(vendeur: u32, appareil: u32) -> Option<IDXGIAdapter3> {
        let fabrique: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.ok()?;
        (0..)
            .map_while(|rang| unsafe { fabrique.EnumAdapters1(rang) }.ok())
            .find(|a: &IDXGIAdapter1| {
                unsafe { a.GetDesc1() }
                    .is_ok_and(|d| d.VendorId == vendeur && d.DeviceId == appareil)
            })?
            .cast::<IDXGIAdapter3>()
            .ok()
    }

    fn carte(a: Option<&IDXGIAdapter3>) -> (u64, u64) {
        let Some(a) = a else { return (0, 0) };
        let lire = |groupe| {
            let mut vu = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
            unsafe { a.QueryVideoMemoryInfo(0, groupe, &mut vu) }
                .map_or(0, |()| vu.CurrentUsage)
        };
        (
            lire(DXGI_MEMORY_SEGMENT_GROUP_LOCAL),
            lire(DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL),
        )
    }

    fn relever(a: Option<&IDXGIAdapter3>) -> Releve {
        let (travail, privee) = processus();
        let (carte_locale, carte_partagee) = carte(a);
        Releve {
            travail,
            privee,
            carte_locale,
            carte_partagee,
        }
    }

    fn dire(marche: &str, avant: Releve, apres: Releve) {
        let d = |a: u64, b: u64| (b as f64 - a as f64) / MO;
        println!(
            "  {marche:<46} travail {:>+8.1}  privee {:>+8.1}  carte {:>+8.1}  partagee {:>+8.1}   \
             (travail {:>7.1} Mo)",
            d(avant.travail, apres.travail),
            d(avant.privee, apres.privee),
            d(avant.carte_locale, apres.carte_locale),
            d(avant.carte_partagee, apres.carte_partagee),
            apres.travail as f64 / MO,
        );
    }

    fn images(dossier: &str) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(dossier)
            .map(|d| {
                d.filter_map(Result::ok)
                    .map(|e| e.path().to_string_lossy().into_owned())
                    .filter(|p| {
                        let p = p.to_lowercase();
                        p.ends_with(".jpg") || p.ends_with(".png") || p.ends_with(".jpeg")
                    })
                    .collect()
            })
            .unwrap_or_default();
        v.sort();
        v
    }

    fn ouvrir_la_carte() -> Option<(wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adaptateur =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
                ..Default::default()
            }))
            .ok()?;
        // Ce que l'application demande, à l'identique (`present::gpu::ouverture`).
        let limites = adaptateur.limits();
        let (p, f) = pollster::block_on(adaptateur.request_device(&wgpu::DeviceDescriptor {
            label: Some("banc memoire"),
            required_features: wgpu::Features::empty(),
            required_limits: limites,
            // Le reglage de l'application ; `PERFORMANCE=1` rejoue l'ancien pour comparer.
            memory_hints: if std::env::var_os("PERFORMANCE").is_some() {
                wgpu::MemoryHints::Performance
            } else {
                wgpu::MemoryHints::MemoryUsage
            },
            ..Default::default()
        }))
        .ok()?;
        Some((adaptateur, p, f))
    }

    fn attendre(p: &wgpu::Device) {
        let _ = p.poll(wgpu::PollType::wait_indefinitely());
    }

    pub fn main() {
        let Some(dossier) = std::env::args().nth(1) else {
            println!("donner le dossier des images");
            return;
        };
        let chemins = images(&dossier);
        println!("{} images dans {dossier}\n", chemins.len());

        let depart = relever(None);
        let Some((adaptateur, peripherique, file)) = ouvrir_la_carte() else {
            println!("aucune carte");
            return;
        };
        let info = adaptateur.get_info();
        let dxgi = adaptateur_dxgi(info.vendor, info.device);
        println!("carte : {} ({:?})\n", info.name, info.backend);
        let mut avant = relever(dxgi.as_ref());
        dire("ouvrir la carte", depart, avant);

        // Les pyramides, par le vrai magasin et son atelier.
        let mut magasin = Magasin::nouveau();
        magasin.ouvrir();
        let t = std::time::Instant::now();
        for c in &chemins {
            let _ = magasin.pyramide(c);
        }
        magasin.attendre_le_chantier();
        let mural = t.elapsed();
        let apres = relever(dxgi.as_ref());
        dire("decoder toutes les pyramides", avant, apres);
        println!(
            "    le magasin dit porter {:.1} Mo pour {} images, decodees en {:.0} ms par l'atelier",
            magasin.octets() as f64 / MO,
            magasin.cache.len(),
            mural.as_secs_f64() * 1e3
        );
        redescendre_au_disque(&chemins);
        avant = apres;

        let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Bgra8Unorm);
        let apres = relever(dxgi.as_ref());
        dire("construire la scene graphique", avant, apres);
        avant = apres;

        // Le niveau qui couvre une photo posée à 400 pixels : ce que la carte reçoit au repos.
        let t = std::time::Instant::now();
        televerser_tout(&mut scene, (&peripherique, &file), &magasin, Some(400.0));
        attendre(&peripherique);
        println!("    243 textures creees et remplies en {:.1} ms", t.elapsed().as_secs_f64() * 1e3);
        let apres = relever(dxgi.as_ref());
        dire("envoyer le niveau pour 400 px", avant, apres);
        avant = apres;

        // Les originaux : ce que la carte reçoit quand on s'approche de chacune.
        televerser_tout(&mut scene, (&peripherique, &file), &magasin, None);
        attendre(&peripherique);
        let apres = relever(dxgi.as_ref());
        dire("remplacer par les originaux", avant, apres);
        avant = apres;

        // Dix allers-retours : un geste de zoom qui change de palier sans cesse.
        let t = std::time::Instant::now();
        for tour in 0..10 {
            let largeur = if tour % 2 == 0 { Some(400.0) } else { None };
            televerser_tout(&mut scene, (&peripherique, &file), &magasin, largeur);
            attendre(&peripherique);
        }
        println!("    dix changements de palier en {:.1} ms", t.elapsed().as_secs_f64() * 1e3);
        let apres = relever(dxgi.as_ref());
        dire("dix changements de palier", avant, apres);
        avant = apres;

        drop(scene);
        attendre(&peripherique);
        let apres = relever(dxgi.as_ref());
        dire("oublier toutes les textures", avant, apres);
        avant = apres;

        avant = etager(&mut magasin, &chemins, avant, dxgi.as_ref());

        drop(magasin);
        let apres = relever(dxgi.as_ref());
        dire("oublier toutes les pyramides", avant, apres);
        avant = apres;

        drop((peripherique, file, adaptateur));
        let apres = relever(dxgi.as_ref());
        dire("fermer la carte", avant, apres);
    }

    /// **ETAGES-1, mesuré** : ce que la mémoire vive tient quand l'écran montre chaque photo
    /// à 150 pixels, quand il n'en montre plus aucune, et ce que coûte de tout faire revenir.
    fn etager(
        magasin: &mut Magasin,
        chemins: &[String],
        mut avant: Releve,
        dxgi: Option<&IDXGIAdapter3>,
    ) -> Releve {
        use glucose_desktop::renderer::photo::Etat;
        let image = |magasin: &mut Magasin, largeur: Option<f32>| {
            magasin.ouvrir();
            for c in chemins {
                if let (Some(l), true) = (largeur, magasin.reclamer(c, 0.0)) {
                    let _ = magasin.cache[c.as_str()].pyramide.meilleur_pour(l);
                }
            }
            magasin.fermer();
            let t = std::time::Instant::now();
            magasin.attendre_le_chantier();
            t.elapsed()
        };
        let dire_les_etages = |magasin: &Magasin| {
            println!(
                "    tenus {:.0} Mo, offerts {:.0} Mo",
                magasin.octets_en(Etat::Tenu) as f64 / MO,
                magasin.octets_en(Etat::Offert) as f64 / MO
            );
        };
        for (marche, largeur) in [
            ("etager : l'ecran les montre a 150 px", Some(150.0)),
            ("etager : l'ecran n'en montre plus", None),
        ] {
            image(magasin, largeur);
            // Une image de plus : ce que la precedente a lu reste tenu jusqu'a celle-ci.
            let duree = image(magasin, largeur);
            let apres = relever(dxgi);
            dire(marche, avant, apres);
            dire_les_etages(magasin);
            println!("    l'atelier a fini en {:.0} ms", duree.as_secs_f64() * 1e3);
            avant = apres;
        }
        let duree = image(magasin, Some(150.0));
        let apres = relever(dxgi);
        dire("etager : tout revient a 150 px", avant, apres);
        dire_les_etages(magasin);
        println!(
            "    reprises faites en {:.0} ms par l'atelier -- {:?}",
            duree.as_secs_f64() * 1e3,
            magasin.mouvements()
        );
        apres
    }

    /// **Ce que coûte une image que le système a reprise** : la relire et la décoder, sur un
    /// seul fil, comme l'atelier le fait pour chacune — par format, et rapporté au mégapixel.
    fn redescendre_au_disque(chemins: &[String]) {
        let mut par_format: std::collections::BTreeMap<String, (f64, f64, usize)> =
            Default::default();
        for c in chemins {
            let t = std::time::Instant::now();
            let Ok(image) = image::open(c) else { continue };
            let rgba = image.to_rgba8();
            let ms = t.elapsed().as_secs_f64() * 1e3;
            let mpx = f64::from(rgba.width()) * f64::from(rgba.height()) / 1e6;
            let format = c.rsplit('.').next().unwrap_or("?").to_lowercase();
            let e = par_format.entry(format).or_default();
            *e = (e.0 + ms, e.1 + mpx, e.2 + 1);
        }
        for (format, (ms, mpx, n)) in par_format {
            println!(
                "    redecoder un {format} sur un fil : {:.1} ms par image, {:.1} ms par Mpx ({n} images)",
                ms / n as f64,
                ms / mpx
            );
        }
    }

    /// Envoie chaque photo du magasin, au niveau qui couvre `largeur`, ou native.
    fn televerser_tout(
        scene: &mut SceneGpu,
        (peripherique, file): (&wgpu::Device, &wgpu::Queue),
        magasin: &Magasin,
        largeur: Option<f32>,
    ) {
        scene.ouvrir();
        for (src, entree) in &magasin.cache {
            let niveau = match largeur {
                Some(l) => entree.pyramide.niveau_pour(l),
                None => entree.pyramide.native(),
            };
            let Some(niveau) = niveau else { continue };
            let cle = format!("{src}@{}", niveau.width());
            scene.televerser(peripherique, file, (src, &cle), niveau);
        }
        file.submit([]);
    }
}
