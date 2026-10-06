//! Ce qu'un dépôt venu du web laisse sur le disque, et rien de ce qui l'a apporté.
//!
//! # Pourquoi cette moitié est à part, et pourquoi elle est pure
//!
//! Le pont de [`super::depot_windows`] est du COM : il ne s'exécute que sur Windows, il ne se
//! teste qu'en glissant une image depuis un navigateur, et son `unsafe` demande de la relire
//! plutôt que de la jouer. **Tout ce qui décide** — comment un nom se nettoie, où les octets
//! s'écrivent, ce qui est trop gros — est donc ici, sans un `unsafe`, sans une ligne de
//! Windows, et couvert par des tests qui s'exécutent sur n'importe quelle machine.
//!
//! # Des octets, et plus aucun fichier temporaire (DEPOT-4)
//!
//! Le navigateur ne donne pas un chemin : il donne un nom suggéré et des octets. On les
//! écrivait dans le répertoire temporaire du système, pour qu'ils redeviennent un fichier
//! comme ceux de l'explorateur — 172 fichiers chez lui, que rien n'effaçait et que Windows
//! pouvait vider. Ils restent désormais **en mémoire**, dans un [`Recu`] :
//! [`crate::interactions::drop`] pose une image directement depuis eux — ses octets entrent
//! dans le document —, et donne au reste un vrai fichier, là où un navigateur l'aurait mis
//! ([`super::telechargements`]).
//!
//! # Le nom vient d'ailleurs, donc il ne décide de rien
//!
//! `cFileName` est choisi par la page web, pas par l'utilisateur. Le laisser désigner un
//! chemin serait accepter qu'une page écrive où elle veut : `..\..\demarrage\x.exe` est un
//! nom de fichier valide pour qui ne regarde pas. Seule la **dernière** composante est
//! retenue, et tout ce qui pourrait séparer ou remonter est remplacé.
//!
//! C'est la même prudence que [`crate::interactions::links`], qui refuse tout ce qui n'est ni
//! `http://` ni `https://`, et que le lanceur de `drop`, qui **révèle** au lieu d'ouvrir.

use std::path::{Path, PathBuf};

/// **Combien d'octets au plus on accepte d'un seul fichier venu d'une page** — promis par le
/// navigateur, ou rapatrié par Glucose.
///
/// Deux cent cinquante-six mébioctets est ce qu'une image de très haute définition atteint au
/// pire ; au-delà, ce n'est plus une image qu'on dépose sur un canevas, et une page peut en
/// promettre autant qu'elle veut. La borne était écrite **deux fois** dans le pont, à
/// l'identique : elle vit ici, où les deux chemins la lisent.
pub const OCTETS_MAX: usize = 256 * 1024 * 1024;

/// Le nom retenu quand la page n'en donne aucun d'utilisable.
const SANS_NOM: &str = "depose";

/// **Ce qu'une page a livré** : le nom qu'elle propose, rendu sûr ([`nom_sur`]), et ses octets,
/// en mémoire. Rien n'est écrit dans le dossier temporaire du système (DEPOT-4).
#[derive(Debug, Clone, PartialEq)]
pub struct Recu {
    pub nom: String,
    pub octets: Vec<u8>,
}

impl Recu {
    /// Ce que la page a livré — ou rien, si elle n'a rien livré : un contenu vide n'est pas un
    /// dépôt, et donnerait une tuile vide, qui ressemble à un bug plutôt qu'à un refus.
    pub fn nouveau(propose: &str, octets: Vec<u8>) -> Option<Self> {
        (!octets.is_empty()).then(|| Self {
            nom: nom_sur(propose),
            octets,
        })
    }

    /// Un raccourci Internet — une adresse dans un habit de fichier ?
    pub fn est_un_raccourci(&self) -> bool {
        est_un_raccourci(Path::new(&self.nom))
    }
}

