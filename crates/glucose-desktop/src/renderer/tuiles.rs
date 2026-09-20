//! Le cache de tuiles — TUILE-1, la moitié qui a les pixels.
//!
//! [`glucose_core::tuile`] dit **où** sont les tuiles et **ce qu'elles portent** ; ce module
//! garde ce qu'elles montrent, et ne le redessine jamais deux fois.
//!
//! # La mémoïsation, et pourquoi elle est indexée par l'empreinte et non par l'adresse
//!
//! C'est le point qui fait toute la différence avec le cache de vignettes qu'il remplace.
//!
//! Indexer par l'**adresse** donnerait un cache ordinaire : chaque case du monde garde son
//! image, et deux cases identiques la peignent chacune. Indexer par l'**empreinte** donne un
//! cache *canonique* : deux régions du canevas qui portent la même chose disposée pareil
//! partagent **un seul** rendu, où qu'elles soient. C'est la propriété de Hashlife, et elle
//! transforme le coût — proportionnel au nombre de contenus **distincts**, plus au nombre de
//! tuiles à l'écran.
//!
//! Le cas le plus fréquent en profite le plus : dans un canevas infini, l'immense majorité des
//! tuiles sont vides. Elles ont toutes la même empreinte, donc **un seul** rendu pour tout le
//! vide du document. Se déplacer sur une zone sans contenu ne dessine rien.
//!
//! # Ce que le cache garde, et ce qu'il jette
//!
//! Il garde ce qui a servi à la dernière image, et jette le reste. La borne n'est donc pas un
//! nombre choisi : c'est **ce que l'écran demande**, et elle suit d'elle-même la taille de la
//! fenêtre et le niveau de zoom. Un écran plus grand garde plus de tuiles parce qu'il en
//! montre plus, pas parce qu'un réglage l'a décidé.
//!
//! # Ce qu'il ne fait pas
//!
//! Il ne décide pas quand s'en servir : le rendu le fait, selon ce que la vue demande. Et il
//! ne construit rien de lui-même en pleine image — il **note**, comme le faisait déjà
//! CASCADE-1, et l'atelier vient ensuite dans le temps libre.

use glucose_core::report::Plages;
use glucose_core::store::Store;
use glucose_core::tuile::{couvrant, Adresse, Empreinte, Occupant, COTE};
use glucose_core::types::Viewport;
use std::collections::HashMap;
use tiny_skia::Pixmap;

/// Ce qu'une tuile montre, prêt à poser.
struct Rendu {
    pixels: Pixmap,
    /// La dernière image où ce rendu a servi. Ce qui n'a pas servi s'en va.
    vu: u64,
    /// Ce que la tuile porte vraiment : la boîte des pixels non transparents, et si elle est
    /// opaque d'un bord à l'autre.
    ///
    /// # Pourquoi c'est mesuré une fois plutôt que composé à chaque image
    ///
    /// Une tuile de photos est transparente partout où aucune photo ne passe. La composer en
    /// entier, en source-over, coûtait cinq millisecondes pour un écran de 2560 × 1600 — huit
    /// fois le plancher de la bande passante. Un balayage de vingt microsecondes au moment de
    /// la ranger dit où sont ses pixels ; le report ne parcourt ensuite que ceux-là, et les
    /// **remplace** quand ils sont opaques, ce qui est un déplacement de mémoire.
    portee: Portee,
}

/// Où sont les pixels d'une tuile, s'ils sont opaques — et, ligne par ligne, où exactement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Portee {
    /// La boîte des pixels non transparents, en pixels de la tuile : `(x0, y0, x1, y1)`,
    /// bords droit et bas exclus. `None` si la tuile est vide.
    pub boite: Option<(u32, u32, u32, u32)>,
    /// Tous les pixels de la boîte sont-ils opaques ? Vrai pour une tuile vide.
    pub opaque: bool,
    /// Les plages de chaque ligne : ce que la composition copie, saute ou mélange, sans
    /// avoir à relire un alpha. Lues ici, une fois, et non à chaque image — `bench_salissure`
    /// mesurait ce balayage répété à 4,8 ms par écran, la moitié d'une image.
    pub plages: Plages,
}

impl Portee {
    /// Mesure une tuile : ses plages, et ce qui s'en déduit.
    ///
    /// Un passage sur les alphas, et rien d'autre. La boîte est l'englobant des plages non
    /// transparentes ; un seul alpha partiel suffit à déclarer la tuile non opaque.
    pub fn de(pixels: &Pixmap) -> Self {
        let (px, _) = pixels.data().as_chunks::<4>();
        let plages = Plages::de(px, pixels.width(), pixels.height());
        Self {
            boite: plages.boite(),
            opaque: plages.opaque(),
            plages,
        }
    }
}

