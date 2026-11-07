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

use crate::present::banc_gpu::carte;

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
        cle: cle.into(),
        identite: cle.into(),
        pose,
    }
}

/// Dessine `poses` dans une cible de `cote` pixels, et rend ses octets RGBA.
fn rendre(cote: u32, poses: &[(String, Pose)], sources: &[(&str, Pixmap)]) -> Option<Vec<u8>> {
    rendre_par(cote, poses, sources, false)
}

/// La même chose, avec ou sans l'atlas (ATLAS-1).
fn rendre_par(
    cote: u32,
    poses: &[(String, Pose)],
    sources: &[(&str, Pixmap)],
    sans_atlas: bool,
) -> Option<Vec<u8>> {
    let (peripherique, file) = carte()?;
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut scene = SceneGpu::nouvelle(&peripherique, format);
    scene.sans_atlas = sans_atlas;
    scene.ouvrir();
    for (cle, source) in sources {
        scene.televerser(
            &peripherique,
            &mut envoi,
            (&Cle::from(*cle), &Cle::from(*cle)),
            source.as_ref(),
        );
    }
    let poses: Vec<APoser> = poses.iter().map(|(c, p)| a_poser(c, *p)).collect();
    let retenues = scene.preparer(&peripherique, &file, (cote as f32, cote as f32), &poses);
    Some(dessiner_la_scene(
        (&peripherique, &file),
        &mut envoi,
        (&scene, &retenues),
        cote,
    ))
}

/// Dessine tout ce que la scène a retenu.
fn dessiner_la_scene(
    carte: (&wgpu::Device, &wgpu::Queue),
    envoi: &mut crate::present::envoi::Envoi,
    (scene, retenues): (&SceneGpu, &Retenues),
    cote: u32,
) -> Vec<u8> {
    dessiner_la_tranche(carte, envoi, (scene, retenues, 0..retenues.len()), cote)
}

