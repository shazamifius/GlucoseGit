//! Ce que la voie graphique doit garantir : qu'elle pose la photo **où on le lui dit**, dans
//! le bon sens, et qu'elle compose comme le noyau.
//!
//! # Pourquoi ces tests ouvrent une vraie carte
//!
//! Un nuanceur ne se relit pas : une erreur de signe sur l'axe vertical, un coin inversé, un
//! mélange qui n'est pas prémultiplié — rien de tout cela ne se voit dans le code, et tout se
//! voit à l'écran. La seule épreuve qui vaille est de dessiner et de relire les pixels.
//!
//! Sur une machine sans carte utilisable, ils se **sautent** au lieu d'échouer : l'absence de
//! matériel n'est pas un défaut du code.

use super::*;
use crate::renderer::voies::APoser;

/// Un périphérique hors fenêtre, ou `None` si cette machine n'en offre pas.
fn carte() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adaptateur = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: false,
        compatible_surface: None,
        ..Default::default()
    }))
    .ok()?;
    pollster::block_on(adaptateur.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

/// Une photo unie et opaque, de la couleur donnée.
fn photo(cote: u32, couleur: [u8; 4]) -> Pixmap {
    let mut p = Pixmap::new(cote, cote).expect("une photo");
    for bloc in p.data_mut().as_chunks_mut::<4>().0 {
        *bloc = couleur;
    }
    p
}

/// Ce qu'une photo pose : son identité est sa clé, puisque ses octets ne changent jamais.
fn a_poser(cle: &str, pose: Pose) -> APoser {
    APoser {
        repli: None,
        cle: cle.to_string(),
        identite: cle.to_string(),
        pose,
    }
}

/// Dessine `poses` dans une cible de `cote` pixels, et rend ses octets RGBA.
fn rendre(cote: u32, poses: &[(String, Pose)], sources: &[(&str, Pixmap)]) -> Option<Vec<u8>> {
    let (peripherique, file) = carte()?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut scene = SceneGpu::nouvelle(&peripherique, format);
    scene.ouvrir();
    for (cle, source) in sources {
        scene.televerser(&peripherique, &file, (cle, cle), source.as_ref());
    }
    let poses: Vec<APoser> = poses.iter().map(|(c, p)| a_poser(c, *p)).collect();
    let retenues = scene.preparer(&peripherique, &file, (cote as f32, cote as f32), &poses);

    let taille = wgpu::Extent3d {
        width: cote,
        height: cote,
        depth_or_array_layers: 1,
    };
    let cible = peripherique.create_texture(&wgpu::TextureDescriptor {
        label: Some("cible"),
        size: taille,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let vue = cible.create_view(&wgpu::TextureViewDescriptor::default());
    // Une ligne de copie s'aligne sur 256 octets : la cible est choisie en consequence.
    let par_ligne = cote * 4;
    assert_eq!(
        par_ligne % 256,
        0,
        "le test choisit un cote qui aligne la copie"
    );
    let lecture = peripherique.create_buffer(&wgpu::BufferDescriptor {
        label: Some("lecture"),
        size: u64::from(par_ligne * cote),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encodeur =
        peripherique.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut passe = encodeur.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scene"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &vue,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        scene.poser(&mut passe, &retenues, 0..retenues.len());
    }
    encodeur.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &cible,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &lecture,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(par_ligne),
                rows_per_image: Some(cote),
            },
        },
        taille,
    );
    file.submit(Some(encodeur.finish()));
    lecture.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = peripherique.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    let octets = lecture
        .slice(..)
        .get_mapped_range()
        .expect("la lecture")
        .to_vec();
    lecture.unmap();
    Some(octets)
}

fn pixel(octets: &[u8], cote: u32, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * cote + x) * 4) as usize;
    [octets[i], octets[i + 1], octets[i + 2], octets[i + 3]]
}