/// Les tuiles déjà peintes, et ce que chaque case du monde porte.
#[derive(Default)]
pub struct Tuiles {
    /// Les pixels, **par empreinte** : c'est la mémoïsation.
    rendus: HashMap<u64, Rendu>,
    /// Ce que chaque case portait la dernière fois qu'on l'a regardée.
    ///
    /// Séparé des pixels parce que ce sont deux questions distinctes : « qu'y a-t-il ici »
    /// dépend du document, « à quoi cela ressemble » n'en dépend pas. Mille cases peuvent
    /// pointer le même rendu.
    portees: HashMap<Adresse, u64>,
    image: u64,
    /// Combien de tuiles ont réellement été peintes depuis le début.
    ///
    /// C'est ce qui rend le cache **testable** : sans ce compte, un cache qui repeindrait tout
    /// à chaque image rendrait les mêmes pixels et passerait toutes les épreuves d'aspect. On
    /// ne saurait qu'il est inutile qu'au chronomètre, c'est-à-dire jamais de façon
    /// déterministe — la leçon des vignettes, qui ont servi à 1 % pendant cinq sessions.
    peintes: u64,
    /// Combien de fois un rendu déjà là a servi tel quel.
    reprises: u64,
}

impl Tuiles {
    pub fn nouveau() -> Self {
        Self::default()
    }

    /// Combien de tuiles ont été peintes depuis le début — le travail réellement fait.
    pub fn peintes(&self) -> u64 {
        self.peintes
    }

    /// Combien de fois un rendu déjà là a servi — le travail épargné.
    pub fn reprises(&self) -> u64 {
        self.reprises
    }

    /// Combien de rendus distincts sont gardés.
    pub fn gardes(&self) -> usize {
        self.rendus.len()
    }

    /// Ouvre une image : ce qui ne servira pas d'ici la fermeture sera oublié.
    pub fn ouvrir(&mut self) {
        self.image += 1;
    }

    /// Oublie ce qui n'a pas servi à cette image.
    ///
    /// La borne du cache est donc **ce que l'écran demande**, et rien d'autre : aucun nombre
    /// n'a eu à être choisi, et elle suit la fenêtre et le zoom d'elle-même.
    pub fn fermer(&mut self) {
        let image = self.image;
        self.rendus.retain(|_, r| r.vu == image);
        // Une case dont plus aucun rendu ne répond n'a plus de raison d'être retenue.
        let vivants: std::collections::HashSet<u64> = self.rendus.keys().copied().collect();
        self.portees.retain(|_, e| vivants.contains(e));
    }

    /// Les pixels de cette empreinte et leur portée, s'ils sont déjà peints. Les marque
    /// comme ayant servi.
    pub fn deja_peinte(&mut self, empreinte: Empreinte) -> Option<(&Pixmap, &Portee)> {
        let image = self.image;
        let rendu = self.rendus.get_mut(&empreinte.valeur())?;
        if rendu.vu != image {
            rendu.vu = image;
        }
        self.reprises += 1;
        Some((&rendu.pixels, &rendu.portee))
    }

    /// Range les pixels d'une empreinte qu'on vient de peindre.
    pub fn ranger(&mut self, empreinte: Empreinte, pixels: Pixmap) {
        self.peintes += 1;
        let portee = Portee::de(&pixels);
        self.rendus.insert(
            empreinte.valeur(),
            Rendu {
                pixels,
                vu: self.image,
                portee,
            },
        );
    }

    /// Note ce qu'une case du monde porte, pour ne pas le recalculer à l'image suivante.
    pub fn noter(&mut self, adresse: Adresse, empreinte: Empreinte) {
        self.portees.insert(adresse, empreinte.valeur());
    }

    /// Ce que cette case portait à la dernière image, si on l'a déjà regardée.
    pub fn portee(&self, adresse: Adresse) -> Option<Empreinte> {
        self.portees.get(&adresse).copied().map(Empreinte::depuis)
    }
}

/// Les tuiles qu'il faut pour couvrir l'écran sous cette vue, et à quel niveau.
///
/// Le niveau est celui dont l'échelle ne dépasse pas celle de la vue : les tuiles sont alors
/// **agrandies** d'un facteur compris entre un et deux, jamais réduites. `bench_tuiles` mesure
/// ce régime à 1,7 ms pour un écran entier, contre 25 ms si l'on interpolait.
pub fn a_l_ecran(vue: Viewport, ecran: (f32, f32)) -> (i32, impl Iterator<Item = Adresse>) {
    let niveau = Adresse::niveau_pour(vue.scale);
    let coin = crate::canvas::screen_to_world(0.0, 0.0, &vue);
    let fin = crate::canvas::screen_to_world(f64::from(ecran.0), f64::from(ecran.1), &vue);
    let region = glucose_core::geometry::Rect::new(
        coin.0,
        coin.1,
        (fin.0 - coin.0).max(0.0),
        (fin.1 - coin.1).max(0.0),
    );
    (niveau, couvrant(niveau, region))
}

