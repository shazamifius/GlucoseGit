//! Le banc du **démarrage** — combien de temps la vue met à atteindre la vitesse que la main
//! demande, quand elle part de zéro, et ce que le lissage qui la retient achète en échange.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_demarrage
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! L'utilisateur, après la session de terrain du 21/09 : *« au commencement d'une action,
//! lorsque tu as zéro vélocité et que tu souhaites te déplacer, que ce soit avec la minimap,
//! le pad ou autre, on a une accélération progressive — et bien cette accélération-là est
//! BEAUCOUP trop lente »*. Il ajoute que le freinage, lui, est bon : la **dissymétrie** est le
//! sujet.
//!
//! Aucune mesure du dépôt ne disait cela. Les bancs chronomètrent ce qu'une image **coûte** ;
//! celui-ci ne chronomètre rien. Il **simule** : une main qui pousse à vitesse constante, une
//! horloge d'affichage parfaitement régulière, et il lit la courbe de ce que l'écran montre.
//! Le résultat ne dépend donc d'aucune machine — c'est de l'arithmétique, et deux exécutions
//! donnent le même chiffre au bit près. C'est la seule raison pour laquelle il a le droit de
//! comparer deux exécutions, ce que la fiche 20 § 5.1 interdit partout ailleurs.
//!
//! # Ce qu'il sépare, et c'est tout son objet
//!
//! Deux causes peuvent faire une rampe molle, et elles se confondent à l'écran :
//!
//! * la **constante de temps de conduite** de [`Elan`], qui ne rembourse qu'une fraction de la
//!   dette à chaque image ;
//! * le **tempo**, qui espace les images de `k` balayages — huit sur le terrain, soit une
//!   image toutes les 33 ms.
//!
//! Le banc joue donc la même main sur plusieurs `k` **dans la même exécution** (fiche 20
//! § 5.1). Si la rampe dure autant à `k = 1` qu'à `k = 8`, le tempo est hors de cause.
//!
//! # Et ce qu'il met en face, parce qu'un lissage s'achète
//!
//! Ce lissage n'est pas gratuit ni inutile : il existe pour absorber l'irrégularité de
//! **livraison**. Windows groupe parfois trois événements dans la même milliseconde, et les
//! appliquer tels quels ferait sauter la vitesse apparente d'un facteur trois — c'est le
//! défaut que l'utilisateur décrivait par « des sauts d'image comme si on avait 15 fps ».
//!
//! Juger la rampe sans mesurer ce qu'elle achète serait donc malhonnête. Le banc joue les deux
//! livraisons — régulière et en rafales de trois — et rend le **grain** : de combien la
//! vitesse apparente varie d'une image à l'autre une fois la rampe finie. C'est le chiffre à
//! ne pas dégrader en accélérant la montée.

use glucose_desktop::interactions::elan::Elan;
use std::time::{Duration, Instant};

/// La période d'un écran à 240 Hz, celle de la machine de l'utilisateur.
const PERIODE: Duration = Duration::from_nanos(4_166_667);

/// Le rythme d'émission d'un pavé tactile ou d'une souris courante : cent par seconde.
const PERIODE_SOURCE: Duration = Duration::from_millis(10);

/// Un glissement franc, en pixels par seconde. La rampe est linéaire en la vitesse, donc les
/// parts et les temps de montée valent pour toute vitesse ; seul le retard leur est
/// proportionnel.
const VITESSE: f64 = 1200.0;

/// La diagonale d'un écran de 2560 × 1600, dont [`Elan`] se sert pour solder les dettes
/// négligeables.
const DIAGONALE: f64 = 3018.3;

/// Combien d'images on simule : de quoi couvrir largement la montée à tous les tempos.
const IMAGES: usize = 200;

/// À partir de quelle image on considère la rampe finie et le régime établi.
const REGIME: usize = 100;

/// La part de la vitesse demandée à partir de laquelle on considère la vue « arrivée ».
const ARRIVEE: f64 = 0.90;

/// Combien d'événements une rafale groupe dans le même instant de livraison.
const RAFALE: u32 = 3;

/// Comment la source livre ce que la main a fait.
#[derive(Clone, Copy, PartialEq)]
enum Livraison {
    /// Un événement à son instant, comme une source idéale.
    Reguliere,
    /// Trois événements retenus puis lâchés ensemble, comme Windows le fait.
    EnRafales,
}

impl Livraison {
    fn nom(self) -> &'static str {
        match self {
            Self::Reguliere => "régulière",
            Self::EnRafales => "rafales de 3",
        }
    }

    /// L'instant auquel le `n`-ième événement est **livré**, qui n'est pas celui où la main
    /// l'a produit.
    fn livre_a(self, base: Instant, n: u32) -> Instant {
        match self {
            Self::Reguliere => base + PERIODE_SOURCE * (n + 1),
            // Le dernier de la rafale arrive à l'heure ; les autres l'ont attendu.
            Self::EnRafales => base + PERIODE_SOURCE * (n.div_ceil(RAFALE) * RAFALE + RAFALE),
        }
    }
}

