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
        scene.televerser(&peripherique, &file, (cle, cle), source);
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
        scene.poser(&mut passe, &retenues);
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
    scene.televerser(&peripherique, &file, ("a", "a"), &photo(4, [1, 2, 3, 255]));
    scene.televerser(&peripherique, &file, ("b", "b"), &photo(4, [4, 5, 6, 255]));
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
    };
    scene.preparer(&peripherique, &file, (8.0, 8.0), &[a_poser("a", pose)]);
    scene.fermer();

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
        &photo(4, [1, 2, 3, 255]),
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
    };
    let retenues = scene.preparer(
        &peripherique,
        &file,
        (16.0, 16.0),
        &[APoser {
            cle: "carte:c1:neuf".to_string(),
            identite: "carte:c1".to_string(),
            pose,
        }],
    );
    assert_eq!(
        retenues,
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
        &photo(4, [1, 2, 3, 255]),
    );

    let pose = Pose {
        x: 0.0,
        y: 0.0,
        largeur: 4.0,
        hauteur: 4.0,
        opacite: 1.0,
        angle: 0.0,
        fenetre: Pose::TOUT,
    };
    let demande = |cle: &str, identite: &str| APoser {
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
        &|_| Some(photo(4, [9, 9, 9, 255])),
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
        &photo(4, [1, 1, 1, 255]),
    );
    scene.televerser(
        &peripherique,
        &file,
        ("c", "c:x2"),
        &photo(4, [2, 2, 2, 255]),
    );
    assert!(scene.connait("c", "c:x2"), "la neuve a pris la place");
    assert!(
        !scene.connait("c", "c:x1"),
        "et l'ancienne n'est plus la : une identite ne porte qu'une texture"
    );
}