/// Ce que cette tuile porte, lu dans le document.
///
/// Les coordonnées sont **relatives au coin de la tuile et en pixels de son niveau** — c'est
/// cette soustraction, et elle seule, qui fait que deux régions identiques du canevas
/// partagent leur rendu.
pub fn ce_que_porte(store: &Store, adresse: Adresse, rangs: &[u32]) -> Empreinte {
    let Some(board) = store.active_board() else {
        return Empreinte::vide();
    };
    let echelle = Adresse::echelle(adresse.niveau);
    let (ox, oy) = adresse.origine_en_pixels();
    let couverte = adresse.couvre();

    let mut empreinte = Empreinte::vide();
    for rang in rangs {
        let Some(img) = board.images.get(*rang as usize) else {
            continue;
        };
        // **La boîte, et non `x`/`y` nus.** Une photo range son CENTRE dans `x` et `y`
        // (`BoardImage::rect`) ; la première version de cette fonction les prenait pour le
        // coin, et une photo à cheval sur deux tuiles n'était comptée que dans celle de
        // droite -- la moitié gauche disparaissait de l'écran. La preuve de redimensionnement
        // l'a vu au premier branchement : « bord à 500 px, le document dit 400 ».
        let boite = img.rect();
        // Hors de la tuile : elle n'en montre rien, donc elle ne doit pas s'en souvenir.
        if boite.right() <= couverte.left
            || boite.left >= couverte.right()
            || boite.bottom() <= couverte.top
            || boite.top >= couverte.bottom()
        {
            continue;
        }
        empreinte.ajouter(Occupant {
            boite: glucose_core::geometry::Rect::new(
                boite.left * echelle - ox,
                boite.top * echelle - oy,
                boite.width * echelle,
                boite.height * echelle,
            ),
            aspect: aspect_de(img),
        });
    }
    empreinte
}

/// Ce qui distingue l'aspect d'une photo — tout ce qui change ses pixels, et rien d'autre.
///
/// # La seule règle que l'empreinte impose, et l'oublier serait invisible
///
/// Deux nœuds de même aspect et de même boîte **doivent** donner les mêmes pixels. En oublier
/// un ferait réutiliser un rendu qui n'est pas le bon — et le défaut ne se verrait pas tout de
/// suite : il faudrait que deux nœuds ne diffèrent *que* par la propriété oubliée.
///
/// D'où le choix de tout hacher, y compris ce qui paraît accessoire : le fichier, la rotation,
/// la nature vidéo, le cadrage, **et les domaines** — qui décident de la couleur du halo, donc
/// de pixels bien au-delà de la photo elle-même.
///
/// Toute propriété ajoutée plus tard à une photo devra passer par ici. C'est le prix d'un
/// cache canonique, et il se paie une fois.
fn aspect_de(img: &glucose_core::types::BoardImage) -> u64 {
    let mut mots = vec![
        mot_du_texte(img.src.as_deref().unwrap_or_default()),
        mot_du_texte(img.fit.as_deref().unwrap_or_default()),
        img.rotation.to_bits(),
        u64::from(img.is_video),
        u64::from(img.locked),
        img.original_width.to_bits(),
        img.original_height.to_bits(),
    ];
    // Les domaines décident de la teinte du halo : deux photos identiques rangées dans deux
    // domaines différents ne donnent pas les mêmes pixels.
    for d in &img.domains {
        mots.push(mot_du_texte(&d.domain_id));
        mots.push(d.weight.to_bits());
    }
    meler(&mots)
}

/// Mêle des mots par FNV-1a. L'ordre compte, et c'est voulu.
fn meler(mots: &[u64]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for mot in mots {
        h = (h ^ mot).wrapping_mul(0x1000_0000_01b3);
    }
    h
}

/// Un mot de soixante-quatre bits qui résume un texte, par FNV-1a.
fn mot_du_texte(texte: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for octet in texte.as_bytes() {
        h = (h ^ u64::from(*octet)).wrapping_mul(0x1000_0000_01b3);
    }
    h
}

/// Le côté d'une tuile, repris ici pour que l'appelant n'ait pas à connaître le noyau.
pub const COTE_TUILE: u32 = COTE;

#[cfg(test)]
mod tests;
