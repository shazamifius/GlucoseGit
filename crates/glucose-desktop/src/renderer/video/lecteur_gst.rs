//! **Une vidéo jouée par GStreamer** (VIDEO-1, fiche 169) : `playbin` démultiplexe, décode (par
//! la carte graphique quand le système le sait), joue le son et cadence les images ; un `appsink`
//! les rend en RGBA, une à une, à l'heure où elles doivent paraître.
//!
//! Ses octets viennent d'un `appsrc` en accès libre : un fichier, une tranche du document ou de
//! la mémoire, lus là où ils sont — aucune copie sur le disque, et un seul chemin pour les trois.

use super::gst::{c, gst, Gst, MapInfo, Ptr, RappelsSource};
use super::{Fin, Lieu};
use crate::renderer::gif::Toile;
use std::ffi::{c_int, c_uint};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Les constantes de l'en-tête de GStreamer 1.x dont le lecteur se sert.
const ETAT_NUL: c_int = 1;
const ETAT_PAUSE: c_int = 3;
const ETAT_LECTURE: c_int = 4;
const CHANGEMENT_RATE: c_int = 0;
const MESSAGE_ERREUR: c_uint = 1 << 1;
const FORMAT_TEMPS: c_int = 3;
const CHERCHER_EN_VIDANT: c_int = 1;
const LIRE: c_int = 1;
const ECRIRE: c_int = 2;
const FLUX_EN_ACCES_LIBRE: c_int = 2;
/// `GST_VIDEO_ORIENTATION_AUTO` : l'orientation que la vidéo dit d'elle-même.
const ORIENTATION_AUTO: c_int = 8;
/// L'image, le son, le désentrelacement ; ni les sous-titres, ni la balance des couleurs.
const DRAPEAUX_JOUER: c_int = 0x1 | 0x2 | 0x10 | 0x200;
/// L'image fixe n'a que faire du son : la sortie audio ne s'ouvre pas pour elle.
const DRAPEAUX_IMAGE_FIXE: c_int = 0x1 | 0x200;
/// Ce qu'un `need-data` sans taille demande.
const MORCEAU: usize = 64 * 1024;

/// **Le temps que GStreamer a pour préparer la première image** (démultiplexer, ouvrir le
/// décodeur) : au-delà, la vidéo est tenue pour illisible ici.
const PREPARATION: Duration = Duration::from_secs(10);

/// **Une vidéo ouverte**, en pause sur sa première image.
pub(crate) struct LecteurVideo {
    gst: &'static Gst,
    pipeline: Ptr,
    sortie: Ptr,
    bus: Ptr,
}

impl LecteurVideo {
    /// **Ouvre la vidéo** de ce lieu, prête à jouer, son coupé ; `None` si GStreamer manque, ne
    /// sait pas la lire, ou ne l'a pas préparée à temps.
    pub(crate) fn ouvrir(lieu: Lieu, avec_le_son: bool) -> Option<Self> {
        let gst = gst()?;
        let flux = Arc::new(Mutex::new(Flux::ouvrir(lieu)?));
        // SAFETY : la table vient de GStreamer 1.x ; chaque objet créé ici est rendu par `Drop`
        // (le pipeline possède sa sortie) ; les propriétés ont le type C que l'en-tête leur donne.
        unsafe {
            let pipeline = (gst.gst_element_factory_make)(c("playbin").as_ptr(), std::ptr::null());
            let sortie = (gst.gst_element_factory_make)(c("appsink").as_ptr(), std::ptr::null());
            if pipeline.is_null() || sortie.is_null() {
                for objet in [pipeline, sortie].into_iter().filter(|o| !o.is_null()) {
                    (gst.gst_object_unref)(objet);
                }
                return None;
            }
            let caps = (gst.gst_caps_from_string)(
                c("video/x-raw,format=RGBA,pixel-aspect-ratio=1/1").as_ptr(),
            );
            poser(gst, sortie, "caps", Valeur::Objet(caps));
            (gst.gst_mini_object_unref)(caps);
            // Une image d'avance au plus ; en retard, la plus vieille tombe plutôt que le son
            // n'attende. `sync` : chaque image sort à son heure, accordée au son — posé plutôt
            // que supposé, un défaut pouvant changer d'une version de GStreamer à l'autre.
            poser(gst, sortie, "sync", Valeur::Entier(1));
            poser(gst, sortie, "max-buffers", Valeur::Naturel(1));
            poser(gst, sortie, "drop", Valeur::Entier(1));
            poser(gst, sortie, "enable-last-sample", Valeur::Entier(0));
            let drapeaux = if avec_le_son {
                DRAPEAUX_JOUER
            } else {
                DRAPEAUX_IMAGE_FIXE
            };
            poser(gst, pipeline, "flags", Valeur::Entier(drapeaux));
            poser(gst, pipeline, "uri", Valeur::Texte("appsrc://"));
            poser(gst, pipeline, "video-sink", Valeur::Objet(sortie));
            poser(gst, pipeline, "mute", Valeur::Entier(1));
            // Une vidéo de téléphone filmée debout est souvent rangée couchée, avec l'ordre de
            // la tourner : `videoflip` l'applique, comme EXIF-1 pour une photo (fiche 165).
            let tourner = (gst.gst_element_factory_make)(c("videoflip").as_ptr(), std::ptr::null());
            if !tourner.is_null() {
                poser(
                    gst,
                    tourner,
                    "video-direction",
                    Valeur::Entier(ORIENTATION_AUTO),
                );
                poser(gst, pipeline, "video-filter", Valeur::Objet(tourner));
            }
            (gst.g_signal_connect_data)(
                pipeline,
                c("source-setup").as_ptr(),
                source_prete as *const () as Ptr,
                Arc::into_raw(flux) as Ptr,
                lacher_le_flux_du_signal as *const () as Ptr,
                0,
            );
            let lecteur = Self {
                gst,
                pipeline,
                sortie,
                bus: (gst.gst_element_get_bus)(pipeline),
            };
            if (gst.gst_element_set_state)(pipeline, ETAT_PAUSE) == CHANGEMENT_RATE {
                return None;
            }
            let (mut etat, mut attendu) = (0, 0);
            let pret = (gst.gst_element_get_state)(
                pipeline,
                &mut etat,
                &mut attendu,
                PREPARATION.as_nanos() as u64,
            );
            (pret != CHANGEMENT_RATE && etat == ETAT_PAUSE).then_some(lecteur)
        }
    }