/// **La photo se pose exactement où on le dit, et dans le bon sens.**
///
/// L'axe vertical est le piège : les coordonnées de dessin montent quand celles de l'écran
/// descendent. Un signe inversé passe toutes les compilations et met l'image à l'envers.
#[test]
fn test_une_photo_se_pose_ou_on_le_dit() {
    let cote = 64;
    let sources = vec![("rouge", photo(8, [200, 0, 0, 255]))];
    let poses = vec![(
        "rouge".to_string(),
        Pose {
            x: 16.0,
            y: 8.0,
            largeur: 32.0,
            hauteur: 16.0,
            opacite: 1.0,
            angle: 0.0,
            fenetre: Pose::TOUT,
            bornes: Pose::PARTOUT,
        },
    )];
    let Some(octets) = rendre(cote, &poses, &sources) else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };

    // Au centre de la boite : la photo.
    assert_eq!(pixel(&octets, cote, 32, 16), [200, 0, 0, 255], "le centre");
    // Juste AU-DESSUS du bord haut : rien. C'est ce qui attrape l'axe inverse.
    assert_eq!(pixel(&octets, cote, 32, 4), [0, 0, 0, 0], "au-dessus");
    // Juste en dessous du bord bas : rien non plus.
    assert_eq!(pixel(&octets, cote, 32, 30), [0, 0, 0, 0], "en dessous");
    // A gauche du bord gauche : rien.
    assert_eq!(pixel(&octets, cote, 8, 16), [0, 0, 0, 0], "a gauche");
}

/// **L'opacite compose en premultiplie**, comme `Melange::Composer` du noyau.
///
/// Si le nuanceur multipliait la couleur sans multiplier l'alpha — l'erreur classique — le
/// resultat serait plus sombre que la moitie, et deux voies rendraient des pixels differents
/// pour la meme scene.
#[test]
fn test_l_opacite_compose_en_premultiplie() {
    let cote = 64;
    let sources = vec![("blanc", photo(8, [255, 255, 255, 255]))];
    let poses = vec![(
        "blanc".to_string(),
        Pose {
            x: 0.0,
            y: 0.0,
            largeur: 64.0,
            hauteur: 64.0,
            opacite: 0.5,
            angle: 0.0,
            fenetre: Pose::TOUT,
            bornes: Pose::PARTOUT,
        },
    )];
    let Some(octets) = rendre(cote, &poses, &sources) else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let vu = pixel(&octets, cote, 32, 32);
    // Sur un fond transparent, un blanc a moitie opaque donne du blanc premultiplie a moitie :
    // les quatre canaux valent 128, a l'arrondi pres.
    for (canal, valeur) in vu.iter().enumerate() {
        assert!(
            valeur.abs_diff(128) <= 2,
            "canal {canal} vaut {valeur}, attendu 128 : la composition n'est pas premultipliee"
        );
    }
}

/// Le magasin oublie ce qui n'a pas servi, et retient ce qui a servi — la même loi que le
/// cache de tuiles, et la seule qui borne la mémoire sans choisir un nombre.
#[test]
fn test_le_magasin_oublie_ce_qui_n_a_pas_servi() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &file,
        ("a", "a"),
        photo(4, [1, 2, 3, 255]).as_ref(),
    );
    scene.televerser(
        &peripherique,
        &file,
        ("b", "b"),
        photo(4, [4, 5, 6, 255]).as_ref(),
    );
    assert!(scene.connait("a", "a") && scene.connait("b", "b"));

    // Une image ou seule `a` sert.
    scene.ouvrir();
    let pose = Pose {
        x: 0.0,
        y: 0.0,
        largeur: 4.0,
        hauteur: 4.0,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
        bornes: Pose::PARTOUT,
    };
    scene.preparer(&peripherique, &file, (8.0, 8.0), &[a_poser("a", pose)]);
    scene.fermer(0);

    assert!(scene.connait("a", "a"), "ce qui a servi reste");
    assert!(!scene.connait("b", "b"), "ce qui n'a pas servi est oublie");
}

