//! Les lois des raccourcis comme donnée (P4, fiche 72).

use super::*;

fn a(texte: &str) -> Accord {
    Accord::lire(texte).unwrap_or_else(|e| panic!("{texte} : {e}"))
}

/// **ACCORD-1 — le clavier français** : AltGr (Ctrl+Alt) a écrit son caractère, Maj a écrit un
/// signe ou un chiffre ; ni l'un ni l'autre ne compte alors. Ctrl compte toujours, Alt seul et
/// Maj sur une lettre comptent.
#[test]
fn test_accord_1_la_loi_du_clavier_francais() {
    // `Ctrl+]` en AZERTY : AltGr + `)`, qui arrive comme Ctrl+Alt.
    assert_eq!(
        Accord::de(true, true, false, Touche::Signe(']')),
        a("Ctrl+]")
    );
    // `1` en AZERTY se tape avec Maj : la touche marquée 1 reste `1`.
    assert_eq!(Accord::de(false, false, true, Touche::Chiffre(1)), a("1"));
    assert_eq!(
        Accord::de(true, false, true, Touche::Chiffre(3)),
        a("Ctrl+3")
    );
    // `?` est Maj + `,` : le signe est déjà là.
    assert_eq!(Accord::de(false, false, true, Touche::Signe('?')), a("?"));
    // Ce qui compte, compte.
    assert_ne!(
        Accord::de(true, false, true, Touche::Lettre('z')),
        a("Ctrl+Z")
    );
    assert_eq!(
        Accord::de(true, false, true, Touche::Lettre('z')),
        a("Ctrl+Maj+Z")
    );
    assert_eq!(
        Accord::de(false, true, false, Touche::Lettre('f')),
        a("Alt+F")
    );
    assert_eq!(
        Accord::de(false, false, true, Touche::Nommee(Nommee::Gauche)),
        a("Maj+Gauche")
    );
}

/// **Un accord se lit comme il s'écrit**, et se dit à l'écran avec ses flèches et ses accents.
#[test]
fn test_accord_1_s_ecrit_et_se_relit() {
    for texte in [
        "Ctrl+Z",
        "Ctrl+Maj+Z",
        "Alt+F",
        "1",
        "Ctrl+1",
        "Ctrl+]",
        "Maj+Gauche",
        "Suppr",
        "F2",
        "Espace",
        "Echap",
        "Ctrl++",
        "+",
    ] {
        assert_eq!(a(texte).ecrire(), texte);
    }
    assert_eq!(
        a("ctrl+shift+z").ecrire(),
        "Ctrl+Maj+Z",
        "les alias anglais se lisent"
    );
    assert_eq!(a("Maj+Gauche").dire(), "Maj+←");
    assert_eq!(a("Echap").dire(), "Échap");
    assert_eq!(a("Échap"), a("Echap"), "le nom affiché se relit aussi");
}

/// **Ce qui ne pourrait jamais se déclencher est refusé, avec sa raison.**
#[test]
fn test_accord_1_l_impossible_est_refuse() {
    for faux in ["Ctrl+Alt+Z", "Maj+1", "Maj+]", "Hyper+Z", "Ctrl+Nulle", ""] {
        assert!(Accord::lire(faux).is_err(), "{faux:?} devrait être refusé");
    }
}

/// **Une couche se relit à l'identique**, et ses lignes fausses sont dites par leur numéro —
/// la couche garde tout le reste.
#[test]
fn test_couche_se_relit_et_dit_ses_lignes_fausses() {
    let texte = "# Les raccourcis de Glucose\n\
                 Ctrl+Z        annuler\n\
                 \n\
                 1             fleche.relation.1 | signet.aller.1\n\
                 Ctrl+Alt+Q    quitter\n\
                 Ctrl+S        -\n\
                 Ctrl+Z        retablir\n\
                 Ctrl+E\n";
    let (couche, erreurs) = Couche::lire("glucose", texte);
    let lignes: Vec<usize> = erreurs.iter().map(|e| e.ligne).collect();
    assert_eq!(lignes, [5, 7, 8], "{erreurs:?}");
    assert_eq!(
        couche.liaison(&a("1")),
        Some(
            &[
                "fleche.relation.1".to_string(),
                "signet.aller.1".to_string()
            ][..]
        )
    );
    assert_eq!(couche.liaison(&a("Ctrl+S")), Some(&[][..]), "un retrait");
    let (relue, rien) = Couche::lire("glucose", &couche.ecrire());
    assert!(rien.is_empty());
    assert_eq!(relue, couche);
}

