//! Le pincement à deux doigts — ce que le système en dit, et que `winit` ne transmet pas.
//!
//! # Le défaut, et il était invisible depuis le code de Glucose
//!
//! Sous Windows, un pavé tactile de précision livre le pincement, à une application qui ne
//! gère pas les gestes natifs, sous la forme d'un **défilement de molette accompagné de
//! `Ctrl`** : le bit `MK_CONTROL` du message `WM_MOUSEWHEEL`. C'est le mécanisme qui fait
//! que `Ctrl` + molette zoome dans tous les logiciels, et le système s'appuie dessus.
//!
//! Mais `winit` ne lit **pas** ce bit. À chaque message de molette il rafraîchit ses
//! modificateurs depuis l'état **clavier** — où aucune touche n'est enfoncée, puisque le
//! `Ctrl` du pincement est virtuel. Glucose reçoit donc un défilement vertical nu, et NAV-2,
//! qui a raison de tout le reste, en conclut un déplacement de la vue.
//!
//! **Conséquence : pincer déplace la vue verticalement**, avec un saut de zoom sporadique
//! quand le delta du doigt tombe par hasard sur un nombre entier de crans. Ni un défaut de
//! cadence, ni un défaut de latence : le geste n'arrivait pas.
//!
//! # La réparation n'invente rien
//!
//! Le bit existe, il arrive, personne ne le lisait : on le lit. `winit` offre pour cela un
//! crochet qui voit chaque message **avant** de le remettre à la fenêtre — donc avant
//! l'événement qui en découle. L'ordre entre la marque et son défilement est garanti par la
//! boucle elle-même, pas par une supposition.
//!
//! Aucune heuristique n'est ajoutée, aucune constante : c'est l'information que Windows
//! envoie déjà, enfin regardée. Sur les plateformes qui n'ont pas cette convention, le pont
//! ne dit rien et la décision reste celle du clavier.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Le défilement en cours de traitement porte-t-il la marque de zoom du système ?
///
/// Écrit par le crochet, lu par le gestionnaire d'événements, sur le même fil et dans cet
/// ordre. L'atome ne sert donc pas à synchroniser : il sert à ce que l'état soit atteignable
/// depuis une fonction que `winit` appelle sans rien nous passer.
static MARQUE: AtomicBool = AtomicBool::new(false);

/// Combien de défilements ont porté cette marque depuis le lancement.
///
/// C'est la **preuve** que le pont sert : si ce compte reste à zéro pendant qu'on pince, la
/// convention supposée ici n'est pas celle de la machine, et il faut chercher ailleurs plutôt
/// que de croire la réparation faite.
static MARQUES: AtomicU64 = AtomicU64::new(0);

/// Le système a-t-il marqué ce défilement comme une demande de zoom ?
///
/// # La marque se **consomme**, et c'est tout l'objet de cette fonction
///
/// La première version la lisait sans l'effacer. Un message de molette qui n'était pas passé
/// par le crochet héritait donc de la marque du précédent — et Windows a plusieurs boucles
/// internes qui pompent les messages sans passer par celle de `winit` : un redimensionnement,
/// un menu système, un glisser natif.
///
/// La conséquence se voyait, et l'utilisateur l'a décrite exactement : « tu peux aller en haut
/// et revenir en bas et tu viens de dézoomer énormément », sur des gestes qui ne contenaient
/// aucun zoom. Un glissement dont chaque unité vaut un quart d'octave au lieu de seize pixels
/// traverse plusieurs octaves en un geste.
///
/// Consommée, la marque ne peut plus servir qu'au message qui l'a posée.
pub fn zoom_du_systeme() -> bool {
    MARQUE.swap(false, Ordering::Relaxed)
}

/// Combien de messages ont porté la marque — pour la chronique.
///
/// À comparer au nombre de pincements que la navigation a comptés : les deux doivent coïncider.
/// Un écart signifie qu'un message marqué n'a pas donné d'événement, ou l'inverse — donc que le
/// pont et la boucle ne voient pas la même chose.
pub fn marques() -> u64 {
    MARQUES.load(Ordering::Relaxed)
}

/// Note ce qu'un message de défilement disait. Appelé par le crochet de plateforme.
fn noter(marque: bool) {
    MARQUE.store(marque, Ordering::Relaxed);
    if marque {
        MARQUES.fetch_add(1, Ordering::Relaxed);
    }
}

/// Branche le pont sur la boucle d'événements, là où la plateforme en a un.
#[cfg(target_os = "windows")]
pub fn brancher(builder: &mut winit::event_loop::EventLoopBuilder<()>) {
    use winit::platform::windows::EventLoopBuilderExtWindows;
    builder.with_msg_hook(|message| {
        plateforme::lire(message);
        // Faux : on ne fait que regarder passer. Rendre vrai priverait `winit` du message.
        false
    });
}

/// Ailleurs, rien à brancher : le pincement n'emprunte pas la molette.
#[cfg(not(target_os = "windows"))]
pub fn brancher(_builder: &mut winit::event_loop::EventLoopBuilder<()>) {}

#[cfg(target_os = "windows")]
mod plateforme {
    use std::ffi::c_void;

    /// `WM_MOUSEWHEEL` — défilement vertical.
    const MOLETTE_VERTICALE: u32 = 0x020A;

    /// `WM_MOUSEHWHEEL` — défilement horizontal.
    ///
    /// Lu lui aussi, non pour le zoom — il n'en porte jamais la marque — mais pour que le
    /// drapeau ne reste pas sur la valeur d'un message précédent quand on glisse en biais.
    const MOLETTE_HORIZONTALE: u32 = 0x020E;

    /// `MK_CONTROL` : le bit que Windows pose sur un pincement, et sur un vrai `Ctrl` + molette.
    const CONTROLE: usize = 0x0008;

    /// La structure que Win32 remet au crochet.
    ///
    /// Déclarée ici plutôt qu'importée d'une bibliothèque de liaisons, comme le fait déjà
    /// [`crate::memoire`] : elle est figée depuis Windows 3.1, et la recopier coûte huit
    /// lignes contre une vingtaine de crates.
    #[repr(C)]
    struct Message {
        /// Les quatre premiers champs ne sont pas lus : ils **placent** les deux suivants.
        _fenetre: *mut c_void,
        code: u32,
        haut: usize,
        _bas: isize,
        _instant: u32,
        _curseur: [i32; 2],
    }

    // La disposition ne se teste pas : elle se **prouve** à la compilation. Si un champ bouge
    // ou si un padding change, le crate cesse de compiler — plutôt que de lire de travers un
    // message sur deux sans que rien ne le signale.
    const _: () = {
        assert!(core::mem::offset_of!(Message, code) == size_of::<usize>());
        assert!(core::mem::offset_of!(Message, haut) == 2 * size_of::<usize>());
    };

    pub(super) fn lire(brut: *const c_void) {
        // Sûr : `winit` documente ce pointeur comme un `MSG` valide le temps de l'appel, et
        // la lecture se limite à deux entiers dont la position dans la structure est fixée
        // par l'ABI de Win32.
        let message = unsafe { &*(brut.cast::<Message>()) };
        if message.code != MOLETTE_VERTICALE && message.code != MOLETTE_HORIZONTALE {
            return;
        }
        super::noter(message.haut & CONTROLE != 0);
    }
}

#[cfg(test)]
mod tests;
