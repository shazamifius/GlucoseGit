//! **Les poses que le tampon garde d'une image à l'autre** (LOT-1, fiche 87).
//!
//! La loi est dans [`crate::renderer::voies`] : ici, ce que la carte en tient. **Un lot n'existe
//! que complet** : chaque photo proposée est là, à jour, et rien n'attend ; sinon la prochaine image
//! doit de toute façon tout revoir. Le tampon garde ces poses en tête, et la copie de leurs octets
//! ne sert que le jour où il doit grandir.

use super::{Retenues, SceneGpu, Suite};
use crate::renderer::voies::{LotEnPlace, Similitude};

/// Le lot que le tampon garde.
pub(super) struct LotTenu {
    numero: u64,
    /// Les dessins de ses poses.
    suites: Vec<Suite>,
    /// Ses poses, pour un tampon neuf.
    octets: Vec<u8>,
}

impl SceneGpu {
    /// **Le lot que la carte tient** : ce que le rendu demande avant de reprendre ses poses.
    pub fn lot_en_place(&self) -> Option<u64> {
        self.lot.as_ref().map(|lot| lot.numero)
    }

    /// Retient le lot si toutes ses photos sont là et à jour, et si rien n'attend.
    pub(super) fn retenir_le_lot(
        &mut self,
        propose: Option<(u64, usize)>,
        a_jour: usize,
        retenues: &Retenues,
        octets: &[u8],
    ) {
        self.lot = propose
            .filter(|(_, nombre)| a_jour == *nombre && !self.inachevee)
            .map(|(numero, _)| LotTenu {
                numero,
                suites: retenues.suites.clone(),
                octets: octets.to_vec(),
            });
    }

    /// **Ce que le lot en place apporte à cette image** : ses poses et ses dessins, sans les
    /// avoir revus.
    pub(super) fn retenues_du_lot(&self, en_place: Option<LotEnPlace>) -> Retenues {
        match (en_place, &self.lot) {
            (Some(lot), Some(tenu)) => Retenues {
                du_lot: lot.nombre,
                suites: tenu.suites.clone(),
                ..Retenues::default()
            },
            _ => Retenues::default(),
        }
    }

    /// **Le tampon porte ce que l'image demande** : il grandit par doublements, donc il cesse de
    /// grandir en quelques images, comme le cache de tuiles. Le lot gardé y retourne.
    pub(super) fn assurer_la_capacite(
        &mut self,
        peripherique: &wgpu::Device,
        file: &wgpu::Queue,
        combien: usize,
        garder_le_lot: bool,
    ) {
        if combien <= self.capacite {
            return;
        }
        self.capacite = combien.next_power_of_two();
        self.poses = Self::tampon_des_poses(peripherique, self.capacite);
        let layout = self.pipeline.get_bind_group_layout(0);
        self.commun = Self::liaison_commune(
            peripherique,
            &layout,
            &self.poses,
            &self.ecran,
            &self.echantillonneur,
        );
        if let Some(lot) = self.lot.as_ref().filter(|_| garder_le_lot) {
            file.write_buffer(&self.poses, 0, &lot.octets);
        } else {
            self.lot = None;
        }
    }

    /// **L'uniforme de l'écran** : sa taille, et ce qui mène le lot à cette vue.
    pub(super) fn ecrire_l_ecran(
        &self,
        file: &wgpu::Queue,
        ecran: (f32, f32),
        en_place: Option<LotEnPlace>,
    ) {
        let (nombre, s) = en_place.map_or((0, Similitude::IDENTITE), |l| (l.nombre, l.similitude));
        let mut cadre = Vec::with_capacity(32);
        for v in [ecran.0, ecran.1, s.rapport as f32] {
            cadre.extend_from_slice(&v.to_le_bytes());
        }
        cadre.extend_from_slice(&(nombre as u32).to_le_bytes());
        for v in [s.tx as f32, s.ty as f32, 0.0, 0.0] {
            cadre.extend_from_slice(&v.to_le_bytes());
        }
        file.write_buffer(&self.ecran, 0, &cadre);
    }
}
