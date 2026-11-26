//! **Les vidéos** (VIDEO-1, fiche 169 ; l'étude, fiche 160) : un nœud photo dont les octets sont
//! une vidéo (`is_video`, déjà dans le format). Son image fixe est sa première image — pyramide,
//! aperçu, export la traitent en photo ; elle joue au clic sur ▶, le son coupé jusqu'au clic sur
//! son icône (sa réponse du 25/11, `relais-q-videos`).
//!
//! Les décodeurs sont ceux du système (la voie A de la fiche 160) : aucun poids livré, aucune
//! licence de codec à porter, la carte graphique décode. Linux (GStreamer), Android
//! (`MediaPlayer`, VIDEO-ANDROID-1, fiche 172) et Windows (Media Foundation, VIDEO-WINDOWS-1,
//! fiche 175) ; sur le Mac, en attendant sa porte, une vidéo n'a pas d'image fixe et ne joue pas.

use crate::persist::objets::Objets;

pub(crate) mod commandes;
pub(crate) mod joueur;
pub(crate) mod signature;

#[cfg(any(windows, test))]
mod bgra;
#[cfg(all(test, any(target_os = "linux", windows)))]
mod epreuves;
#[cfg(target_os = "linux")]
mod gst;
#[cfg(target_os = "linux")]
mod lecteur_gst;
#[cfg(target_os = "linux")]
pub(crate) use lecteur_gst::LecteurVideo;
#[cfg(windows)]
mod lecteur_mf;
#[cfg(windows)]
pub(crate) use lecteur_mf::LecteurVideo;
#[cfg(any(target_os = "linux", target_os = "android", windows))]
mod lieu;
#[cfg(any(target_os = "linux", target_os = "android", windows))]
pub(crate) use lieu::Lieu;
#[cfg(windows)]
mod mf;
#[cfg(any(target_os = "android", test))]
mod telephone;
#[cfg(target_os = "android")]
pub(crate) use telephone::LecteurDuTelephone as LecteurVideo;
#[cfg(any(target_os = "android", test))]
mod yuv;

/// **Le temps que le système a pour préparer la première image** (démultiplexer, ouvrir le
/// décodeur) : au-delà, la vidéo est tenue pour illisible ici.
#[cfg(any(target_os = "linux", windows))]
const PREPARATION: std::time::Duration = std::time::Duration::from_secs(10);

/// **Pourquoi une vidéo ne donne plus d'image.**
#[cfg(any(target_os = "linux", target_os = "android", windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fin {
    /// Elle est allée au bout.
    AuBout,
    /// Le système n'a pas su la lire.
    Faute,
}

/// **Annonce à GStreamer les greffons connus à la compilation** (sous Nix), avant qu'aucun fil ne
/// parte : à appeler en tête de `main`. `gst_init` lit `GST_PLUGIN_PATH_1_0` et range ce qu'il y
/// trouve dans son registre en cache : le premier lancement lit les greffons (~1 s), les suivants
/// trouvent le cache (~9 ms). Ajoutés après l'ouverture (`gst_registry_scan_path`), ils n'entraient
/// jamais dans ce cache : chaque lancement les relisait — une seconde de fenêtre figée au premier
/// dépôt d'une vidéo (fiche 169 § 4).
pub fn annoncer_les_greffons() {
    #[cfg(target_os = "linux")]
    if let Some(greffons) = gst::GREFFONS {
        const NOM: &str = "GST_PLUGIN_PATH_1_0";
        std::env::set_var(NOM, chemins_des_greffons(greffons, std::env::var_os(NOM)));
    }
}

/// Les greffons de Glucose d'abord, puis ceux qu'on avait déjà annoncés.
#[cfg(target_os = "linux")]
fn chemins_des_greffons(greffons: &str, deja: Option<std::ffi::OsString>) -> std::ffi::OsString {
    let mut chemins = std::ffi::OsString::from(greffons);
    if let Some(deja) = deja.filter(|d| !d.is_empty()) {
        chemins.push(":");
        chemins.push(deja);
    }
    chemins
}

/// **Ce système a-t-il un lecteur de vidéos ?** Linux (GStreamer), Android (MediaPlayer) et
/// Windows (Media Foundation) ; le Mac, pas encore (fiche 175). Le nom de « Ajouter des images… » et le filtre de
/// son dialogue le suivent (fiche 172 § 6) : ils ne promettent une vidéo que là où elle peut
/// jouer. [`lisible_ici`] le dit au moment de la poser — sans GStreamer, elle devient un
/// lanceur, comme déposée — : l'interroger pour un nom, c'était charger GStreamer sur le fil qui
/// dessine, à l'ouverture d'un menu.
pub(crate) const PREVUES_ICI: bool = cfg!(any(target_os = "linux", target_os = "android", windows));

/// **Ce système sait-il lire une vidéo ?** Sous Linux, si GStreamer est là ; sous Android, si sa
/// porte est branchée ; sous Windows, si Media Foundation l'est et décode le H.264 ; sur le Mac,
/// pas encore (sa porte suit, fiche 175).
pub(crate) fn lisible_ici() -> bool {
    #[cfg(target_os = "linux")]
    {
        gst::gst().is_some()
    }
    #[cfg(target_os = "android")]
    {
        telephone::lisible_ici()
    }
    #[cfg(windows)]
    {
        lecteur_mf::lisible_ici()
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", windows)))]
    {
        false
    }
}

/// **L'image fixe d'une vidéo** : sa première image, en RGBA ; `None` si ce système ne sait pas
/// la lire.
pub(crate) fn image_fixe(
    src: &str,
    objets: Option<&Objets>,
) -> Option<crate::renderer::gif::Toile> {
    #[cfg(any(target_os = "linux", windows))]
    {
        LecteurVideo::ouvrir(Lieu::de(src, objets)?, false)?.image_arretee()
    }
    #[cfg(target_os = "android")]
    {
        telephone::image_fixe(Lieu::de(src, objets)?)
    }
    #[cfg(not(any(target_os = "linux", target_os = "android", windows)))]
    {
        let _ = (src, objets);
        None
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    #[test]
    fn video_1_les_greffons_de_glucose_passent_devant_ceux_deja_annonces() {
        let chemins =
            |deja: Option<&str>| super::chemins_des_greffons("/g/a:/g/b", deja.map(Into::into));
        assert_eq!(chemins(None), "/g/a:/g/b");
        assert_eq!(chemins(Some("")), "/g/a:/g/b");
        assert_eq!(chemins(Some("/autre")), "/g/a:/g/b:/autre");
    }

    /// Les greffons s'annoncent **avant tout fil** : la première chose que fait `main`. Plus tard,
    /// changer l'environnement pendant qu'un autre fil le lit est une course ; jamais, et chaque
    /// lancement relit les greffons (une seconde de fenêtre figée).
    #[test]
    fn video_1_main_annonce_les_greffons_avant_tout() {
        let main = include_str!("../main.rs");
        let corps = main.split("fn main()").nth(1).expect("main");
        let premiere = (corps.lines().skip(1))
            .map(str::trim)
            .find(|l| !l.is_empty() && !l.starts_with("//"));
        assert_eq!(
            premiere,
            Some("glucose_desktop::renderer::annoncer_les_greffons();")
        );
    }
}