/// **Un quart de tour transpose la photo autour de son centre.**
///
/// Sans ce test, l'angle pourrait tourner autour du COIN -- l'erreur la plus courante -- et
/// rien ne le dirait : une photo carree non tournee passe les deux epreuves precedentes.
///
/// La photo choisie est large et plate, et les deux pixels lus sont dehors dans un cas et
/// dedans dans l'autre : ils ne peuvent pas etre justes tous les deux par accident.
#[test]
fn test_un_quart_de_tour_tourne_autour_du_centre() {
    let cote = 64;
    let sources = vec![("vert", photo(8, [0, 180, 0, 255]))];
    // Sans rotation : x de 16 a 48, y de 28 a 36. Centre en (32, 32).
    let poses = vec![(
        "vert".to_string(),
        Pose {
            x: 16.0,
            y: 28.0,
            largeur: 32.0,
            hauteur: 8.0,
            opacite: 1.0,
            angle: std::f32::consts::FRAC_PI_2,
            fenetre: Pose::TOUT,
            bornes: Pose::PARTOUT,
        },
    )];
    let Some(octets) = rendre(cote, &poses, &sources) else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    // Tournee d'un quart de tour autour du centre : x de 28 a 36, y de 16 a 48.
    assert_eq!(
        pixel(&octets, cote, 32, 20),
        [0, 180, 0, 255],
        "dedans une fois tournee, dehors sans rotation"
    );
    assert_eq!(
        pixel(&octets, cote, 20, 32),
        [0, 0, 0, 0],
        "dehors une fois tournee, dedans sans rotation"
    );
}

/// **CASCADE-2 : un ancien palier se pose tant que le nouveau n'est pas prêt.**
///
/// C'est la propriété qui rend le report sûr. Sans elle, borner le rendu des textures ferait
/// **disparaître** les composants pas encore refaits — et un trou est infiniment pire qu'un
/// flou d'une image.
///
/// # Ce test porte sa preuve
///
/// Il rejoue l'ancienne loi — une mémoire indexée par **clé** — en demandant la clé neuve à
/// une carte qui ne détient que l'ancienne, et vérifie que cette question-là répond `false`
/// pendant que la question par **identité** répond `true`. Sur l'ancienne implémentation, la
/// pose ne trouvait rien et le composant n'était pas dessiné du tout.
#[test]
fn test_cascade_l_ancien_palier_se_pose_tant_que_le_nouveau_manque() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    // La carte détient la carte de texte `c1` à son ancien palier.
    scene.televerser(
        &peripherique,
        &file,
        ("carte:c1", "carte:c1:ancien"),
        photo(4, [1, 2, 3, 255]).as_ref(),
    );

    // La scène en demande maintenant un palier neuf.
    assert!(
        !scene.connait("carte:c1", "carte:c1:neuf"),
        "la clé neuve n'est pas détenue : c'est ce qui la fera rendre quand il y aura du temps"
    );
    assert!(
        scene.detient("carte:c1"),
        "l'identité, elle, est détenue -- et c'est ce qui evite le trou"
    );

    // Et elle se pose : l'ancienne texture, à la place et à la taille demandées.
    let pose = Pose {
        x: 0.0,
        y: 0.0,
        largeur: 8.0,
        hauteur: 8.0,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
        bornes: Pose::PARTOUT,
    };
    let retenues = scene.preparer(
        &peripherique,
        &file,
        (16.0, 16.0),
        &[APoser {
            repli: None,
            cle: "carte:c1:neuf".to_string(),
            identite: "carte:c1".to_string(),
            pose,
        }],
    );
    assert_eq!(
        retenues.cles,
        vec!["carte:c1".to_string()],
        "l'ancien palier doit se poser ; sous l'ancienne loi, indexee par cle, il disparaissait"
    );
}

