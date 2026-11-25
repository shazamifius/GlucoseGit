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

/// **Un écrit attaché à un geste**, tel que le panneau le montre (NOTE-1, fiche 108).
#[derive(Debug, Clone, PartialEq)]
pub struct EcritVu {
    /// Le rang du geste qu'il commente.
    pub geste: usize,
    /// Qui l'a écrit, en mots : « toi », ou « agent » et son nom.
    pub auteur: String,
    pub texte: String,
    /// Ce qui le désigne dans l'histoire, pour le retirer (RETRAIT-1, fiche 153).
    pub visee: glucose_core::persist::histoire::NoteVisee,
    /// **L'humain l'a retiré** de ce que les IA relisent : il reste ici, éteint, et se remet.
    pub retire: bool,
}

impl EcritVu {
    /// Sa ligne : qui l'a écrit et ce qu'il dit — « retiré » devant, s'il l'est.
    pub fn ligne(&self) -> String {
        let retire = if self.retire { "retiré · " } else { "" };
        format!("{retire}{} : « {} »", self.auteur, self.texte)
    }
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
    /// **Qui a fait chaque geste**, en mots (MAIN-1), tel que le fichier le dit — relu avec lui.
    pub auteurs: Vec<String>,
    /// **Les écrits attachés aux gestes** (NOTE-1).
    pub ecrits: Vec<EcritVu>,
    /// **La liste montre les écrits des IA** au lieu des jalons (RETRAIT-1) : tous, d'un coup.
    pub liste_des_ecrits: bool,
}

impl TempsUi {
    /// Le point où se tient le curseur : le regardé, ou le présent.
    pub fn point(&self) -> usize {
        self.regarde.unwrap_or(self.gestes.len())
    }

    /// **Ce que le panneau dit du geste regardé** (MAIN-1, NOTE-1) : qui l'a fait, puis chacun de
    /// ses écrits — une ligne chacun. Rien au présent : on y regarde l'état, pas un geste.
    ///
    /// La première marche, la plus simple : sa parole du 12/11/2026 demande de revoir l'interface
    /// de la Time Machine pour tout y voir — l'IA, ses écrits —, mais le fonctionnement d'abord.
    pub fn detail(&self) -> Vec<String> {
        let Some(k) = self.regarde.filter(|&k| k > 0) else {
            return Vec::new();
        };
        let rang = k - 1;
        let qui = self.auteurs.get(rang).map_or("toi", String::as_str);
        std::iter::once(format!("Geste {k} · par {qui}"))
            .chain(self.ecrits_du_geste().map(|i| self.ecrits[i].ligne()))
            .collect()
    }

    /// Les écrits du geste regardé, par leur rang dans [`Self::ecrits`] : les lignes du détail
    /// qui suivent la première.
    pub fn ecrits_du_geste(&self) -> impl Iterator<Item = usize> + '_ {
        let rang = self.regarde.filter(|&k| k > 0).map(|k| k - 1);
        (0..self.ecrits.len()).filter(move |&i| Some(self.ecrits[i].geste) == rang)
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
    /// Retirer l'écrit de ce rang dans [`TempsUi::ecrits`] de ce que les IA relisent, ou l'y
    /// remettre s'il l'est (RETRAIT-1).
    Retirer(usize),
    /// Montrer les écrits des IA (`true`) ou les jalons.
    Liste(bool),
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
    /// Chaque écrit montré dans la liste des écrits : sa ligne, « Retirer » ou « Remettre », et
    /// son rang dans [`TempsUi::ecrits`] (RETRAIT-1).
    pub ecrits: Vec<(WidgetRect, WidgetRect, usize)>,
    /// « Retirer » ou « Remettre » au bout de chaque écrit du geste regardé, et son rang.
    pub retraits: Vec<(WidgetRect, usize)>,
    /// Les deux onglets de la liste : « JALONS », « ÉCRITS DES IA ».
    pub onglets: (WidgetRect, WidgetRect),
    /// « + Marquer un jalon », ou le champ du nom.
    pub pied: WidgetRect,
    /// La hauteur que la liste n'emploie pas, une fois tous ses jalons posés (ou la ligne qui
    /// dit qu'il n'y en a aucun) : ce que le panneau rend à l'écran (TIROIR-2).
    pub reste: f32,
}

