//! Instrumentation de performance optionnelle, activée par `GLUCOSE_PERF=1`.
//!
//! Aucune allocation ni aucun appel à `Instant::now()` n'est effectué lorsque la
//! variable d'environnement est absente : le coût est alors une simple lecture
//! d'un booléen mis en cache.

use std::cell::{Cell, RefCell};
use std::sync::OnceLock;
use std::time::Instant;

static LEVEL: OnceLock<u8> = OnceLock::new();

/// Niveau de trace : 0 = désactivé, 1 = une ligne par frame, 2 = chaque étape en direct.
fn level() -> u8 {
    *LEVEL.get_or_init(|| match std::env::var("GLUCOSE_PERF") {
        Ok(value) => {
            let v = value.trim();
            if v.is_empty() || v == "0" || v.eq_ignore_ascii_case("false") {
                0
            } else if v == "2" {
                2
            } else {
                1
            }
        }
        Err(_) => 0,
    })
}

/// Indique si la trace de performance est demandée (`GLUCOSE_PERF=1`).
pub fn enabled() -> bool {
    level() > 0
}

thread_local! {
    static FRAME_START: Cell<Option<Instant>> = const { Cell::new(None) };
    static LAST_MARK: Cell<Option<Instant>> = const { Cell::new(None) };
    static STAGES: RefCell<Vec<(&'static str, f64)>> = const { RefCell::new(Vec::new()) };
    static COMPTEURS: RefCell<Vec<(&'static str, f64)>> = const { RefCell::new(Vec::new()) };
    static FRAME_INDEX: Cell<u64> = const { Cell::new(0) };
}

/// Ouvre une nouvelle frame de mesure.
///
/// # Pourquoi la mesure a lieu meme sans `GLUCOSE_PERF`
///
/// La trace verbeuse est une aide de developpement ; la **chronique**, elle, enregistre en
/// permanence chez l'utilisateur, et c'est d'elle que viendra la connaissance du terrain.
/// Les deux lisent les memes postes, donc les postes se mesurent toujours.
///
/// Le cout est une quinzaine d'appels a `Instant::now()` par image, soit environ trois cents
/// nanosecondes : un huit-millieme du budget d'une image a 400 fps. La variable d'environnement
/// ne decide plus de MESURER, seulement d'AFFICHER.
pub fn frame_begin() {
    let now = Instant::now();
    if level() >= 2 {
        eprintln!("[perf] --- frame begin ---");
    }
    FRAME_START.with(|c| c.set(Some(now)));
    LAST_MARK.with(|c| c.set(Some(now)));
    STAGES.with(|s| s.borrow_mut().clear());
    COMPTEURS.with(|c| c.borrow_mut().clear());
}

/// Note une **quantité** de la frame, à côté de ses durées.
///
/// Une durée seule ne dit pas d'où elle vient : « le dessin des images coûte 429 ms » ne
/// distingue pas trente-six images chères d'une seule redessinée trente-six fois. Un compteur
/// tranche ce que le chronomètre laisse ambigu — combien de nœuds, combien de pixels, combien
/// de mégaoctets — et c'est la seule façon de ne pas avoir à deviner.
pub fn compteur(label: &'static str, valeur: f64) {
    COMPTEURS.with(|c| c.borrow_mut().push((label, valeur)));
}

/// Les postes de la frame en cours, tels que la chronique les lira.
pub fn postes() -> Vec<(&'static str, f64)> {
    STAGES.with(|s| s.borrow().clone())
}

/// La derniere valeur declaree sous ce nom pendant la frame en cours.
pub fn valeur_du_compteur(label: &str) -> Option<f64> {
    COMPTEURS.with(|c| {
        c.borrow()
            .iter()
            .rev()
            .find(|(nom, _)| *nom == label)
            .map(|(_, v)| *v)
    })
}

/// Enregistre la durée écoulée depuis le repère précédent sous le nom `label`.
///
/// # Un nom cumule, il n'écrase pas
///
/// Un poste mesuré plusieurs fois dans la même image — une fois par photo, par exemple — doit
/// rendre **la somme** de ses passages. La première version en empilait autant d'entrées que
/// d'appels, et la chronique, qui indexe par nom, ne gardait que la dernière : un poste
/// parcouru quatre-vingt-neuf fois se rapportait comme s'il l'avait été une seule.
///
/// Cumuler rend donc possible de mesurer **à l'intérieur** d'une boucle, ce qui est la seule
/// façon de savoir où va le temps d'une passe qui traite des dizaines d'objets.
pub fn stage(label: &'static str) {
    let now = Instant::now();
    let previous = LAST_MARK.with(|c| c.replace(Some(now)));
    if let Some(prev) = previous {
        let ms = now.duration_since(prev).as_secs_f64() * 1000.0;
        if level() >= 2 {
            eprintln!("[perf]   {label}={ms:.2}ms");
        }
        STAGES.with(|s| {
            let mut postes = s.borrow_mut();
            match postes.iter_mut().find(|(nom, _)| *nom == label) {
                Some((_, total)) => *total += ms,
                None => postes.push((label, ms)),
            }
        });
    }
}

/// Clôt la frame courante et écrit la ligne de trace sur stderr.
pub fn frame_end() {
    if !enabled() {
        // Les postes restent en place : la chronique les lit apres coup, et `frame_begin`
        // les videra a la prochaine image.
        return;
    }
    let total_ms = FRAME_START
        .with(|c| c.replace(None))
        .map(|start| start.elapsed().as_secs_f64() * 1000.0)
        .unwrap_or(0.0);
    LAST_MARK.with(|c| c.set(None));
    let index = FRAME_INDEX.with(|c| {
        let next = c.get().wrapping_add(1);
        c.set(next);
        next
    });
    let detail = STAGES.with(|s| {
        s.borrow()
            .iter()
            .map(|(label, ms)| format!("{label}={ms:.2}"))
            .collect::<Vec<_>>()
            .join(" ")
    });
    let compteurs = COMPTEURS.with(|c| {
        c.borrow()
            .iter()
            .map(|(label, valeur)| format!(" {label}={valeur:.1}"))
            .collect::<String>()
    });
    eprintln!("[perf] frame #{index} total={total_ms:.2}ms {detail}{compteurs}");
}

/// Écrit une mesure ponctuelle hors frame (démarrage, chargement, etc.).
pub fn event(label: &str, started: Instant) {
    if !enabled() {
        return;
    }
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    eprintln!("[perf] {label}={ms:.2}ms");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_api_is_safe_whatever_the_level() {
        // Activée ou non, l'API ne doit ni paniquer ni laisser d'état résiduel.
        frame_begin();
        stage("etape-a");
        stage("etape-b");
        frame_end();
        event("evenement", Instant::now());

        // Une étape hors frame est ignorée silencieusement.
        stage("orpheline");
        assert_eq!(enabled(), level() > 0);
    }
}