/// **L'absent passe avant le périmé, et au moins un se rend toujours.**
///
/// L'ordre des deux tours est la priorité : sans texture un composant ne se dessine pas, donc
/// l'urgent mange le budget en premier. Mais le budget vaut pour lui aussi — quatre cent
/// quatre-vingts cartes qui entrent ensemble coûtaient 57 ms sur une image, et geler un
/// vingtième de seconde se voit plus que deux cents cartes qui paraissent une image plus tard.
///
/// Un budget nul est le cas extrême qui montre les deux règles à la fois : l'absent est servi,
/// le périmé attend, et **au moins une texture se rend** pour qu'une scène finisse toujours
/// par se compléter.
#[test]
fn test_cascade_le_budget_reporte_le_perime_et_sert_l_absent_d_abord() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &file,
        ("vieux", "vieux:ancien"),
        photo(4, [1, 2, 3, 255]).as_ref(),
    );

    let pose = Pose {
        x: 0.0,
        y: 0.0,
        largeur: 4.0,
        hauteur: 4.0,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
        bornes: Pose::PARTOUT,
    };
    let demande = |cle: &str, identite: &str| APoser {
        repli: None,
        cle: cle.to_string(),
        identite: identite.to_string(),
        pose,
    };
    // Budget nul : l'image n'a plus une milliseconde à donner.
    scene.assurer(
        &peripherique,
        &file,
        (
            &[demande("vieux:neuf", "vieux"), demande("neuf", "neuf")],
            std::time::Duration::ZERO,
        ),
        &|_| Some(Pixels::Rendues(photo(4, [9, 9, 9, 255]))),
    );

    assert!(
        scene.connait("neuf", "neuf"),
        "l'absent passe en premier, et au moins un se rend toujours -- sinon une scene ne se \
         completerait jamais sur une machine qui depasse le plancher a chaque image"
    );
    assert!(
        !scene.connait("vieux", "vieux:neuf"),
        "ce qui a vieilli attend le budget ; c'est tout l'objet de la cascade"
    );
    assert!(
        scene.detient("vieux"),
        "et il garde son ancien palier, donc il se pose"
    );
}

/// **La chronique sait ce que le rendu des textures a pris**, à part de leur envoi.
///
/// Pendant qu'il écrit, sa chronique donne 11,6 ms de `textures` par image, et le banc 2,2 ms
/// pour la même carte sur la même machine : seul ce compteur départagera un rendu plus lent
/// chez lui d'un envoi plus lent. Un compteur déclaré et jamais lu vaut zéro (fiche 17) — et
/// un compteur jamais nourri aussi : une source qui prend trois millisecondes doit s'y lire.
#[test]
fn test_la_cascade_compte_ce_que_le_rendu_a_pris() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    let pose = Pose {
        x: 0.0,
        y: 0.0,
        largeur: 4.0,
        hauteur: 4.0,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
        bornes: Pose::PARTOUT,
    };
    let demande = APoser {
        repli: None,
        cle: "carte:c:1".to_string(),
        identite: "carte:c".to_string(),
        pose,
    };
    let lente = |_: &str| {
        std::thread::sleep(std::time::Duration::from_millis(3));
        Some(Pixels::Rendues(photo(4, [9, 9, 9, 255])))
    };
    scene.assurer(
        &peripherique,
        &file,
        (&[demande], std::time::Duration::MAX),
        &lente,
    );
    let rendu = crate::perf::valeur_du_compteur("textures_rendu_us").unwrap_or(0.0);
    assert!(
        rendu >= 3000.0,
        "une source de trois millisecondes n'en a laisse que {rendu} us au compteur"
    );
}

/// **Une identité ne porte jamais deux textures.** Sans quoi la mémoire de la carte
/// doublerait à chaque changement de palier, et la borne du magasin — ce que l'écran
/// demande — cesserait d'en être une.
#[test]
fn test_cascade_un_nouveau_palier_remplace_l_ancien_et_ne_s_ajoute_pas() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &file,
        ("c", "c:x1"),
        photo(4, [1, 1, 1, 255]).as_ref(),
    );
    scene.televerser(
        &peripherique,
        &file,
        ("c", "c:x2"),
        photo(4, [2, 2, 2, 255]).as_ref(),
    );
    assert!(scene.connait("c", "c:x2"), "la neuve a pris la place");
    assert!(
        !scene.connait("c", "c:x1"),
        "et l'ancienne n'est plus la : une identite ne porte qu'une texture"
    );
}

