//! Le panneau TIME MACHINE : l'histoire du document, parcourue geste par geste (fiche 10
//! § 5.7, fiche 33 § 5.4).
//!
//! Il ne sait rien du disque. L'application lui donne ce qu'il montre — l'instant de chaque
//! geste, les jalons, le point regardé — et traduit ce qu'il demande ([`TempsIntent`]) en
//! aperçu, en retour au présent, en restauration ou en jalon (`interactions::temps`).
//!
//! Comme chez Glucose Tauri : une **réglette** des gestes, un **aperçu** du passé bordé
//! d'ambre, « Restaurer cet état », et la liste des **jalons**. La différence est dessous :
//! chaque point de la réglette se relit du fichier, depuis l'instantané le plus proche, et
//! restaurer est un geste comme un autre — annulable, écrit dans l'histoire, qui ne détruit
//! rien de ce qu'il défait.

use super::WidgetRect;
use crate::interactions::text_entry::TextEntry;
use crate::params::{Pointer, ScaledRect};

pub mod paint;

/// Un jalon tel que le panneau le montre.
#[derive(Debug, Clone, PartialEq)]
pub struct JalonVu {
    /// Après combien de gestes il a été posé : c'est le point qu'il désigne.
    pub apres: usize,
    pub libelle: String,
    /// Posé et nommé par l'utilisateur, ou par `Ctrl+S`.
    pub nomme: bool,
    pub instant: i64,
    /// Sa date à l'horloge de l'utilisateur, si le système la donne (registre de Tauri, 12).
    pub date: Option<crate::plateforme::heure::HeureLocale>,
}

/// Ce que la Time Machine montre, et ce qu'on y est en train de faire.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TempsUi {
    /// L'instant de chaque geste, dans l'ordre (millisecondes Unix).
    pub gestes: Vec<i64>,
    pub jalons: Vec<JalonVu>,
    /// Le point regardé en aperçu — l'état après ce nombre de gestes. `None` : le présent.
    pub regarde: Option<usize>,
    /// Le nom du jalon en cours de frappe.
    pub nom: Option<TextEntry>,
    /// L'instant de référence des « il y a… », posé quand le panneau se remplit.
    pub maintenant: i64,
    /// La réglette est tenue : tant que le bouton l'est, le passé suit le curseur.
    pub glisse: bool,
}

impl TempsUi {
    /// Le point où se tient le curseur : le regardé, ou le présent.
    pub fn point(&self) -> usize {
        self.regarde.unwrap_or(self.gestes.len())
    }

    /// Des gestes viennent de s'écrire : la réglette s'allonge sans relire le fichier.
    pub fn noter(&mut self, nombre: usize, instant: i64) {
        self.gestes.extend(std::iter::repeat_n(instant, nombre));
        self.maintenant = instant;
    }
}

/// Ce que l'utilisateur demande au panneau. Aucune variante ne touche au document : elles
/// nomment un geste, l'application le fait (standard § 1.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TempsIntent {
    /// Le point de la réglette sous le curseur — appuyé, puis glissé : un point du passé, ou
    /// le présent quand c'est le dernier. La réglette reste tenue jusqu'au relâchement.
    Reglette(usize),
    /// Regarder l'état après ce nombre de gestes.
    Voir(usize),
    Maintenant,
    /// Faire de l'état regardé l'état présent.
    Restaurer,
    /// Restaurer l'état d'un jalon, par le nombre de gestes qui le précèdent.
    RestaurerLeJalon(usize),
    CommencerUnJalon,
}

/// La géométrie du panneau — la seule, lue par le dessin et par le clic.
#[derive(Debug, Clone, PartialEq)]
pub struct TempsLayout {
    pub badge: WidgetRect,
    pub reglette: WidgetRect,
    /// « ← Maintenant » et « Restaurer cet état », en aperçu seulement.
    pub boutons: Option<(WidgetRect, WidgetRect)>,
    /// Le haut de la liste des jalons.
    pub liste_y: f32,
    /// Chaque jalon montré : sa ligne, son bouton, et son rang dans [`TempsUi::jalons`].
    pub lignes: Vec<(WidgetRect, WidgetRect, usize)>,
    /// « + Marquer un jalon », ou le champ du nom.
    pub pied: WidgetRect,
}

/// Hauteur d'une ligne de jalon, en unités de la fiche.
const LIGNE: f32 = 38.0;

