//! Ce que la loi de la perception doit garantir — sans écran, et sans œil.

use super::*;

/// Combien de pixels physiques par seconde font tant de degrés, à cette échelle.
fn a_tant_de_degres(degres: f64, echelle: f64) -> f64 {
    degres / DEGRES_PAR_PIXEL_LOGIQUE * echelle
}

/// **À l'arrêt, rien n'est dégradable.** C'est la netteté intégrale que la charte exige, et
/// elle ne doit tenir à aucun seuil : zéro pixel par seconde donne un, point.
#[test]
fn a_l_arret_la_nettete_est_entiere() {
    assert_eq!(
        Perception::a_la_vitesse(0.0, 0.0, 1.0).facteur_admissible(),
        1
    );
    assert_eq!(
        Perception::a_la_vitesse(0.0, 0.0, 3.0).facteur_admissible(),
        1
    );
    assert!(!Perception::a_la_vitesse(0.0, 0.0, 1.0).autorise_a_degrader());
    assert_eq!(Perception::nette().facteur_admissible(), 1);
}

/// **Tant que l'œil peut poursuivre, il voit tout.** En dessous de vingt degrés par seconde
/// il stabilise le contenu sur sa rétine : dégrader s'y verrait, quelle que soit la vitesse
/// en pixels.
#[test]
fn sous_la_vitesse_de_poursuite_rien_n_est_degradable() {
    for degres in [1.0, 5.0, 12.0, 19.9] {
        let p = Perception::a_la_vitesse(a_tant_de_degres(degres, 1.0), 0.0, 1.0);
        assert_eq!(
            p.facteur_admissible(),
            1,
            "a {degres} deg/s l'oeil poursuit encore : il voit net"
        );
        assert!(!p.autorise_a_degrader());
    }
}

/// **Le facteur ne décroît jamais quand la vitesse monte** — c'est la monotonie sans laquelle
/// la finesse oscillerait pendant un geste, et l'oscillation se voit davantage que le grain.
#[test]
fn le_facteur_admissible_ne_decroit_jamais_avec_la_vitesse() {
    let mut precedent = 1;
    for px in (0..8_000).step_by(25) {
        let facteur = Perception::a_la_vitesse(f64::from(px), 0.0, 1.0).facteur_admissible();
        assert!(
            facteur >= precedent,
            "a {px} px/s le facteur est retombe a {facteur} apres {precedent}"
        );
        precedent = facteur;
    }
}

/// **La netteté revient en même temps que le mouvement s'éteint.** C'est le défaut signalé,
/// et ce test le verrouille sur la trajectoire réelle d'un amortissement exponentiel.
#[test]
fn un_amortissement_exponentiel_redevient_net_avant_de_s_eteindre() {
    const TAU: f64 = 0.45;
    let depart = 4_000.0;

    // Au lâcher, la vue file : l'œil ne suit plus, on a le droit d'abîmer.
    assert!(Perception::a_la_vitesse(depart, 0.0, 1.0).autorise_a_degrader());

    // On intègre l'amortissement, et on cherche l'instant où la netteté revient.
    let mut retour = None;
    for pas in 0..200 {
        let t = f64::from(pas) * 0.01;
        let vitesse = depart * (-t / TAU).exp();
        if !Perception::a_la_vitesse(vitesse, 0.0, 1.0).autorise_a_degrader() {
            retour = Some(t);
            break;
        }
    }
    let retour = retour.expect("la nettete doit revenir");

    // Elle revient **avant** que l'élan ne s'éteigne — trois constantes de temps mettent la
    // vitesse à 5 % de sa valeur initiale, et la netteté ne doit pas attendre jusque-là.
    assert!(
        retour < 3.0 * TAU,
        "la nettete revient a {retour:.2} s, l'elan dure {:.2} s",
        3.0 * TAU
    );
    // Et pas trop tôt non plus : sinon la loi n'autoriserait jamais rien, et le geste rapide
    // paierait le rendu fin pour rien.
    assert!(
        retour > 0.2,
        "la nettete revient des {retour:.2} s : le geste rapide n'a rien gagne"
    );
}

