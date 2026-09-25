//! **La fenêtre de l'éditeur, placée** — une fonction pure : le dessin et le clic lisent la
//! même mise en page (loi L4).
//!
//! Les mesures sont celles de `ArrowTextEditor.tsx`, en points, un peu plus grandes là où il a
//! trouvé l'interface de Glucose Rust trop petite : la fenêtre de 480, ses marges de 20 et 24,
//! la zone de texte de 300 au plus, les puces, les boutons.

use super::{ActionDAncrage, Ancrage, Etape};
use crate::renderer::math::MathRenderer;
use crate::typography::{Face, Typography};

/// Un rectangle `(x, y, largeur, hauteur)`, en pixels de l'écran.
pub type Rect4 = (f32, f32, f32, f32);

/// La largeur de la fenêtre.
const LARGEUR: f32 = 480.0;
/// Ses marges intérieures.
const MARGE_X: f32 = 24.0;
const MARGE_Y: f32 = 20.0;
/// Le rayon de ses coins.
pub(super) const RAYON: f32 = 10.0;
/// Le corps de l'entête, de la consigne et des puces.
pub(super) const CORPS: f32 = 12.0;
/// La hauteur d'une ligne d'entête ou de consigne.
const LIGNE: f32 = 18.0;
/// L'écart entre deux blocs de la fenêtre.
const ECART: f32 = 12.0;
/// La hauteur la plus grande de la zone de texte : au-delà, elle défile.
const ZONE_MAX: f32 = 300.0;
/// Le côté d'un numéro d'étape.
const ETAPE: f32 = 18.0;
/// L'encadré de ce qui est choisi : sa marge, la hauteur d'une puce, la marge d'une puce.
const CHOISI_MARGE: f32 = 10.0;
const PUCE_H: f32 = 24.0;
const PUCE_MARGE: f32 = 8.0;
/// Le côté de la croix qui retire une puce.
pub(super) const CROIX: f32 = 8.0;
/// Les boutons du bas : leur corps, leur hauteur, leur marge.
pub(super) const CORPS_BOUTON: f32 = 13.0;
const BOUTON_H: f32 = 32.0;
const BOUTON_MARGE: f32 = 14.0;
/// La longueur, en caractères, au-delà de laquelle une citation se coupe dans sa puce.
const CITATION_MAX: usize = 32;

/// La fenêtre, une fois placée.
#[derive(Debug, Clone, PartialEq)]
pub struct Fenetre {
    /// L'échelle de l'interface : combien de pixels font un point.
    pub s: f32,
    pub rect: Rect4,
    pub entete: Entete,
    pub zone: Zone,
    pub choisi: Option<Choisi>,
    pub boutons: Vec<BoutonDAncrage>,
}

/// **L'entête de la fenêtre, et sa consigne.**
#[derive(Debug, Clone, PartialEq)]
pub struct Entete {
    /// Le haut de la ligne d'entête.
    pub haut: f32,
    /// Où s'écrit le titre de l'étape — sa pastille se pose devant —, puis sa consigne.
    pub x_titre: f32,
    pub x_sous_titre: f32,
    /// Les numéros d'étape, s'il y en a deux : le libellé, la boîte, s'il est l'étape en cours.
    pub etapes: Vec<(&'static str, Rect4, bool)>,
    /// Le haut des deux lignes de la consigne ; la touche `Ctrl` se pose sur la seconde.
    pub consigne: (f32, f32),
    pub touche: Rect4,
}

/// **La zone où le texte de la carte se sélectionne.**
///
/// Le texte y est une carte virtuelle d'origine `(0, 0)`, large de `largeur_monde` unités, que
/// l'on regarde à l'échelle `s` — une unité, un point — et défilée de `defilement` unités.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zone {
    pub rect: Rect4,
    pub largeur_monde: f32,
    /// La hauteur du texte entier, marges de la carte comprises.
    pub hauteur_monde: f32,
    pub defilement: f32,
    pub s: f32,
}

impl Zone {
    /// Le défilement le plus grand : ce qui déborde de la zone, et rien de plus.
    pub fn defilement_max(&self) -> f32 {
        (self.hauteur_monde - self.rect.3 / self.s).max(0.0)
    }

