//! Les textures qui portent l'image, en anneau (BLIT-1).
//!
//! # Pourquoi plusieurs, alors qu'une seule est affichée
//!
//! Écrire dans une texture que la carte est encore en train de lire pour l'image précédente
//! fait **attendre l'écriture** : le pilote n'a pas d'autre choix que de faire patienter le
//! processeur jusqu'à ce qu'elle se libère.
//!
//! Mesuré chez l'utilisateur, avec une seule texture :
//!
//! ```text
//!    2.5s   501.30ms  repos   1 noeud,  0 photo   dont blit 488.72ms
//! ```
//!
//! Quatre cent quatre-vingt-huit millisecondes de téléversement sur une scène **sans une
//! seule photo**. Ce n'était pas le dessin qui coûtait, c'était l'attente — et aucune
//! optimisation du rendu n'y pouvait quoi que ce soit.
//!
//! # Leur nombre ne se choisit pas
//!
//! C'est celui des images que la chaîne garde en vol, **plus celle qu'on écrit**. En dessous,
//! l'écriture attend ; au-dessus, on garde de la mémoire pour rien. La chaîne le dit
//! elle-même, on ne le suppose pas.

/// De quoi fabriquer les textures : tout ce qui ne change pas d'une image à l'autre.
pub(super) struct Fabrique<'a> {
    pub device: &'a wgpu::Device,
    pub layout: &'a wgpu::BindGroupLayout,
    pub sampler: &'a wgpu::Sampler,
    pub format: wgpu::TextureFormat,
    pub combien: usize,
}

/// Les textures de l'image, et celle où écrire la prochaine.
pub(super) struct Anneau {
    textures: Vec<(wgpu::Texture, wgpu::BindGroup)>,
    taille: (u32, u32),
    prochaine: usize,
    courante: usize,
}

impl Anneau {
    pub(super) fn nouveau() -> Self {
        Self {
            textures: Vec::new(),
            taille: (0, 0),
            prochaine: 0,
            courante: 0,
        }
    }

    /// La texture où écrire cette image, en avançant d'un cran.
    ///
    /// Les refait toutes quand la fenêtre change de taille : elles n'ont alors plus rien à
    /// porter, et la chaîne d'images est de toute façon reconstruite au même moment.
    pub(super) fn pour(&mut self, f: &Fabrique<'_>, w: u32, h: u32) -> wgpu::Texture {
        if self.taille != (w, h) || self.textures.len() != f.combien {
            self.textures = (0..f.combien.max(1))
                .map(|_| faire_une_texture(f, w, h))
                .collect();
            self.taille = (w, h);
            self.prochaine = 0;
        }
        self.courante = self.prochaine;
        self.prochaine = (self.prochaine + 1) % self.textures.len();
        self.textures[self.courante].0.clone()
    }

    /// Le groupe de liaison de la texture qu'on vient d'écrire.
    ///
    /// # Panique
    ///
    /// Seulement si personne n'a jamais appelé [`Anneau::pour`], ce que la présentation fait
    /// toujours avant de dessiner : le téléversement précède l'affichage, dans cet ordre.
    pub(super) fn courante(&self) -> &wgpu::BindGroup {
        &self.textures[self.courante].1
    }
}

/// Une texture de l'anneau, avec le groupe de liaison qui la donne au nuanceur.
fn faire_une_texture(f: &Fabrique<'_>, w: u32, h: u32) -> (wgpu::Texture, wgpu::BindGroup) {
    let texture = f.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glucose-image"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: f.format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind = f.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("glucose-image"),
        layout: f.layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(f.sampler),
            },
        ],
    });
    (texture, bind)
}