/// Dessine cette tranche de ce que la scène a retenu dans une cible de `cote` pixels, et rend ses
/// octets RGBA.
fn dessiner_la_tranche(
    (peripherique, file): (&wgpu::Device, &wgpu::Queue),
    envoi: &mut crate::present::envoi::Envoi,
    (scene, retenues, tranche): (&SceneGpu, &Retenues, std::ops::Range<usize>),
    cote: u32,
) -> Vec<u8> {
    let format = wgpu::TextureFormat::Rgba8Unorm;
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
        scene.poser(&mut passe, retenues, tranche);
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
    envoi.soumettre();
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
    octets
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("a"), &Cle::from("a")),
        photo(4, [1, 2, 3, 255]).as_ref(),
    );
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("b"), &Cle::from("b")),
        photo(4, [4, 5, 6, 255]).as_ref(),
    );
    assert!(scene.connait(&"a".into(), &"a".into()) && scene.connait(&"b".into(), &"b".into()));

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

    assert!(
        scene.connait(&"a".into(), &"a".into()),
        "ce qui a servi reste"
    );
    assert!(
        !scene.connait(&"b".into(), &"b".into()),
        "ce qui n'a pas servi est oublie"
    );
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    // La carte détient la carte de texte `c1` à son ancien palier.
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("carte:c1"), &Cle::from("carte:c1:ancien")),
        photo(4, [1, 2, 3, 255]).as_ref(),
    );

    // La scène en demande maintenant un palier neuf.
    assert!(
        !scene.connait(&"carte:c1".into(), &"carte:c1:neuf".into()),
        "la clé neuve n'est pas détenue : c'est ce qui la fera rendre quand il y aura du temps"
    );
    assert!(
        scene.detient(&"carte:c1".into()),
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
            cle: "carte:c1:neuf".into(),
            identite: "carte:c1".into(),
            pose,
        }],
    );
    assert_eq!(
        retenues.cles,
        vec![Cle::from("carte:c1")],
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("vieux"), &Cle::from("vieux:ancien")),
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
        cle: cle.into(),
        identite: identite.into(),
        pose,
    };
    // Budget nul : l'image n'a plus une milliseconde à donner.
    scene.assurer(
        &peripherique,
        &mut envoi,
        (
            &[demande("vieux:neuf", "vieux"), demande("neuf", "neuf")],
            &[],
            std::time::Duration::ZERO,
        ),
        &|_| Some(Pixels::Rendues(photo(4, [9, 9, 9, 255]))),
    );

    assert!(
        scene.connait(&"neuf".into(), &"neuf".into()),
        "l'absent passe en premier, et au moins un se rend toujours -- sinon une scene ne se \
         completerait jamais sur une machine qui depasse le plancher a chaque image"
    );
    assert!(
        !scene.connait(&"vieux".into(), &"vieux:neuf".into()),
        "ce qui a vieilli attend le budget ; c'est tout l'objet de la cascade"
    );
    assert!(
        scene.detient(&"vieux".into()),
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
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
        cle: "carte:c:1".into(),
        identite: "carte:c".into(),
        pose,
    };
    let lente = |_: &Cle| {
        std::thread::sleep(std::time::Duration::from_millis(3));
        Some(Pixels::Rendues(photo(4, [9, 9, 9, 255])))
    };
    scene.assurer(
        &peripherique,
        &mut envoi,
        (&[demande], &[], std::time::Duration::MAX),
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("c"), &Cle::from("c:x1")),
        photo(4, [1, 1, 1, 255]).as_ref(),
    );
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("c"), &Cle::from("c:x2")),
        photo(4, [2, 2, 2, 255]).as_ref(),
    );
    assert!(
        scene.connait(&"c".into(), &"c:x2".into()),
        "la neuve a pris la place"
    );
    assert!(
        !scene.connait(&"c".into(), &"c:x1".into()),
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
        cle: "carte:c@6#0,0:k".into(),
        identite: "carte:c@6#0,0".into(),
        pose,
        repli: Some(Box::new(APoser {
            cle: "carte:c:k".into(),
            identite: "carte:c".into(),
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
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
        &mut envoi,
        (&Cle::from("carte:c"), &Cle::from("carte:c:k")),
        photo(4, [7, 7, 7, 255]).as_ref(),
    );
    let retenues = scene.preparer(&peripherique, &file, (8.0, 8.0), &demande);
    assert_eq!(
        retenues.cles,
        vec![Cle::from("carte:c")],
        "la tuile manque : son repli se pose"
    );
    scene.fermer(0);

    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("carte:c@6#0,0"), &Cle::from("carte:c@6#0,0:k")),
        photo(4, [9, 9, 9, 255]).as_ref(),
    );
    let retenues = scene.preparer(&peripherique, &file, (8.0, 8.0), &demande);
    assert_eq!(
        retenues.cles,
        vec![Cle::from("carte:c@6#0,0")],
        "la tuile est la : elle se pose"
    );
    scene.fermer(0);
    assert!(
        scene.connait(&"carte:c".into(), &"carte:c:k".into()),
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
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
    let source = |_: &Cle| Some(Pixels::Rendues(photo(4, [9, 9, 9, 255])));

    // La tuile est deja la, et l'image n'a plus une milliseconde : rien n'est force.
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("carte:c@6#0,0"), &Cle::from("carte:c@6#0,0:k")),
        photo(4, [1, 1, 1, 255]).as_ref(),
    );
    scene.assurer(
        &peripherique,
        &mut envoi,
        (&demande, &[], std::time::Duration::ZERO),
        &source,
    );
    assert!(
        !scene.detient(&"carte:c".into()),
        "sans temps, le repli attend : il ne force jamais une image"
    );

    // Du temps : il se rend, bien que sa tuile soit la.
    scene.assurer(
        &peripherique,
        &mut envoi,
        (&demande, &[], std::time::Duration::MAX),
        &source,
    );
    assert!(
        scene.connait(&"carte:c".into(), &"carte:c:k".into()),
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
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
    let mut jouer = |gardable: u64| {
        let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
        for nom in ["a", "b", "c"] {
            scene.ouvrir();
            scene.televerser(
                &peripherique,
                &mut envoi,
                (&Cle::from(nom), &Cle::from(nom)),
                photo(4, [1, 2, 3, 255]).as_ref(),
            );
            scene.preparer(&peripherique, &file, (8.0, 8.0), &[a_poser(nom, pose)]);
            scene.fermer(u64::MAX);
        }
        scene.ouvrir();
        scene.televerser(
            &peripherique,
            &mut envoi,
            (&Cle::from("d"), &Cle::from("d")),
            photo(4, [4, 5, 6, 255]).as_ref(),
        );
        scene.preparer(&peripherique, &file, (8.0, 8.0), &[a_poser("d", pose)]);
        scene.fermer(gardable);
        scene
    };
    let scene = jouer(128);
    assert!(scene.detient(&"d".into()), "ce qui sert reste toujours");
    assert!(
        scene.detient(&"c".into()) && scene.detient(&"b".into()),
        "les deux plus recentes tiennent dans le budget : elles restent"
    );
    assert!(
        !scene.detient(&"a".into()),
        "la plus ancienne ne tient plus : elle part"
    );
    assert_eq!(scene.octets_en_cache(), 128);

    let scene = jouer(0);
    assert!(scene.detient(&"d".into()));
    assert!(
        !scene.detient(&"a".into()) && !scene.detient(&"b".into()) && !scene.detient(&"c".into()),
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
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
        cle: cle.into(),
        identite: cle.into(),
        pose,
    };
    let pretee = photo(4, [1, 2, 3, 255]);
    let demandes = [a_poser("photo"), a_poser("carte")];
    scene.assurer(
        &peripherique,
        &mut envoi,
        (&demandes, &[], std::time::Duration::MAX),
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
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.debit.noter(1, std::time::Duration::from_secs(1));
    scene.ouvrir();
    let a_poser = |cle: &str| APoser {
        repli: None,
        cle: cle.into(),
        identite: cle.into(),
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
        &mut envoi,
        (
            &[a_poser("premiere"), a_poser("seconde")],
            &[],
            std::time::Duration::from_millis(5),
        ),
        &|_| Some(Pixels::Rendues(photo(4, [9, 9, 9, 255]))),
    );
    assert!(
        scene.connait(&"premiere".into(), &"premiere".into()),
        "la premiere passe toujours"
    );
    assert!(
        !scene.connait(&"seconde".into(), &"seconde".into()),
        "prevue bien au-dela des cinq millisecondes du budget : elle attend"
    );
}

/// **NETTETE-1 — la cascade dit qu'elle n'a pas fini.**
///
/// Ce qu'elle reporte attend l'image suivante — et sans qu'elle le dise, l'image suivante
/// n'arrivait qu'au prochain geste : des tuiles floues restaient posées au repos. À l'envers,
/// une scène servie en entier est finie, et ne redemande rien.
///
/// **Un repli qui attend ne la rend pas inachevée.** Il ne force jamais une image
/// (`test_de_pres_un_repli_se_rend_sur_le_temps_qui_reste_et_jamais_de_force`) : sur une machine
/// dont chaque image dépasse le plancher, il attendrait toujours, et la boucle redessinerait
/// sans fin au repos pour lui.
#[test]
fn test_nettete_1_la_cascade_dit_qu_elle_n_a_pas_fini() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
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
    let source = |_: &Cle| Some(Pixels::Rendues(photo(4, [9, 9, 9, 255])));
    let mut servir = |scene: &mut SceneGpu, demandes: &[APoser], budget| {
        scene.ouvrir();
        scene.assurer(&peripherique, &mut envoi, (demandes, &[], budget), &source);
        scene.inachevee()
    };
    let deux = [a_poser("a", pose), a_poser("b", pose)];
    assert!(
        servir(&mut scene, &deux, std::time::Duration::ZERO),
        "une texture reportee laisse la scene inachevee"
    );
    assert!(
        !servir(&mut scene, &deux, std::time::Duration::MAX),
        "servie en entier, la scene est finie"
    );

    let tuile = APoser {
        repli: Some(Box::new(a_poser("entier-bas", pose))),
        ..a_poser("tuile", pose)
    };
    let tuile = std::slice::from_ref(&tuile);
    assert!(
        !servir(&mut scene, tuile, std::time::Duration::ZERO),
        "la tuile est servie ; son repli attend, et ne garde pas la boucle eveillee"
    );
    assert!(
        !scene.detient(&"entier-bas".into()),
        "le montage : le repli a bien attendu"
    );
}

