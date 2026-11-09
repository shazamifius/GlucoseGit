//! **AGENT-8 — ce que l'agent ne dit pas ne change pas** (fiche 98).
//!
//! Poser un nœud sous un identifiant qui existe déjà le met à jour (AGENT-4). La pose bâtissait
//! pourtant un nœud **neuf** et le mettait à la place de l'ancien : tout ce que l'appel ne
//! redisait pas retombait à vide. Recolorer une membrane effaçait son titre — et ses rideaux, son
//! mode, son ancrage dans le temps ; réécrire un texte effaçait la couleur, la taille de lettre ou
//! l'opérateur que l'humain y avait mis. Seuls l'appartenance et les domaines étaient gardés,
//! chacun par une ligne à lui (MEMB-8, fiche 88) : la règle était juste, mais partielle.
//!
//! Elle devient générale, et dans le bon sens : une mise à jour **part du nœud tel qu'il est**,
//! et l'outil n'y écrit que ce que l'appel nomme. Un champ qu'on ajoutera au modèle demain sera
//! gardé sans que personne ait à y penser — l'inverse d'une liste de champs à recopier.

use glucose_core::geometry::Rect;
use glucose_core::store::Store;
use glucose_core::types::Annotation;

/// **Le nœud qu'une pose d'agent écrit** : celui qui porte déjà cet identifiant s'il est de la
/// même sorte — l'appel n'y changera que ce qu'il nomme —, sinon le neuf. Un nœud d'une autre
/// sorte part du neuf, et la pose le refusera (« autre sorte »).
pub(super) fn partir_de(s: &Store, board: &str, neuf: Annotation) -> Annotation {
    match s.project.annotation(board, neuf.id()) {
        Some(a) if std::mem::discriminant(a) == std::mem::discriminant(&neuf) => a.clone(),
        _ => neuf,
    }
}

/// **Un texte ou un post-it, mis à jour** : sa boîte et son texte, que l'appel dit toujours ; sa
/// couleur s'il la dit. Le reste — sa taille de lettre, son opérateur, son fond, son miroir, son
/// ancrage dans le temps — est celui du nœud.
pub(super) fn un_texte(
    s: &Store,
    board: &str,
    neuf: Annotation,
    boite: Rect,
    couleur: Option<&str>,
) -> Annotation {
    let texte = neuf.own_text().unwrap_or_default();
    let mut ann = partir_de(s, board, neuf);
    if let Annotation::Text {
        x,
        y,
        width,
        height,
        text,
        color,
        cursor_pos,
        ..
    }
    | Annotation::Sticky {
        x,
        y,
        width,
        height,
        text,
        color,
        cursor_pos,
        ..
    } = &mut ann
    {
        (*x, *y) = (boite.left, boite.top);
        (*width, *height) = (Some(boite.width), Some(boite.height));
        // La place d'un curseur ne survit pas au texte qu'on réécrit.
        (*text, *cursor_pos) = (texte, None);
        if let Some(c) = couleur {
            *color = Some(c.to_string());
        }
    }
    ann
}

/// **Une membrane, mise à jour** : sa boîte ; son titre et sa couleur, s'il les dit. Le reste —
/// son mode, ses rideaux, son miroir, son ancrage dans le temps — est celui du nœud.
pub(super) fn une_membrane(
    s: &Store,
    board: &str,
    id: &str,
    boite: Rect,
    (titre, couleur): (Option<&str>, Option<&str>),
) -> Annotation {
    let neuve = Annotation::membrane(id, boite.left, boite.top, boite.width, boite.height);
    let mut ann = partir_de(s, board, neuve);
    if let Annotation::Membrane {
        x,
        y,
        width,
        height,
        text,
        color,
        ..
    } = &mut ann
    {
        (*x, *y, *width, *height) = (boite.left, boite.top, boite.width, boite.height);
        if let Some(t) = titre {
            *text = Some(t.to_string());
        }
        if let Some(c) = couleur {
            *color = Some(c.to_string());
        }
    }
    ann
}