    /// **L'image où la vidéo est arrêtée** : la première, juste après l'ouverture.
    pub(crate) fn image_arretee(&self) -> Option<Toile> {
        // SAFETY : la sortie vit autant que le lecteur ; l'échantillon rendu est lâché ici.
        unsafe {
            let echantillon = (self.gst.gst_app_sink_try_pull_preroll)(self.sortie, 0);
            self.toile(echantillon)
        }
    }

    /// **L'image suivante**, attendue au plus `delai` : `Ok(None)` si elle n'est pas encore là,
    /// `Err` à la fin de la vidéo ou à la première faute.
    pub(crate) fn suivante(&self, delai: Duration) -> Result<Option<Toile>, Fin> {
        // SAFETY : comme `image_arretee` ; le bus vit autant que le lecteur.
        unsafe {
            let faute = (self.gst.gst_bus_pop_filtered)(self.bus, MESSAGE_ERREUR);
            if !faute.is_null() {
                (self.gst.gst_mini_object_unref)(faute);
                return Err(Fin::Faute);
            }
            let echantillon =
                (self.gst.gst_app_sink_try_pull_sample)(self.sortie, delai.as_nanos() as u64);
            if echantillon.is_null() {
                return if (self.gst.gst_app_sink_is_eos)(self.sortie) != 0 {
                    Err(Fin::AuBout)
                } else {
                    Ok(None)
                };
            }
            Ok(self.toile(echantillon))
        }
    }

    /// **Revient au début**, pour une vidéo qui joue en boucle (VIDEO-2) ; `false` si le système
    /// refuse.
    pub(crate) fn revenir_au_debut(&self) -> bool {
        // SAFETY : le pipeline vit autant que le lecteur ; la source en accès libre sait chercher.
        unsafe {
            (self.gst.gst_element_seek_simple)(self.pipeline, FORMAT_TEMPS, CHERCHER_EN_VIDANT, 0)
                != 0
        }
    }

    /// **Joue** ou **met en pause**.
    pub(crate) fn jouer(&self, oui: bool) {
        let etat = if oui { ETAT_LECTURE } else { ETAT_PAUSE };
        // SAFETY : le pipeline vit autant que le lecteur.
        unsafe { (self.gst.gst_element_set_state)(self.pipeline, etat) };
    }

    /// **Coupe ou rend le son.**
    pub(crate) fn couper_le_son(&self, oui: bool) {
        // SAFETY : `mute` est un booléen de `playbin`.
        unsafe {
            poser(
                self.gst,
                self.pipeline,
                "mute",
                Valeur::Entier(c_int::from(oui)),
            );
        }
    }