/// **MEMOIRE-2 — rendre à la carte ce qui se reconstruit** (fiche 66). Trois photos en cache
/// avec un budget généreux, une quatrième posée : au premier plan, seule celle que l'écran pose
/// reste, quel que soit le budget ; derrière une autre application, plus rien. Ce que la carte
/// détient se compte, avant et après.
#[test]
fn test_memoire_2_rendre_garde_l_ecran_au_premier_plan_et_rien_derriere() {
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
    let preparee = || {
        let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
        let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
        for nom in ["a", "b", "c", "d"] {
            scene.ouvrir();
            let p = photo(4, [1, 2, 3, 255]);
            scene.televerser(
                &peripherique,
                &mut envoi,
                (&Cle::from(nom), &Cle::from(nom)),
                p.as_ref(),
            );
            scene.preparer(&peripherique, &file, (8.0, 8.0), &[a_poser(nom, pose)]);
            scene.fermer(u64::MAX);
        }
        scene
    };
    let mut devant = preparee();
    // Quatre photos de seize pixels, rangées dans une page de 128 pixels de côté, qui occupe sa
    // place entière (ATLAS-1).
    let page = 128 * 128 * 4;
    assert_eq!(devant.octets(), page, "la page entière : la carte la tient");
    devant.rendre(false);
    assert!(devant.detient(&"d".into()), "ce que l'écran pose reste");
    assert!(
        !devant.detient(&"a".into())
            && !devant.detient(&"b".into())
            && !devant.detient(&"c".into()),
        "le cache part, quel que soit le budget"
    );
    assert_eq!(
        devant.octets(),
        page,
        "la page vit tant qu'une photo y reste"
    );
    assert_eq!(devant.octets_en_cache(), 0);

    let mut derriere = preparee();
    derriere.rendre(true);
    assert!(
        !derriere.detient(&"d".into()),
        "personne ne regarde : tout part"
    );
    assert_eq!(derriere.octets(), 0);
    assert_eq!(derriere.octets_en_cache(), 0);
}

