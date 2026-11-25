//! **L'ombre des nœuds sous la main** (Q10, fiche 155) : l'icône d'un glisser qui part de
//! Glucose. `wl_data_device.start_drag` prend une surface sans rôle, qui devient l'icône et suit
//! le pointeur ; son coin haut gauche est d'abord au point chaud, puis un décalage le place — le
//! point de l'ombre où la main tient les nœuds reste sous elle.
//!
//! # Prudence
//!
//! La connexion est celle de la fenêtre : une faute de protocole fermerait Glucose. Chaque
//! requête suit la version de sa surface, écrite à côté : `attach` avec un décalage non nul est
//! une faute dès la version 5 (`invalid_offset`), qui a `offset` à sa place ; `set_buffer_scale`
//! naît en version 3, `damage_buffer` en 4 ; et la taille du tampon est un multiple de son
//! échelle (`invalid_size`).

use super::etat::Etat;
use crate::plateforme::Ombre;
use std::io::Write;
use std::os::fd::{AsFd, FromRawFd, OwnedFd};
use std::sync::atomic::{AtomicBool, Ordering};
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_shm::{Format, WlShm};
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Proxy, QueueHandle};

/// De quoi faire une icône : le compositeur (la surface) et la mémoire partagée (ses pixels).
pub(super) struct Fabrique {
    pub compositeur: WlCompositor,
    pub memoire: WlShm,
}

/// **Une icône de glisser** : sa surface, son tampon, et le fichier en mémoire qui porte ses
/// pixels — gardés jusqu'à la fin du glisser.
pub(super) struct Icone {
    surface: WlSurface,
    tampon: WlBuffer,
    reserve: WlShmPool,
    _pixels: std::fs::File,
    retiree: AtomicBool,
}

impl Icone {
    /// **Une surface neuve et ses pixels** — sans rôle encore : `start_drag` le lui donne.
    pub(super) fn nouvelle(
        fabrique: &Fabrique,
        qh: &QueueHandle<Etat>,
        ombre: &Ombre,
    ) -> Result<Self, String> {
        let (l, h, d) = (ombre.largeur, ombre.hauteur, ombre.densite);
        if l == 0 || h == 0 || d == 0 || l % d != 0 || h % d != 0 {
            return Err(format!("une ombre de {l} × {h} à la densité {d}"));
        }
        let octets = en_argb(&ombre.rgba);
        let taille = i32::try_from(octets.len()).map_err(|_| "une ombre trop grande")?;
        if octets.len() != l as usize * h as usize * 4 {
            return Err("une ombre qui n'a pas ses pixels".into());
        }
        let mut pixels = std::fs::File::from(memoire_anonyme()?);
        pixels.write_all(&octets).map_err(|e| e.to_string())?;
        let reserve = fabrique.memoire.create_pool(pixels.as_fd(), taille, qh, ());
        let (l, h) = (l as i32, h as i32);
        let tampon = reserve.create_buffer(0, l, h, l * 4, Format::Argb8888, qh, ());
        let surface = fabrique.compositeur.create_surface(qh, ());
        Ok(Self {
            surface,
            tampon,
            reserve,
            _pixels: pixels,
            retiree: AtomicBool::new(false),
        })
    }

    pub(super) fn surface(&self) -> &WlSurface {
        &self.surface
    }

    /// **Montre l'ombre**, une fois la surface devenue l'icône : le point sous la main y reste.
    pub(super) fn montrer(&self, ombre: &Ombre) {
        let s = &self.surface;
        let d = ombre.densite as i32;
        let (x, y) = (-ombre.point.0 / d, -ombre.point.1 / d);
        if d > 1 && s.version() >= 3 {
            s.set_buffer_scale(d);
        }
        if s.version() >= 5 {
            s.attach(Some(&self.tampon), 0, 0);
            s.offset(x, y);
        } else {
            s.attach(Some(&self.tampon), x, y);
        }
        if s.version() >= 4 {
            s.damage_buffer(0, 0, i32::MAX, i32::MAX);
        } else {
            s.damage(0, 0, i32::MAX, i32::MAX);
        }
        s.commit();
    }

    /// **Le glisser est fini** : la surface, le tampon et la réserve partent — une seule fois.
    pub(super) fn retirer(&self) {
        if !self.retiree.swap(true, Ordering::Relaxed) {
            self.surface.destroy();
            self.tampon.destroy();
            self.reserve.destroy();
        }
    }
}

/// Un fichier en mémoire, sans nom sur le disque.
fn memoire_anonyme() -> Result<OwnedFd, String> {
    // SAFETY : un nom terminé par un zéro, et des drapeaux valides ; le descripteur rendu, s'il
    // est valide, n'appartient qu'à nous.
    let fd = unsafe { libc::memfd_create(c"glucose-ombre".as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    // SAFETY : `fd` vient de naître, valide, et personne d'autre ne le tient.
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

/// Rouge, vert, bleu, alpha prémultipliés → `ARGB8888` de Wayland, petit-boutiste : bleu, vert,
/// rouge, alpha — prémultipliés aussi, comme Wayland les veut.
fn en_argb(rgba: &[u8]) -> Vec<u8> {
    let (pixels, _) = rgba.as_chunks::<4>();
    pixels
        .iter()
        .flat_map(|&[r, g, b, a]| [b, g, r, a])
        .collect()
}

wayland_client::delegate_noop!(Etat: ignore WlCompositor);
wayland_client::delegate_noop!(Etat: ignore WlShm);
wayland_client::delegate_noop!(Etat: ignore WlShmPool);
wayland_client::delegate_noop!(Etat: ignore WlBuffer);
wayland_client::delegate_noop!(Etat: ignore WlSurface);

#[cfg(test)]
mod tests {
    use super::en_argb;

    /// Wayland lit `ARGB8888` en petit-boutiste : bleu d'abord, l'alpha à la fin.
    #[test]
    fn ombre_1_les_pixels_partent_bleu_vert_rouge_alpha() {
        assert_eq!(en_argb(&[1, 2, 3, 4, 5, 6, 7, 8]), [3, 2, 1, 4, 7, 6, 5, 8]);
    }
}
