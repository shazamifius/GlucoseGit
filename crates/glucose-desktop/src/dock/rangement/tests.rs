//! TIROIR-5 — le rangement des tiroirs dans l'écran, éprouvé sur ses lois seules.

use super::*;

fn tiroir(tab: TabId, largeur: f32, naturelle: f32) -> Tiroir {
    Tiroir {
        tab,
        largeur,
        naturelle,
    }
}

/// **Le partage max-min** : le minuteur (184) tient dans sa part égale et reste entier ; Ordonner
/// (410) et la Time Machine (600) cèdent à une hauteur commune ; la colonne prend toute la place,
/// et pas un pixel de plus.
#[test]
fn test_tiroir_5_le_partage_est_equitable() {
    let colonne = [
        tiroir(TabId::Temps, 300.0, 600.0),
        tiroir(TabId::Pomodoro, 160.0, 184.0),
        tiroir(TabId::Organize, 250.0, 410.0),
    ];
    let h = partager_la_hauteur(&colonne, 700.0);
    assert_eq!(h[1], 184.0, "le minuteur reste entier");
    assert_eq!(h[0], h[2], "les deux plus hauts cèdent à la même hauteur");
    assert!(
        (h.iter().sum::<f32>() - 700.0).abs() < 1e-3,
        "toute la place : {h:?}"
    );
    // Tout tient : chacun garde sa hauteur.
    assert_eq!(
        partager_la_hauteur(&colonne, 2_000.0),
        vec![600.0, 184.0, 410.0]
    );
}

/// **Quand tout tient, rien ne change** : le rangement de TIROIR-4 — deux tiroirs empilés, le
/// troisième dans la colonne voisine —, chacun à sa hauteur naturelle.
#[test]
fn test_tiroir_5_quand_tout_tient_le_premier_qui_tient_reste() {
    let gauche = vec![
        tiroir(TabId::Organize, 250.0, 426.0),
        tiroir(TabId::Pomodoro, 160.0, 200.0),
        tiroir(TabId::Storyboard, 260.0, 300.0),
    ];
    let [g, d] = ranger([gauche, Vec::new()], 1440.0, 722.0);
    assert!(d.is_empty());
    assert_eq!(g.len(), 2, "le Storyboard ouvre la colonne voisine");
    let tabs: Vec<Vec<(TabId, f32)>> = g
        .iter()
        .map(|c| c.tiroirs.iter().map(|(t, h)| (t.tab, *h)).collect())
        .collect();
    assert_eq!(
        tabs,
        vec![
            vec![(TabId::Organize, 426.0), (TabId::Pomodoro, 200.0)],
            vec![(TabId::Storyboard, 300.0)],
        ]
    );
    assert_eq!((g[0].largeur, g[1].largeur), (250.0, 260.0));
}

/// Le cas de la fiche 122 § 7 : 1440 × 900, la bande de 78, huit domaines — trois colonnes à
/// droite, deux à gauche.
fn le_pire_cas() -> [Vec<Tiroir>; 2] {
    [
        vec![
            tiroir(TabId::Organize, 250.0, 426.0),
            tiroir(TabId::Temps, 300.0, 520.0),
            tiroir(TabId::Storyboard, 260.0, 420.0),
            tiroir(TabId::Pomodoro, 160.0, 200.0),
        ],
        vec![
            tiroir(TabId::Plugins, 340.0, 610.0),
            tiroir(TabId::Preset, 280.0, 400.0),
            tiroir(TabId::Domains, 320.0, 510.0),
        ],
    ]
}