/// Une photo de bruit, pour que toute lecture hors de sa cellule se voie : deux voisines
/// différentes, un texel de trop, et l'image change.
fn bruit(largeur: u32, hauteur: u32, graine: u64) -> Pixmap {
    let mut p = Pixmap::new(largeur, hauteur).expect("une photo");
    let mut g = graine.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    for bloc in p.data_mut().as_chunks_mut::<4>().0 {
        g ^= g << 13;
        g ^= g >> 7;
        g ^= g << 17;
        *bloc = [g as u8, (g >> 8) as u8, (g >> 16) as u8, 255];
    }
    p
}

fn pose_de(x: f32, y: f32, largeur: f32, hauteur: f32) -> Pose {
    Pose {
        x,
        y,
        largeur,
        hauteur,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
        bornes: Pose::PARTOUT,
    }
}

/// **Les mêmes pixels avec et sans l'atlas** (ATLAS-1) : la promesse qui rend l'atlas permis. Des
/// photos de bruit, voisines dans leur page, réduites, agrandies, tournées, recadrées, à demi
/// transparentes : un texel lu chez la voisine se verrait aussitôt.
///
/// **L'écart permis se déduit** : une carte lit à 1/2⁸ de texel près sur chaque axe
/// (`D3D12_SUBPIXEL_FRACTIONAL_BIT_COUNT`, et le palier Roadmap 2022 de Vulkan), et la page arrondit
/// la même place autrement que la texture seule. Sur du bruit, où deux voisins diffèrent de 255, cela
/// fait au plus 255 × 2 / 2⁸ : moins de deux unités. Le « 1 » d'avant n'était qu'une supposition, que
/// la carte de Windows sans matériel (WARP) a démentie (fiche 88). Une lecture du demi-texel
/// voisin, elle, en coûte des dizaines.
#[test]
fn test_atlas_1_les_memes_pixels_avec_et_sans_l_atlas() {
    let cote = 128;
    let sources = vec![
        ("a", bruit(8, 8, 1)),
        ("b", bruit(16, 7, 2)),
        ("c", bruit(5, 13, 3)),
        ("d", bruit(64, 64, 4)),
        ("e", bruit(65, 65, 5)),
        ("f", bruit(16, 16, 6)),
    ];
    let cle = |c: &str, p: Pose| (c.to_string(), p);
    let poses = vec![
        cle("a", pose_de(2.0, 2.0, 37.3, 41.7)),
        cle("b", pose_de(40.5, 3.25, 5.5, 3.1)),
        cle(
            "c",
            Pose {
                angle: 0.7,
                ..pose_de(50.0, 10.0, 30.0, 50.0)
            },
        ),
        cle("d", pose_de(5.0, 50.0, 33.0, 33.0)),
        cle(
            "e",
            Pose {
                opacite: 0.5,
                ..pose_de(20.0, 70.0, 50.0, 50.0)
            },
        ),
        cle(
            "f",
            Pose {
                fenetre: [0.25, 0.25, 0.5, 0.5],
                bornes: [0.25, 0.25, 0.75, 0.75],
                ..pose_de(80.0, 70.0, 45.0, 45.0)
            },
        ),
    ];
    let Some(avec) = rendre_par(cote, &poses, &sources, false) else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let sans = rendre_par(cote, &poses, &sources, true).expect("la même carte");
    let pire = avec
        .iter()
        .zip(&sans)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    let permis = (255.0_f32 * 2.0 / 256.0).ceil() as u8;
    assert!(
        pire <= permis,
        "un écart de {pire} sur 255, pour {permis} permis : une lecture hors de la cellule"
    );
}