    /// Le point de la carte virtuelle sous ce point de l'écran.
    pub fn vers_le_monde(&self, (px, py): (f32, f32)) -> (f32, f32) {
        (
            (px - self.rect.0) / self.s,
            (py - self.rect.1) / self.s + self.defilement,
        )
    }

    /// La vue qui pose la carte virtuelle dans une image de la taille de la zone.
    pub fn vue(&self) -> glucose_core::types::Viewport {
        glucose_core::types::Viewport {
            x: 0.0,
            y: -f64::from(self.defilement * self.s),
            scale: f64::from(self.s),
        }
    }
}

/// L'encadré de ce qui est choisi.
#[derive(Debug, Clone, PartialEq)]
pub struct Choisi {
    pub rect: Rect4,
    /// « Sélectionné (2) : », et le haut de sa ligne.
    pub titre: (String, f32),
    /// « Effacer », à droite de la même ligne.
    pub effacer: Rect4,
    pub puces: Vec<Puce>,
}

/// Un passage choisi : sa citation, et la croix qui le retire.
#[derive(Debug, Clone, PartialEq)]
pub struct Puce {
    pub rect: Rect4,
    pub texte: String,
    pub croix: Rect4,
    pub rang: usize,
}

/// Un bouton du bas : ce qu'il demande, ce qu'il écrit, où il est, et s'il est le principal —
/// celui que `Entrée` presse.
#[derive(Debug, Clone, PartialEq)]
pub struct BoutonDAncrage {
    pub action: ActionDAncrage,
    pub libelle: &'static str,
    pub rect: Rect4,
    pub principal: bool,
}

/// Le titre d'une étape.
pub fn titre(etape: Etape) -> &'static str {
    match etape {
        Etape::Source => "SOURCE",
        Etape::Cible => "CIBLE",
    }
}

/// La consigne de l'entête, et les deux lignes de la consigne — la touche `Ctrl` se pose entre
/// les deux morceaux de la seconde.
pub const SOUS_TITRE: &str = "— Sélectionnez le texte exact";
pub const CONSIGNE_1: &str = "Glissez sur le passage exact, ou cliquez sur un mot.";
pub const CONSIGNE_2: (&str, &str) = ("Maintenez", "pour en ajouter plusieurs.");
pub const TOUCHE: &str = "Ctrl";

/// **La fenêtre de l'édition en cours**, pour le texte `texte` de la carte de l'étape.
pub fn layout_ancrage(
    ancrage: &Ancrage,
    texte: &str,
    (typographie, math): (&Typography, &MathRenderer),
    ecran: (f32, f32),
    scale: f32,
) -> Fenetre {
    let s = crate::theme::clamp_ui_scale(scale);
    let largeur = LARGEUR.min(ecran.0 / s - 2.0 * MARGE_X);
    let interieur = largeur - 2.0 * MARGE_X;
    let hauteur_monde =
        crate::renderer::card::text_card_fit_height(typographie, math, texte, f64::from(interieur))
            as f32;
    let hauteur_zone = hauteur_monde.min(ZONE_MAX);
    let choisi = mesurer_le_choisi(ancrage, typographie, interieur, s);
    let hauteur = MARGE_Y * 2.0
        + LIGNE
        + ECART
        + LIGNE * 2.0
        + ECART
        + hauteur_zone
        + choisi.as_ref().map_or(0.0, |(_, h)| ECART + h)
        + ECART
        + BOUTON_H;
    let x = (ecran.0 - largeur * s) / 2.0;
    let y = ((ecran.1 - hauteur * s) / 2.0).max(0.0);
    let gauche = x + MARGE_X * s;
    let mut curseur = y + MARGE_Y * s;

    let entete = placer_l_entete(
        ancrage,
        typographie,
        (gauche, x + (largeur - MARGE_X) * s, curseur),
        s,
    );
    curseur += (LIGNE * 3.0 + ECART * 2.0) * s;
    let zone = Zone {
        rect: (gauche, curseur, interieur * s, hauteur_zone * s),
        largeur_monde: interieur,
        hauteur_monde,
        defilement: ancrage
            .defilement
            .clamp(0.0, (hauteur_monde - hauteur_zone).max(0.0)),
        s,
    };
    curseur += (hauteur_zone + ECART) * s;
    let choisi = choisi.map(|(c, h)| {
        let c = placer_le_choisi(c, (gauche, curseur), (interieur, h), s);
        curseur += (h + ECART) * s;
        c
    });
    let boutons = boutons(
        ancrage,
        typographie,
        (x + (largeur - MARGE_X) * s, curseur),
        s,
    );
    Fenetre {
        s,
        rect: (x, y, largeur * s, hauteur * s),
        entete,
        zone,
        choisi,
        boutons,
    }
}

