//! **La voie graphique : les photos vivent sur la carte, et la vue n'est qu'une matrice.**
//!
//! # Ce que ce module change, et pourquoi il existe
//!
//! Jusqu'ici la carte ne faisait que **poser l'image finie** ([`super::gpu`]). Toute la scène
//! se composait sur le processeur, pixel par pixel, à chaque image — et c'est ce qui obligeait
//! à pixeliser en mouvement : interpoler quatre texels par pixel d'écran coûtait quatre fois
//! le budget d'une image.
//!
//! Ici, une photo décodée devient une **texture téléversée une fois**. Déplacer la vue ne
//! relit plus un seul octet de la source : cela change quatre nombres par photo. Et le
//! filtrage bilinéaire est **câblé dans le silicium** — il ne coûte rien, à aucune échelle.
//!
//! `bench_voie_gpu` le chiffre, sur 429 photos et 2560 × 1600, à une échelle non entière :
//!
//! ```text
//!     voie processeur, seize fils      4,2 a 6,3 ms   et PIXELISEE
//!     Intel Arc 140T (integre)              1,22 ms   et nette
//!     RTX 5070 Laptop (dediee)              0,26 ms   et nette
//! ```
//!
//! # Ce que ce module ne fait pas, et ne fera pas
//!
//! Ni le texte, ni les formes, ni les lueurs : ils restent au processeur, composés autour des
//! photos (fiche 21, étape 1). Il ne décide pas non plus **quand** la voie graphique sert —
//! c'est l'arbitre qui le fera, en observant le débit de chacune, et il n'existe pas encore.
//!
//! # Un dessin par photo, et pourquoi c'est le bon départ
//!
//! Chaque grande photo a sa texture, donc sa liaison de ressources, donc son dessin. **Les petites
//! se rangent dans des pages** ([`atlas`]), et les poses consécutives d'une page partent en un
//! seul dessin : un banc l'a réclamé à huit mille images (ATLAS-1, fiche 86).

mod atlas;
mod cascade;
mod cle;
mod construction;
mod lot;
mod memoire;
mod pose;
pub use atlas::octets_d_une_texture;
pub use cle::Cle;
pub use pose::Pose;
use pose::OCTETS_POSE;

use crate::renderer::voies::APoser;
use tiny_skia::Pixmap;

/// **Ce qui donne les pixels d'une texture que la carte ne connaît pas encore.**
///
/// Pas un `Pixmap` : un composant se rend et se donne, mais une photo existe déjà dans le
/// magasin, et la copier avant de la téléverser coûtait plus que le téléversement lui-même —
/// 10,5 ms de copie pour 2,9 ms d'envoi sur une épingle de 27 Mo (`bench_televersement`).
/// Elle se prête.
pub type Source<'s> = dyn Fn(&Cle) -> Option<Pixels<'s>> + 's;

/// **Les pixels d'une texture** : prêtés par le magasin, ou rendus pour l'occasion.
///
/// Un niveau de pyramide ne vit plus forcément dans un `Pixmap` — ses pages peuvent avoir été
/// offertes au système et reprises (ETAGES-1) —, d'où une vue empruntée plutôt qu'un `Cow`.
pub enum Pixels<'s> {
    Pretes(tiny_skia::PixmapRef<'s>),
    Rendues(Pixmap),
}

impl Pixels<'_> {
    /// Les pixels eux-mêmes, d'où qu'ils viennent.
    pub fn vue(&self) -> tiny_skia::PixmapRef<'_> {
        match self {
            Self::Pretes(p) => *p,
            Self::Rendues(p) => p.as_ref(),
        }
    }
}