/// **Une suite par lot, dans l'ordre du modèle** (ATLAS-1) : les petites d'une page font un seul
/// dessin, et une grande photo posée entre elles coupe la suite à sa place — sinon elle passerait
/// dessus ou dessous ce que le document dit.
#[test]
fn test_atlas_1_une_suite_par_lot_dans_l_ordre_du_modele() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    for nom in ["a", "b", "c"] {
        scene.televerser(
            &peripherique,
            &mut envoi,
            (&Cle::from(nom), &Cle::from(nom)),
            photo(8, [9, 9, 9, 255]).as_ref(),
        );
    }
    let grande = photo(128, [1, 1, 1, 255]);
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("g"), &Cle::from("g")),
        grande.as_ref(),
    );
    let pose = pose_de(0.0, 0.0, 4.0, 4.0);
    let poses: Vec<APoser> = ["a", "b", "g", "c"]
        .iter()
        .map(|n| a_poser(n, pose))
        .collect();
    let retenues = scene.preparer(&peripherique, &file, (8.0, 8.0), &poses);
    let suites: Vec<(u32, u32)> = retenues.suites.iter().map(|s| (s.debut, s.fin)).collect();
    assert_eq!(suites, [(0, 2), (2, 3), (3, 4)]);
    assert_eq!(
        retenues.suites[0].lot, retenues.suites[2].lot,
        "la même page, et pourtant deux dessins : la grande est entre les deux"
    );
}

/// **Mille photos d'une page, un seul dessin** : c'est tout l'objet d'ATLAS-1. La page d'après
/// commence le suivant, et aucune autre coupure n'existe.
#[test]
fn test_atlas_1_les_photos_d_une_page_font_un_seul_dessin() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    let petite = photo(4, [9, 9, 9, 255]);
    let noms: Vec<String> = (0..1500).map(|i| format!("p{i}")).collect();
    for nom in &noms {
        scene.televerser(
            &peripherique,
            &mut envoi,
            (&Cle::from(nom), &Cle::from(nom)),
            petite.as_ref(),
        );
    }
    let pose = pose_de(0.0, 0.0, 4.0, 4.0);
    let poses: Vec<APoser> = noms.iter().map(|n| a_poser(n, pose)).collect();
    let retenues = scene.preparer(&peripherique, &file, (8.0, 8.0), &poses);
    let suites: Vec<(u32, u32)> = retenues.suites.iter().map(|s| (s.debut, s.fin)).collect();
    assert_eq!(
        suites,
        [(0, 1024), (1024, 1500)],
        "deux pages, deux dessins"
    );
}

/// **Une cellule rendue se reprend, et une page vide se rend** (ATLAS-1) : ce qui part de la carte
/// ne se perd pas dans une page que plus personne ne lit.
#[test]
fn test_atlas_1_une_page_vide_se_rend_et_une_cellule_se_reprend() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    let petite = photo(4, [9, 9, 9, 255]);
    for nom in ["a", "b"] {
        scene.televerser(
            &peripherique,
            &mut envoi,
            (&Cle::from(nom), &Cle::from(nom)),
            petite.as_ref(),
        );
    }
    let page = 128 * 128 * 4;
    assert_eq!(scene.octets(), page);
    scene.oublier_la_texture(&"a".into());
    assert_eq!(scene.octets(), page, "une seule partie : la page reste");
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("c"), &Cle::from("c")),
        petite.as_ref(),
    );
    assert_eq!(
        scene.octets(),
        page,
        "la cellule se reprend, sans page de plus"
    );
    scene.oublier_la_texture(&"b".into());
    scene.oublier_la_texture(&"c".into());
    assert_eq!(scene.octets(), 0, "plus personne : la page se rend");
}

/// **Une photo qui change de niveau reprend sa cellule** : remplacer la texture d'une identité ne
/// laisse rien derrière elle, ni dans le compte ni dans la page.
#[test]
fn test_atlas_1_changer_de_niveau_ne_laisse_rien_derriere() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("a"), &Cle::from("a@2")),
        photo(8, [9, 9, 9, 255]).as_ref(),
    );
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("a"), &Cle::from("a@1")),
        photo(16, [9, 9, 9, 255]).as_ref(),
    );
    assert_eq!(
        scene.octets(),
        512 * 512 * 4,
        "la page de 8 est rendue, reste celle de 16"
    );
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&Cle::from("a"), &Cle::from("a@0")),
        photo(128, [9, 9, 9, 255]).as_ref(),
    );
    assert_eq!(
        scene.octets(),
        128 * 128 * 4,
        "sa texture à elle : plus aucune page"
    );
}