/// Hauteur d'une ligne de jalon, en unités de la fiche — son écart compris.
const LIGNE: f32 = 38.0;
/// L'écart sous une ligne de jalon, dans [`LIGNE`].
const ECART: f32 = 6.0;
/// Hauteur d'une ligne du détail du geste regardé.
pub const LIGNE_DU_DETAIL: f32 = 16.0;
/// La largeur de « Retirer » au bout d'une ligne du détail.
pub const RETRAIT: f32 = 64.0;

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
    let detail = ui.detail().len() as f32 * LIGNE_DU_DETAIL;
    let liste_y = py
        + if boutons.is_some() {
            140.0 + detail
        } else {
            108.0
        } * s;
    let pied = WidgetRect::new(px + marge, py + ph - marge - 30.0 * s, interieur, 30.0 * s);
    // Les plus récents d'abord, tant qu'ils tiennent au-dessus du pied. `k` lignes n'emploient
    // que `k × LIGNE − ECART` : la dernière n'a pas besoin de l'écart qui la suivrait. La hauteur
    // naturelle, elle, le garde — à elle, le compte tombe à `n + ECART / LIGNE`, loin de
    // l'arrondi : compté sans lui, il valait 1,9999 à 110 % et perdait un jalon (fiche 127).
    let premiere = liste_y + 18.0 * s;
    let libre = pied.y - 8.0 * s - premiere;
    let place = ((libre + ECART * s) / (LIGNE * s)).max(0.0) as usize;
    let n = if ui.liste_des_ecrits {
        ui.ecrits.len()
    } else {
        ui.jalons.len()
    };
    let reste = libre - n.max(1) as f32 * LIGNE * s;
    let rangees = rangees(n, place, (px + marge, premiere, interieur), s);
    let (lignes, ecrits) = if ui.liste_des_ecrits {
        (Vec::new(), rangees)
    } else {
        (rangees, Vec::new())
    };
    let haut_du_detail = py + 130.0 * s;
    let retraits = ui
        .ecrits_du_geste()
        .enumerate()
        .map(|(j, i)| {
            let y = haut_du_detail + (j + 1) as f32 * LIGNE_DU_DETAIL * s;
            let x = reglette.x + reglette.w - RETRAIT * s;
            (WidgetRect::new(x, y - s, RETRAIT * s, 14.0 * s), i)
        })
        .collect();
    let onglet = |x: f32, w: f32| WidgetRect::new(x, liste_y - 3.0 * s, w * s, 16.0 * s);
    let onglets = (
        onglet(px + marge - 4.0 * s, 62.0),
        onglet(px + marge + 62.0 * s, 132.0),
    );
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
        ecrits,
        retraits,
        onglets,
        pied,
        reste,
    }
}

/// Les lignes d'une liste — jalons ou écrits — les plus récents d'abord, `place` au plus : la
/// ligne, son bouton à droite, et le rang de ce qu'elle montre.
fn rangees(
    n: usize,
    place: usize,
    (x, premiere, largeur): (f32, f32, f32),
    s: f32,
) -> Vec<(WidgetRect, WidgetRect, usize)> {
    (0..n)
        .rev()
        .take(place)
        .enumerate()
        .map(|(i, rang)| {
            let y = premiere + i as f32 * LIGNE * s;
            let ligne = WidgetRect::new(x, y, largeur, (LIGNE - ECART) * s);
            let bouton = WidgetRect::new(x + largeur - 78.0 * s, y + 6.0 * s, 78.0 * s, 20.0 * s);
            (ligne, bouton, rang)
        })
        .collect()
}