/// Les photos que la carte détient, et de quoi les poser.
pub struct SceneGpu {
    pipeline: wgpu::RenderPipeline,
    /// Ce qui ne change pas d'une photo à l'autre : les poses, l'écran, le filtre.
    commun: wgpu::BindGroup,
    /// Ce qui change : la texture de la photo courante.
    disposition_photo: wgpu::BindGroupLayout,
    echantillonneur: wgpu::Sampler,
    poses: wgpu::Buffer,
    ecran: wgpu::Buffer,
    /// Combien de poses le tampon peut porter avant d'être refait.
    capacite: usize,
    /// Ce que la carte détient, **indexé par identité et non par clé**.
    ///
    /// Une identité porte au plus une texture : quand un composant change de palier, la
    /// nouvelle remplace l'ancienne, et la mémoire ne double jamais. Tant que la nouvelle
    /// n'est pas rendue, c'est l'ancienne qui se pose — un peu floue, jamais absente.
    photos: std::collections::HashMap<Cle, Televersee, glucose_core::rapide::Rapide>,
    /// Les pages qui rangent les petites textures (ATLAS-1).
    atlas: atlas::Atlas,
    /// `GLUCOSE_SANS_ATLAS` : un instrument, comme `GLUCOSE_CARTE`, pour comparer les deux chemins
    /// dans les mêmes conditions.
    sans_atlas: bool,
    image: u64,
    /// Ce que les textures gardées hors de l'écran occupent, en octets (VRAM-1).
    en_cache: u64,
    /// Ce qu'une texture a coûté sur cette machine, par pixel posé (CASCADE-3).
    debit: cascade::Debit,
    /// **La dernière cascade a-t-elle reporté quelque chose ?** (NETTETE-1) Ce qui attend
    /// demande l'image suivante, même quand plus rien ne bouge.
    inachevee: bool,
    /// **Les poses que le tampon garde** (LOT-1) : les premières, écrites par une image complète.
    lot: Option<lot::LotTenu>,
}

struct Televersee {
    liaison: wgpu::BindGroup,
    /// **Le numéro de cette liaison** : les poses consécutives d'un même lot sont un seul dessin.
    lot: u64,
    /// Sa place dans une page, si elle en a une (ATLAS-1).
    cellule: Option<atlas::Cellule>,
    /// **Ce que cette texture montre** : la clé qui l'a produite.
    ///
    /// Quand elle diffère de celle que la scène demande, la texture est périmée — elle se
    /// pose quand même, et se refera dès qu'il y aura du temps pour elle.
    cle: Cle,
    /// La dernière image où cette photo a été posée.
    vue: u64,
    /// Ce qu'elle occupe sur la carte, en octets.
    octets: u64,
    /// Une photo, et non une carte de texte : ses pixels ont été prêtés par le magasin
    /// (ETAGES-3). C'est ce que le cran des photos peut réduire.
    photo: bool,
}

/// Combien de poses le tampon porte au départ — il grandit si l'écran en demande plus.
///
/// Cinq cent douze : le document de l'utilisateur en compte quatre cent trente, et un écran
/// n'en montre jamais beaucoup plus à la fois puisque le culling les a déjà filtrées.
const POSES_AU_DEPART: usize = 512;

impl SceneGpu {
    /// Construit le pipeline. `format` est celui de la cible où les photos se poseront.
    pub fn nouvelle(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let (commun_layout, disposition_photo) = Self::dispositions(peripherique);
        let pipeline =
            Self::pipeline_des_photos(peripherique, format, &commun_layout, &disposition_photo);
        let echantillonneur = peripherique.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("scene: bilineaire"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let poses = Self::tampon_des_poses(peripherique, POSES_AU_DEPART);
        let ecran = peripherique.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene: ecran"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let commun = Self::liaison_commune(
            peripherique,
            &commun_layout,
            &poses,
            &ecran,
            &echantillonneur,
        );
        Self {
            pipeline,
            commun,
            disposition_photo,
            echantillonneur,
            poses,
            ecran,
            capacite: POSES_AU_DEPART,
            photos: std::collections::HashMap::default(),
            image: 0,
            en_cache: 0,
            atlas: atlas::Atlas::default(),
            sans_atlas: std::env::var_os("GLUCOSE_SANS_ATLAS").is_some(),
            debit: cascade::Debit::default(),
            inachevee: false,
            lot: None,
        }
    }