    /// Les pixels d'un échantillon, à sa taille, rangée par rangée ; l'échantillon est lâché.
    unsafe fn toile(&self, echantillon: Ptr) -> Option<Toile> {
        if echantillon.is_null() {
            return None;
        }
        let g = self.gst;
        let toile = (|| {
            let caps = (g.gst_sample_get_caps)(echantillon);
            let tampon = (g.gst_sample_get_buffer)(echantillon);
            if caps.is_null() || tampon.is_null() {
                return None;
            }
            let structure = (g.gst_caps_get_structure)(caps, 0);
            let (mut l, mut h) = (0, 0);
            if (g.gst_structure_get_int)(structure, c("width").as_ptr(), &mut l) == 0
                || (g.gst_structure_get_int)(structure, c("height").as_ptr(), &mut h) == 0
            {
                return None;
            }
            let (l, h) = (u32::try_from(l).ok()?, u32::try_from(h).ok()?);
            let mut carte = MapInfo::vide();
            if (g.gst_buffer_map)(tampon, &mut carte, LIRE) == 0 {
                return None;
            }
            let octets = std::slice::from_raw_parts(carte.data, carte.size);
            let rgba = rangees(octets, l as usize, h as usize);
            (g.gst_buffer_unmap)(tampon, &mut carte);
            Some(Toile {
                largeur: l,
                hauteur: h,
                rgba: rgba?,
                delai: Duration::ZERO,
            })
        })();
        (g.gst_mini_object_unref)(echantillon);
        toile
    }
}

/// La valeur d'une propriété GObject, du type C que `g_object_set` attend pour elle.
enum Valeur {
    Entier(c_int),
    Naturel(c_uint),
    Long(i64),
    Objet(Ptr),
    Texte(&'static str),
}

/// **Pose une propriété** d'un objet de GStreamer (`g_object_set`, terminé par un nul).
///
/// # Safety
/// `objet` est un GObject vivant qui a cette propriété, de ce type.
unsafe fn poser(g: &Gst, objet: Ptr, nom: &str, valeur: Valeur) {
    let (nom, fin) = (c(nom), std::ptr::null::<u8>());
    match valeur {
        Valeur::Entier(v) => (g.g_object_set)(objet, nom.as_ptr(), v, fin),
        Valeur::Naturel(v) => (g.g_object_set)(objet, nom.as_ptr(), v, fin),
        Valeur::Long(v) => (g.g_object_set)(objet, nom.as_ptr(), v, fin),
        Valeur::Objet(v) => (g.g_object_set)(objet, nom.as_ptr(), v, fin),
        Valeur::Texte(v) => (g.g_object_set)(objet, nom.as_ptr(), c(v).as_ptr(), fin),
    }
}

/// Les pixels d'une image RGBA de `l` × `h` : sans `GstVideoMeta` annoncée par la sortie, le
/// décodeur rend la disposition par défaut — des rangées de `4 l` octets (multiple de 4), serrées.
fn rangees(octets: &[u8], l: usize, h: usize) -> Option<Vec<u8>> {
    let n = l.checked_mul(4)?.checked_mul(h)?;
    octets.get(..n).map(<[u8]>::to_vec)
}

impl Drop for LecteurVideo {
    fn drop(&mut self) {
        // SAFETY : à l'état nul, plus aucun fil de GStreamer ne touche au flux ; le pipeline
        // emporte sa sortie, sa source et le flux (par la fin de son signal).
        unsafe {
            (self.gst.gst_element_set_state)(self.pipeline, ETAT_NUL);
            (self.gst.gst_object_unref)(self.bus);
            (self.gst.gst_object_unref)(self.pipeline);
        }
    }
}

/// **Les octets de la vidéo**, là où ils sont, et où la lecture en est.
struct Flux {
    lieu: Ouvert,
    longueur: u64,
    position: u64,
}

enum Ouvert {
    Fichier { fichier: std::fs::File, debut: u64 },
    Octets(Arc<[u8]>),
}

impl Flux {
    fn ouvrir(lieu: Lieu) -> Option<Self> {
        let (lieu, longueur) = match lieu {
            Lieu::Fichier {
                chemin,
                debut,
                longueur,
            } => {
                let fichier = std::fs::File::open(chemin).ok()?;
                let longueur = match longueur {
                    Some(n) => n,
                    None => fichier.metadata().ok()?.len(),
                };
                (Ouvert::Fichier { fichier, debut }, longueur)
            }
            Lieu::Octets(octets) => {
                let n = octets.len() as u64;
                (Ouvert::Octets(octets), n)
            }
        };
        Some(Self {
            lieu,
            longueur,
            position: 0,
        })
    }