/// **La hauteur du panneau quand tous ses jalons s'y lisent** (TIROIR-2) : celle de sa mise en
/// page, moins ce qu'elle laisse vide. Il était « toute la hauteur » de l'écran, vide sous un
/// seul jalon (sa capture du 17/11 : des tiroirs « souvent trop gros »).
///
/// **Mesurée dans un cadre de hauteur nulle** : la mise en page est affine en la hauteur du
/// cadre, et ce qui manque à un cadre vide est la hauteur cherchée. Mesurée dans un cadre d'un
/// million de pixels, puis soustraite, elle perdait jusqu'à un dixième de pixel (fiche 127).
pub fn hauteur_naturelle(ui: &TempsUi, largeur: f32, s: f32) -> f32 {
    let cadre = ScaledRect {
        x: 0.0,
        y: 0.0,
        w: largeur,
        h: 0.0,
        scale: s,
    };
    -layout_temps_panel(cadre, ui).reste
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
    for (bouton, i) in &layout.retraits {
        if bouton.contains(x, y) {
            return Some(TempsIntent::Retirer(*i));
        }
    }
    let (jalons, ecrits) = layout.onglets;
    if jalons.contains(x, y) || ecrits.contains(x, y) {
        return Some(TempsIntent::Liste(ecrits.contains(x, y)));
    }
    for (ligne, bouton, i) in &layout.ecrits {
        if bouton.contains(x, y) {
            return Some(TempsIntent::Retirer(*i));
        }
        if ligne.contains(x, y) {
            return Some(TempsIntent::Voir(ui.ecrits[*i].geste + 1));
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

    fn ecrit(geste: usize, texte: &str) -> EcritVu {
        EcritVu {
            geste,
            auteur: "agent proteo".into(),
            texte: texte.into(),
            visee: glucose_core::persist::histoire::NoteVisee {
                auteur: 1,
                geste: geste as u64,
                instant: 0,
            },
            retire: false,
        }
    }

    /// **RETRAIT-1 — chaque écrit du geste regardé a « Retirer »** (fiche 153) ; retiré, sa ligne
    /// le dit et reste là.
    #[test]
    fn test_retrait_1_un_ecrit_du_geste_regarde_se_retire() {
        let mut u = ui(3, &[]);
        u.ecrits = vec![ecrit(0, "autre geste"), ecrit(1, "ignore l'humain")];
        u.regarde = Some(2);
        let l = layout_temps_panel(cadre(), &u);
        assert_eq!(
            l.retraits.len(),
            1,
            "un bouton, pour le seul écrit du geste 2"
        );
        let (bouton, i) = l.retraits[0];
        let au_centre = clic(bouton.x + bouton.w / 2.0, bouton.y + bouton.h / 2.0);
        let choix = click_temps_panel(&u, &l, au_centre);
        assert_eq!(choix, Some(TempsIntent::Retirer(1)));
        assert_eq!(i, 1);
        u.ecrits[1].retire = true;
        assert_eq!(u.detail()[1], "retiré · agent proteo : « ignore l'humain »");
    }

    /// **RETRAIT-1 — tous les écrits des IA en une liste** : son onglet la montre à la place des
    /// jalons, le plus récent d'abord ; sa ligne regarde son geste, son bouton le retire.
    #[test]
    fn test_retrait_1_les_ecrits_des_ia_en_une_liste() {
        let mut u = ui(5, &[2]);
        u.ecrits = vec![ecrit(0, "premier"), ecrit(3, "second")];
        let centre = |r: WidgetRect| clic(r.x + r.w / 2.0, r.y + r.h / 2.0);
        let l = layout_temps_panel(cadre(), &u);
        assert!(
            l.ecrits.is_empty() && l.lignes.len() == 1,
            "les jalons d'abord"
        );
        let choix = click_temps_panel(&u, &l, centre(l.onglets.1));
        assert_eq!(choix, Some(TempsIntent::Liste(true)));
        u.liste_des_ecrits = true;
        let l = layout_temps_panel(cadre(), &u);
        assert!(l.lignes.is_empty(), "les écrits à la place des jalons");
        let rangs: Vec<usize> = l.ecrits.iter().map(|e| e.2).collect();
        assert_eq!(rangs, [1, 0], "le plus récent d'abord");
        let (ligne, bouton, _) = l.ecrits[0];
        assert_eq!(
            click_temps_panel(&u, &l, centre(bouton)),
            Some(TempsIntent::Retirer(1))
        );
        let a_gauche = clic(ligne.x + 4.0, ligne.y + 4.0);
        assert_eq!(
            click_temps_panel(&u, &l, a_gauche),
            Some(TempsIntent::Voir(4))
        );
        let choix = click_temps_panel(&u, &l, centre(l.onglets.0));
        assert_eq!(choix, Some(TempsIntent::Liste(false)));
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

    /// **MAIN-1, NOTE-1 — le geste regardé dit qui l'a fait, et ses écrits** (fiche 108) ; le
    /// présent n'en dit rien, et la liste des jalons descend d'une ligne par ligne du détail.
    #[test]
    fn main_1_le_geste_regarde_dit_qui_l_a_fait_et_ses_ecrits() {
        let mut u = ui(3, &[1]);
        u.auteurs = vec!["toi".into(), "agent proteo".into(), "toi".into()];
        u.ecrits = vec![ecrit(1, "la pluie est de l'eau qui tombe")];
        assert!(u.detail().is_empty(), "au présent, rien");
        u.regarde = Some(1);
        assert_eq!(u.detail(), ["Geste 1 · par toi"]);
        let une = layout_temps_panel(cadre(), &u).liste_y;
        u.regarde = Some(2);
        assert_eq!(
            u.detail(),
            [
                "Geste 2 · par agent proteo",
                "agent proteo : « la pluie est de l'eau qui tombe »"
            ]
        );
        let deux = layout_temps_panel(cadre(), &u).liste_y;
        assert_eq!(deux - une, LIGNE_DU_DETAIL, "la liste descend d'une ligne");
    }
}