/// **TIROIR-5 — au pire cas, les colonnes tiennent dans l'écran** : leur largeur totale ne
/// dépasse pas l'écran, aucune colonne n'est plus haute que la place, et chaque tiroir est rangé
/// une fois. Preuve à l'envers : le « first fit » seul, celui de TIROIR-4, débordait la largeur.
#[test]
fn test_tiroir_5_au_pire_cas_les_colonnes_tiennent_dans_l_ecran() {
    let (largeur, place) = (1440.0, 822.0);
    let seul: f32 = le_pire_cas()
        .iter()
        .flat_map(|c| premier_qui_tient(c, place))
        .map(|col| largeur_de(&col))
        .sum();
    assert!(seul > largeur, "le cas : {seul} de colonnes pour {largeur}");
    let cotes = ranger(le_pire_cas(), largeur, place);
    let prise: f32 = cotes.iter().flatten().map(|c| c.largeur).sum();
    assert!(prise <= largeur, "{prise} de colonnes pour {largeur}");
    for colonne in cotes.iter().flatten() {
        let haut: f32 = colonne.tiroirs.iter().map(|(_, h)| h).sum();
        assert!(haut <= place + 1e-3, "une colonne de {haut} pour {place}");
        for (t, h) in &colonne.tiroirs {
            assert!(*h <= t.naturelle && *h > 0.0, "{:?} : {h}", t.tab);
        }
    }
    let ranges: usize = cotes.iter().flatten().map(|c| c.tiroirs.len()).sum();
    assert_eq!(ranges, 7, "chaque tiroir, une fois");
    // La colonne fondue va dans la moins pleine de son côté : Domaines sous Preset (400 de
    // haut), pas sous Plugins (610).
    let avec_domaines = cotes[1]
        .iter()
        .find(|c| c.tiroirs.iter().any(|(t, _)| t.tab == TabId::Domains))
        .expect("Domaines rangé");
    let voisins: Vec<TabId> = avec_domaines.tiroirs.iter().map(|(t, _)| t.tab).collect();
    assert_eq!(voisins, vec![TabId::Preset, TabId::Domains]);
}

/// **Un écran trop étroit pour une seule colonne par côté** : rien de plus ne se peut — le
/// rangement s'arrête sans perdre de tiroir.
#[test]
fn test_tiroir_5_trop_etroit_aucun_tiroir_ne_se_perd() {
    let cotes = ranger(le_pire_cas(), 400.0, 300.0);
    assert_eq!((cotes[0].len(), cotes[1].len()), (1, 1));
    let ranges: usize = cotes.iter().flatten().map(|c| c.tiroirs.len()).sum();
    assert_eq!(ranges, 7);
}

/// Ce qu'un tiroir seul montre, rangé sur un écran assez grand pour ne jamais le rétrécir : la
/// boîte du rangement, à cette échelle.
fn seul_et_entier(dock: &DockManager, store: &Store, s: f32) -> PanelLayoutBox {
    let b = compute_panel_layouts(dock, store, 4000.0, 8000.0, 78.0, s).remove(0);
    assert_eq!(b.height, b.naturelle, "le cas : il n'est pas rétréci");
    b
}

/// `n` jalons nommés, du plus ancien au plus récent.
fn jalons(n: usize) -> Vec<crate::dock::temps::JalonVu> {
    (0..n)
        .map(|k| crate::dock::temps::JalonVu {
            apres: k,
            libelle: format!("jalon {k}"),
            nomme: true,
            instant: 0,
            date: None,
        })
        .collect()
}

/// Un document à `n` domaines.
fn avec_des_domaines(n: usize) -> Store {
    let mut store = Store::new("essai");
    for k in 0..n {
        let domaine = glucose_core::types::Domain {
            id: format!("d-{k}"),
            name: format!("domaine {k}"),
            color: "#888888".into(),
            icon: "◆".into(),
            defini_a: 0,
        };
        store.try_add_domain(domaine).expect("des noms distincts");
    }
    store
}

/// **TIROIR-2 — la hauteur naturelle est exacte** : mesurée dans un cadre de hauteur nulle, elle
/// est celle qu'on lit dans un cadre de mille pixels, au millième de pixel près — la mise en page
/// est affine en la hauteur du cadre. Mesurée dans un cadre d'un million de pixels puis soustraite,
/// elle s'en écartait jusqu'à un dixième de pixel (fiche 127).
#[test]
fn test_tiroir_2_la_hauteur_naturelle_est_exacte() {
    use crate::dock::{domains, temps};
    let store = avec_des_domaines(3);
    let ui_temps = temps::TempsUi {
        jalons: jalons(3),
        ..Default::default()
    };
    let ui_domaines = domains::DomainsUi::default();
    let mut echecs = Vec::new();
    for pas in 10..=60 {
        let s = pas as f32 * 0.05;
        let cadre = crate::params::ScaledRect {
            x: 0.0,
            y: 0.0,
            w: 324.0 * s,
            h: 1000.0,
            scale: s,
        };
        let (tm, dom) = (
            temps::hauteur_naturelle(&ui_temps, cadre.w, s),
            domains::hauteur_naturelle(&store, &ui_domaines, cadre.w, s),
        );
        let lu_tm = 1000.0 - temps::layout_temps_panel(cadre, &ui_temps).reste;
        let lu_dom = 1000.0 - domains::layout_domains_panel(cadre, &store, &ui_domaines).reste;
        for (quoi, mesure, lu) in [("Time Machine", tm, lu_tm), ("Domaines", dom, lu_dom)] {
            if (mesure - lu).abs() > 1e-3 {
                echecs.push(format!("{quoi} à {s} : {mesure} mesurée, {lu} lue"));
            }
        }
    }
    assert!(echecs.is_empty(), "{}", echecs.join("\n"));
}

