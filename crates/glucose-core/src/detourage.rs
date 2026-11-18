//! **DETOURER-1 — la forme principale d'une image, seule, en transparence** (fiche 121).
//!
//! Sa parole du 18/11/2026 : *« il n'y a pas de système qui permet de passer une image à fond uni
//! blanc ou complexe à juste la forme principale — par exemple une boule sur un fond multicolore, et
//! qu'on souhaite avoir juste en transparence que la boule […]. Pour l'instant, on sort la version en
//! beta en utilisant la création de quelqu'un — à voir quel est le meilleur algorithme […] ; attention
//! aux licences »*.
//!
//! # Deux gestes, parce que deux images
//!
//! * **Détourer** ([`detourer`]) : un modèle de détourage dit, pour chaque pixel, s'il est de la forme
//!   principale ; puis [`premier_plan`] retire du bord la couleur du fond (DETOURER-2). Le modèle
//!   embarqué est **u2netp** (Xuebin Qin et al., *U²-Net*, 2020 ; licence Apache 2.0) : 4,5 Mo, 1,1
//!   million de paramètres, 19,8 milliards de multiplications pour une image — moins d'une seconde
//!   sur un processeur ordinaire. Sur sa boule sur fond multicolore : 0,3 % d'erreur moyenne.
//! * **Retirer le fond uni** ([`fond_uni`], DETOURER-3) : sans IA, exact — le seul juste pour un
//!   dessin au trait, où le modèle garde le blanc entre les traits.
//!
//! # Un moteur à nous, et le modèle est une donnée
//!
//! Comme l'IA du sens ([`crate::sens`]), Glucose ne dépend d'aucun moteur d'inférence : le modèle
//! est un fichier (`donnees/detourage-u2netp.bin`, sa provenance dans `donnees/LISEZ-MOI.md`), tiré
//! une fois d'ONNX par `outils/detourage/convertir.py`. Il n'emploie que sept opérations (convolution,
//! ReLU, regroupement par maximum, agrandissement bilinéaire, concaténation, addition, sigmoïde) :
//! [`modele`] les lit, `calcul` et `conv` les exécutent, à l'identique de l'étalon en numpy
//! (`outils/detourage/reference.py`), qui rend lui-même le masque d'ONNX Runtime à 2·10⁻⁵ près. Le
//! même moteur fait tourner ISNet (44 M de paramètres, 1024²) : le modèle plus fort, selon la machine.

mod calcul;
mod conv;
mod echantillon;
pub mod fond_uni;
pub mod modele;
pub mod premier_plan;

pub use calcul::Tenseur;
pub use modele::Modele;

use std::sync::OnceLock;

/// **Le modèle que Glucose embarque** : u2netp, 4,5 Mo dans le programme — aucune machine n'a à le
/// télécharger. Sa provenance, sa licence, sa conversion et ses empreintes : `donnees/LISEZ-MOI.md`.
pub static U2NETP: &[u8] = include_bytes!("../donnees/detourage-u2netp.bin");

/// **Le modèle embarqué, lu une seule fois** — ou rien s'il ne se lit pas, ce qu'une épreuve
/// interdit.
pub fn embarque() -> Option<&'static Modele> {
    static LU: OnceLock<Option<Modele>> = OnceLock::new();
    LU.get_or_init(|| Modele::lire(U2NETP).ok()).as_ref()
}

/// **L'entrée du modèle** : l'image (RVBA, `l × h`) réduite à `taille × taille`, divisée par sa plus
/// grande valeur puis centrée et réduite — le prétraitement des auteurs d'U²-Net (`ToTensorLab`).
pub fn entree(modele: &Modele, rvba: &[u8], (l, h): (usize, usize)) -> Tenseur {
    let t = modele.taille;
    let plans = plans_rvb(rvba);
    let max = rvba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| &p[..3])
        .copied()
        .max()
        .unwrap_or(0);
    let max = f32::from(max.max(1)) / 255.0;
    let mut v = Vec::with_capacity(3 * t * t);
    for (c, plan) in plans.iter().enumerate() {
        let petit = echantillon::reechantillonner(plan, (l, h), (t, t));
        v.extend(
            petit
                .iter()
                .map(|x| (x / max - modele.moyenne[c]) / modele.ecart[c]),
        );
    }
    Tenseur {
        c: 3,
        h: t,
        l: t,
        v,
    }
}

/// Les trois plans de couleur d'une image RVBA, entre 0 et 1.
fn plans_rvb(rvba: &[u8]) -> [Vec<f32>; 3] {
    std::array::from_fn(|c| {
        rvba.as_chunks::<4>()
            .0
            .iter()
            .map(|p| f32::from(p[c]) / 255.0)
            .collect()
    })
}

/// **Le masque de la forme principale**, une opacité de 0 à 1 par pixel de l'image : le masque
/// brut du modèle, étiré de son minimum à son maximum (`normPRED`, les auteurs), remis à la taille
/// de l'image.
pub fn masque(modele: &Modele, rvba: &[u8], (l, h): (usize, usize), fils: usize) -> Vec<f32> {
    let brut = calcul::executer(modele, entree(modele, rvba, (l, h)), fils).v;
    let (min, max) = brut
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    let etendue = (max - min).max(1e-6);
    let etire: Vec<f32> = brut.iter().map(|x| (x - min) / etendue).collect();
    let t = modele.taille;
    echantillon::reechantillonner(&etire, (t, t), (l, h))
        .into_iter()
        .map(|a| a.clamp(0.0, 1.0))
        .collect()
}

/// **L'image détourée**, en RVBA non prémultiplié : la forme principale, sa couleur sans le fond
/// (DETOURER-2), le reste transparent. L'opacité qu'avait déjà un pixel se garde.
pub fn detourer(modele: &Modele, rvba: &[u8], (l, h): (usize, usize), fils: usize) -> Vec<u8> {
    let alpha = masque(modele, rvba, (l, h), fils);
    let couleurs = premier_plan::estimer(&plans_rvb(rvba), &alpha, (l, h));
    let mut sortie = Vec::with_capacity(rvba.len());
    for (i, p) in rvba.as_chunks::<4>().0.iter().enumerate() {
        for plan in &couleurs {
            sortie.push((plan[i] * 255.0).round() as u8);
        }
        sortie.push((alpha[i] * f32::from(p[3])).round() as u8);
    }
    sortie
}

/// **Combien de fils** donner au détourage : ceux que la machine a, moins un pour l'image —
/// Glucose se met là où il reste de la place (VISION § 2).
pub fn fils_disponibles() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get().saturating_sub(1).max(1))
}
#[cfg(test)]
mod tests;
