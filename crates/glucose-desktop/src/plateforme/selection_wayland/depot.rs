//! **Ce qu'un dépôt Wayland apporte**, une fois lâché sur la fenêtre — la même [`Moisson`] que le
//! pont de Windows ([`super::super::depot_windows`]), dans le même ordre, du plus sûr au plus
//! pauvre :
//!
//! 1. **un lot de Glucose** (`application/x-glucose-lot`) — un bloc glissé d'un autre Glucose ;
//! 2. **des fichiers** (`text/uri-list`) — Nautilus, Dolphin, un bureau ;
//! 3. **l'image elle-même** (`image/png`, puis tout `image/*`) — ce qu'un navigateur livre d'une
//!    image qu'on glisse hors de sa page ;
//! 4. **l'adresse seule** — une page qui ne livre rien d'autre (un lien, Pinterest) : elle part se
//!    faire rapatrier ([`super::super::rapatrier`]), et se pose en lien si rien ne vient.
//!
//! Tout ici est pur : `lire` est le seul accès au monde, et les épreuves le simulent.

use super::super::moisson::{adresses_dans, Moisson, Recu};
use super::super::selection_unix::{lire_la_liste, LISTE_D_URI, LOT, PNG, TEXTES};

/// Les types d'image que le décodeur sait lire, dans l'ordre où on les préfère : le PNG est sans
/// perte, et c'est ce que les navigateurs donnent d'abord.
const IMAGES: [&str; 6] = [
    PNG,
    "image/webp",
    "image/jpeg",
    "image/gif",
    "image/bmp",
    "image/x-bmp",
];

/// L'adresse d'un lien glissé de Firefox : l'URL, puis son titre, en UTF-16.
const LIEN_DE_FIREFOX: &str = "text/x-moz-url";

/// **Le type qu'on accepte à l'entrée d'un glisser** : il dit à l'application d'où l'on glisse que
/// Glucose prendra quelque chose — son curseur le montre. Rien, si rien n'est posable.
pub(super) fn type_accepte(offerts: &[String]) -> Option<&String> {
    let preferes = [LOT, LISTE_D_URI]
        .into_iter()
        .chain(IMAGES)
        .chain([LIEN_DE_FIREFOX])
        .chain(TEXTES);
    preferes
        .into_iter()
        .find_map(|t| offerts.iter().find(|o| o.as_str() == t))
}

/// **La moisson d'un dépôt**, et les adresses à rapatrier quand elle ne porte rien d'autre.
///
/// `lire` rend les octets d'un type offert — ou rien, s'il n'est pas venu.
pub(super) fn recolter(
    offerts: &[String],
    mut lire: impl FnMut(&str) -> Option<Vec<u8>>,
) -> (Moisson, Vec<String>) {
    // Un glisser parti de ce Glucose ne se dépose pas ici, même si un compositeur le lâche alors
    // qu'on l'a refusé à l'entrée : les nœuds n'ont pas bougé, et les recoller les doublerait.
    if super::vient_d_ici(offerts) {
        return (Moisson::default(), Vec::new());
    }
    let offert = |t: &str| offerts.iter().any(|o| o == t);
    let mut moisson = Moisson::default();
    if offert(LOT) {
        moisson.lot = lire(LOT);
        if moisson.lot.is_some() {
            return (moisson, Vec::new());
        }
    }
    let mut adresses = Vec::new();
    if offert(LISTE_D_URI) {
        if let Some(liste) = lire(LISTE_D_URI) {
            let (chemins, web) = lire_la_liste(&String::from_utf8_lossy(&liste));
            moisson.chemins = chemins.into_iter().filter(|c| c.exists()).collect();
            adresses = web;
        }
    }
    if moisson.chemins.is_empty() {
        let image = IMAGES.into_iter().find(|t| offert(t));
        if let Some((t, octets)) = image.and_then(|t| Some((t, lire(t)?))) {
            moisson.recus.extend(Recu::nouveau(&nom_de(t), octets));
        }
    }
    if moisson.chemins.is_empty() && moisson.recus.is_empty() && adresses.is_empty() {
        let texte = [LIEN_DE_FIREFOX]
            .into_iter()
            .chain(TEXTES)
            .find(|t| offert(t));
        if let Some(octets) = texte.and_then(&mut lire) {
            adresses = adresses_dans(&octets);
        }
    }
    let a_rapatrier = if moisson.chemins.is_empty() && moisson.recus.is_empty() {
        adresses
    } else {
        Vec::new()
    };
    (moisson, a_rapatrier)
}

