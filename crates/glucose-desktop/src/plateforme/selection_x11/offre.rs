//! **Ce que la sélection offre sous X11** : la table commune à X11 et Wayland
//! ([`super::super::selection_unix`]), dite en atomes — X11 nomme ses cibles par des atomes, que
//! la connexion a demandés une fois sous les noms mêmes de la table ([`super::Atomes`]).

use super::super::selection_unix::{
    FICHIERS_GNOME, LISTE_D_URI, LOT, PNG, TEXTE, TEXTE_UTF8, UTF8_STRING,
};
use super::Atomes;
use crate::interactions::clipboard::{Formes, Pour};
use x11rb::protocol::xproto::Atom;

pub(super) use super::super::selection_unix::Rendu;

/// L'offre de X11 : ses cibles sont des atomes.
pub(super) type Offre = super::super::selection_unix::Offre<Atom>;

/// **Les formes qui partent, dites en atomes.**
pub(super) fn nouvelle(formes: Formes, a: &Atomes) -> Offre {
    Offre::nouvelle(formes, Pour::PressePapiers, |nom| atome(a, nom))
}

/// L'atome d'un nom de la table. Les épreuves de X11 demandent chaque cible par un programme qui
/// n'est pas le nôtre (`xclip`) : un nom sans atome y tomberait.
fn atome(a: &Atomes, nom: &str) -> Atom {
    match nom {
        LOT => a.LOT,
        UTF8_STRING => a.UTF8_STRING,
        TEXTE_UTF8 => a.TEXTE_UTF8,
        TEXTE => a.TEXTE,
        PNG => a.PNG,
        LISTE_D_URI => a.LISTE_D_URI,
        FICHIERS_GNOME => a.FICHIERS_GNOME,
        _ => x11rb::NONE,
    }
}
