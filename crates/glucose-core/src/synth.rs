//! Documents synthétiques reproductibles — le travail B.5 du plan de marche.
//!
//! Un banc d'essai qui génère ses données au hasard ne mesure rien : deux exécutions ne
//! comparent pas la même chose. Tout ici est **déterministe à la graine près** — même graine,
//! même document, octet pour octet, sur toutes les machines.
//!
//! # Pourquoi pas de générateur aléatoire externe
//!
//! Le noyau n'a aucune dépendance, et il n'en prend pas une pour ça : une suite congruentielle
//! linéaire suffit, tient en dix lignes, et a l'avantage décisif d'être **exactement
//! reproductible** sans dépendre de la version d'une bibliothèque. Ses constantes sont celles
//! de Knuth (MMIX), et l'on n'en garde que les bits de poids fort — les bits de poids faible
//! d'un LCG ont une période courte, et c'est le piège classique de cette famille.
//!
//! # Les formes
//!
//! [`Shape`] existe parce qu'un document réel n'est pas une pluie uniforme de nœuds. Un canva
//! de travail est fait d'**amas** — un sujet, ses notes, ses images — reliés par de **longues
//! arêtes**. C'est cette forme-là qui met un index spatial et un cache de tuiles en difficulté,
//! pas la pluie uniforme, et c'est donc elle qu'il faut savoir fabriquer.

use crate::store::Store;
use crate::types::{Annotation, BoardImage, CanvasFolder, Viewport};

/// Suite congruentielle linéaire — le hasard reproductible du projet.
///
/// Les constantes sont celles de MMIX (Knuth). `next` ne rend que les bits de poids fort :
/// dans un LCG, le bit de poids faible alterne avec une période de 2, le suivant de 4, et ainsi
/// de suite — s'en servir donnerait une suite visiblement régulière.
#[derive(Debug, Clone)]
pub struct Lcg(u64);

impl Lcg {
    /// Une suite à partir de sa graine. La même graine rend toujours la même suite.
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// L'entier suivant de la suite.
    ///
    /// Nommee `next_u64` et non `next` : une methode `next` sur un type qui n'est pas un
    /// iterateur se confond avec celle d'`Iterator`, et le lecteur s'attend alors a pouvoir
    /// ecrire une boucle `for`.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 11
    }

    /// Un réel dans `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() % 1_000_000) as f64 / 1_000_000.0
    }

    /// Un réel dans `[0, span)`.
    pub fn upto(&mut self, span: f64) -> f64 {
        self.unit() * span
    }

    /// Un entier dans `[0, n)`. Rend `0` si `n` vaut zéro.
    pub fn index(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }

    /// Un réel centré sur `0`, d'étendue `± span`, resserré vers le centre.
    ///
    /// La somme de deux tirages uniformes donne une **distribution triangulaire** : la densité
    /// décroît linéairement de part et d'autre du centre. C'est exact, ça tient en une ligne, et
    /// c'est assez pour faire un amas qui ressemble à un amas — une vraie gaussienne
    /// demanderait un logarithme et un cosinus pour un résultat visuellement identique ici.
    pub fn centered(&mut self, span: f64) -> f64 {
        (self.unit() + self.unit() - 1.0) * span
    }
}

/// La forme d'un document synthétique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Une pluie uniforme sur le carré. Le cas facile : densité constante partout.
    Uniform,
    /// Des amas, chacun dense, séparés par du vide. La forme d'un vrai canva de travail.
    Clustered,
    /// Des amas **reliés entre eux** par de longues flèches. La forme d'une carte de
    /// connaissances, et le cas difficile : une arête traverse tout le document, donc toutes
    /// les cellules d'un index et toutes les tuiles d'un cache.
    Wikipedia,
}

/// Le tableau d'un document synthétique — toujours le même identifiant, pour que les bancs
/// n'aient pas à le deviner.
pub const BOARD: &str = "main";