    /// La carte détient-elle **exactement** ce que cette texture montre ?
    ///
    /// Une identité connue dont la clé diffère n'est pas « connue » : elle est périmée, et
    /// elle se refera quand le budget le permettra.
    pub fn connait(&self, identite: &Cle, cle: &Cle) -> bool {
        self.photos.get(identite).is_some_and(|t| t.cle == *cle)
    }

    /// La carte détient-elle quelque chose à poser pour cette identité, fût-ce périmé ?
    pub fn detient(&self, identite: &Cle) -> bool {
        self.photos.contains_key(identite)
    }

    /// **Téléverse une photo décodée, une fois pour toutes.**
    ///
    /// C'est le seul moment où ses octets traversent le bus. Tout le reste de sa vie —
    /// déplacements, zooms, changements d'opacité — ne coûte que sa pose.
    pub fn televerser(
        &mut self,
        peripherique: &wgpu::Device,
        envoi: &mut crate::present::envoi::Envoi,
        (identite, cle): (&Cle, &Cle),
        source: tiny_skia::PixmapRef<'_>,
    ) {
        if self.televerser_dans_l_atlas(peripherique, envoi, (identite, cle), source) {
            return;
        }
        let (l, h) = (source.width(), source.height());
        let taille = wgpu::Extent3d {
            width: l,
            height: h,
            depth_or_array_layers: 1,
        };
        let texture = peripherique.create_texture(&wgpu::TextureDescriptor {
            label: Some("photo"),
            size: taille,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // `tiny-skia` produit du RGBA huit bits premultiplie : les octets partent tels
            // quels, comme pour l'image de presentation.
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        envoi.texture(&texture, (0, 0), (l, h), (source.data(), l as usize * 4));
        let vue = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let liaison = peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("photo"),
            layout: &self.disposition_photo,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&vue),
            }],
        });
        let image = self.image;
        // La nouvelle texture **remplace** celle que cette identité portait : une identité
        // n'en a jamais deux, donc la mémoire ne double pas pendant un changement de palier.
        self.oublier_la_texture(identite);
        let lot = self.atlas.lot_neuf();
        self.photos.insert(
            identite.clone(),
            Televersee {
                liaison,
                lot,
                cellule: None,
                cle: cle.clone(),
                vue: image,
                octets: u64::from(l) * u64::from(h) * 4,
                photo: false,
            },
        );
    }

    /// Marque cette texture comme une photo : ses pixels venaient du magasin.
    pub(super) fn marquer_photo(&mut self, identite: &Cle) {
        if let Some(t) = self.photos.get_mut(identite) {
            t.photo = true;
        }
    }

    /// **Ce que les photos posées à cette image occupent sur la carte** (ETAGES-3).
    pub fn octets_des_photos_posees(&self) -> u64 {
        self.photos
            .values()
            .filter(|t| t.photo && t.vue == self.image)
            .map(|t| t.octets)
            .sum()
    }

    /// Ouvre une image : ce qui ne servira pas d'ici à [`SceneGpu::fermer`] sera oublié.
    pub fn ouvrir(&mut self) {
        self.image += 1;
    }

    /// **Oublie ce qui n'a pas servi à cette image, au-delà de ce que la carte peut garder**
    /// (VRAM-1).
    ///
    /// Ce qui a servi reste toujours. Ce qui a quitté l'écran se garde, **les plus récemment
    /// vues d'abord**, tant que le tout tient dans `gardable` octets — ce que le budget de la
    /// carte accorde à un cache ([`crate::memoire::MemoireGraphique::part_pour_un_cache`]).
    /// Revenir sur une photo gardée ne la téléverse pas : elle est là, et elle se pose.
    ///
    /// Sans budget connu, `gardable` vaut zéro, et la borne redevient **ce que l'écran demande**
    /// — la loi d'avant, qui ne choisissait aucun nombre et n'en choisit toujours pas.
    pub fn fermer(&mut self, gardable: u64) {
        let image = self.image;
        let mut hors_ecran: Vec<(u64, u64, Cle)> = self
            .photos
            .iter()
            .filter(|(_, t)| t.vue != image)
            .map(|(identite, t)| (t.vue, t.octets, identite.clone()))
            .collect();
        hors_ecran.sort_by_key(|(vue, ..)| std::cmp::Reverse(*vue));
        self.en_cache = 0;
        for (_, octets, identite) in hors_ecran {
            if self.en_cache + octets <= gardable {
                self.en_cache += octets;
            } else {
                self.oublier_la_texture(&identite);
            }
        }
    }

    /// Ce que les textures gardées hors de l'écran occupent, en octets.
    pub fn octets_en_cache(&self) -> u64 {
        self.en_cache
    }

    /// **Écrit les poses dans le tampon**, et rend celles que la carte saura dessiner.
    ///
    /// À appeler avant d'ouvrir la passe : un tampon ne s'écrit pas pendant qu'on dessine.
    /// Une photo que la carte ne connaît pas est simplement omise — c'est à l'appelant de la
    /// téléverser avant, et de savoir qu'elle manque.
    pub fn preparer(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        ecran: (f32, f32),
        photos: &[APoser],
    ) -> Retenues {
        self.preparer_la_vue(peripherique, file, ecran, photos, (None, None))
    }

    /// **La même chose, avec le lot** (LOT-1) : `en_place` dit que les premières poses sont déjà
    /// dans le tampon, et `propose` que les `nombre` premières demandées forment un lot neuf.
    pub fn preparer_la_vue(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        ecran: (f32, f32),
        photos: &[APoser],
        (en_place, propose): (
            Option<crate::renderer::voies::LotEnPlace>,
            Option<(u64, usize)>,
        ),
    ) -> Retenues {
        // Le rendu n'a demandé ce lot qu'après avoir lu que la carte le tenait, et rien ne passe
        // entre les deux. S'il manquait quand même, une image incomplète, et la suivante refait tout.
        let en_place = en_place.filter(|_| self.lot.is_some());
        if en_place.is_none() {
            self.lot = None;
        }
        let du_lot = en_place.map_or(0, |lot| lot.nombre);
        self.assurer_la_capacite(
            peripherique,
            file,
            du_lot + photos.len(),
            en_place.is_some(),
        );
        let mut octets = Vec::with_capacity(photos.len() * OCTETS_POSE);
        let mut retenues = self.retenues_du_lot(en_place);
        let image = self.image;
        let mut perimees = 0.0_f64;
        let mut a_jour = 0;
        for (rang, t) in photos.iter().enumerate() {
            // **Le repli se garde tant que sa photo peut en avoir besoin** (ETAGES-1). Il ne se
            // pose qu'à la place d'une photo absente ; oublié à la fin de chaque image où elle
            // est là, il manquerait au premier zoom, qui change son niveau.
            if let Some(gardee) = t
                .repli
                .as_deref()
                .and_then(|r| self.photos.get_mut(&r.identite))
            {
                gardee.vue = image;
            }
            // **On pose ce que la carte détient pour cette identité**, à jour ou non. Une
            // texture périmée est l'ancien palier : elle se pose à la place demandée, donc
            // au bon endroit et à la bonne taille, et la carte la filtre. C'est un demi-pixel
            // d'adoucissement pendant une image ou deux, contre une image qui gèle. À défaut,
            // son repli : un peu flou, jamais un trou.
            let posee = match t.repli.as_deref() {
                Some(repli) if !self.photos.contains_key(&t.identite) => repli,
                _ => t,
            };
            let Some(televersee) = self.photos.get_mut(&posee.identite) else {
                continue;
            };
            if televersee.cle != t.cle {
                perimees += 1.0;
            } else {
                a_jour += 1;
            }
            televersee.vue = image;
            televersee
                .cellule
                .map_or(posee.pose, |c| c.dans_la_page(posee.pose))
                .ecrire(&mut octets);
            retenues.retenir(&posee.identite, rang + du_lot, televersee);
            // Le lot neuf ne se juge que sur ses photos : ce qui suit n'en fait pas partie.
            if propose.is_some_and(|(_, nombre)| rang + 1 == nombre) {
                self.retenir_le_lot(propose, a_jour, &retenues, &octets);
            }
        }
        crate::perf::compteur("textures_perimees", perimees);
        if !octets.is_empty() {
            file.write_buffer(&self.poses, (du_lot * OCTETS_POSE) as u64, &octets);
        }
        self.ecrire_l_ecran(file, ecran, en_place);
        retenues
    }

    /// **Pose dans la passe en cours** une tranche de ce que [`SceneGpu::preparer`] a retenu.
    ///
    /// Une tranche et non le tout : les lueurs des cartes passent **entre** les photos et les
    /// cartes (LUEUR-3), donc la pose se fait en deux fois, autour d'elles. Le rang d'une
    /// texture reste celui de sa pose dans le tampon, quelle que soit la tranche.
    pub fn poser(
        &self,
        passe: &mut wgpu::RenderPass<'_>,
        retenues: &Retenues,
        tranche: std::ops::Range<usize>,
    ) {
        if tranche.is_empty() {
            return;
        }
        passe.set_pipeline(&self.pipeline);
        passe.set_bind_group(0, &self.commun, &[]);
        let (debut, fin) = (tranche.start as u32, tranche.end as u32);
        for suite in &retenues.suites {
            let (d, f) = (suite.debut.max(debut), suite.fin.min(fin));
            if d < f {
                passe.set_bind_group(1, &suite.liaison, &[]);
                passe.draw(0..6, d..f);
            }
        }
    }
}

