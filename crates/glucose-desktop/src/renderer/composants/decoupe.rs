//! **DE-PRES-1 — le découpage d'un composant plus grand que l'écran.**
//!
//! # La grille, et pourquoi elle est ancrée au composant
//!
//! Les tuiles sont des carrés de [`COTE`] pixels, comptés depuis le coin de la texture que le
//! composant aurait s'il était entier. Ancrées à lui et non à l'écran : se déplacer de près ne
//! change ni leur contenu ni leur clé — seulement lesquelles on voit et où elles se posent.
//! Seules celles qui entrent se rendent.
//!
//! [`COTE`] est le côté que `bench_tuiles` a mesuré pour la grille du canevas (TUILE-1) : le
//! même compromis — un dessin par carré contre le travail qu'une invalidation jette — et la
//! même réponse. Aucun nombre nouveau.
//!
//! # Ce que la mesure a établi avant d'écrire une ligne
//!
//! Une carte rendue par tuiles, reposée, donne les pixels d'une carte rendue d'un tenant :
//! texte, fond et bords droits au bit près, **aucune couture** aux frontières des tuiles. Les
//! seuls écarts sont sur l'arrondi des coins, un pixel de large, jusqu'à 48 niveaux — et la
//! texture entière d'avant a exactement les mêmes contre le rendu en place, dès l'échelle 2 :
//! `tiny-skia` approche un arc par des segments en virgule fixe, et leur arrondi dépend de la
//! position absolue de l'arc. Le découpage n'ajoute rien à ce qui existait.
//!
//! # La gouttière : un texel, le rayon du filtre
//!
//! En mouvement, la carte graphique interpole entre quatre texels. Au bord d'une tuile, deux
//! d'entre eux appartiennent à la voisine : sans eux, le filtre répéterait le dernier texel de
//! la tuile, et une couture d'un pixel paraîtrait pendant le geste. Chaque tuile se rend donc
//! avec **un texel de ses voisines** tout autour, et se pose en ne montrant que son centre : le
//! filtre bilinéaire ne lit jamais plus loin qu'un texel, c'est son rayon.
//!
//! À l'arrêt, la tuile se pose sur un pixel entier sans agrandissement, et le centre de chaque
//! pixel tombe sur le centre d'un texel : la gouttière n'est pas lue.
//!
//! # Le repli : jamais un trou
//!
//! Quand l'échelle change, les tuiles de la nouvelle échelle n'existent pas encore, et leur
//! identité change avec elle — poser l'ancienne tuile de même numéro montrerait un autre
//! morceau du composant. Chaque tuile porte donc son **repli** : le même rectangle, lu dans la
//! texture entière du composant à un palier plus bas. C'est ce que fait un navigateur quand une
//! tuile nette manque : il pose la version basse résolution qu'il garde toujours — un peu flou
//! le temps d'une image ou deux, jamais absent.

use super::{empreinte, Composant, Regime};
use crate::present::scene_gpu::Pose;
use crate::renderer::voies::APoser;
use glucose_core::tuile::COTE;
use std::sync::Arc;

/// **Les tuiles de `entier` que l'écran montre**, rangée par rangée, chacune avec son repli.
pub(super) fn tuiles(
    regime: &Regime,
    entier: &Composant,
    repli: Option<&Composant>,
) -> Vec<Composant> {
    let rapport = regime.rapport();
    let pas = COTE as f32 * rapport;
    let colonnes = visibles(
        entier.pose.x,
        (0.0, regime.clip.width),
        pas,
        entier.pixels.0,
    );
    let rangees = visibles(
        entier.pose.y,
        (regime.clip.top, regime.clip.height),
        pas,
        entier.pixels.1,
    );
    // Le contenu, l'échelle et la phase sont ceux du composant entier : son empreinte vaut
    // pour toutes ses tuiles, et le numéro de chacune est dans son identité.
    let empreinte = empreinte(entier);
    let mut tuiles = Vec::with_capacity(colonnes.len() * rangees.len());
    for ty in rangees {
        for tx in colonnes.clone() {
            tuiles.push(tuile(entier, (tx, ty), (rapport, empreinte), repli));
        }
    }
    tuiles
}