/// **Une bande retirée ne revient pas sur le bord qu'elle touchait** — sur la carte, comme
/// dans le report du noyau (BORDURES-4).
///
/// C'est le défaut que la capture de l'utilisateur du 23/09 montrait au bord droit de sa
/// forêt : 123 là où l'image vaut 78, parce que le filtre lisait la colonne blanche que
/// `Ctrl+B` venait de retirer. Dix colonnes blanches, un contenu uni, la fenêtre qui retire
/// les dix : chaque pixel posé doit valoir le contenu, à toute échelle et toute phase. Et sans
/// les bornes, la même pose doit ramener le blanc — sinon ce test ne prouverait rien.
#[test]
fn test_une_bande_retiree_ne_revient_pas_sur_le_bord() {
    const CONTENU: [u8; 4] = [40, 40, 40, 255];
    let (l, h) = (40u32, 20u32);
    let mut source = Pixmap::new(l, h).expect("une photo");
    for (i, bloc) in source
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        *bloc = if (i as u32) % l < 10 {
            [255; 4]
        } else {
            CONTENU
        };
    }
    let crop = glucose_core::types::Recadrage::depuis_les_marges(0.25, 0.0, 0.0, 0.0);
    let cote = 128;
    let mut blanc_sans_bornes = 0;
    for echelle in [0.61f32, 1.5, 3.7] {
        for phase in [0.0f32, 0.3, 0.5, 0.8] {
            for borne in [true, false] {
                let pose = Pose {
                    x: 5.0 + phase,
                    y: 3.0 + phase,
                    largeur: 30.0 * echelle,
                    hauteur: 20.0 * echelle,
                    opacite: 1.0,
                    angle: 0.0,
                    fenetre: Pose::fenetre_de(crop),
                    bornes: if borne {
                        Pose::bornes_de(crop, (l, h), (1, (l, h)))
                    } else {
                        Pose::PARTOUT
                    },
                };
                let sources = vec![("cadree", source.clone())];
                let Some(octets) = rendre(cote, &[("cadree".to_string(), pose)], &sources) else {
                    eprintln!("aucune carte graphique : test saute");
                    return;
                };
                let poses: Vec<[u8; 4]> = octets
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .copied()
                    .filter(|p| p[3] != 0)
                    .collect();
                assert!(!poses.is_empty());
                let clairs = poses.iter().filter(|p| p[0] > CONTENU[0]).count();
                if borne {
                    assert_eq!(
                        clairs, 0,
                        "echelle {echelle}, phase {phase} : le blanc revient"
                    );
                } else {
                    blanc_sans_bornes += usize::from(clairs > 0);
                }
            }
        }
    }
    assert!(
        blanc_sans_bornes > 0,
        "sans les bornes, le blanc devait revenir -- sinon ce test ne prouve rien"
    );
}

// -- DE-PRES-1 : le repli des tuiles ------------------------------------------

/// Une tuile qu'on demande, et son repli : la même place, une autre texture.
fn tuile_et_repli(pose: Pose) -> APoser {
    APoser {
        cle: "carte:c@6#0,0:k".to_string(),
        identite: "carte:c@6#0,0".to_string(),
        pose,
        repli: Some(Box::new(APoser {
            cle: "carte:c:k".to_string(),
            identite: "carte:c".to_string(),
            pose,
            repli: None,
        })),
    }
}

/// **Une tuile absente se remplace par son repli, et le repli survit aux images où il ne sert
/// à rien** (DE-PRES-1).
///
/// Sans le premier, une carte vue de près se trouerait pendant que ses tuiles se rendent ;
/// sans le second, le repli serait oublié dès que toutes sont là, et le premier zoom — qui
/// les change toutes — n'aurait plus rien pour combler.
#[test]
fn test_de_pres_une_tuile_absente_se_remplace_par_son_repli_qui_reste_garde() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    let pose = Pose {
        x: 0.0,
        y: 0.0,
        largeur: 4.0,
        hauteur: 4.0,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
        bornes: Pose::PARTOUT,
    };
    let demande = [tuile_et_repli(pose)];

    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &file,
        ("carte:c", "carte:c:k"),
        photo(4, [7, 7, 7, 255]).as_ref(),
    );
    let retenues = scene.preparer(&peripherique, &file, (8.0, 8.0), &demande);
    assert_eq!(
        retenues.cles,
        vec!["carte:c".to_string()],
        "la tuile manque : son repli se pose"
    );
    scene.fermer(0);

    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &file,
        ("carte:c@6#0,0", "carte:c@6#0,0:k"),
        photo(4, [9, 9, 9, 255]).as_ref(),
    );
    let retenues = scene.preparer(&peripherique, &file, (8.0, 8.0), &demande);
    assert_eq!(
        retenues.cles,
        vec!["carte:c@6#0,0".to_string()],
        "la tuile est la : elle se pose"
    );
    scene.fermer(0);
    assert!(
        scene.connait("carte:c", "carte:c:k"),
        "le repli n'a pas ete pose, et il est garde pour le prochain zoom"
    );
}