/// Un document de `n` nœuds répartis sur un carré de `span` unités, dans la forme demandée.
///
/// Le remplissage a lieu **sous transaction live** : sans elle, chaque ajout clonerait le
/// document entier et la génération serait quadratique — inatteignable au-delà de quelques
/// milliers de nœuds. La pile d'annulation est vidée à la fin : un banc mesure un document,
/// pas l'historique de sa fabrication.
pub fn document(n: usize, span: f64, shape: Shape, seed: u64) -> Store {
    let mut store = Store::new("synthetique");
    let mut rng = Lcg::new(seed);
    store.begin_live_edit();

    // Le nombre d'amas suit la racine du nombre de nœuds : c'est ce qui garde une densité
    // d'amas à peu près constante quand le document grandit, plutôt que des amas qui enflent.
    let clusters = match shape {
        Shape::Uniform => 0,
        Shape::Clustered | Shape::Wikipedia => ((n as f64).sqrt() / 8.0).ceil().max(1.0) as usize,
    };
    let centers: Vec<(f64, f64)> =
        (0..clusters).map(|_| (rng.upto(span), rng.upto(span))).collect();
    // Un amas occupe une fraction du carré telle que les amas se touchent à peine.
    let radius = span / (clusters.max(1) as f64).sqrt() / 3.0;

    for i in 0..n {
        let (x, y) = if centers.is_empty() {
            (rng.upto(span), rng.upto(span))
        } else {
            let c = centers[rng.index(centers.len())];
            (c.0 + rng.centered(radius), c.1 + rng.centered(radius))
        };
        match i % 4 {
            0 => {
                let mut img = BoardImage::new(format!("img-{i}"), x, y, 200.0, 150.0);
                img.tags.push("synth".to_string());
                store.add_image(BOARD, img);
            }
            1 => store.add_annotation(
                BOARD,
                Annotation::text(format!("txt-{i}"), x, y, "Un fait, écrit à la main."),
            ),
            2 => store.add_annotation(
                BOARD,
                Annotation::sticky(format!("sti-{i}"), x, y, "une note brève"),
            ),
            _ => store.add_annotation(BOARD, Annotation::membrane(format!("mem-{i}"), x, y, 420.0, 320.0)),
        }
    }

    // Les longues arêtes : une par amas, vers un autre amas. C'est le cas qui fait mal, et il
    // n'existe que dans cette forme.
    if shape == Shape::Wikipedia && centers.len() > 1 {
        for (k, from) in centers.iter().enumerate() {
            let to = centers[(k + 1 + rng.index(centers.len() - 1)) % centers.len()];
            store.add_annotation(
                BOARD,
                Annotation::arrow(format!("arr-{k}"), from.0, from.1, to.0, to.1),
            );
        }
    }

    store.end_live_edit();
    store.clear_selection();
    store.journal.clear();
    store
}

/// **La scène témoin** : un exemplaire de chaque chose que Glucose sait dessiner, à des
/// positions fixes, dans un cadrage fixe.
///
/// Elle n'est pas un document réaliste et n'essaie pas de l'être. Son rôle est d'être
/// **capturée** : tant que son image ne change pas, le rendu n'a pas bougé ; le jour où elle
/// change, c'est qu'une modification a touché l'écran, et on la regarde. C'est le filet que
/// six cent quarante et un tests unitaires ne tendent pas — aucun d'eux ne voit une carte
/// posée un pixel trop bas.
///
/// Toute fonctionnalité de rendu qui arrive **doit** y ajouter son exemplaire, sinon elle
/// n'est couverte par rien.
pub fn witness() -> Store {
    let mut store = Store::new("temoin");
    store.begin_live_edit();

    store.add_annotation(
        BOARD,
        Annotation::text(
            "t-markdown",
            -520.0,
            -260.0,
            "# Titre de premier rang\n## Sous-titre\nUn corps de texte assez long pour que le \
             retour à la ligne ait lieu et se voie.\n- une puce\n- une autre puce",
        ),
    );
    store.add_annotation(
        BOARD,
        Annotation::text("t-accents", -520.0, 40.0, "Accents : éàçùôêîï — « guillemets »"),
    );
    store.add_annotation(
        BOARD,
        Annotation::text(
            "t-maths",
            60.0,
            340.0,
            "Une formule, seule sur sa ligne :\n$$\\int_0^\\infty e^{-x^2}\\,dx = \\frac{\\sqrt{\\pi}}{2}$$\nEt une qui ne compile pas :\n$$\\frac{a}{$$",
        ),
    );
    store.add_annotation(BOARD, Annotation::sticky("s-simple", -520.0, 180.0, "un pense-bête"));
    store.add_annotation(BOARD, Annotation::arrow("a-droite", -120.0, 200.0, 200.0, 60.0));

    let mut membrane = Annotation::membrane("m-titree", 60.0, -260.0, 420.0, 300.0);
    if let Annotation::Membrane { text, .. } = &mut membrane {
        *text = Some("Membrane témoin".to_string());
    }
    store.add_annotation(BOARD, membrane);

    let mut folder = CanvasFolder::new("f-temoin", "Dossier témoin", String::new());
    folder.x = 60.0;
    folder.y = 120.0;
    folder.width = 260.0;
    folder.height = 190.0;
    store.create_folder(BOARD, folder);

    // Le cadrage fait partie de la scène : sans lui, la capture dépendrait du viewport par
    // défaut, qui pose l'origine du monde dans le coin de la fenêtre — et la moitié du témoin
    // tomberait hors champ. Ces valeurs mettent tout le contenu sous le bandeau et dans la
    // fenêtre, à l'échelle 1, pour une capture de [`WITNESS_SIZE`].
    let board_id = store.project.active_board_id.clone();
    store.set_viewport(&board_id, Viewport { x: 720.0, y: 374.0, scale: 1.0 });

    store.end_live_edit();
    store.clear_selection();
    store.journal.clear();
    store
}