/// L'entête et la consigne : la pastille de l'étape, son titre, sa consigne, les numéros
/// d'étape à droite ; puis les deux lignes de la consigne, la touche `Ctrl` entre les deux
/// morceaux de la seconde.
fn placer_l_entete(
    ancrage: &Ancrage,
    typographie: &Typography,
    (gauche, droite, haut): (f32, f32, f32),
    s: f32,
) -> Entete {
    let x_titre = gauche + 14.0 * s;
    let x_sous_titre = x_titre
        + typographie
            .measure_text(titre(ancrage.etape), CORPS * s, Face::Bold)
            .0
        + 8.0 * s;
    let etapes = numeros(ancrage, (droite, haut), s);
    let consigne = (haut + (LIGNE + ECART) * s, haut + (LIGNE * 2.0 + ECART) * s);
    let avant = typographie
        .measure_text(CONSIGNE_2.0, CORPS * s, Face::Regular)
        .0;
    let touche_l = typographie
        .measure_text(TOUCHE, (CORPS - 1.0) * s, Face::Mono)
        .0
        + 10.0 * s;
    let touche = (gauche + avant + 5.0 * s, consigne.1, touche_l, LIGNE * s);
    Entete {
        haut,
        x_titre,
        x_sous_titre,
        etapes,
        consigne,
        touche,
    }
}

/// Les numéros « 1 » et « 2 », alignés à droite de l'entête — seulement s'il y a deux étapes.
fn numeros(
    ancrage: &Ancrage,
    (droite, haut): (f32, f32),
    s: f32,
) -> Vec<(&'static str, Rect4, bool)> {
    if ancrage.etapes() < 2 {
        return Vec::new();
    }
    let cote = ETAPE * s;
    let premier = droite - cote * 2.0 - 4.0 * s;
    [("1", Etape::Source), ("2", Etape::Cible)]
        .into_iter()
        .enumerate()
        .map(|(i, (n, e))| {
            let rect = (premier + i as f32 * (cote + 4.0 * s), haut, cote, cote);
            (n, rect, ancrage.etape == e)
        })
        .collect()
}

/// Les puces de ce qui est choisi, mesurées mais pas encore placées, et la hauteur de
/// l'encadré — `None` s'il n'y a rien de choisi.
fn mesurer_le_choisi(
    ancrage: &Ancrage,
    typographie: &Typography,
    interieur: f32,
    s: f32,
) -> Option<(Vec<(String, f32)>, f32)> {
    if ancrage.ancres().is_empty() {
        return None;
    }
    let dispo = interieur - 2.0 * CHOISI_MARGE;
    let puces: Vec<(String, f32)> = ancrage
        .ancres()
        .iter()
        .map(|a| {
            let texte = citation(&a.quote);
            let l = typographie.measure_text(&texte, CORPS * s, Face::Italic).0 / s
                + PUCE_MARGE * 3.0
                + CROIX;
            (texte, l.min(dispo))
        })
        .collect();
    let mut lignes = 1;
    let mut x = 0.0;
    for (_, l) in &puces {
        if x > 0.0 && x + l > dispo {
            lignes += 1;
            x = 0.0;
        }
        x += l + 6.0;
    }
    let hauteur = CHOISI_MARGE * 2.0 + LIGNE + 6.0 + lignes as f32 * (PUCE_H + 6.0) - 6.0;
    Some((puces, hauteur))
}