/// Ce qu'un dépôt a réellement apporté, une fois le pont passé.
///
/// Les chemins sont ceux de fichiers qui **existent** — ceux de l'explorateur ; ce qu'une page
/// a livré est en mémoire. L'application n'a donc aucun cas d'échec à traiter que le
/// glisser-déposer d'un fichier ordinaire n'ait déjà.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Moisson {
    /// Les fichiers à poser, dans l'ordre où l'explorateur les a donnés.
    pub chemins: Vec<PathBuf>,
    /// Ce qu'une page a livré, dans l'ordre où elle l'a donné.
    pub recus: Vec<Recu>,
    /// **Les adresses qu'une page donne quand elle ne promet aucun contenu.**
    ///
    /// Un lien glissé depuis la barre d'adresse, ou une image dont le navigateur ne veut pas
    /// livrer les octets. Poser le lien est un repli visible ; ne rien faire serait le geste
    /// sans effet qu'on vient de corriger.
    pub liens: Vec<String>,
    /// Où le curseur a lâché, en pixels physiques depuis le coin de la fenêtre.
    ///
    /// C'est ce que le glisser-déposer de `winit` ne transmet pas, et pourquoi un fichier
    /// déposé se pose aujourd'hui au centre de l'écran — `interactions::drop` le dit
    /// lui-même : *« la position exacte viendra avec la couche `IDropTarget` propre au
    /// projet »*.
    pub ou: Option<(f64, f64)>,
    /// **Un lot de nœuds** glissé depuis une autre fenêtre de Glucose (fiche 51 § 2) : il se
    /// colle là où l'on lâche, comme un `Ctrl+V`.
    pub lot: Option<Vec<u8>>,
}

/// **Ce qu'un dépôt fait parvenir à la boucle d'images**, dans l'ordre où cela arrive.
///
/// Un dépôt qui porte ses fichiers se pose d'un coup. Un dépôt qui ne porte que des adresses
/// — Pinterest, dix fois sur dix — part se faire rapatrier, et la seconde que cela prend ne
/// doit pas être un geste sans réponse : Nielsen place à **un dixième de seconde** la limite
/// d'une manipulation directe qui semble répondre. L'annonce part donc à l'instant du lâcher,
/// et la livraison la rejoint par son numéro.
#[derive(Debug, Clone, PartialEq)]
pub enum Depot {
    /// **Un téléchargement commence** : le numéro qui le reliera à sa livraison, le point de
    /// l'écran où le curseur a lâché, et le site d'où l'image vient.
    EnChemin {
        numero: u64,
        ou: Option<(f64, f64)>,
        hote: String,
    },
    /// **Ce qu'un dépôt apporte** — rapatrié s'il porte le numéro de son annonce. Une livraison
    /// annoncée arrive **toujours**, vide s'il le faut : sans elle, l'annonce resterait à
    /// l'écran pour toujours.
    Pose {
        numero: Option<u64>,
        moisson: Moisson,
    },
}

impl Moisson {
    /// Rien n'a été récolté.
    pub fn est_vide(&self) -> bool {
        self.chemins.is_empty()
            && self.recus.is_empty()
            && self.liens.is_empty()
            && self.lot.is_none()
    }
}

/// **Le nom de fichier qu'on accepte de cette page**, et il ne peut désigner qu'un fichier.
///
/// Trois choses lui arrivent, dans cet ordre : on ne garde que ce qui suit le dernier
/// séparateur — c'est ce qui rend `..\..\x` inoffensif sans avoir à reconnaître `..` —, on
/// remplace ce que Windows interdit dans un nom, et on retombe sur un nom neutre s'il ne
/// reste rien.
///
/// L'extension n'est pas vérifiée, et c'est voulu : `drop` **tente l'en-tête** plutôt que de
/// croire une extension, précisément parce qu'une liste diverge toujours de ce que le
/// décodeur sait lire.
pub fn nom_sur(propose: &str) -> String {
    let dernier = propose
        .rsplit(['/', '\\', ':'])
        .next()
        .unwrap_or(propose)
        .trim();
    let net: String = dernier
        .chars()
        .map(|c| match c {
            // Les caractères que Windows refuse dans un nom, et ceux de contrôle, qui ne se
            // voient pas à l'écran et se lisent très bien par un programme.
            '<' | '>' | '"' | '|' | '?' | '*' => '_',
            c if (c as u32) < 0x20 => '_',
            c => c,
        })
        .collect();
    // Un nom fait uniquement de points remonterait d'un dossier tout en passant le filtre du
    // séparateur : `.` et `..` n'ont pas de dernière composante différente d'eux-mêmes.
    if net.is_empty() || net.chars().all(|c| c == '.') {
        return SANS_NOM.to_string();
    }
    net
}