pub fn layout_temps_panel(frame: ScaledRect, ui: &TempsUi) -> TempsLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (px, py, pw, ph) = (frame.x, frame.y, frame.w, frame.h);
    let marge = 14.0 * s;
    let interieur = pw - 2.0 * marge;
    let reglette = WidgetRect::new(px + marge, py + 64.0 * s, interieur, 22.0 * s);
    let boutons = ui.regarde.map(|_| {
        let y = py + 96.0 * s;
        let gauche = WidgetRect::new(px + marge, y, 112.0 * s, 24.0 * s);
        let droite = WidgetRect::new(px + marge + 118.0 * s, y, interieur - 118.0 * s, 24.0 * s);
        (gauche, droite)
    });
    let liste_y = py + if boutons.is_some() { 140.0 } else { 108.0 } * s;
    let pied = WidgetRect::new(px + marge, py + ph - marge - 30.0 * s, interieur, 30.0 * s);
    // Les jalons les plus récents d'abord, tant qu'ils tiennent au-dessus du pied.
    let premiere = liste_y + 18.0 * s;
    let place = ((pied.y - 8.0 * s - premiere) / (LIGNE * s)).max(0.0) as usize;
    let lignes = (0..ui.jalons.len())
        .rev()
        .take(place)
        .enumerate()
        .map(|(i, rang)| {
            let y = premiere + i as f32 * LIGNE * s;
            let ligne = WidgetRect::new(px + marge, y, interieur, (LIGNE - 6.0) * s);
            let bouton = WidgetRect::new(
                ligne.x + ligne.w - 78.0 * s,
                y + 6.0 * s,
                78.0 * s,
                20.0 * s,
            );
            (ligne, bouton, rang)
        })
        .collect();
    TempsLayout {
        badge: WidgetRect::new(
            px + pw - marge - 72.0 * s,
            py + 22.0 * s,
            72.0 * s,
            16.0 * s,
        ),
        reglette,
        boutons,
        liste_y,
        lignes,
        pied,
    }
}

/// Le point de la réglette sous l'abscisse `x` : de 0 (avant tout geste) à `n` (le présent).
pub fn point_sous(reglette: WidgetRect, n: usize, x: f32) -> usize {
    if n == 0 || reglette.w <= 0.0 {
        return 0;
    }
    let t = ((x - reglette.x) / reglette.w).clamp(0.0, 1.0);
    (t * n as f32).round() as usize
}

pub fn click_temps_panel(
    ui: &TempsUi,
    layout: &TempsLayout,
    pointer: Pointer,
) -> Option<TempsIntent> {
    let (x, y) = (pointer.x, pointer.y);
    if layout.reglette.contains(x, y) {
        return Some(TempsIntent::Reglette(point_sous(
            layout.reglette,
            ui.gestes.len(),
            x,
        )));
    }
    if let Some((maintenant, restaurer)) = layout.boutons {
        if maintenant.contains(x, y) {
            return Some(TempsIntent::Maintenant);
        }
        if restaurer.contains(x, y) {
            return Some(TempsIntent::Restaurer);
        }
    }
    for (ligne, bouton, rang) in &layout.lignes {
        let apres = ui.jalons[*rang].apres;
        if bouton.contains(x, y) {
            return Some(TempsIntent::RestaurerLeJalon(apres));
        }
        if ligne.contains(x, y) {
            return Some(TempsIntent::Voir(apres));
        }
    }
    (ui.nom.is_none() && layout.pied.contains(x, y)).then_some(TempsIntent::CommencerUnJalon)
}

