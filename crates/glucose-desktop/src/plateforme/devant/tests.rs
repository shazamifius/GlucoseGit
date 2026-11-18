//! Les épreuves de DEVANT-1 : ce que Glucose demande à chaque bureau, lu sans le jouer.

use super::*;

fn env<'a>(paires: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |cle| {
        paires
            .iter()
            .find(|(k, _)| *k == cle)
            .map(|(_, v)| v.to_string())
    }
}

#[test]
fn test_devant_1_le_bureau_se_lit_dans_son_environnement() {
    let niri = [
        ("NIRI_SOCKET", "/run/user/1000/niri.sock"),
        ("XDG_CURRENT_DESKTOP", "niri"),
    ];
    assert_eq!(Bureau::de(env(&niri)), Bureau::Niri);
    assert_eq!(
        Bureau::de(env(&[("HYPRLAND_INSTANCE_SIGNATURE", "a1")])),
        Bureau::Hyprland
    );
    assert_eq!(
        Bureau::de(env(&[("SWAYSOCK", "/run/sway.sock")])),
        Bureau::Sway
    );
    assert_eq!(
        Bureau::de(env(&[("XDG_CURRENT_DESKTOP", "KDE")])),
        Bureau::Plasma
    );
    assert_eq!(
        Bureau::de(env(&[("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")])),
        Bureau::Gnome
    );
    assert_eq!(
        Bureau::de(env(&[("XDG_CURRENT_DESKTOP", "XFCE")])),
        Bureau::Autre
    );
    // Une prise vide n'est pas un bureau : elle reste d'une session qu'on a quittée.
    assert_eq!(Bureau::de(env(&[("NIRI_SOCKET", "")])), Bureau::Autre);
}

#[test]
fn test_devant_1_gnome_et_l_inconnu_le_disent_sans_rien_jouer() {
    assert_eq!(garder_devant(Bureau::Gnome, true), Devant::AlaMain(GNOME));
    assert_eq!(garder_devant(Bureau::Autre, true), Devant::AlaMain(AUTRE));
}

#[test]
fn test_devant_1_la_fenetre_se_designe_par_son_processus() {
    // Ce que niri 26.04 écrit, réduit à deux fenêtres : Glucose n'a pas le focus.
    let fenetres = r#"[{"id":4,"title":"Discord","app_id":"vesktop","pid":4784,"is_focused":true},
        {"id":9,"title":"Glucose","app_id":"glucose","pid":5123,"is_focused":false}]"#;
    assert_eq!(id_niri(fenetres, 5123), Some(9));
    assert_eq!(id_niri(fenetres, 1), None);
    assert_eq!(
        commande(Bureau::Niri, 5123, true, Some(9)),
        [
            "niri",
            "msg",
            "action",
            "move-window-to-floating",
            "--id",
            "9"
        ]
    );
    assert_eq!(
        commande(Bureau::Niri, 5123, false, Some(9)),
        [
            "niri",
            "msg",
            "action",
            "move-window-to-tiling",
            "--id",
            "9"
        ]
    );
    assert_eq!(
        commande(Bureau::Hyprland, 5123, true, None),
        ["hyprctl", "dispatch", "setfloating", "pid:5123"]
    );
    assert_eq!(
        commande(Bureau::Sway, 5123, false, None),
        ["swaymsg", "[pid=5123] floating disable"]
    );
}

#[test]
fn test_devant_1_kwin_pose_keep_above_sur_ce_processus() {
    let script = script_kwin(5123, true);
    assert!(script.contains("f.pid === 5123") && script.contains("f.keepAbove = true"));
    assert!(script_kwin(5123, false).contains("f.keepAbove = false"));
    let reponse = "method return time=1 sender=:1.4 -> destination=:1.9 serial=7\n   int32 3\n";
    assert_eq!(numero_du_script(reponse), Some(3));
    // `loadScript` rend -1 quand un script de ce nom est déjà là.
    assert_eq!(numero_du_script("method return\n   int32 -1\n"), None);
}