/// Un coin de photo à l'écran, pour une vue : la règle que `pose_tenue` applique.
fn a_la_vue(pose: Pose, (vx, vy, s): (f32, f32, f32)) -> Pose {
    Pose {
        x: pose.x * s + vx,
        y: pose.y * s + vy,
        largeur: pose.largeur * s,
        hauteur: pose.hauteur * s,
        ..pose
    }
}

/// Ce qui mène de la vue `a` à la vue `b`.
fn lot_entre(
    a: (f32, f32, f32),
    b: (f32, f32, f32),
    nombre: usize,
) -> crate::renderer::voies::LotEnPlace {
    let rapport = f64::from(b.2) / f64::from(a.2);
    crate::renderer::voies::LotEnPlace {
        nombre,
        similitude: crate::renderer::voies::Similitude {
            rapport,
            tx: f64::from(b.0) - rapport * f64::from(a.0),
            ty: f64::from(b.1) - rapport * f64::from(a.1),
        },
    }
}

/// **Un lot repris à une autre vue donne les pixels d'une image complète à cette vue** (LOT-1) : la
/// promesse qui le rend permis. Des photos de bruit, dans une page et hors d'elle, tournées et
/// recadrées, et ce qui n'est pas du lot — une carte de texte — reste où il est dit, sans similitude.
#[test]
fn test_lot_1_un_lot_repris_donne_les_pixels_d_une_image_complete() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let cote = 128;
    let ecran = (cote as f32, cote as f32);
    let sources = vec![
        ("a", bruit(16, 16, 1)),
        ("b", bruit(32, 16, 2)),
        ("g", bruit(128, 128, 3)),
        ("c", bruit(16, 16, 4)),
    ];
    let monde = [
        (
            "a",
            Pose {
                angle: 0.4,
                ..pose_de(10.0, 20.0, 60.0, 60.0)
            },
        ),
        ("b", pose_de(90.0, 10.0, 80.0, 40.0)),
        ("g", pose_de(30.0, 90.0, 90.0, 90.0)),
    ];
    let (va, vb) = ((3.0, -2.0, 0.5), (-11.5, 7.25, 0.8125));
    let poses_a = |v| -> Vec<APoser> {
        monde
            .iter()
            .map(|(n, p)| a_poser(n, a_la_vue(*p, v)))
            .collect()
    };
    let carte_de_texte = a_poser("c", pose_de(60.0, 60.0, 20.0, 20.0));
    let charger = |scene: &mut SceneGpu, envoi: &mut crate::present::envoi::Envoi| {
        scene.ouvrir();
        for (nom, source) in &sources {
            let cle = Cle::from(*nom);
            scene.televerser(&peripherique, envoi, (&cle, &cle), source.as_ref());
        }
    };

    // L'image complète à la vue B, le témoin.
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut temoin = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    charger(&mut temoin, &mut envoi);
    let mut tout = poses_a(vb);
    tout.push(carte_de_texte.clone());
    let retenues = temoin.preparer(&peripherique, &file, ecran, &tout);
    let attendu = dessiner_la_scene(
        (&peripherique, &file),
        &mut envoi,
        (&temoin, &retenues),
        cote,
    );

    // Une image complète à la vue A propose le lot, et la suivante le reprend à la vue B.
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    charger(&mut scene, &mut envoi);
    let photos = poses_a(va);
    scene.preparer_la_vue(&peripherique, &file, ecran, &photos, (None, Some((5, 3))));
    assert_eq!(scene.lot_en_place(), Some(5), "complet : il est retenu");
    let lot = lot_entre(va, vb, 3);
    let reste = std::slice::from_ref(&carte_de_texte);
    let retenues = scene.preparer_la_vue(&peripherique, &file, ecran, reste, (Some(lot), None));
    assert_eq!(retenues.len(), 4);
    assert_eq!(
        retenues.avant(2),
        2,
        "les rangs du lot sont ceux de ses poses"
    );
    assert_eq!(retenues.avant(3), 3);
    assert_eq!(retenues.avant(4), 4, "et ce qui suit vient après lui");
    let obtenu = dessiner_la_scene(
        (&peripherique, &file),
        &mut envoi,
        (&scene, &retenues),
        cote,
    );
    let pire = attendu
        .iter()
        .zip(&obtenu)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert!(
        pire <= 2,
        "un écart de {pire} sur 255 entre le lot repris et l'image complète"
    );

    // À l'envers : sans similitude, les mêmes poses seraient restées à la vue A.
    let fige = crate::renderer::voies::LotEnPlace {
        nombre: 3,
        similitude: crate::renderer::voies::Similitude::IDENTITE,
    };
    let retenues = scene.preparer_la_vue(&peripherique, &file, ecran, reste, (Some(fige), None));
    let sans = dessiner_la_scene(
        (&peripherique, &file),
        &mut envoi,
        (&scene, &retenues),
        cote,
    );
    let pire = attendu
        .iter()
        .zip(&sans)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert!(
        pire > 50,
        "l'épreuve doit voir une vue qui n'a pas suivi : {pire}"
    );
}