/// « il y a … » : l'écart entre deux instants, dit comme on le dit.
pub fn il_y_a(maintenant: i64, instant: i64) -> String {
    let s = (maintenant - instant).max(0) / 1000;
    match s {
        0..=4 => "à l'instant".to_string(),
        5..=59 => format!("il y a {s} s"),
        60..=3599 => format!("il y a {} min", s / 60),
        3600..=86_399 => format!("il y a {} h", s / 3600),
        _ => format!("il y a {} j", s / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cadre() -> ScaledRect {
        ScaledRect {
            x: 1000.0,
            y: 60.0,
            w: 324.0,
            h: 800.0,
            scale: 1.0,
        }
    }

    fn ui(n: usize, jalons: &[usize]) -> TempsUi {
        TempsUi {
            gestes: vec![0; n],
            jalons: jalons
                .iter()
                .map(|&apres| JalonVu {
                    apres,
                    libelle: format!("après {apres}"),
                    nomme: true,
                    instant: 0,
                    date: None,
                })
                .collect(),
            ..TempsUi::default()
        }
    }

    fn clic(x: f32, y: f32) -> Pointer {
        Pointer { x, y }
    }

    /// La réglette va du début (0) au présent (`n`) : ses deux bouts sont ces deux points.
    #[test]
    fn test_la_reglette_va_du_debut_au_present() {
        let u = ui(10, &[]);
        let l = layout_temps_panel(cadre(), &u);
        let r = l.reglette;
        let milieu = r.y + r.h / 2.0;
        assert_eq!(
            click_temps_panel(&u, &l, clic(r.x + 1.0, milieu)),
            Some(TempsIntent::Reglette(0))
        );
        assert_eq!(
            click_temps_panel(&u, &l, clic(r.x + r.w / 2.0, milieu)),
            Some(TempsIntent::Reglette(5))
        );
        assert_eq!(
            click_temps_panel(&u, &l, clic(r.x + r.w - 1.0, milieu)),
            Some(TempsIntent::Reglette(10))
        );
    }

    /// Les jalons se lisent du plus récent au plus ancien ; leur bouton restaure, leur ligne
    /// montre.
    #[test]
    fn test_un_jalon_se_montre_ou_se_restaure() {
        let u = ui(9, &[2, 7]);
        let l = layout_temps_panel(cadre(), &u);
        let (ligne, bouton, rang) = l.lignes[0];
        assert_eq!(u.jalons[rang].apres, 7, "le plus récent en haut");
        let centre = |r: WidgetRect| clic(r.x + r.w / 2.0, r.y + r.h / 2.0);
        assert_eq!(
            click_temps_panel(&u, &l, centre(bouton)),
            Some(TempsIntent::RestaurerLeJalon(7))
        );
        assert_eq!(
            click_temps_panel(&u, &l, clic(ligne.x + 4.0, ligne.y + 4.0)),
            Some(TempsIntent::Voir(7))
        );
    }

    /// Les boutons de l'aperçu n'existent qu'en aperçu, et le pied ne répond plus pendant
    /// qu'on nomme un jalon.
    #[test]
    fn test_les_boutons_suivent_l_etat() {
        let mut u = ui(4, &[]);
        assert!(layout_temps_panel(cadre(), &u).boutons.is_none());
        u.regarde = Some(1);
        let l = layout_temps_panel(cadre(), &u);
        let (m, r) = l.boutons.expect("en aperçu");
        assert_eq!(
            click_temps_panel(&u, &l, clic(m.x + 2.0, m.y + 2.0)),
            Some(TempsIntent::Maintenant)
        );
        assert_eq!(
            click_temps_panel(&u, &l, clic(r.x + 2.0, r.y + 2.0)),
            Some(TempsIntent::Restaurer)
        );
        let pied = clic(l.pied.x + 2.0, l.pied.y + 2.0);
        assert_eq!(
            click_temps_panel(&u, &l, pied),
            Some(TempsIntent::CommencerUnJalon)
        );
        u.nom = Some(TextEntry::default());
        assert_eq!(
            click_temps_panel(&u, &layout_temps_panel(cadre(), &u), pied),
            None
        );
    }

    #[test]
    fn test_il_y_a_se_dit_comme_on_le_dit() {
        assert_eq!(il_y_a(10_000, 9_000), "à l'instant");
        assert_eq!(il_y_a(100_000, 70_000), "il y a 30 s");
        assert_eq!(il_y_a(3_600_000, 0), "il y a 1 h");
        assert_eq!(
            il_y_a(0, 5_000),
            "à l'instant",
            "une horloge qui recule ne dit pas « dans »"
        );
    }

    /// **Un jalon dit sa date exacte** (registre de Tauri, 12), et la durée relative seulement
    /// quand le système ne donne pas l'heure locale.
    #[test]
    fn test_un_jalon_dit_sa_date_exacte() {
        let mut j = JalonVu {
            apres: 12,
            libelle: "avant la refonte".to_string(),
            nomme: true,
            instant: 1_000_000,
            date: Some(crate::plateforme::heure::HeureLocale {
                annee: 2026,
                mois: 9,
                jour: 25,
                heure: 22,
                minute: 19,
            }),
        };
        assert_eq!(
            paint::detail_du_jalon(&j, 1_000_000 + 3 * 3_600_000),
            "nommé · 25/09/2026 22:19 · geste 12"
        );
        j.date = None;
        assert_eq!(
            paint::detail_du_jalon(&j, 1_000_000 + 3 * 3_600_000),
            "nommé · il y a 3 h · geste 12"
        );
    }
}