/// **Ce que la carte saura poser à cette image**, dans l'ordre de la pose : l'identité de
/// chaque texture retenue, et son rang dans la liste demandée.
///
/// Le rang est ce qui permet de couper la pose là où la scène change de nature — après les
/// photos, avant les cartes de texte — sans supposer qu'aucune n'a été omise : une photo que
/// la carte ne détient pas encore est absente des retenues, et la frontière se déplace avec
/// elle.
#[derive(Debug, Clone, Default)]
pub struct Retenues {
    /// Ce que cette image a préparé : le lot en place n'y est pas (LOT-1).
    pub cles: Vec<Cle>,
    rangs: Vec<usize>,
    /// Combien de poses viennent du lot en place : les premières.
    du_lot: usize,
    /// **Les dessins de cette image** : une suite de poses consécutives d'un même lot en est un
    /// seul (ATLAS-1). L'ordre du modèle reste la loi : une pose d'ailleurs coupe la suite à sa
    /// place.
    suites: Vec<Suite>,
}

/// Un dessin : une liaison, et les poses du tampon qu'il couvre.
#[derive(Debug, Clone)]
struct Suite {
    lot: u64,
    liaison: wgpu::BindGroup,
    debut: u32,
    fin: u32,
}

impl Retenues {
    /// Retient une pose de plus : elle prolonge le dernier dessin si elle lit le même lot.
    fn retenir(&mut self, identite: &Cle, rang: usize, televersee: &Televersee) {
        let i = (self.du_lot + self.cles.len()) as u32;
        match self.suites.last_mut() {
            Some(suite) if suite.lot == televersee.lot => suite.fin = i + 1,
            _ => self.suites.push(Suite {
                lot: televersee.lot,
                liaison: televersee.liaison.clone(),
                debut: i,
                fin: i + 1,
            }),
        }
        self.cles.push(identite.clone());
        self.rangs.push(rang);
    }

    /// Combien de textures la carte posera.
    pub fn len(&self) -> usize {
        self.du_lot + self.cles.len()
    }

    /// Aucune ?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// **Combien des retenues viennent des `n` premières demandées** : là où la pose se coupe.
    ///
    /// Les rangs croissent avec la liste demandée, donc une dichotomie suffit.
    pub fn avant(&self, n: usize) -> usize {
        // Le lot est complet : ses rangs sont ceux de ses poses.
        if n <= self.du_lot {
            return n;
        }
        self.du_lot + self.rangs.partition_point(|&r| r < n)
    }
}

#[cfg(test)]
mod tests;