/// **Un repli se rend sur le temps qui reste, et jamais au prix d'une image.**
///
/// Toujours demandé tant qu'une carte est découpée — une carte ouverte de près n'en aurait
/// sinon jamais —, mais après tout le reste, et jamais par la règle « au moins une par
/// image », qui existe pour que le contenu se complète, pas pour préparer un zoom.
#[test]
fn test_de_pres_un_repli_se_rend_sur_le_temps_qui_reste_et_jamais_de_force() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let pose = Pose {
        x: 0.0,
        y: 0.0,
        largeur: 4.0,
        hauteur: 4.0,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
        bornes: Pose::PARTOUT,
    };
    let demande = [tuile_et_repli(pose)];
    let source = |_: &str| Some(Pixels::Rendues(photo(4, [9, 9, 9, 255])));

    // La tuile est deja la, et l'image n'a plus une milliseconde : rien n'est force.
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &file,
        ("carte:c@6#0,0", "carte:c@6#0,0:k"),
        photo(4, [1, 1, 1, 255]).as_ref(),
    );
    scene.assurer(
        &peripherique,
        &file,
        (&demande, std::time::Duration::ZERO),
        &source,
    );
    assert!(
        !scene.detient("carte:c"),
        "sans temps, le repli attend : il ne force jamais une image"
    );

    // Du temps : il se rend, bien que sa tuile soit la.
    scene.assurer(
        &peripherique,
        &file,
        (&demande, std::time::Duration::MAX),
        &source,
    );
    assert!(
        scene.connait("carte:c", "carte:c:k"),
        "avec du temps, le repli se prepare"
    );
}

/// **Les bornes d'un recadrage se rapportent au niveau envoyé, pas à la texture native**
/// (NIVEAU-GPU-1).
///
/// Quarante colonnes natives, dix coupées à gauche, un niveau deux fois réduit de vingt
/// colonnes : le premier texel lisible est le cinquième du niveau — celui qui ne moyenne que
/// des colonnes gardées —, et sa borne est son centre, 5,5 sur vingt. Rapportée aux quarante
/// colonnes natives, elle tomberait au milieu de la bande coupée.
#[test]
fn test_les_bornes_d_un_recadrage_se_rapportent_au_niveau_envoye() {
    let crop = glucose_core::types::Recadrage::depuis_les_marges(0.25, 0.0, 0.0, 0.0);
    let bornes = Pose::bornes_de(crop, (40, 20), (2, (20, 10)));
    assert_eq!(
        bornes[0],
        5.5 / 20.0,
        "le premier texel lisible du niveau, en son centre"
    );
    assert_eq!(
        bornes[2],
        19.5 / 20.0,
        "et le dernier, qui ne touche aucune coupe"
    );
}

// -- VRAM-1 : la carte garde ce que son budget permet ---------------------------

/// **Hors de l'écran, la carte garde les plus récemment vues, tant qu'elles tiennent dans ce
/// que son budget accorde** — et rien de plus.
///
/// Trois photos de 4 × 4 (64 octets chacune) : `a` vue à l'image 1, `b` à l'image 2, `c` à la
/// 3 ; puis une image où seule `d` sert. Avec 128 octets à garder, `c` et `b` restent — les
/// plus récentes —, `a` part. Avec zéro, c'est la loi d'avant : ce que l'écran demande.
#[test]
fn test_vram_hors_de_l_ecran_la_carte_garde_les_plus_recentes_dans_son_budget() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let pose = Pose {
        x: 0.0,
        y: 0.0,
        largeur: 4.0,
        hauteur: 4.0,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
        bornes: Pose::PARTOUT,
    };
    let jouer = |gardable: u64| {
        let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
        for nom in ["a", "b", "c"] {
            scene.ouvrir();
            scene.televerser(
                &peripherique,
                &file,
                (nom, nom),
                photo(4, [1, 2, 3, 255]).as_ref(),
            );
            scene.preparer(&peripherique, &file, (8.0, 8.0), &[a_poser(nom, pose)]);
            scene.fermer(u64::MAX);
        }
        scene.ouvrir();
        scene.televerser(
            &peripherique,
            &file,
            ("d", "d"),
            photo(4, [4, 5, 6, 255]).as_ref(),
        );
        scene.preparer(&peripherique, &file, (8.0, 8.0), &[a_poser("d", pose)]);
        scene.fermer(gardable);
        scene
    };
    let scene = jouer(128);
    assert!(scene.detient("d"), "ce qui sert reste toujours");
    assert!(
        scene.detient("c") && scene.detient("b"),
        "les deux plus recentes tiennent dans le budget : elles restent"
    );
    assert!(
        !scene.detient("a"),
        "la plus ancienne ne tient plus : elle part"
    );
    assert_eq!(scene.octets_en_cache(), 128);

    let scene = jouer(0);
    assert!(scene.detient("d"));
    assert!(
        !scene.detient("a") && !scene.detient("b") && !scene.detient("c"),
        "sans budget, la loi d'avant : ce que l'ecran demande, et rien de plus"
    );
    assert_eq!(scene.octets_en_cache(), 0);
}