/// **Une adresse, écrite de façon qu'un `Ctrl`+clic la suive.**
///
/// `links::url_at` ne reconnaît que ce que la syntaxe Markdown désigne : une adresse écrite
/// nue reste du texte. On pose donc `[adresse](adresse)`, ce qui l'affiche telle quelle et la
/// rend cliquable.
///
/// **Les parenthèses sont ce qui casse cette forme**, et elles n'ont rien d'exotique :
/// `…/wiki/Paris_(homonymie)` en porte deux. Elles s'écrivent donc `%28` et `%29` dans la
/// partie adresse — la même adresse pour le serveur, une syntaxe qui tient pour le lecteur.
/// Ce qui est montré, lui, garde ses parenthèses : c'est ce que l'utilisateur relit.
///
/// **Je n'avais encodé que la fermante**, en raisonnant que seule elle pouvait fermer le lien
/// trop tôt. Le test l'a démenti au premier essai : l'ouvrante suffit à empêcher la tranche
/// d'être reconnue comme un lien, et `Ctrl`+clic ne suivait plus rien du tout. La preuve va
/// donc jusqu'à `url_at`, celui que le clic emploie, plutôt que de s'arrêter à la forme de la
/// chaîne — une chaîne bien formée en apparence ne prouve pas qu'un lien s'ouvre.
pub fn lien_markdown(adresse: &str) -> String {
    let montre = adresse.trim();
    let cible = montre.replace('(', "%28").replace(')', "%29");
    format!("[{montre}]({cible})")
}

/// **Ce fichier est-il un raccourci Internet** — une adresse dans un habit de fichier ?
///
/// # Le défaut que cette question ferme
///
/// Glisser une épingle depuis la grille de Pinterest ne donne pas l'image : Chrome promet un
/// fichier `.url`, soixante-quatorze octets qui disent `URL=https://fr.pinterest.com/pin/…`.
/// Le pont le traitait comme n'importe quel fichier promis — le deuxième format le plus sûr,
/// devant le bitmap — et `drop` le posait en carte portant son **nom** :
/// *« 1790160747344231500-0-fr.pinterest.com.url »*. Ni l'image, ni un lien qu'on puisse
/// suivre. C'est ce que l'utilisateur a vu le 23/09.
///
/// Un raccourci n'apporte aucun contenu : il n'apporte qu'une adresse, et c'est le format le
/// **plus pauvre** qu'un dépôt puisse porter. Il se lit donc comme tel, où qu'il vienne —
/// promis par un navigateur ou glissé depuis le bureau.
pub fn est_un_raccourci(chemin: &Path) -> bool {
    chemin
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("url"))
}

/// **L'adresse qu'un raccourci Internet désigne**, ou rien s'il n'en porte pas.
///
/// Le format est un fichier INI de Windows : une section `[InternetShortcut]` et une clé
/// `URL=`. On ne lit que cette clé-là, dans cette section-là — un raccourci peut porter une
/// icône, des dates, un numéro de favori, et aucun ne désigne ce qu'on a voulu déposer.
pub fn adresse_du_raccourci(texte: &str) -> Option<String> {
    let mut dans_la_section = false;
    for ligne in texte.lines().map(str::trim) {
        if ligne.starts_with('[') {
            dans_la_section = ligne.eq_ignore_ascii_case("[InternetShortcut]");
            continue;
        }
        let Some((cle, valeur)) = ligne.split_once('=') else {
            continue;
        };
        if dans_la_section && cle.trim().eq_ignore_ascii_case("URL") {
            let adresse = valeur.trim();
            return (!adresse.is_empty()).then(|| adresse.to_string());
        }
    }
    None
}

/// **Sépare les raccourcis du reste**, et rend leurs adresses à la place de leurs fichiers.
///
/// Un raccourci illisible reste un fichier : le lanceur le montrera, et c'est un repli
/// visible plutôt qu'un dépôt qui disparaît.
pub fn lire_les_raccourcis(chemins: &[PathBuf]) -> (Vec<PathBuf>, Vec<String>) {
    let mut fichiers = Vec::with_capacity(chemins.len());
    let mut adresses = Vec::new();
    for chemin in chemins {
        let adresse = est_un_raccourci(chemin)
            .then(|| std::fs::read(chemin).ok())
            .flatten()
            .and_then(|octets| adresse_du_raccourci(&String::from_utf8_lossy(&octets)));
        match adresse {
            Some(a) => adresses.push(a),
            None => fichiers.push(chemin.clone()),
        }
    }
    (fichiers, adresses)
}