/// **TIROIR-2 — à sa hauteur naturelle, un tiroir montre tout, à toute échelle** : la Time
/// Machine tous ses jalons, Domaines toutes ses rangées, le Storyboard toutes ses cases sur une
/// page — de 50 à 300 %, par pas de 5, à travers le vrai rangement. Regardé au pilote à 110 % (fiche
/// 127), la Time Machine avait perdu son second jalon : sa hauteur se mesurait dans un cadre d'un
/// million de pixels puis se soustrayait, et ses jalons se comptaient avec un écart de trop — un
/// rapport qui valait 1,9999.
#[test]
fn test_tiroir_2_a_sa_hauteur_naturelle_un_tiroir_montre_tout_a_toute_echelle() {
    let mut echecs = Vec::new();
    for pas in 10..=60 {
        let s = pas as f32 * 0.05;
        for n in [1_usize, 2, 3, 5, 8] {
            let mut dock = ouverts(&[TabId::Temps]);
            dock.temps.jalons = jalons(n);
            let b = seul_et_entier(&dock, &Store::new("essai"), s);
            let l = crate::dock::temps::layout_temps_panel(b.contenu(s), &dock.temps);
            if l.lignes.len() != n {
                echecs.push(format!(
                    "Time Machine à {s} : {} jalons sur {n}",
                    l.lignes.len()
                ));
            }
            let store = avec_des_domaines(n);
            let dock = ouverts(&[TabId::Domains]);
            let b = seul_et_entier(&dock, &store, s);
            let l = crate::dock::domains::layout_domains_panel(b.contenu(s), &store, &dock.domains);
            if l.hidden != 0 {
                echecs.push(format!(
                    "Domaines à {s} : {} rangées cachées sur {n}",
                    l.hidden
                ));
            }
            let mut dock = ouverts(&[TabId::Storyboard]);
            dock.storyboard.contenu.cases = (0..n)
                .map(|k| (format!("c-{k}"), k as i32, String::new()))
                .collect();
            let b = seul_et_entier(&dock, &Store::new("essai"), s);
            let l = crate::dock::storyboard::layout_storyboard_panel(
                b.contenu(s),
                &dock.storyboard,
                &dock.storyboard.contenu,
            );
            if l.lignes.len() != n || l.pages.is_some() {
                echecs.push(format!(
                    "Storyboard à {s} : {} cases sur {n}, pages {:?}",
                    l.lignes.len(),
                    l.pages.map(|p| p.3)
                ));
            }
        }
    }
    assert!(
        echecs.is_empty(),
        "{} échecs :\n{}",
        echecs.len(),
        echecs.join("\n")
    );
}

// ── TIROIR-6 : ce qu'un tiroir ne montre plus défile ─────────────────────────────

use crate::dock::{render_docks, DockCache, DockManager, DockPass};
use crate::params::{Pointer, ScreenFrame};
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::store::Store;

const TOUS: [TabId; 8] = [
    TabId::Organize,
    TabId::Pomodoro,
    TabId::Storyboard,
    TabId::Plugins,
    TabId::Preset,
    TabId::Domains,
    TabId::Temps,
    TabId::Discussion,
];

fn ecran(largeur: f32, hauteur: f32) -> ScreenFrame {
    ScreenFrame {
        width: largeur,
        height: hauteur,
        header_h: 78.0,
        scale: 1.0,
    }
}

