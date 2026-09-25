//! **Le focus sur un tableau entier** : ce que l'application demande, image après image.
//!
//! La décision et la visibilité du focus se définissent dans le module parent, sur des
//! éléments décrits un à un. Les décrire tous à chaque image coûtait 8 ms pour 10 000 nœuds :
//! ce module en tire ce que l'application lit — la carte des membranes, une fois par état du
//! document, puis la décision par image sur les seules membranes ; et le masque du focus, en
//! un passage.

use super::ScreenSize;
use super::{annotation_visible_under_focus, decider_sur, FocusAction, FocusState, MembraneVue};
use crate::geometry::Rect;
use crate::types::{Annotation, Viewport};
use std::collections::{HashMap, HashSet};

/// **Ce que la décision du focus retient d'un tableau** : chaque membrane, la boîte qu'on voit
/// d'elle et celle de son contenu.
///
/// # Pourquoi elle existe
///
/// Décider à chaque image en redécrivant tout le tableau coûtait **8 ms pour 10 000 nœuds, et
/// 165 ms pour 100 000** — mesuré : chaque image d'un zoom l'aurait payé. Or seule la vue
/// change pendant un zoom. La carte se calcule donc une fois par état du document, en un seul
/// passage qui ne recopie aucun identifiant de nœud ; la décision, à chaque image, ne parcourt
/// plus que les membranes ([`decider`]).
///
/// La géométrie affichée ne diffère des coordonnées que sous une membrane minimisée : c'est
/// seulement là que le repère complet des membranes est calculé.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CarteDesMembranes {
    membranes: Vec<MembraneVue>,
}

impl CarteDesMembranes {
    pub fn du_tableau(board: &crate::types::Board) -> Self {
        let mut membranes: Vec<MembraneVue> = Vec::new();
        let mut rang: HashMap<&str, usize> = HashMap::new();
        let mut minimisee = false;
        for a in &board.annotations {
            if let Annotation::Membrane { mode, .. } = a {
                minimisee |= *mode == crate::types::MembraneMode::Minimized;
                if let Some(r) = a.rect() {
                    rang.insert(a.id(), membranes.len());
                    membranes.push(MembraneVue {
                        id: a.id().to_string(),
                        vue: Some(r),
                        cadre: r,
                    });
                }
            }
        }
        if membranes.is_empty() {
            return Self::default();
        }
        // L'étendue du contenu direct de chaque membrane, depuis son coin : ce que
        // `focus_box` calcule, en un seul passage.
        let mut etendue = vec![(0.0f64, 0.0f64); membranes.len()];
        let mut etendre = |parent: Option<&str>, soi: &str, r: Rect| {
            let Some(&k) = parent.filter(|p| *p != soi).and_then(|p| rang.get(p)) else {
                return;
            };
            let m = membranes[k].cadre;
            etendue[k].0 = etendue[k].0.max(r.right() - m.left);
            etendue[k].1 = etendue[k].1.max(r.bottom() - m.top);
        };
        for img in &board.images {
            etendre(img.membrane_id.as_deref(), &img.id, img.rect());
        }
        for a in &board.annotations {
            if let (false, Some(r)) = (matches!(a, Annotation::Arrow { .. }), a.rect()) {
                etendre(a.membrane_id(), a.id(), r);
            }
        }
        for (m, (w, h)) in membranes.iter_mut().zip(etendue) {
            m.cadre.width = m.cadre.width.max(w);
            m.cadre.height = m.cadre.height.max(h);
        }
        if minimisee {
            let items = crate::membrane_space::items_of_board(board);
            let vus = crate::membrane_space::resolve_items(&items, Default::default());
            for m in &mut membranes {
                m.vue = vus.get(&m.id).map(|r| r.rect());
            }
        }
        Self { membranes }
    }

    pub fn est_vide(&self) -> bool {
        self.membranes.is_empty()
    }
}

/// La décision du focus pour cette vue, sur la carte d'un tableau.
pub fn decider(
    carte: &CarteDesMembranes,
    vp: Viewport,
    screen: ScreenSize,
    state: FocusState,
    now: i64,
) -> FocusAction {
    if carte.est_vide() && state.membrane_id.is_none() {
        return FocusAction::Stay(state);
    }
    decider_sur(&carte.membranes, vp, screen, state, now)
}

/// **Ce que le focus laisse voir d'un tableau** — calculé une fois par état du document, lu
/// par le rendu et par le clic.
#[derive(Debug, Clone, PartialEq)]
pub struct MasqueFocus {
    /// Par rang de présentation — les images, puis les annotations, puis les dossiers, dans
    /// l'ordre de l'index spatial : ce nœud se voit-il ?
    pub permis: Vec<bool>,
    /// Les mêmes, par identifiant.
    pub ids: HashSet<String>,
    /// La couleur de la membrane focalisée : elle teinte le fond de la scène.
    pub couleur: Option<String>,
}

impl MasqueFocus {
    pub fn laisse_voir(&self, id: &str) -> bool {
        self.ids.contains(id)
    }
}

