//! **La photo que le magasin tient**, telle que la carte graphique la pose : son fichier, le
//! niveau voulu, sa pose — et le niveau qui la remplace tant que le voulu n'est pas tenu.

use super::cran;
use crate::present::scene_gpu::Pose;
use glucose_core::types::Viewport;

/// **Ce que la carte doit savoir d'une photo que le magasin tient** : son fichier, le niveau
/// voulu et sa pose — et, si ce niveau n'est pas tenu, celui qui le remplace en attendant.
pub(super) struct PhotoTenue {
    pub src: String,
    pub facteur: u32,
    pub pose: Pose,
    pub repli: Option<(u32, Pose)>,
}

/// **Où la carte posera cette photo, et à quel niveau** — ou rien si le magasin ne tient
/// encore rien d'elle : elle est alors en chemin, et se dessine comme telle.
pub(super) fn pose_tenue(
    magasin: &mut crate::renderer::magasin::Magasin,
    img: &glucose_core::types::BoardImage,
    (vp, cran, plafond): (&Viewport, u32, Option<u32>),
) -> Option<PhotoTenue> {
    let src = img.src.as_deref()?;
    if !magasin.reclamer(src, img.width) {
        return None;
    }
    let entree = magasin.cache.get(src)?;
    // Le modele place une photo par son CENTRE : le coin s'en deduit, et c'est le piege
    // que `ce_que_porte` avait deja paye une fois.
    let (x, y) =
        crate::canvas::world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, vp);
    let boite = (x, y, img.width * vp.scale, img.height * vp.scale);
    // **NIVEAU-GPU-1 : la carte reçoit le niveau qui couvre encore la taille posée**, et
    // non la texture native. La règle est celle de la voie processeur (MIP-1), lue au même
    // endroit : sur la taille à laquelle la SOURCE entière se pose, qu'un recadrage rend
    // plus grande que la boîte. Une épingle de 27 Mo posée en vignette partait entière sur
    // le bus — treize millisecondes de processeur pour un envoi, d'où les photos qui
    // arrivaient en vagues —, et le filtre lisait un texel sur dix : du crénelage.
    // ETAGES-3 : le cran commun, quand la carte ne tient pas l'écran entier.
    let largeur_source = cran::largeur_source(img, vp) / 2f64.powi(cran as i32);
    // **ETAGES-1 : la carte demande toujours le niveau VOULU.** S'il n'est pas tenu — offert
    // au système, il revient —, la carte pose ce qu'elle détient déjà pour cette photo, net
    // s'il était resté dans son cache. Et seulement si elle ne détient rien, le meilleur
    // niveau tenu, sous une identité à lui : poser le repli sous l'identité de la photo
    // remplacerait une texture nette par une floue, le temps d'une reprise.
    let pyramide = &entree.pyramide;
    // **PLAFOND-1 : jamais une texture plus grande que la carte n'accepte** — elle la
    // refuserait, et la photo partirait en morceaux. Le voulu se réduit jusqu'à tenir ; un
    // repli trop grand ne se pose pas, le voulu attend.
    let voulu = pyramide.tenir_dans(pyramide.facteur_pour(largeur_source as f32), plafond);
    let (tenu, _) = pyramide.meilleur_pour(largeur_source as f32)?;
    let tenu = pyramide.repli_qui_tient(tenu, voulu, plafond);
    let pose = |facteur: u32| Pose {
        x: x as f32,
        y: y as f32,
        largeur: boite.2 as f32,
        hauteur: boite.3 as f32,
        opacite: 1.0,
        angle: img.rotation as f32,
        // Le recadrage du modele, lu et jamais calcule : la carte montre la fenetre de la
        // photo que le document dit (RECADRAGE-1).
        fenetre: Pose::fenetre_de(img.crop),
        // Ce que le filtre a le droit de lire se decide sur ce niveau-la (BORDURES-4).
        bornes: Pose::bornes_de(
            img.crop,
            pyramide.dimensions_natives(),
            (facteur, pyramide.dimensions(facteur)),
        ),
    };
    Some(PhotoTenue {
        src: src.to_string(),
        facteur: voulu,
        pose: pose(voulu),
        repli: (tenu != voulu).then(|| (tenu, pose(tenu))),
    })
}