/// **La pile** : la couche du dessus remplace la liste d'un accord, ou la retire ; les accords
/// d'une commande sont ceux qui y mènent encore — ceux du dessus d'abord, puis dans l'ordre
/// du fichier.
#[test]
fn test_clavier_la_couche_du_dessus_l_emporte() {
    let (base, _) = Couche::lire(
        "glucose",
        "Ctrl+Z annuler\nCtrl+Y retablir\nCtrl+Maj+Z retablir\nCtrl+S enregistrer\n1 a | b\n",
    );
    let (dessus, _) = Couche::lire("figma", "Ctrl+Y dupliquer\nCtrl+S -\n1 c\n");
    let clavier = Clavier::de(vec![base, dessus]);
    assert_eq!(clavier.commandes(&a("Ctrl+Z")), ["annuler"]);
    assert_eq!(clavier.commandes(&a("Ctrl+Y")), ["dupliquer"]);
    assert!(clavier.commandes(&a("Ctrl+S")).is_empty(), "retiré");
    assert!(clavier.commandes(&a("Ctrl+Q")).is_empty(), "inconnu");
    assert_eq!(
        clavier.commandes(&a("1")),
        ["c"],
        "la liste entière est remplacée"
    );
    assert_eq!(clavier.accords_de("retablir"), [a("Ctrl+Maj+Z")]);
    assert_eq!(clavier.accords_de("dupliquer"), [a("Ctrl+Y")]);
    assert!(clavier.accords_de("enregistrer").is_empty());
    let (seule, _) = Couche::lire("g", "Ctrl+Maj+A mode\nAlt+T mode\n");
    let (dessus, _) = Couche::lire("u", "Ctrl+Q mode\n");
    assert_eq!(
        Clavier::de(vec![seule.clone()]).accords_de("mode"),
        [a("Ctrl+Maj+A"), a("Alt+T")],
        "le premier écrit d'abord : c'est lui que le menu dit"
    );
    assert_eq!(
        Clavier::de(vec![seule, dessus]).accords_de("mode")[0],
        a("Ctrl+Q"),
        "la couche du dessus passe avant"
    );
}

/// Un clavier de deux couches : un preset, et celle de l'utilisateur, vide.
fn preset_et_utilisateur(preset: &str) -> Clavier {
    let (dessous, erreurs) = Couche::lire("preset", preset);
    assert!(erreurs.is_empty(), "{erreurs:?}");
    Clavier::de(vec![dessous, Couche::nommee("utilisateur")])
}

fn mene(clavier: &Clavier, accord: &str) -> Vec<String> {
    clavier.commandes(&a(accord)).to_vec()
}

/// **Changer un raccourci remplace** : la commande n'a plus que le nouvel accord, l'accord ne
/// mène plus qu'à elle, et ce qu'il faisait d'autre est rendu — pour être dit. Le preset ne
/// bouge pas : tout s'écrit dans la couche du dessus.
#[test]
fn test_changer_un_raccourci_remplace_et_dit_ce_qu_il_remplace() {
    let mut c = preset_et_utilisateur(
        "Ctrl+D  dupliquer\nCtrl+K  dupliquer\nCtrl+J  joindre\n1  relation.1 | signet.aller.1\n",
    );
    let preset = c.couches()[0].clone();
    let remplacees = c.lier_seul("dupliquer", a("Ctrl+J"));
    assert_eq!(remplacees, vec!["joindre".to_string()]);
    assert_eq!(c.accords_de("dupliquer"), vec![a("Ctrl+J")]);
    assert_eq!(mene(&c, "Ctrl+J"), vec!["dupliquer"]);
    assert!(mene(&c, "Ctrl+D").is_empty() && mene(&c, "Ctrl+K").is_empty());
    assert!(c.accords_de("joindre").is_empty());
    assert_eq!(c.couches()[0], preset, "le preset a bougé");
    // Un accord qui menait à deux commandes : en prendre une en laisse l'autre ailleurs.
    let remplacees = c.lier_seul("signet.aller.1", a("Ctrl+1"));
    assert!(remplacees.is_empty());
    assert_eq!(mene(&c, "1"), vec!["relation.1"]);
    // La couche de l'utilisateur s'écrit et se relit à l'identique.
    let ecrite = c.couches()[1].ecrire();
    let (relue, erreurs) = Couche::lire("utilisateur", &ecrite);
    assert!(erreurs.is_empty());
    assert_eq!(relue.ecrire(), ecrite);
}

/// **Retirer un raccourci** : plus aucun accord ne mène à la commande ; un accord partagé mène
/// encore à l'autre ; et rien ne retombe sur ce que disait le preset.
#[test]
fn test_retirer_un_raccourci() {
    let mut c = preset_et_utilisateur("Ctrl+D  dupliquer\n1  relation.1 | signet.aller.1\n");
    c.delier("dupliquer");
    c.delier("relation.1");
    assert!(c.accords_de("dupliquer").is_empty());
    assert!(mene(&c, "Ctrl+D").is_empty());
    assert_eq!(mene(&c, "1"), vec!["signet.aller.1"]);
}

/// **Rétablir rend le preset** — à cette commande, et à ce que ses accords faisaient d'autre ;
/// les autres changements de l'utilisateur restent.
#[test]
fn test_retablir_rend_le_preset_a_cette_commande() {
    let mut c = preset_et_utilisateur(
        "Ctrl+D  dupliquer\nCtrl+J  joindre\nCtrl+S  enregistrer\n1  relation.1 | signet.aller.1\n",
    );
    c.lier_seul("dupliquer", a("Ctrl+J"));
    c.lier_seul("enregistrer", a("F2"));
    c.delier("relation.1");
    c.retablir("dupliquer");
    assert_eq!(c.accords_de("dupliquer"), vec![a("Ctrl+D")]);
    assert_eq!(
        mene(&c, "Ctrl+J"),
        vec!["joindre"],
        "Ctrl+J n'est pas rendu"
    );
    assert_eq!(
        c.accords_de("enregistrer"),
        vec![a("F2")],
        "un autre changement perdu"
    );
    c.retablir("relation.1");
    assert_eq!(mene(&c, "1"), vec!["relation.1", "signet.aller.1"]);
    c.retablir("enregistrer");
    assert!(c.couches()[1].est_vide(), "{}", c.couches()[1].ecrire());
}