/// La définition pour laquelle le cadrage de [`witness`] est réglé.
pub const WITNESS_SIZE: (u32, u32) = (1400, 900);

/// La boîte englobante du contenu de la scène témoin, en unités monde — ce que le cadrage doit
/// contenir, et ce qu'un test peut vérifier sans dessiner.
pub const WITNESS_CONTENT: (f64, f64, f64, f64) = (-520.0, -260.0, 480.0, 490.0);

/// **La vitrine** : un document soigné, fait pour être montré.
///
/// Elle n'a pas le rôle de [`witness`], qui existe pour être comparée à elle-même. Celle-ci
/// existe pour être **regardée** : c'est elle qui produit les captures du dépôt, et elle doit
/// donc montrer ce que Glucose sait faire, arrangé comme un vrai tableau de travail plutôt que
/// comme une liste d'exemples.
///
/// Elle est, comme tout le reste de ce module, entièrement déterministe : la capture du dépôt
/// se refait à l'identique par une commande.
pub fn showcase() -> Store {
    let mut store = Store::new("Théorie de l'information");
    store.begin_live_edit();

    // La colonne de gauche : le fil du raisonnement.
    store.add_annotation(
        BOARD,
        Annotation::text(
            "s-titre",
            -880.0,
            -420.0,
            "# Entropie de Shannon\n## Ce que mesure l'information\nUne source qui ne surprend \
             jamais n'apprend rien. L'entropie compte la surprise moyenne.",
        ),
    );
    store.add_annotation(
        BOARD,
        Annotation::text(
            "s-formule",
            -880.0,
            -180.0,
            "$$H(X) = -\\sum_{i=1}^{n} p_i \\log_2 p_i$$",
        ),
    );
    store.add_annotation(
        BOARD,
        Annotation::text(
            "s-cas",
            -880.0,
            -20.0,
            "- une pièce équilibrée : 1 bit\n- une pièce truquée : moins\n- une pièce à deux faces : zéro",
        ),
    );
    store.add_annotation(
        BOARD,
        Annotation::sticky("s-note", -880.0, 200.0, "à relier au codage de Huffman"),
    );

    // La membrane de droite : le domaine voisin.
    let mut membrane = Annotation::membrane("s-membrane", -180.0, -420.0, 700.0, 460.0);
    if let Annotation::Membrane { text, color, .. } = &mut membrane {
        *text = Some("Théorie du codage".to_string());
        *color = Some("#7c5cff".to_string());
    }
    store.add_annotation(BOARD, membrane);

    store.add_annotation(
        BOARD,
        Annotation::text(
            "s-kraft",
            -140.0,
            -330.0,
            "## Inégalité de Kraft\nUn code préfixe existe si et seulement si :",
        ),
    );
    store.add_annotation(
        BOARD,
        Annotation::text("s-kraft-f", -140.0, -180.0, "$$\\sum_{i=1}^{n} 2^{-\\ell_i} \\le 1$$"),
    );
    store.add_annotation(
        BOARD,
        Annotation::text(
            "s-borne",
            -140.0,
            -30.0,
            "La longueur moyenne d'un code optimal est bornée :\n$$H(X) \\le \\bar{\\ell} < H(X) + 1$$",
        ),
    );

    // Les flèches du raisonnement.
    store.add_annotation(BOARD, Annotation::arrow("s-a1", -560.0, -120.0, -160.0, -140.0));
    store.add_annotation(BOARD, Annotation::arrow("s-a2", -560.0, 60.0, -160.0, 20.0));

    // Un dossier : le sous-tableau des démonstrations.
    let mut dossier = CanvasFolder::new("s-preuves", "Démonstrations", String::new());
    dossier.x = -880.0;
    dossier.y = 320.0;
    dossier.width = 300.0;
    dossier.height = 200.0;
    dossier.color = "#4ade80".to_string();
    store.create_folder(BOARD, dossier);

    let board_id = store.project.active_board_id.clone();
    store.set_viewport(&board_id, Viewport { x: 1_010.0, y: 500.0, scale: 1.0 });

    store.end_live_edit();
    store.clear_selection();
    store.journal.clear();
    store
}

#[cfg(test)]
mod tests;
