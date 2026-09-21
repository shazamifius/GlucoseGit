//! En combien de bandes découper une passe qui couvre l'écran — et pourquoi ce nombre se
//! **constate** au lieu de se choisir.
//!
//! # Pourquoi ce module existe
//!
//! Deux passes se découpent aujourd'hui en bandes horizontales : la composition des tuiles et
//! les lueurs. Elles ne partagent pas leur boucle — l'une écrit des pixels prémultipliés par
//! quatre octets, l'autre passe par `PixmapMut` — mais elles posent exactement la même
//! question, et une réponse écrite deux fois finit toujours par diverger.

/// Deux bornes, et la seconde est la seule qui dise quelque chose.
///
/// * **Ce que la machine annonce** : au-delà, les bandes se disputeraient les mêmes cœurs.
/// * **Le nombre de LIGNES que le travail touche** : une bande doit contenir au moins une
///   ligne à peindre, sinon le fil se lance pour rien — une vingtaine de microsecondes.
///
/// # Pourquoi ce n'est pas le nombre d'OBJETS, et c'est une erreur que j'ai faite
///
/// La première version bornait par le nombre d'objets à poser, en se disant qu'une bande sans
/// objet ne fait rien. C'est vrai, et hors sujet : **un seul objet peut couvrir tout l'écran**.
///
/// La chronique l'a montré sans appel. Après avoir découpé les lueurs, `glisser un noeud`
/// tombait de 18,08 à 2,44 ms — sept fois moins — mais `editer du texte` restait à 9,74. La
/// différence entre les deux gestes : on édite **une** carte à la fois. Un objet, donc un fil,
/// donc aucun partage, pour une lueur qui couvrait la moitié de l'écran.
///
/// Le travail ne se compte pas en objets. Il se compte en lignes.
///
/// Le nombre de fils de la machine ne change pas d'une image à l'autre : on le lit une fois.
/// Ce qui change — un jeu qui occupe les cœurs — se verra dans le **débit**, et c'est un autre
/// mécanisme que celui-ci.
pub(super) fn bandes_utiles(lignes_de_travail: u32) -> usize {
    static FILS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let fils = *FILS.get_or_init(|| {
        std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
    });
    fils.min(lignes_de_travail as usize).max(1)
}

#[cfg(test)]
mod tests {
    use super::bandes_utiles;

    #[test]
    fn test_le_nombre_de_bandes_se_compte_en_lignes_et_pas_en_objets() {
        assert_eq!(bandes_utiles(0), 1, "rien a peindre : aucun fil a lancer");
        assert_eq!(bandes_utiles(1), 1, "une seule ligne ne se partage pas");
        assert_eq!(bandes_utiles(3), 3, "jamais plus de bandes que de lignes");
        // Le cas qui a coute une session : UN objet qui couvre l'ecran merite tous les fils.
        assert!(
            bandes_utiles(800) > 1,
            "huit cents lignes de travail doivent se partager, fussent-elles d'un seul objet"
        );
    }
}