/// Le nom d'une image livrée sans nom : `depose` et l'extension de son type — `drop` lit de toute
/// façon l'en-tête, jamais l'extension.
fn nom_de(type_image: &str) -> String {
    let extension = type_image.rsplit(['/', '-']).next().unwrap_or("png");
    format!("depose.{extension}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Une application qui offre ces types, et livre ces octets.
    fn offre(types: &[(&str, &[u8])]) -> (Vec<String>, HashMap<String, Vec<u8>>) {
        let offerts = types.iter().map(|(t, _)| t.to_string()).collect();
        let octets = types
            .iter()
            .map(|(t, o)| (t.to_string(), o.to_vec()))
            .collect();
        (offerts, octets)
    }

    fn recolter_de(types: &[(&str, &[u8])]) -> (Moisson, Vec<String>, Vec<String>) {
        let (offerts, octets) = offre(types);
        let mut lus = Vec::new();
        let (moisson, adresses) = recolter(&offerts, |t| {
            lus.push(t.to_string());
            octets.get(t).cloned()
        });
        (moisson, adresses, lus)
    }

    /// **Un lot d'un autre Glucose passe avant tout**, et rien d'autre n'est lu.
    #[test]
    fn test_un_lot_passe_avant_tout() {
        let (m, a, lus) = recolter_de(&[(LISTE_D_URI, b"file:///tmp\r\n"), (LOT, b"lot")]);
        assert_eq!(m.lot.as_deref(), Some(b"lot".as_slice()));
        assert!(m.chemins.is_empty() && a.is_empty());
        assert_eq!(lus, vec![LOT.to_string()]);
    }

    /// **Les fichiers de Nautilus se posent**, seulement ceux qui existent ; une image offerte
    /// à côté ne se lit pas en double.
    #[test]
    fn test_des_fichiers_se_posent_sans_leur_image() {
        let ici = std::env::temp_dir();
        let liste = format!(
            "{}\r\nfile:///n/existe/pas\r\n",
            super::super::super::selection_unix::uri_de(&ici)
        );
        let (m, a, lus) = recolter_de(&[(LISTE_D_URI, liste.as_bytes()), (PNG, b"png")]);
        assert_eq!(m.chemins, vec![ici]);
        assert!(m.recus.is_empty() && a.is_empty());
        assert!(!lus.contains(&PNG.to_string()), "{lus:?}");
    }

    /// **Un glisser parti de ce Glucose ne se moissonne pas**, même lâché sur sa fenêtre : rien
    /// n'est lu, rien ne se pose — les nœuds glissés y sont encore.
    #[test]
    fn test_un_glisser_parti_d_ici_ne_se_moissonne_pas() {
        let marque = format!("{};n={}.1", super::super::MARQUE, std::process::id());
        let (m, a, lus) = recolter_de(&[(LOT, b"lot"), (marque.as_str(), b"")]);
        assert!(m.est_vide() && a.is_empty());
        assert!(lus.is_empty(), "{lus:?}");
    }

    /// **Une image glissée d'une page arrive en octets**, et son adresse ne part pas se faire
    /// rapatrier : l'image est déjà là.
    #[test]
    fn test_une_image_d_une_page_arrive_en_octets() {
        let (m, a, _) = recolter_de(&[
            (LISTE_D_URI, b"https://ex.org/i.png\r\n"),
            ("image/jpeg", b"jpeg"),
        ]);
        assert_eq!(m.recus.len(), 1);
        assert_eq!(m.recus[0].nom, "depose.jpeg");
        assert!(a.is_empty(), "l'image est là : rien à rapatrier");
    }

    /// **Une adresse seule part se faire rapatrier** — un lien de Firefox en UTF-16 aussi.
    #[test]
    fn test_une_adresse_seule_se_rapatrie() {
        let (m, a, _) = recolter_de(&[(LISTE_D_URI, b"https://ex.org/page\r\n")]);
        assert!(m.est_vide());
        assert_eq!(a, vec!["https://ex.org/page".to_string()]);

        let large: Vec<u8> = "https://ex.org/x\ntitre"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let (_, a, _) = recolter_de(&[(LIEN_DE_FIREFOX, &large)]);
        assert_eq!(a, vec!["https://ex.org/x".to_string()]);
    }

    /// **Le type accepté à l'entrée suit le même ordre** que la récolte, et rien n'est accepté
    /// de ce qu'on ne sait pas poser.
    #[test]
    fn test_le_type_accepte_suit_l_ordre_de_la_recolte() {
        let types = |t: &[&str]| t.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let pris = |t: &[&str]| type_accepte(&types(t)).cloned();
        assert_eq!(
            pris(&["text/html", PNG, LISTE_D_URI]),
            Some(LISTE_D_URI.into())
        );
        assert_eq!(
            pris(&["text/html", "image/webp"]),
            Some("image/webp".into())
        );
        assert_eq!(pris(&["text/html", "application/x-truc"]), None);
    }
}