fn ouverts(tabs: &[TabId]) -> DockManager {
    let mut dock = DockManager::new();
    for &tab in tabs {
        dock.toggle_tab(tab);
    }
    dock
}

fn dessin(dock: &DockManager, store: &Store, e: ScreenFrame) -> tiny_skia::Pixmap {
    dessin_avec(dock, store, e, Pointer { x: 0.0, y: 0.0 }, None)
}

/// Le dock dessiné, ce pointeur posé, par ce cache ou directement.
fn dessin_avec(
    dock: &DockManager,
    store: &Store,
    e: ScreenFrame,
    pointer: Pointer,
    cache: Option<&DockCache>,
) -> tiny_skia::Pixmap {
    let (theme, typo) = (Theme::dark(), Typography::new());
    let mut p = tiny_skia::Pixmap::new(e.width as u32, e.height as u32).expect("tampon");
    p.fill(theme.bg_canvas);
    let pass = DockPass {
        typo: &typo,
        theme: &theme,
        screen: e,
        pointer,
        cache,
    };
    render_docks(&mut p.as_mut(), dock, store, &pass);
    p
}

/// Ordonner seul, défilé de 64, sur un écran qui ne lui laisse que 300 pixels.
fn ordonner_defile() -> (DockManager, ScreenFrame) {
    let mut dock = ouverts(&[TabId::Organize]);
    dock.defilement.insert(TabId::Organize, 64.0);
    (dock, ecran(1440.0, 378.0))
}

/// **TIROIR-6 — le cache suit le défilement** : la clé d'un tampon porte le cadre entier du
/// panneau, défilement compris (DOCK-CACHE-1) — sans quoi, dans le vrai Glucose qui dessine par
/// le cache, la molette changerait le nombre et pas l'image. Rempli avant le défilement, le
/// cache rend ensuite l'image d'un rendu neuf, et elle a bougé : le cas est exercé.
#[test]
fn test_tiroir_6_le_cache_suit_le_defilement() {
    let store = Store::new("essai");
    let (defile, e) = ordonner_defile();
    let haut = ouverts(&[TabId::Organize]);
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let cache = DockCache::new();
    let avant = dessin_avec(&haut, &store, e, dehors, Some(&cache));
    let par_cache = dessin_avec(&defile, &store, e, dehors, Some(&cache));
    let neuf = dessin_avec(&defile, &store, e, dehors, None);
    assert!(
        avant.data() != neuf.data(),
        "le cas : le défilement change l'image"
    );
    assert!(
        par_cache.data() == neuf.data(),
        "le cache a gardé l'image d'avant le défilement"
    );
}

/// **TIROIR-6 et SURVOL-2 — un tiroir défilé se survole à travers le cache comme sans lui** : son
/// contenu visible se dessine par un pinceau à lui (`Brush::vers`), dont les questions de survol
/// reviennent au panneau — c'est par elles seules que le cache sait quand se refaire. Rempli le
/// pointeur dehors, puis le pointeur sur un bouton de la part visible : l'image du cache est celle
/// d'un rendu neuf. L'épreuve cherche elle-même un point où le survol change l'image : le cas est
/// exercé.
#[test]
fn test_tiroir_6_un_tiroir_defile_se_survole_a_travers_le_cache() {
    let store = Store::new("essai");
    let (dock, e) = ordonner_defile();
    let vu = crate::dock::places(&dock, &store, e)
        .remove(0)
        .contenu_visible(1.0);
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let sans = dessin_avec(&dock, &store, e, dehors, None);
    let dessus = (1..4)
        .flat_map(|i| {
            let x = vu.x + vu.w * i as f32 / 4.0;
            (0..(vu.h / 4.0) as u32).map(move |k| Pointer {
                x,
                y: vu.y + 4.0 * k as f32,
            })
        })
        .find(|&p| dessin_avec(&dock, &store, e, p, None).data() != sans.data())
        .expect("le cas : un bouton de la part visible change au survol");
    let cache = DockCache::new();
    dessin_avec(&dock, &store, e, dehors, Some(&cache));
    let par_cache = dessin_avec(&dock, &store, e, dessus, Some(&cache));
    let neuf = dessin_avec(&dock, &store, e, dessus, None);
    assert!(
        par_cache.data() == neuf.data(),
        "le survol du contenu défilé diverge à travers le cache ({}, {})",
        dessus.x,
        dessus.y
    );
}