/// **Sépare les raccourcis de ce qu'une page a livré**, et rend leurs adresses à leur place.
/// Un raccourci illisible reste un reçu : le lanceur le montrera.
pub fn separer_les_raccourcis(recus: Vec<Recu>) -> (Vec<Recu>, Vec<String>) {
    let mut gardes = Vec::with_capacity(recus.len());
    let mut adresses = Vec::new();
    for recu in recus {
        let adresse = recu
            .est_un_raccourci()
            .then(|| adresse_du_raccourci(&String::from_utf8_lossy(&recu.octets)))
            .flatten();
        match adresse {
            Some(a) => adresses.push(a),
            None => gardes.push(recu),
        }
    }
    (gardes, adresses)
}

/// **Les adresses web que ces octets transportent**, dans l'ordre et sans doublon.
///
/// # Pourquoi l'instrument en a besoin
///
/// Glisser une épingle depuis la grille de Pinterest n'apporte, le plus souvent, **aucun**
/// format standard : ni image, ni fichier, ni adresse — seulement `DragImageBits`, la vignette
/// qui suit le curseur, et `Chromium Web Custom MIME Data Format`, les données que la page a
/// posées elle-même dans le glisser (session économe du 23/09, six dépôts sur dix). La seule
/// question qui reste est de savoir si l'adresse de l'image y est **cachée** — et elle ne se
/// devine pas.
///
/// On cherche donc `http://` et `https://` en UTF-8 **et** en UTF-16 — Chromium range ses
/// données dans un « pickle » de chaînes larges —, sur les deux alignements possibles d'une
/// chaîne large. Une adresse s'arrête au premier caractère qu'une adresse ne peut pas porter
/// sans être encodée : c'est la définition de la RFC 3986, pas un choix.
pub fn adresses_dans(octets: &[u8]) -> Vec<String> {
    let mut textes = vec![String::from_utf8_lossy(octets).into_owned()];
    for decalage in 0..2 {
        let (paires, _) = octets.get(decalage..).unwrap_or_default().as_chunks::<2>();
        let larges: Vec<u16> = paires.iter().map(|p| u16::from_le_bytes(*p)).collect();
        textes.push(String::from_utf16_lossy(&larges));
    }
    let mut trouvees: Vec<String> = Vec::new();
    for texte in &textes {
        for (debut, _) in texte.match_indices("http") {
            let reste = &texte[debut..];
            if !(reste.starts_with("http://") || reste.starts_with("https://")) {
                continue;
            }
            let fin = reste
                .find(|c: char| !c.is_ascii_graphic() || "\"'<>\\^`{|}".contains(c))
                .unwrap_or(reste.len());
            let adresse = sans_ponctuation_finale(&reste[..fin]);
            if adresse.len() > "https://".len() && !trouvees.iter().any(|a| a == adresse) {
                trouvees.push(adresse.to_string());
            }
        }
    }
    trouvees
}

/// **Une adresse sans la ponctuation qui la suivait dans le texte.**
///
/// Une parenthèse fermante est permise dans une adresse — `…/wiki/Paris_(homonymie)` —, et
/// pourtant `url(https://…/h.png)` la colle à celle qui n'en a pas : le premier essai réel a
/// demandé `…h.png)` et reçu un refus. On ne la garde que si l'adresse en ouvre une ; un point,
/// une virgule ou un point-virgule final appartiennent toujours à la phrase.
fn sans_ponctuation_finale(adresse: &str) -> &str {
    let mut a = adresse;
    loop {
        let avant = a;
        a = a.trim_end_matches(['.', ',', ';', ':', '!']);
        if a.ends_with(')') && a.matches('(').count() < a.matches(')').count() {
            a = &a[..a.len() - 1];
        }
        if a == avant {
            return a;
        }
    }
}

#[cfg(test)]
mod tests;