/// Ce qu'une simulation a montré.
struct Course {
    /// Le temps écoulé depuis le début du geste quand la vitesse montrée atteint [`ARRIVEE`].
    montee: Option<Duration>,
    /// De combien de pixels la vue traîne derrière la main à la fin de la simulation.
    retard: f64,
    /// Le pire écart relatif de la vitesse apparente à sa moyenne, une fois le régime établi.
    grain: f64,
}

fn main() {
    println!("\n  Le démarrage d'un geste — ce que l'écran montre quand la main part de zéro");
    println!(
        "\n  main à {VITESSE:.0} px/s, source à {:.0} Hz, écran à 240 Hz, {IMAGES} images",
        1.0 / PERIODE_SOURCE.as_secs_f64()
    );
    println!(
        "  la montée est atteinte à {:.0} % de la vitesse demandée\n",
        ARRIVEE * 100.0
    );
    for livraison in [Livraison::Reguliere, Livraison::EnRafales] {
        println!("  livraison {} :", livraison.nom());
        println!("    tempo          image      montée      retard      grain");
        for k in [1_u32, 2, 3, 5, 8] {
            let intervalle = PERIODE * k;
            let c = courir(intervalle, livraison);
            let montee = match c.montee {
                Some(d) => format!("{:6.1} ms", d.as_secs_f64() * 1000.0),
                None => " jamais  ".to_string(),
            };
            println!(
                "    {k} balayage(s)  {:6.2} ms  {montee}  {:6.1} px  {:6.1} %",
                intervalle.as_secs_f64() * 1000.0,
                c.retard,
                c.grain * 100.0
            );
        }
        println!();
    }
    println!(
        "  La montée ne dépend pas du tempo : c'est la constante de temps qui la fixe, et le\n  \
         retard en régime vaut la vitesse fois cette même constante. Le grain est ce que ce\n  \
         lissage achète, et c'est lui qu'il ne faut pas dégrader en accélérant la montée.\n"
    );
}

/// Joue une main qui pousse à vitesse constante depuis l'arrêt, sur une horloge régulière.
fn courir(intervalle: Duration, livraison: Livraison) -> Course {
    let base = Instant::now();
    let mut elan = Elan::default();
    let mut montre = 0.0_f64;
    let mut emis = 0_u32;
    let mut montee = None;
    let mut vitesses = Vec::with_capacity(IMAGES);
    for n in 0..IMAGES {
        let rang = u32::try_from(n).unwrap_or(u32::MAX) + 1;
        let instant = base + intervalle * rang;
        emis = pousser_jusqu_a(&mut elan, base, instant, emis, livraison);
        let avance = elan
            .avancer(instant, intervalle, DIAGONALE)
            .map_or(0.0, |m| m.pan.0);
        montre += avance;
        let vitesse = avance / intervalle.as_secs_f64();
        if n >= REGIME {
            vitesses.push(vitesse);
        }
        if montee.is_none() && vitesse >= VITESSE * ARRIVEE {
            montee = Some(instant.duration_since(base));
        }
    }
    Course {
        montee,
        retard: f64::from(emis) * pas_de_source() - montre,
        grain: grain(&vitesses),
    }
}

/// Le pire écart relatif d'une vitesse apparente à la moyenne de la série.
///
/// C'est bien le **pire** et non un écart type : une seule image qui avance de moitié se voit,
/// et la fiche 19 a établi qu'une moyenne cache exactement ce qu'on cherche ici.
fn grain(vitesses: &[f64]) -> f64 {
    if vitesses.is_empty() {
        return 0.0;
    }
    let moyenne = vitesses.iter().sum::<f64>() / vitesses.len() as f64;
    if moyenne <= 0.0 {
        return 0.0;
    }
    vitesses
        .iter()
        .map(|v| (v - moyenne).abs() / moyenne)
        .fold(0.0_f64, f64::max)
}

/// Ce qu'un événement de la source apporte, en pixels.
fn pas_de_source() -> f64 {
    VITESSE * PERIODE_SOURCE.as_secs_f64()
}

/// Pousse dans l'élan tous les événements que la source a **livrés** jusqu'à cet instant.
///
/// Rend le nombre total d'événements poussés depuis le début, qui dit ce que la main a
/// demandé et sert à mesurer le retard.
fn pousser_jusqu_a(
    elan: &mut Elan,
    base: Instant,
    instant: Instant,
    deja: u32,
    livraison: Livraison,
) -> u32 {
    let mut pousses = deja;
    while livraison.livre_a(base, pousses) <= instant {
        elan.pousser_pan(pas_de_source(), 0.0, livraison.livre_a(base, pousses));
        pousses += 1;
    }
    pousses
}