/// Le plus grand écart, canal par canal, entre deux régions de même taille.
fn ecart(
    a: &tiny_skia::Pixmap,
    b: &tiny_skia::Pixmap,
    (xa, ya): (u32, u32),
    (xb, yb): (u32, u32),
    (l, h): (u32, u32),
) -> u8 {
    let mut pire = 0;
    for dy in 0..h {
        for dx in 0..l {
            let (p, q) = (
                a.pixel(xa + dx, ya + dy).expect("a"),
                b.pixel(xb + dx, yb + dy).expect("b"),
            );
            pire = pire
                .max(p.red().abs_diff(q.red()))
                .max(p.green().abs_diff(q.green()))
                .max(p.blue().abs_diff(q.blue()));
        }
    }
    pire
}

/// **TIROIR-6 — défilé de d, le contenu est le contenu entier remonté de d** : Ordonner (426 de
/// haut) sur un écran qui ne lui laisse que 300, défilé de 64, montre ce qu'il montre entier sur un
/// grand écran, 64 pixels plus bas. Le dessin et le clic lisent le même cadre (L4) : le clic tombe
/// donc là où le dessin le montre. Et sa poignée est celle du tiroir entier — le contenu n'y passe
/// pas (TIROIR-1). **Aucun niveau d'écart** : le tampon du contenu part du fond du tiroir et se
/// recopie à sa place — le même arrondi que le dessin direct. Comparé hors du filet du cadre, qui
/// reste au bord quand le contenu défile (la mesure d'abord prise sur lui disait 9 niveaux, fiche
/// 127).
#[test]
fn test_tiroir_6_defile_le_contenu_est_le_contenu_entier_remonte() {
    let store = Store::new("essai");
    let (court, haut) = (ecran(1440.0, 378.0), ecran(1440.0, 900.0));
    let mut dock = ouverts(&[TabId::Organize]);
    let b = crate::dock::places(&dock, &store, court).remove(0);
    assert!(
        b.naturelle > b.height,
        "le cas : il cache une part de son contenu"
    );
    dock.defilement.insert(TabId::Organize, 64.0);
    let b = crate::dock::places(&dock, &store, court).remove(0);
    assert_eq!(b.defile, 64.0);
    let entier = crate::dock::places(&ouverts(&[TabId::Organize]), &store, haut).remove(0);
    let (a, e) = (
        dessin(&dock, &store, court),
        dessin(&ouverts(&[TabId::Organize]), &store, haut),
    );
    let vu = b.contenu_visible(1.0);
    // Le filet du cadre excepté, tout autour : il reste au bord (et celui du bord intérieur dit
    // la part montrée).
    let (x0, l) = (vu.x as u32 + 2, vu.w as u32 - 4);
    let (y0, h) = (vu.y as u32 + 2, vu.h as u32 - 2);
    let contenu = ecart(&a, &e, (x0, y0), (x0, y0 + 64), (l, h));
    assert_eq!(
        contenu, 0,
        "le contenu défilé s'écarte de {contenu} niveaux"
    );
    let g = b.grip_height as u32;
    let poignee = ecart(
        &a,
        &e,
        (x0, b.grip_y as u32),
        (x0, entier.grip_y as u32),
        (l, g),
    );
    assert_eq!(poignee, 0, "la poignée d'un tiroir entier");
}

/// **TIROIR-5 et 6 — au pire cas, aucun tiroir ne se pose sur un autre** : tous ouverts sur un
/// écran de 1440 × 600 — aucune paire ne se croise, chacun reste dans l'écran, et au moins un cache
/// une part de son contenu : le cas est exercé.
#[test]
fn test_tiroir_6_au_pire_cas_aucun_tiroir_ne_se_pose_sur_un_autre() {
    let store = Store::new("essai");
    let e = ecran(1440.0, 600.0);
    let boites = crate::dock::places(&ouverts(&TOUS), &store, e);
    assert_eq!(boites.len(), TOUS.len(), "chaque tiroir ouvert est rangé");
    assert!(
        boites.iter().any(|b| b.naturelle > b.height),
        "le cas : un tiroir défile"
    );
    for (i, a) in boites.iter().enumerate() {
        assert!(
            a.x >= 0.0 && a.x + a.width <= e.width,
            "{:?} hors de l'écran",
            a.tab
        );
        assert!(
            a.y + a.height <= e.height + 1e-3,
            "{:?} sous l'écran",
            a.tab
        );
        for b in &boites[i + 1..] {
            let croise = a.x < b.x + b.width
                && b.x < a.x + a.width
                && a.y < b.y + b.height - 1e-3
                && b.y < a.y + a.height - 1e-3;
            assert!(!croise, "{:?} et {:?} se croisent", a.tab, b.tab);
        }
    }
}

