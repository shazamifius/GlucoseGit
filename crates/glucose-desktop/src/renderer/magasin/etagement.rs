//! **ETAGES-1 — tenu ce que l'écran montre, offert tout le reste.**
//!
//! Extrait de [`super`], à six cent quinze lignes quand PHOTO-1 y a ajouté le numéro des images :
//! ici ce que chaque niveau devient, là-bas ce que le magasin tient.

use super::super::atelier::{Atelier, Deplacement};
use super::super::photo::{Etat, Transit};
use super::{Entree, Magasin};

impl Magasin {
    /// Pour chaque image à revoir, chaque niveau : voulu à cette image ou dans la vue
    /// d'ensemble, il doit être tenu — offert, on le reprend ; perdu, on redécode. Ni l'un ni
    /// l'autre, et tenu, on l'offre. La borne de ce qui reste tenu n'est donc pas un nombre :
    /// c'est ce que l'écran demande, comme pour les tuiles et les vignettes.
    ///
    /// Une image voulue à cette image se revoit à la suivante : c'est là qu'on saura si elle
    /// a quitté l'écran, et qu'elle s'offrira.
    pub(super) fn etager(&mut self) {
        let Self {
            cache,
            atelier,
            a_revoir,
            ..
        } = self;
        let mut encore = Vec::with_capacity(a_revoir.len());
        let mut deplacements = Vec::new();
        for numero in std::mem::take(a_revoir) {
            let Some((src, entree)) = cache.chemin_et_entree(numero) else {
                continue;
            };
            // Une image remise à revoir deux fois (redécodée entre-temps) ne s'examine qu'une : la
            // seconde lirait des voulus déjà pris, et offrirait ce que l'écran montre.
            if !std::mem::take(&mut entree.a_revoir) {
                continue;
            }
            let voulus = entree.pyramide.prendre_les_voulus();
            // **PHOTO-1 : rien n'a changé depuis un examen qui n'a rien trouvé à faire.** Le même
            // examen rendrait le même résultat, et il coûtait 10 % du fil qui dessine sur huit
            // mille images à l'écran.
            if entree.reglee != Some((voulus, entree.queue)) {
                let regle = examiner(src, entree, voulus, (atelier, &mut deplacements));
                entree.reglee = regle.then_some((voulus, entree.queue));
            }
            if voulus != 0 {
                encore.push(numero);
            }
        }
        atelier.deplacer_tout(deplacements);
        for numero in &encore {
            if let Some(entree) = cache.par_numero_mut(*numero) {
                entree.a_revoir = true;
            }
        }
        a_revoir.append(&mut encore);
    }
}

/// **Un examen** : pour chaque niveau, ce qu'il doit devenir. Rend vrai si tout est réglé — rien
/// n'est parti, rien n'est en route, rien ne manque : tant que rien ne change, un nouvel examen ne
/// ferait rien de plus.
fn examiner(
    src: &str,
    entree: &mut Entree,
    voulus: u64,
    (atelier, deplacements): (&mut Atelier, &mut Vec<Deplacement<Transit>>),
) -> bool {
    let mut regle = true;
    let mut redecoder = false;
    for rang in 0..entree.pyramide.niveaux_construits() {
        let garder = voulus & (1 << rang) != 0 || rang >= entree.queue;
        let etat = entree.pyramide.etat(rang);
        let geste = match etat {
            Etat::Offert => garder,
            Etat::Tenu => !garder && entree.pyramide.offrable(rang),
            Etat::Perdu => {
                redecoder |= garder;
                false
            }
            Etat::EnChemin => false,
        };
        // Un niveau tenu trop petit pour s'offrir le restera : être offrable ne change pas.
        regle &= !geste && !matches!(etat, Etat::EnChemin | Etat::Perdu);
        if let Some(transit) = geste.then(|| entree.pyramide.sortir(rang)).flatten() {
            deplacements.push(Deplacement {
                src: src.to_string(),
                generation: entree.generation,
                rang,
                charge: transit,
            });
        }
    }
    // Ce qui manque ne se trouve pas dans l'aperçu : un niveau perdu se redécode depuis
    // le fichier entier (ETAGES-4).
    if redecoder {
        atelier.redecoder(src);
    }
    // La vue d'ensemble est connue dès que l'écran a montré l'image une fois.
    if entree.apercu_a_ecrire && entree.queue_vue != 0 {
        entree.apercu_a_ecrire = false;
        if let Some(a) = entree.pyramide.apercu(entree.queue) {
            atelier.ecrire_l_apercu(src, a);
        }
    }
    regle && !entree.apercu_a_ecrire
}
