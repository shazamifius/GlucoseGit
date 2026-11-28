//! **LECTEUR-1** (fiche 193) : un lecteur ne touche pas le canevas partagé — ses éditions sont
//! refusées au journal, avant toute image.

use super::*;

/// **Ce qu'il pose n'apparaît pas, même un instant**, et ne laisse rien à annuler.
#[test]
fn lecteur_1_ce_qu_il_pose_n_apparait_pas() {
    let (salle, _alice, _bob) = depart();
    let mut carol = salle.rejoindre("c9", false);
    assert!(carol.lecteur());
    let version = carol.version;
    carol.add_image(T, img("z", 9.0));
    assert_eq!(x_de(&carol, "z"), None, "refusé avant d'être vu");
    assert!(!carol.can_undo(), "rien au journal");
    assert_eq!(carol.version, version, "le document n'a pas changé");
    assert_eq!(
        carol.partage.as_ref().map(|p| p.defaits_au_lecteur),
        Some(1),
        "il le saura"
    );
}

/// **Ce qu'il glisse ne bouge pas sous sa main** : pas même un pas.
#[test]
fn lecteur_1_ce_qu_il_glisse_ne_bouge_pas() {
    let (salle, _alice, _bob) = depart();
    let mut carol = salle.rejoindre("c9", false);
    carol.select_image("b".into(), false);
    carol.begin_live_edit();
    carol.glisser_la_selection(T, (40.0, 0.0));
    assert_eq!(x_de(&carol, "b"), Some(1.0), "au milieu du geste");
    carol.glisser_la_selection(T, (80.0, 0.0));
    carol.end_live_edit();
    assert_eq!(x_de(&carol, "b"), Some(1.0), "au relâcher");
    assert!(!carol.can_undo());
}

/// **Une saisie en bloc** (ORDONNER, plusieurs éditions d'un coup) est retournée tout entière.
#[test]
fn lecteur_1_plusieurs_editions_d_un_coup_sont_toutes_retournees() {
    let (salle, _alice, _bob) = depart();
    let mut carol = salle.rejoindre("c9", false);
    carol.select_image("a".into(), false);
    carol.select_image("c".into(), true);
    carol.delete_selected(T);
    assert_eq!(xs(&carol), [("a", 0.0), ("b", 1.0), ("c", 2.0)]);
}

/// **Celui qui écrit n'est pas un lecteur** : son geste s'applique.
#[test]
fn lecteur_1_celui_qui_ecrit_pose() {
    let (_salle, _alice, mut bob) = depart();
    assert!(!bob.lecteur());
    bob.add_image(T, img("z", 9.0));
    assert_eq!(x_de(&bob, "z"), Some(9.0));
}
