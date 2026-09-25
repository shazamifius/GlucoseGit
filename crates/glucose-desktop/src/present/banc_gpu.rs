//! **Le harnais qui fait tourner une passe graphique hors fenêtre**, pour que les tests
//! puissent comparer les deux voies pixel par pixel.
//!
//! # Pourquoi il existe à part
//!
//! La garantie centrale de la charte est que **deux voies d'une même opération produisent le
//! même résultat** — au bit près quand c'est possible, à un écart borné et mesuré sinon. Une
//! passe graphique ne se relit pas : une erreur de signe sur l'axe vertical, un mélange qui
//! n'est pas prémultiplié, un arrondi qui part dans l'autre sens, rien de cela ne se voit dans
//! le code et tout se voit à l'écran.
//!
//! La seule épreuve qui vaille est donc de **dessiner et de relire les pixels**, et ce module
//! est ce qu'il faut pour le faire : ouvrir une carte, rendre dans une texture, la ramener.
//!
//! Sur une machine sans carte utilisable, les tests se **sautent** au lieu d'échouer :
//! l'absence de matériel n'est pas un défaut du code.

use tiny_skia::Pixmap;

/// Le format des cibles de test : celui où les octets partent tels quels, sans conversion.
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Un périphérique hors fenêtre, ou `None` si cette machine n'en offre pas.
pub fn carte() -> Option<(wgpu::Device, wgpu::Queue)> {
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

/// Une cible hors fenêtre de `taille` pixels, et sa vue.
pub fn cible(peripherique: &wgpu::Device, taille: (u32, u32)) -> wgpu::Texture {
    peripherique.create_texture(&wgpu::TextureDescriptor {
        label: Some("banc: cible"),
        size: wgpu::Extent3d {
            width: taille.0,
            height: taille.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// Ouvre une passe qui **efface** la cible à cette couleur, puis laisse `dessiner` la remplir.
///
/// L'effacement joue le rôle du `fill` de la voie processeur : les deux partent du même fond,
/// sans quoi la comparaison ne dirait rien.
pub fn passe(
    encodeur: &mut wgpu::CommandEncoder,
    vue: &wgpu::TextureView,
    fond: wgpu::Color,
    dessiner: impl FnOnce(&mut wgpu::RenderPass<'_>),
) {
    let mut passe = encodeur.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("banc"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: vue,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(fond),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    dessiner(&mut passe);
}

/// Ramène les pixels d'une cible dans un `Pixmap`, lignes dépadées.
///
/// La copie d'une texture impose des lignes alignées sur 256 octets ; le pixmap, lui, les
/// veut jointives. Le rembourrage se retire ici, et nulle part ailleurs : c'est une contrainte
/// de transfert, pas une propriété de l'image.
pub fn relire(
    peripherique: &wgpu::Device,
    file: &wgpu::Queue,
    texture: &wgpu::Texture,
    taille: (u32, u32),
) -> Option<Pixmap> {
    let par_ligne = (taille.0 * 4).div_ceil(256) * 256;
    let tampon = peripherique.create_buffer(&wgpu::BufferDescriptor {
        label: Some("banc: relecture"),
        size: u64::from(par_ligne) * u64::from(taille.1),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    encodeur.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &tampon,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(par_ligne),
                rows_per_image: Some(taille.1),
            },
        },
        wgpu::Extent3d {
            width: taille.0,
            height: taille.1,
            depth_or_array_layers: 1,
        },
    );
    file.submit(Some(encodeur.finish()));
    tampon.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    peripherique
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .ok()?;

    let mut pixmap = Pixmap::new(taille.0, taille.1)?;
    {
        let lu = tampon.slice(..).get_mapped_range().ok()?;
        let destination = pixmap.data_mut();
        for y in 0..taille.1 as usize {
            let depuis = y * par_ligne as usize;
            let vers = y * taille.0 as usize * 4;
            let large = taille.0 as usize * 4;
            destination[vers..vers + large].copy_from_slice(&lu[depuis..depuis + large]);
        }
    }
    tampon.unmap();
    Some(pixmap)
}

/// Le pire écart, canal par canal, entre deux images de même taille.
///
/// C'est la seule grandeur qui vaille pour cette garantie : une moyenne cacherait un pixel
/// faux parmi un million de justes, et c'est exactement la leçon que la chronique a coûté
/// cher à apprendre (fiche 20 § 4.5).
pub fn pire_ecart(a: &Pixmap, b: &Pixmap) -> u8 {
    a.data()
        .iter()
        .zip(b.data())
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

/// Combien de canaux diffèrent de plus de `tolerance`.
pub fn canaux_hors_tolerance(a: &Pixmap, b: &Pixmap, tolerance: u8) -> usize {
    a.data()
        .iter()
        .zip(b.data())
        .filter(|(x, y)| x.abs_diff(**y) > tolerance)
        .count()
}

/// **Compose les cinq temps hors fenêtre**, exactement comme la présentation le fait, et
/// rend l'image.
///
/// Écrite une fois ici plutôt que dans chaque banc et chaque épreuve : deux copies de la
/// même composition finiraient par ne plus composer la même chose, et c'est précisément ce
/// que ces épreuves existent pour attraper.
///
/// `source` donne les pixels d'une texture que la carte ne connaît pas : une photo décodée,
/// ou un composant rendu à la demande (COMPOSANT-1).
pub fn composer_les_cinq_temps(
    (peripherique, file): (&wgpu::Device, &wgpu::Queue),
    taille: (u32, u32),
    confie: &crate::renderer::Confie,
    (dessous, dessus): (&Pixmap, &Pixmap),
    source: &crate::present::scene_gpu::Source<'_>,
) -> Option<Pixmap> {
    use crate::present::{couches, fond_gpu, lueurs_gpu, membranes_gpu, scene_gpu};
    let ecran = (taille.0 as f32, taille.1 as f32);
    let mut fond = fond_gpu::FondGpu::nouveau(peripherique, FORMAT);
    let mut lueurs = lueurs_gpu::Lueurs::nouvelles(peripherique, FORMAT);
    let mut membranes = membranes_gpu::Membranes::nouvelles(peripherique, FORMAT);
    let mut scene = scene_gpu::SceneGpu::nouvelle(peripherique, FORMAT);
    let mut deux = couches::Couches::nouvelles(peripherique, FORMAT);

    fond.preparer(file, ecran, confie.fond);
    lueurs.preparer(peripherique, file, ecran, &confie.lueurs);
    membranes.preparer(peripherique, file, ecran, &confie.membranes);
    let textures = confie.textures();
    scene.ouvrir();
    // Le banc et l'epreuve des deux voies comparent des IMAGES : elles doivent etre
    // completes, donc aucun report. Un budget illimite est la seule valeur juste ici --
    // une cascade rendrait la comparaison dependante du temps qu'il fait.
    scene.assurer(
        peripherique,
        file,
        (&textures, std::time::Duration::MAX),
        source,
    );
    let retenues = scene.preparer(peripherique, file, ecran, &textures);
    let utile = confie.fond.is_none() || confie.dessous_porte_quelque_chose;
    // Le banc et l'epreuve des deux voies composent des images COMPLETES : tout part.
    deux.televerser(
        peripherique,
        file,
        (
            dessous,
            utile,
            &crate::present::bandes::Bandes::tout(dessous.height()),
        ),
        (
            dessus,
            &crate::present::bandes::Bandes::tout(dessus.height()),
        ),
    );

    let cible = cible(peripherique, taille);
    let vue = cible.create_view(&Default::default());
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    couches::composer(
        &mut encodeur,
        &vue,
        couches::Temps {
            fond: &fond,
            lueurs: &lueurs,
            membranes: &membranes,
            couches: &deux,
            scene: &scene,
            retenues: &retenues,
        },
    );
    file.submit(Some(encodeur.finish()));
    relire(peripherique, file, &cible, taille)
}