/// **Un écran dense ne dégrade pas plus vite.** La loi s'exprime en degrés vus : à la même
/// vitesse *apparente*, deux écrans de densités différentes doivent tolérer la même chose.
#[test]
fn deux_ecrans_de_densites_differentes_tolerent_la_meme_chose() {
    for degres in [25.0, 40.0, 80.0] {
        let simple = Perception::a_la_vitesse(a_tant_de_degres(degres, 1.0), 0.0, 1.0);
        let dense = Perception::a_la_vitesse(a_tant_de_degres(degres, 2.5), 0.0, 2.5);
        assert_eq!(
            simple.facteur_admissible(),
            dense.facteur_admissible(),
            "a {degres} deg/s, l'echelle de l'ecran ne doit rien changer"
        );
    }
}

/// Le facteur reste une puissance de deux — les seuls paliers qu'un rendu réduit sait viser.
#[test]
fn le_facteur_est_toujours_une_puissance_de_deux() {
    for px in (0..20_000).step_by(37) {
        let f = Perception::a_la_vitesse(f64::from(px), 0.0, 1.0).facteur_admissible();
        assert!(f >= 1 && f.is_power_of_two(), "{px} px/s donne {f}");
    }
}

/// Une vitesse absurde — un saut de caméra, une valeur corrompue — ne doit ni déborder ni
/// rendre zéro. Le pire cas reste un facteur, grand mais fini.
#[test]
fn une_vitesse_insensee_ne_casse_rien() {
    for vitesse in [1e9, f64::MAX, f64::INFINITY] {
        let f = Perception::a_la_vitesse(vitesse, 0.0, 1.0).facteur_admissible();
        assert!(f >= 1 && f.is_power_of_two(), "{vitesse} donne {f}");
    }
    // Et une vitesse négative — qui n'a pas de sens — se lit comme un arrêt.
    assert_eq!(
        Perception::a_la_vitesse(-500.0, 0.0, 1.0).facteur_admissible(),
        1
    );
}

/// **Un zoom n'est pas poursuivable, et c'est ce qui le distingue d'un déplacement.**
///
/// L'œil ne poursuit qu'une trajectoire. Un déplacement emporte tout l'écran dans le même
/// sens — une seule poursuite les annule tous. Un zoom écarte le contenu radialement : chaque
/// point part ailleurs, et rien ne peut suivre ailleurs qu'au point fixe.
///
/// À vitesse apparente égale, un zoom doit donc autoriser **plus** qu'un déplacement.
#[test]
fn un_zoom_autorise_plus_qu_un_deplacement_de_meme_vitesse() {
    for degres in [8.0, 15.0, 30.0, 60.0] {
        let px = a_tant_de_degres(degres, 1.0);
        let pan = Perception::a_la_vitesse(px, 0.0, 1.0).facteur_admissible();
        let zoom = Perception::a_la_vitesse(0.0, px, 1.0).facteur_admissible();
        assert!(
            zoom >= pan,
            "a {degres} deg/s : un zoom tolere {zoom}, un deplacement {pan}"
        );
    }

    // Et sous la vitesse de poursuite, l'écart est franc : le déplacement n'autorise rien,
    // le zoom autorise déjà quelque chose.
    let px = a_tant_de_degres(10.0, 1.0);
    assert_eq!(
        Perception::a_la_vitesse(px, 0.0, 1.0).facteur_admissible(),
        1
    );
    assert!(Perception::a_la_vitesse(0.0, px, 1.0).autorise_a_degrader());
}

/// **Les deux mouvements s'additionnent.** Zoomer en se déplaçant glisse davantage sous l'œil
/// que l'un des deux seul — sans quoi une diagonale zoomée passerait pour un simple pan.
#[test]
fn le_zoom_s_ajoute_a_ce_que_le_deplacement_laisse_passer() {
    let rapide = a_tant_de_degres(40.0, 1.0);
    let un_peu = a_tant_de_degres(6.0, 1.0);

    let seul = Perception::a_la_vitesse(rapide, 0.0, 1.0).facteur_admissible();
    let avec = Perception::a_la_vitesse(rapide, un_peu, 1.0).facteur_admissible();
    assert!(
        avec >= seul,
        "un zoom par-dessus un pan rapide : {avec} contre {seul}"
    );
}