/// Le masque du focus sur cette membrane, ou `None` si elle n'est pas sur ce tableau. Les
/// dossiers ne se voient pas en focus : ils mènent ailleurs.
///
/// # Un passage, et les chaînes réglées sur les seules membranes
///
/// Décrire tout le tableau pour en déduire le masque coûtait **7 ms pour 10 000 nœuds et 78
/// pour 100 000** — à chaque modification faite en focus. Seule une membrane peut être parente :
/// tout ce qui concerne les chaînes d'appartenance — une boucle, être sous la membrane
/// focalisée — se règle donc une fois sur les membranes, et chaque nœud ne coûte plus qu'une
/// lecture. Les règles restent exactement celles de [`visible_under_focus`] et
/// [`focus_frame_of`] : la membrane et ses descendants, ce qui est posé dans sa boîte sans
/// appartenir à personne, et une appartenance qui mène à une boucle ne compte pas. L'épreuve
/// `test_le_masque_rend_ce_que_rendait_la_description_complete` les confronte.
pub fn masque_du_focus(board: &crate::types::Board, membrane_id: &str) -> Option<MasqueFocus> {
    let membrane = board
        .annotations
        .iter()
        .find(|a| a.id() == membrane_id && matches!(a, Annotation::Membrane { .. }))?;
    let m_rect = membrane.rect()?;
    let couleur = match membrane {
        Annotation::Membrane { color, .. } => color.clone(),
        _ => None,
    };
    let chaines = Chaines::des_membranes(board, membrane_id);
    // L'appartenance qui compte : une membrane (pas soi-même), par une chaîne sans boucle.
    let parent = |id: &str, p: Option<&str>| -> Option<usize> {
        let k = *chaines.rang.get(p?)?;
        let depart = chaines.rang.get(id).copied().unwrap_or(k);
        (p != Some(id) && chaines.saine[depart]).then_some(k)
    };
    let focus = chaines.rang[membrane_id];
    let (mut w, mut h) = (0.0f64, 0.0f64);
    let mut vus: HashSet<String> = HashSet::new();
    let mut juger = |id: &str, r: Rect, p: Option<&str>| -> bool {
        // La membrane focalisée se voit toujours, quelle que soit sa propre parente.
        let voit = id == membrane_id
            || match parent(id, p) {
                Some(k) => {
                    if k == focus {
                        w = w.max(r.right() - m_rect.left);
                        h = h.max(r.bottom() - m_rect.top);
                    }
                    chaines.sous_le_focus[k]
                }
                None => m_rect.contains_center(r),
            };
        if voit {
            vus.insert(id.to_string());
        }
        voit
    };
    let total = board.images.len() + board.annotations.len() + board.folders.len();
    let mut permis = Vec::with_capacity(total);
    for img in &board.images {
        permis.push(juger(&img.id, img.rect(), img.membrane_id.as_deref()));
    }
    for a in &board.annotations {
        permis.push(match a.rect() {
            Some(r) => juger(a.id(), r, a.membrane_id()),
            None => false,
        });
    }
    let cadre = Rect {
        width: m_rect.width.max(w),
        height: m_rect.height.max(h),
        ..m_rect
    };
    // Les annotations se jugent enfin par la règle du focus : une carte mesurée se voit si
    // elle est visible, une flèche si ses deux bouts le sont.
    let debut = board.images.len();
    for (k, a) in board.annotations.iter().enumerate() {
        permis[debut + k] = annotation_visible_under_focus(a, Some(&vus), Some(cadre));
    }
    permis.resize(total, false);
    let ids = board
        .images
        .iter()
        .map(|i| i.id.as_str())
        .chain(board.annotations.iter().map(Annotation::id))
        .zip(&permis)
        .filter(|(_, voit)| **voit)
        .map(|(id, _)| id.to_string())
        .collect();
    Some(MasqueFocus {
        permis,
        ids,
        couleur,
    })
}

/// Les chaînes d'appartenance des membranes d'un tableau, réglées une fois : laquelle mène
/// à une boucle, laquelle est sous la membrane focalisée.
struct Chaines<'a> {
    rang: HashMap<&'a str, usize>,
    /// La chaîne qui part de cette membrane ne boucle pas.
    saine: Vec<bool>,
    /// Cette membrane est la focalisée, ou lui appartient par une chaîne saine.
    sous_le_focus: Vec<bool>,
}

impl<'a> Chaines<'a> {
    fn des_membranes(board: &'a crate::types::Board, focus: &str) -> Self {
        let membranes: Vec<&Annotation> = board
            .annotations
            .iter()
            .filter(|a| matches!(a, Annotation::Membrane { .. }) && a.rect().is_some())
            .collect();
        let rang: HashMap<&str, usize> = membranes
            .iter()
            .enumerate()
            .map(|(k, a)| (a.id(), k))
            .collect();
        let brut: Vec<Option<usize>> = membranes
            .iter()
            .map(|a| {
                a.membrane_id()
                    .filter(|p| *p != a.id())
                    .and_then(|p| rang.get(p).copied())
            })
            .collect();
        let n = membranes.len();
        // Une chaîne plus longue que le nombre de membranes repasse forcément par l'une d'elles.
        let saine: Vec<bool> = (0..n)
            .map(|depart| {
                let mut cur = Some(depart);
                for _ in 0..=n {
                    match cur {
                        None => return true,
                        Some(c) => cur = brut[c],
                    }
                }
                false
            })
            .collect();
        let focus = rang.get(focus).copied();
        let sous_le_focus = (0..n)
            .map(|depart| {
                if !saine[depart] {
                    return Some(depart) == focus;
                }
                // Bornée elle aussi : une chaîne jugée saine à tort rendrait une réponse fausse,
                // que les épreuves verraient — jamais une application figée.
                let mut cur = Some(depart);
                for _ in 0..=n {
                    match cur {
                        Some(c) if Some(c) == focus => return true,
                        Some(c) => cur = brut[c],
                        None => return false,
                    }
                }
                false
            })
            .collect();
        Self {
            rang,
            saine,
            sous_le_focus,
        }
    }
}