/// **TIROIR-6 — le contenu défile sous le filet du haut** : à chaque défilement, de 4 en 4, la
/// rangée du filet qui borde le tiroir sous la bande est celle d'un tiroir qui ne défile pas — le
/// contenu passe dessous, il ne le recouvre jamais.
#[test]
fn test_tiroir_6_le_contenu_defile_sous_le_filet_du_haut() {
    let store = Store::new("essai");
    let e = ecran(1440.0, 378.0);
    let immobile = dessin(&ouverts(&[TabId::Organize]), &store, e);
    let b = crate::dock::places(&ouverts(&[TabId::Organize]), &store, e).remove(0);
    let (x0, l, y) = (b.x as u32 + 2, b.width as u32 - 4, b.y as u32);
    for d in (0..=(b.naturelle - b.height) as u32).step_by(4) {
        let mut dock = ouverts(&[TabId::Organize]);
        dock.defilement.insert(TabId::Organize, d as f32);
        let pire = ecart(
            &dessin(&dock, &store, e),
            &immobile,
            (x0, y),
            (x0, y),
            (l, 1),
        );
        assert_eq!(pire, 0, "défilé de {d}, le contenu passe sur le filet");
    }
}

/// **TIROIR-6 — la molette défile le tiroir qui cache quelque chose, et s'arrête au bout** : d'une
/// ligne de canevas par cran ; tout au bout, elle reste au tiroir. Sur un tiroir montré entier, elle
/// passe au canevas, comme avant. **Les deux cas se construisent** (fiche 127) : à 1440 × 900, sur
/// un document vide, aucun tiroir ne déborde — l'interface agrandie deux fois (ECHELLE-1), Ordonner
/// ne tient plus sous la bande ; le minuteur seul, à l'échelle 1, se montre entier.
#[test]
fn test_tiroir_6_la_molette_defile_et_s_arrete_au_bout() {
    use winit::event::MouseScrollDelta::LineDelta;
    let seul = |tab: TabId, echelle: f32| {
        let mut app = crate::app::GlucoseApp::new();
        app.ui.scale_factor = echelle;
        app.dock_manager.toggle_tab(tab);
        let (l, h) = app.taille_de_la_fenetre();
        let ecran = app.screen_frame(l, h);
        let b = crate::dock::places(&app.dock_manager, &app.store, ecran)
            .into_iter()
            .find(|b| b.tab == tab)
            .expect("ouvert");
        app.mouse_pos = (f64::from(b.x + b.width / 2.0), f64::from(b.y + 10.0));
        (app, b, crate::theme::clamp_ui_scale(ecran.scale))
    };
    let (mut app, b, s) = seul(TabId::Organize, 2.0);
    assert!(b.naturelle > b.height, "le cas : Ordonner cache une part");
    assert!(app.molette_sur_un_tiroir(LineDelta(0.0, -1.0)));
    assert_eq!(
        app.dock_manager.defile(b.tab),
        32.0 * s,
        "une ligne de canevas"
    );
    for _ in 0..1_000 {
        assert!(
            app.molette_sur_un_tiroir(LineDelta(0.0, -1.0)),
            "au bout, il la garde"
        );
    }
    assert_eq!(
        app.dock_manager.defile(b.tab),
        b.naturelle - b.height,
        "arrêtée au bout"
    );
    let (mut app, entier, _) = seul(TabId::Pomodoro, 1.0);
    assert!(entier.naturelle <= entier.height, "le cas : montré entier");
    assert!(
        !app.molette_sur_un_tiroir(LineDelta(0.0, -1.0)),
        "entier, il la laisse"
    );
}
