//! En combien de bandes découper une passe qui couvre l'écran — et pourquoi ce nombre se
//! **constate** au lieu de se choisir.
//!
//! # Pourquoi ce module existe
//!
//! Deux passes se découpent aujourd'hui en bandes horizontales : la composition des tuiles et
//! les lueurs. Elles ne partagent pas leur boucle — l'une écrit des pixels prémultipliés par
//! quatre octets, l'autre passe par `PixmapMut` — mais elles posent exactement la même
//! question, et une réponse écrite deux fois finit toujours par diverger.

/// Les trois bornes du découpage, et chacune dit pourquoi elle existe.
///
/// * **Ce que la machine annonce** : au-delà, les bandes se disputeraient les mêmes cœurs.
/// * **Le nombre d'objets à poser** : une bande qui n'a rien à faire ne fait rien, et un fil
///   lancé pour rien coûte une vingtaine de microsecondes.
/// * **La hauteur en pixels** : une bande contient au moins une ligne.
///
/// Le nombre de fils de la machine ne change pas d'une image à l'autre : on le lit une fois.
/// Ce qui change — un jeu qui occupe les cœurs — se verra dans le **débit**, et c'est un autre
/// mécanisme que celui-ci.
pub(super) fn bandes_utiles(objets: usize, hauteur: u32) -> usize {
    static FILS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let fils = *FILS.get_or_init(|| {
        std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
    });
    fils.min(objets).min(hauteur as usize).max(1)
}

#[cfg(test)]
mod tests {
    use super::bandes_utiles;

    #[test]
    fn test_le_nombre_de_bandes_se_constate_et_ne_depasse_rien() {
        assert_eq!(
            bandes_utiles(0, 1080),
            1,
            "aucun objet : aucun fil a lancer"
        );
        assert_eq!(bandes_utiles(1, 1080), 1, "un seul objet ne se partage pas");
        assert_eq!(
            bandes_utiles(400, 3),
            3,
            "jamais plus de bandes que de lignes"
        );
        assert!(bandes_utiles(400, 1080) >= 1);
    }
}