/// **Un lot n'existe que complet** (LOT-1) : une photo absente, périmée, ou de la cascade en attente,
/// et la prochaine image doit de toute façon tout revoir.
#[test]
fn test_lot_1_un_lot_n_existe_que_complet() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let ecran = (8.0, 8.0);
    let pose = pose_de(0.0, 0.0, 4.0, 4.0);
    let (a, b) = (Cle::from("a"), Cle::from("b"));
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&a, &a),
        photo(4, [1, 1, 1, 255]).as_ref(),
    );
    let demande = [a_poser("a", pose), a_poser("b", pose)];
    scene.preparer_la_vue(&peripherique, &file, ecran, &demande, (None, Some((1, 2))));
    assert_eq!(scene.lot_en_place(), None, "une photo manque");

    scene.televerser(
        &peripherique,
        &mut envoi,
        (&b, &Cle::from("b:vieille")),
        photo(4, [1, 1, 1, 255]).as_ref(),
    );
    scene.preparer_la_vue(&peripherique, &file, ecran, &demande, (None, Some((2, 2))));
    assert_eq!(scene.lot_en_place(), None, "une photo est périmée");

    scene.televerser(
        &peripherique,
        &mut envoi,
        (&b, &b),
        photo(4, [1, 1, 1, 255]).as_ref(),
    );
    scene.preparer_la_vue(&peripherique, &file, ecran, &demande, (None, Some((3, 2))));
    assert_eq!(scene.lot_en_place(), Some(3), "toutes là, à jour");

    scene.preparer_la_vue(&peripherique, &file, ecran, &demande, (None, None));
    assert_eq!(
        scene.lot_en_place(),
        None,
        "une image complète sans proposition le retire"
    );

    scene.inachevee = true;
    scene.preparer_la_vue(&peripherique, &file, ecran, &demande, (None, Some((4, 2))));
    assert_eq!(scene.lot_en_place(), None, "la cascade attend encore");
}

/// **Une photo du lot qui part défait le lot** ; une photo hors de l'écran, non (LOT-1).
#[test]
fn test_lot_1_une_photo_du_lot_qui_part_le_defait() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let ecran = (8.0, 8.0);
    let pose = pose_de(0.0, 0.0, 4.0, 4.0);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    let pretee = photo(4, [1, 1, 1, 255]);
    let source = |_: &Cle| Some(Pixels::Pretes(pretee.as_ref()));
    let demande = [a_poser("a", pose), a_poser("b", pose)];
    let tout = |scene: &mut SceneGpu, envoi: &mut crate::present::envoi::Envoi, numero| {
        scene.ouvrir();
        scene.assurer(
            &peripherique,
            envoi,
            (&demande, &[], std::time::Duration::MAX),
            &source,
        );
        scene.preparer_la_vue(
            &peripherique,
            &file,
            ecran,
            &demande,
            (None, Some((numero, 2))),
        );
    };
    tout(&mut scene, &mut envoi, 1);
    assert_eq!(scene.lot_en_place(), Some(1));

    let ailleurs = Cle::from("hors de l'écran");
    scene.image -= 1;
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&ailleurs, &ailleurs),
        pretee.as_ref(),
    );
    scene.marquer_photo(&ailleurs);
    scene.image += 1;
    scene.oublier_la_texture(&ailleurs);
    assert_eq!(scene.lot_en_place(), Some(1), "elle n'était pas à l'écran");

    scene.oublier_la_texture(&"a".into());
    assert_eq!(scene.lot_en_place(), None, "elle en faisait partie");

    tout(&mut scene, &mut envoi, 2);
    assert_eq!(scene.lot_en_place(), Some(2));
    scene.rendre(true);
    assert_eq!(scene.lot_en_place(), None, "la carte a tout rendu");
}