/// Les numéros des tuiles que l'intervalle d'écran `[debut, fin)` montre, sur un axe où le
/// composant commence en `origine` et mesure `pixels` texels — chaque tuile couvrant `pas`
/// pixels d'écran.
fn visibles(origine: f32, (debut, fin): (f32, f32), pas: f32, pixels: u32) -> std::ops::Range<u32> {
    let nombre = pixels.div_ceil(COTE);
    let apres = (((fin - origine) / pas).ceil().max(0.0) as u32).min(nombre);
    let premiere = (((debut - origine) / pas).floor().max(0.0) as u32).min(apres);
    premiere..apres
}

/// La tuile `(tx, ty)` de `entier` : sa texture, gouttière comprise, et où elle se pose.
fn tuile(
    entier: &Composant,
    (tx, ty): (u32, u32),
    (rapport, empreinte): (f32, u64),
    repli: Option<&Composant>,
) -> Composant {
    let debut = (tx * COTE, ty * COTE);
    let taille = (
        (entier.pixels.0 - debut.0).min(COTE),
        (entier.pixels.1 - debut.1).min(COTE),
    );
    // L'échelle est dans l'identité : une tuile d'une autre échelle montre un autre morceau
    // du composant, et la poser à la place de celle-ci serait faux, pas flou.
    let identite = format!(
        "{}@{:x}#{tx},{ty}",
        entier.identite,
        entier.echelle.to_bits()
    );
    let pose = Pose {
        x: entier.pose.x + debut.0 as f32 * rapport,
        y: entier.pose.y + debut.1 as f32 * rapport,
        largeur: taille.0 as f32 * rapport,
        hauteur: taille.1 as f32 * rapport,
        // La gouttière est dans la texture, pas à l'écran : la fenêtre n'en montre que le
        // centre, et le filtre, lui, a le droit de la lire.
        fenetre: [
            1.0 / (taille.0 + 2) as f32,
            1.0 / (taille.1 + 2) as f32,
            taille.0 as f32 / (taille.0 + 2) as f32,
            taille.1 as f32 / (taille.1 + 2) as f32,
        ],
        ..entier.pose
    };
    Composant {
        cle: format!("{identite}:{empreinte:016x}"),
        repli: repli.map(|r| APoser {
            cle: r.cle.clone(),
            identite: r.identite.clone(),
            pose: Pose {
                fenetre: fenetre_dans(r, entier, debut, taille),
                ..pose
            },
            repli: None,
        }),
        identite,
        pose,
        contenu: Arc::clone(&entier.contenu),
        echelle: entier.echelle,
        phase: entier.phase,
        marge: entier.marge,
        pixels: (taille.0 + 2, taille.1 + 2),
        depart: (debut.0 as f32 - 1.0, debut.1 as f32 - 1.0),
    }
}

/// **Où le rectangle `[debut, debut + taille)` de la texture entière tombe dans celle du
/// repli**, en fractions de celle-ci.
///
/// Un texel de la texture entière est à `(texel - marge - phase) / echelle` du coin du contenu,
/// dans le monde ; le repli place ce point à `monde × palier + marge + phase`. Le rapport des
/// deux échelles fait tout le passage.
fn fenetre_dans(
    repli: &Composant,
    entier: &Composant,
    debut: (u32, u32),
    taille: (u32, u32),
) -> [f32; 4] {
    let k = (repli.echelle / entier.echelle) as f32;
    let x0 = (debut.0 as f32 - entier.marge - entier.phase.0) * k + repli.marge + repli.phase.0;
    let y0 = (debut.1 as f32 - entier.marge - entier.phase.1) * k + repli.marge + repli.phase.1;
    [
        x0 / repli.pixels.0 as f32,
        y0 / repli.pixels.1 as f32,
        taille.0 as f32 * k / repli.pixels.0 as f32,
        taille.1 as f32 * k / repli.pixels.1 as f32,
    ]
}

#[cfg(test)]
mod tests;