/// L'encadré placé : son titre, « Effacer », et ses puces qui passent à la ligne.
fn placer_le_choisi(
    puces: Vec<(String, f32)>,
    (x, y): (f32, f32),
    (largeur, hauteur): (f32, f32),
    s: f32,
) -> Choisi {
    let (gauche, droite) = (x + CHOISI_MARGE * s, x + (largeur - CHOISI_MARGE) * s);
    let haut = y + CHOISI_MARGE * s;
    let effacer_l = 64.0 * s;
    let n = puces.len();
    let mut placees = Vec::with_capacity(n);
    let (mut cx, mut cy) = (gauche, haut + (LIGNE + 6.0) * s);
    for (rang, (texte, l)) in puces.into_iter().enumerate() {
        let w = l * s;
        if cx > gauche && cx + w > droite {
            cx = gauche;
            cy += (PUCE_H + 6.0) * s;
        }
        let rect = (cx, cy, w, PUCE_H * s);
        let croix = (
            cx + w - (PUCE_MARGE + CROIX) * s,
            cy + (PUCE_H - CROIX) / 2.0 * s,
            CROIX * s,
            CROIX * s,
        );
        placees.push(Puce {
            rect,
            texte,
            croix,
            rang,
        });
        cx += w + 6.0 * s;
    }
    let titre = if n > 1 {
        format!("Sélectionné ({n}) :")
    } else {
        "Sélectionné :".to_string()
    };
    Choisi {
        rect: (x, y, largeur * s, hauteur * s),
        titre: (titre, haut),
        effacer: (droite - effacer_l, haut, effacer_l, LIGNE * s),
        puces: placees,
    }
}

/// Une citation dans sa puce : entre guillemets, coupée au-delà de trente-deux caractères.
fn citation(quote: &str) -> String {
    if quote.chars().count() > CITATION_MAX {
        let debut: String = quote.chars().take(CITATION_MAX - 2).collect();
        format!("« {debut}… »")
    } else {
        format!("« {quote} »")
    }
}

/// Les boutons du bas, alignés à droite : « Annuler », puis le principal — celui qu'`Entrée`
/// presse, à droite, comme dans Tauri.
fn boutons(
    ancrage: &Ancrage,
    typographie: &Typography,
    (droite, haut): (f32, f32),
    s: f32,
) -> Vec<BoutonDAncrage> {
    let principal = if !ancrage.a_une_suite() {
        (ActionDAncrage::Terminer, "Terminer")
    } else if ancrage.ancres().is_empty() {
        (ActionDAncrage::Suivant, "Passer →")
    } else {
        (ActionDAncrage::Suivant, "Valider → Cible")
    };
    let largeur = |t: &str| {
        typographie
            .measure_text(t, CORPS_BOUTON * s, Face::Regular)
            .0
            + BOUTON_MARGE * 2.0 * s
    };
    let lp = largeur(principal.1);
    let la = largeur("Annuler");
    let xp = droite - lp;
    let xa = xp - 8.0 * s - la;
    vec![
        BoutonDAncrage {
            action: ActionDAncrage::Annuler,
            libelle: "Annuler",
            rect: (xa, haut, la, BOUTON_H * s),
            principal: false,
        },
        BoutonDAncrage {
            action: principal.0,
            libelle: principal.1,
            rect: (xp, haut, lp, BOUTON_H * s),
            principal: true,
        },
    ]
}

impl Fenetre {
    /// Ce qu'un clic en `(px, py)` demande à un bouton, à « Effacer », ou à la croix d'une
    /// puce — `None` ailleurs.
    pub fn action_sous(&self, px: f32, py: f32) -> Option<ActionDAncrage> {
        let dans = |r: Rect4| crate::ui::action_bar::dans(r, px, py);
        if let Some(b) = self.boutons.iter().find(|b| dans(b.rect)) {
            return Some(b.action);
        }
        let choisi = self.choisi.as_ref()?;
        if dans(choisi.effacer) {
            return Some(ActionDAncrage::Effacer);
        }
        choisi
            .puces
            .iter()
            .find(|p| dans(p.rect) && px >= p.croix.0 - PUCE_MARGE * self.s)
            .map(|p| ActionDAncrage::Retirer(p.rang))
    }

    /// Le point est-il dans la zone du texte ?
    pub fn dans_la_zone(&self, px: f32, py: f32) -> bool {
        crate::ui::action_bar::dans(self.zone.rect, px, py)
    }
}