/// **Le tampon qui grandit garde le lot** (LOT-1) : sinon, une image où beaucoup de textures
/// arrivent d'un coup perdrait toutes ses photos.
#[test]
fn test_lot_1_le_tampon_qui_grandit_garde_le_lot() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let cote = 64;
    let ecran = (cote as f32, cote as f32);
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    let rouge = Cle::from("rouge");
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&rouge, &rouge),
        photo(4, [200, 0, 0, 255]).as_ref(),
    );
    let (x, y, l, h) = (0.0, 0.0, 16.0, 16.0);
    let lot = [a_poser("rouge", pose_de(x, y, l, h))];
    scene.preparer_la_vue(&peripherique, &file, ecran, &lot, (None, Some((1, 1))));
    let mut reste = Vec::new();
    for i in 0..(POSES_AU_DEPART + 8) {
        let nom = format!("t{i}");
        let cle = Cle::from(&nom);
        scene.televerser(
            &peripherique,
            &mut envoi,
            (&cle, &cle),
            photo(1, [0, 0, 200, 255]).as_ref(),
        );
        reste.push(a_poser(&nom, pose_de(40.0, 40.0, 4.0, 4.0)));
    }
    let en_place = lot_entre((0.0, 0.0, 1.0), (32.0, 0.0, 1.0), 1);
    let retenues =
        scene.preparer_la_vue(&peripherique, &file, ecran, &reste, (Some(en_place), None));
    assert!(scene.capacite > POSES_AU_DEPART, "le tampon a dû grandir");
    let octets = dessiner_la_scene(
        (&peripherique, &file),
        &mut envoi,
        (&scene, &retenues),
        cote,
    );
    assert_eq!(
        pixel(&octets, cote, 40, 8),
        [200, 0, 0, 255],
        "le lot, décalé de 32"
    );
    assert_eq!(
        pixel(&octets, cote, 42, 42),
        [0, 0, 200, 255],
        "ce qui suit le lot"
    );
}

/// **Une tranche ne dessine que ses poses, même au milieu d'un seul dessin** (ATLAS-1) : les lueurs
/// passent entre les photos et les cartes, et une photo et une carte peuvent vivre dans la même page.
/// Sans cette coupe, la carte passerait sous la lueur qui devait la suivre.
#[test]
fn test_atlas_1_une_tranche_ne_dessine_que_ses_poses() {
    let Some((peripherique, file)) = carte() else {
        eprintln!("aucune carte graphique : test saute");
        return;
    };
    let cote = 64;
    let mut envoi = crate::present::envoi::Envoi::nouveau(&peripherique, &file);
    let mut scene = SceneGpu::nouvelle(&peripherique, wgpu::TextureFormat::Rgba8Unorm);
    scene.ouvrir();
    let (a, b) = (Cle::from("a"), Cle::from("b"));
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&a, &a),
        photo(8, [200, 0, 0, 255]).as_ref(),
    );
    scene.televerser(
        &peripherique,
        &mut envoi,
        (&b, &b),
        photo(8, [0, 200, 0, 255]).as_ref(),
    );
    let poses = [
        a_poser("a", pose_de(0.0, 0.0, 16.0, 16.0)),
        a_poser("b", pose_de(32.0, 0.0, 16.0, 16.0)),
    ];
    let retenues = scene.preparer(&peripherique, &file, (cote as f32, cote as f32), &poses);
    assert_eq!(retenues.suites.len(), 1, "la même page : un seul dessin");
    let carte_a = (&peripherique, &file);
    let premiere = dessiner_la_tranche(carte_a, &mut envoi, (&scene, &retenues, 0..1), cote);
    assert_eq!(pixel(&premiere, cote, 8, 8), [200, 0, 0, 255]);
    assert_eq!(
        pixel(&premiere, cote, 40, 8),
        [0, 0, 0, 0],
        "la seconde n'en est pas"
    );
    let seconde = dessiner_la_tranche(carte_a, &mut envoi, (&scene, &retenues, 1..2), cote);
    assert_eq!(
        pixel(&seconde, cote, 8, 8),
        [0, 0, 0, 0],
        "la première non plus"
    );
    assert_eq!(pixel(&seconde, cote, 40, 8), [0, 200, 0, 255]);
}