/// **La carte sait ce qu'occupent ses photos à l'écran** (ETAGES-3) : ce que le magasin lui a
/// prêté, et non ce qu'un composant a rendu ; posé à cette image, et non gardé en cache.
#[test]
fn test_la_carte_compte_ce_qu_occupent_ses_photos_a_l_ecran() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    let pose = Pose {
        x: 0.0,
        y: 0.0,
        largeur: 4.0,
        hauteur: 4.0,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
        bornes: Pose::PARTOUT,
    };
    let a_poser = |cle: &str| APoser {
        repli: None,
        cle: cle.to_string(),
        identite: cle.to_string(),
        pose,
    };
    let pretee = photo(4, [1, 2, 3, 255]);
    let demandes = [a_poser("photo"), a_poser("carte")];
    scene.assurer(
        &peripherique,
        &file,
        (&demandes, std::time::Duration::MAX),
        &|cle| {
            Some(if cle == "photo" {
                Pixels::Pretes(pretee.as_ref())
            } else {
                Pixels::Rendues(photo(8, [4, 5, 6, 255]))
            })
        },
    );
    scene.preparer(&peripherique, &file, (8.0, 8.0), &demandes);
    assert_eq!(
        scene.octets_des_photos_posees(),
        4 * 4 * 4,
        "la photo, et pas la carte de texte"
    );
    scene.ouvrir();
    assert_eq!(
        scene.octets_des_photos_posees(),
        0,
        "ce qui n'est pas pose a cette image ne compte pas"
    );
}

/// **La cascade n'entame pas ce qu'elle ne finira pas** (CASCADE-3) : sur une machine qui a
/// montré qu'un pixel posé coûte une seconde, une texture de seize pixels n'a aucune chance
/// de tenir dans cinq millisecondes — elle attend l'image suivante, même s'il en restait
/// presque cinq quand on l'a regardée. La première passe toujours : c'est elle qui mesure, et
/// une scène doit finir par se compléter.
///
/// Une seconde et non une milliseconde : la première texture, mesurée pour de bon en quelques
/// microsecondes, entre dans la moyenne — et une milliseconde semée s'y diluait assez pour
/// laisser passer la seconde. Le débit suit ce que la machine montre, c'est voulu.
#[test]
fn test_la_cascade_prevoit_avant_d_entamer() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.debit.noter(1, std::time::Duration::from_secs(1));
    scene.ouvrir();
    let a_poser = |cle: &str| APoser {
        repli: None,
        cle: cle.to_string(),
        identite: cle.to_string(),
        pose: Pose {
            x: 0.0,
            y: 0.0,
            largeur: 4.0,
            hauteur: 4.0,
            opacite: 1.0,
            angle: 0.0,
            fenetre: Pose::TOUT,
            bornes: Pose::PARTOUT,
        },
    };
    scene.assurer(
        &peripherique,
        &file,
        (
            &[a_poser("premiere"), a_poser("seconde")],
            std::time::Duration::from_millis(5),
        ),
        &|_| Some(Pixels::Rendues(photo(4, [9, 9, 9, 255]))),
    );
    assert!(
        scene.connait("premiere", "premiere"),
        "la premiere passe toujours"
    );
    assert!(
        !scene.connait("seconde", "seconde"),
        "prevue bien au-dela des cinq millisecondes du budget : elle attend"
    );
}