    /// Lit jusqu'à `tampon.len()` octets à la position courante ; 0 à la fin.
    fn lire(&mut self, tampon: &mut [u8]) -> usize {
        let reste = self.longueur.saturating_sub(self.position);
        let n = tampon
            .len()
            .min(usize::try_from(reste).unwrap_or(usize::MAX));
        let lus = match &self.lieu {
            Ouvert::Fichier { fichier, debut } => {
                use std::os::unix::fs::FileExt;
                fichier
                    .read_at(&mut tampon[..n], debut + self.position)
                    .unwrap_or(0)
            }
            Ouvert::Octets(octets) => {
                let de = usize::try_from(self.position).unwrap_or(usize::MAX);
                tampon[..n].copy_from_slice(&octets[de..de + n]);
                n
            }
        };
        self.position += lus as u64;
        lus
    }
}

/// `source-setup` : la source d'`appsrc://` est née ; elle lit le flux en accès libre.
unsafe extern "C" fn source_prete(_pipeline: Ptr, source: Ptr, donnee: Ptr) {
    let Some(g) = gst() else { return };
    let flux = donnee as *const Mutex<Flux>;
    // Une référence de plus, que la source lâchera elle-même (`lacher_le_flux`).
    Arc::increment_strong_count(flux);
    let longueur = (*flux).lock().map_or(0, |f| f.longueur);
    poser(
        g,
        source,
        "stream-type",
        Valeur::Entier(FLUX_EN_ACCES_LIBRE),
    );
    poser(
        g,
        source,
        "size",
        Valeur::Long(i64::try_from(longueur).unwrap_or(i64::MAX)),
    );
    static RAPPELS: RappelsSource = RappelsSource {
        need_data: donner,
        enough_data: assez,
        seek_data: chercher,
        reserved: [0; 4],
    };
    (g.gst_app_src_set_callbacks)(
        source,
        &RAPPELS,
        flux as Ptr,
        lacher_le_flux as *const () as Ptr,
    );
}

/// La source demande `longueur` octets à la position où elle a cherché.
unsafe extern "C" fn donner(source: Ptr, longueur: c_uint, donnee: Ptr) {
    let Some(g) = gst() else { return };
    let flux = &*(donnee as *const Mutex<Flux>);
    let Ok(mut flux) = flux.lock() else { return };
    let voulu = if longueur == c_uint::MAX {
        MORCEAU
    } else {
        longueur as usize
    };
    let reste = usize::try_from(flux.longueur.saturating_sub(flux.position)).unwrap_or(usize::MAX);
    let n = voulu.min(reste);
    if n == 0 {
        (g.gst_app_src_end_of_stream)(source);
        return;
    }
    let tampon = (g.gst_buffer_new_allocate)(std::ptr::null_mut(), n, std::ptr::null_mut());
    if tampon.is_null() {
        (g.gst_app_src_end_of_stream)(source);
        return;
    }
    let mut carte = MapInfo::vide();
    if (g.gst_buffer_map)(tampon, &mut carte, ECRIRE) == 0 {
        (g.gst_mini_object_unref)(tampon);
        (g.gst_app_src_end_of_stream)(source);
        return;
    }
    let lus = flux.lire(std::slice::from_raw_parts_mut(carte.data, n));
    (g.gst_buffer_unmap)(tampon, &mut carte);
    if lus == 0 {
        (g.gst_mini_object_unref)(tampon);
        (g.gst_app_src_end_of_stream)(source);
        return;
    }
    (g.gst_buffer_set_size)(tampon, lus as isize);
    // La source prend le tampon.
    (g.gst_app_src_push_buffer)(source, tampon);
}

unsafe extern "C" fn assez(_source: Ptr, _donnee: Ptr) {}

/// La source cherche : la lecture suivante part de `position`.
unsafe extern "C" fn chercher(_source: Ptr, position: u64, donnee: Ptr) -> c_int {
    let flux = &*(donnee as *const Mutex<Flux>);
    match flux.lock() {
        Ok(mut f) if position <= f.longueur => {
            f.position = position;
            1
        }
        _ => 0,
    }
}

/// La référence de la source au flux tombe (`GDestroyNotify`).
unsafe extern "C" fn lacher_le_flux(donnee: Ptr) {
    drop(Arc::from_raw(donnee as *const Mutex<Flux>));
}

/// Celle du signal tombe avec lui (`GClosureNotify` : la donnée, puis la fermeture).
unsafe extern "C" fn lacher_le_flux_du_signal(donnee: Ptr, _fermeture: Ptr) {
    lacher_le_flux(donnee);
}

#[cfg(test)]
mod tests;
