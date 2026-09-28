//! **Ce qui part vers la carte, par des tampons qui restent** (ENVOI-1, fiche 43).
//!
//! # Le défaut que ce module répare
//!
//! La carte s'ouvre avec l'allocateur au plus juste (`MemoryHints::MemoryUsage`, ETAGES-1) :
//! il ne réserve rien d'avance, et c'est ce qui rend la mémoire graphique aux applications
//! d'à côté. Mais ses blocs de mémoire visible du processeur font alors quatre mébioctets, et
//! `Queue::write_texture` range chaque envoi dans un tampon de transfert **neuf** : tout envoi
//! plus gros qu'un bloc reçoit une allocation dédiée, créée puis rendue au pilote à chaque
//! fois.
//!
//! Mesuré sur sa RTX (`bench_envoi`) : la couche du dessus entière — une Time Machine ouverte
//! en fait partir toutes les lignes, 11 Mo — coûtait **5,27 ms** au lieu de 1,20 ; un niveau de
//! photo neuf de 9 Mo, **3,69 ms** au lieu de 0,97. Sur le terrain, avec les images en vol,
//! `blit` et `soumettre` en portaient 10 et 9 à chaque image, panneau immobile.
//!
//! # Ce qui le remplace
//!
//! Des tampons de transfert qui **restent** : chacun a la taille de ce qu'il a porté la
//! première fois, et sert de nouveau dès que la carte a fini de le lire (`StagingBelt` de
//! wgpu, sans taille de tranche choisie — zéro : un tampon est ce qu'il porte). L'allocation
//! se paie une fois ; ensuite, un envoi ne coûte que la copie de ses octets. Ces tampons vivent
//! dans la mémoire du système, pas dans celle de la carte : la raison d'ETAGES-1 tient.
//!
//! Les copies s'enregistrent dans une commande à part, confiée à la carte d'un bloc
//! ([`Envoi::soumettre`]) **avant** la commande de l'image qui s'en sert : sur une même file, la
//! carte exécute les soumissions dans l'ordre.

/// Où une copie vers une texture peut commencer dans un tampon : au début d'un texel, soit
/// quatre octets pour les textures RGBA de huit bits que Glucose envoie.
const DEBUT_D_UN_TEXEL: wgpu::BufferSize = match wgpu::BufferSize::new(4) {
    Some(n) => n,
    None => unreachable!(),
};

/// Les envois d'une image, préparés dans des tampons qui restent.
pub struct Envoi {
    peripherique: wgpu::Device,
    file: wgpu::Queue,
    tapis: wgpu::util::StagingBelt,
    /// Les copies de l'image en cours, s'il y en a.
    copies: Option<wgpu::CommandEncoder>,
}

impl Envoi {
    pub fn nouveau(peripherique: &wgpu::Device, file: &wgpu::Queue) -> Self {
        Self {
            peripherique: peripherique.clone(),
            file: file.clone(),
            tapis: wgpu::util::StagingBelt::new(peripherique.clone(), 0),
            copies: None,
        }
    }

    /// **Envoie un rectangle de texels** : `largeur × hauteur`, lus dans `source` à raison de
    /// `rang_source` octets d'une rangée à la suivante, vers `cible` à partir de `origine`.
    ///
    /// Une copie de tampon vers texture exige des rangées alignées sur 256 octets : elles le
    /// sont dans le tampon de transfert, qui les reçoit une par une. La source, elle, garde les
    /// siennes — une couche de 2 160 pixels a des rangées de 8 640 octets.
    pub fn texture(
        &mut self,
        cible: &wgpu::Texture,
        origine: (u32, u32),
        (largeur, hauteur): (u32, u32),
        (source, rang_source): (&[u8], usize),
    ) {
        let rang = largeur * 4;
        let aligne = rang.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let Some(total) = wgpu::BufferSize::new(u64::from(aligne) * u64::from(hauteur)) else {
            return;
        };
        let copies = self.copies.get_or_insert_with(|| {
            self.peripherique
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("glucose-envoi"),
                })
        });
        let tranche = self.tapis.allocate(total, DEBUT_D_UN_TEXEL);
        {
            // La vue se rend avant la commande : une tranche projetée ne se copie pas.
            let mut vue = tranche.get_mapped_range_mut().expect(
                "le tapis ne rend que des tranches de tampons projetés en écriture : il les crée                  projetés, et ne les reprend qu'une fois projetés de nouveau",
            );
            let (rang, aligne) = (rang as usize, aligne as usize);
            for y in 0..hauteur as usize {
                let depart = y * rang_source;
                vue.slice(y * aligne..y * aligne + rang)
                    .copy_from_slice(&source[depart..depart + rang]);
            }
        }
        copies.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: tranche.buffer(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: tranche.offset(),
                    bytes_per_row: Some(aligne),
                    rows_per_image: Some(hauteur),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: cible,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: origine.0,
                    y: origine.1,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: largeur,
                height: hauteur,
                depth_or_array_layers: 1,
            },
        );
    }

    /// **Confie à la carte tout ce qui a été préparé**, d'un bloc — à faire avant de soumettre
    /// ce qui s'en sert. Les tampons reviennent d'eux-mêmes quand la carte les a lus.
    pub fn soumettre(&mut self) {
        let Some(copies) = self.copies.take() else {
            return;
        };
        self.tapis.finish_and_recall_on_submit(&copies);
        self.file.submit(Some(copies.finish()));
    }
}

#[cfg(test)]
mod tests;
