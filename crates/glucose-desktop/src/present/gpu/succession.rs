//! Comment les images se succèdent devant l'écran — et le défaut que ce module répare.
//!
//! # Le mode de présentation n'était choisi par personne
//!
//! `present/gpu.rs` documentait « `Fifo`, le défaut, est ce qu'on veut à l'usage ». Ce n'en
//! était pas un : [`wgpu::Surface::get_default_config`] retient `present_modes.first()`,
//! c'est-à-dire **ce que le pilote annonce en premier**. Sur la machine de l'utilisateur — un
//! Intel Arc — c'est `Immediate`, et la bannière de démarrage l'écrivait à chaque lancement
//! sans que personne y prête attention.
//!
//! `Immediate` ne synchronise rien. L'image part au milieu d'un balayage, donc l'instant où
//! elle devient visible est arbitraire : le mouvement calculé pour un instant se voit à un
//! autre, et l'écart change à chaque image. **Aucune mesure de coût ne le montre** — la
//! cadence reste excellente pendant que le contenu tressaute.
//!
//! Le choix se fait donc ici, explicitement, et il se dit dans la chronique.

/// La façon de présenter demandée par l'environnement, s'il en demande une.
///
/// # Pourquoi ce réglage existe
///
/// La chronique a montré que `present` coûte 28 ms sur un canevas **vide**, soit 68 % de
/// l'image. Or `get_current_texture` **bloque** en mode `Fifo` : il attend que l'écran ait
/// fini de balayer. Une durée seule ne distingue donc pas un travail lent d'une attente, et
/// c'est exactement l'ambiguïté qui a déjà fait chercher au mauvais endroit cette semaine.
///
/// `GLUCOSE_PRESENT=immediate` supprime l'attente : ce qui reste est le travail réel. La
/// comparaison des deux tranche la question au lieu de la raisonner.
///
/// * `immediate` — aucune attente, l'image part tout de suite (déchirure possible) ;
/// * `mailbox` — sans attente ni déchirure, quand la carte le propose ;
/// * `fifo` — le défaut : l'image attend le balayage.
pub(super) fn cadence_demandee() -> Option<wgpu::PresentMode> {
    match std::env::var("GLUCOSE_PRESENT")
        .ok()?
        .trim()
        .to_lowercase()
        .as_str()
    {
        "immediate" => Some(wgpu::PresentMode::Immediate),
        "mailbox" => Some(wgpu::PresentMode::Mailbox),
        "fifo" => Some(wgpu::PresentMode::Fifo),
        autre => {
            eprintln!("[Glucose] GLUCOSE_PRESENT={autre} inconnu (immediate, mailbox, fifo)");
            None
        }
    }
}

/// Combien d'images la chaîne garde en vol, si l'environnement le demande.
///
/// # Pourquoi ce réglage existe, et ce qu'il doit trancher
///
/// La chronique du 21/09 au soir donne `acquerir` à **19,5 ms en médiane au repos**, premier
/// poste de la session et loin devant tout le reste. `get_current_texture` ne dessine rien :
/// il demande à la chaîne une image libre, et s'il attend vingt millisecondes, c'est qu'il
/// n'y en a pas.
///
/// Le compteur `surf` a déjà écarté la première hypothèse — aucune reconfiguration de
/// surface sur les images lentes. Reste la **profondeur de la chaîne** : `wgpu` la déduit de
/// `desired_maximum_frame_latency`, que `get_default_config` fixe à deux. Sur DXGI en modèle
/// « flip », le compositeur retient une image pendant qu'il en affiche une autre, et deux ne
/// suffisent pas toujours : l'application attend alors qu'une se libère, à chaque image.
///
/// Trois ou quatre images coûtent quelques mébioctets et **une** image de latence
/// supplémentaire en théorie — contre vingt millisecondes d'attente mesurées. La comparaison
/// tranche la question au lieu de la raisonner, comme `GLUCOSE_CARTE` l'a fait pour les gels
/// de `present`.
///
/// ```text
///     GLUCOSE_IMAGES=3    la chaîne garde trois images en vol
/// ```
///
/// # Ce que le terrain a répondu, le 22/09 : l'hypothèse est morte
///
/// Deux sessions, mêmes gestes, même document de 482 nœuds, sur la RTX en `mailbox` :
///
/// ```text
///                        deux images      trois images
///     images par seconde        51               43
///     a l'ecran, median      19,48 ms         23,17 ms
///     latence mediane        19,5 ms          23,2 ms
///     au-dessus de 10 ms        8 %             10 %
///     judder                   16 %             19 %
///     tempo a 5 balayages      53 %             66 %
/// ```
///
/// **Trois images en vol dégradent tout**, et de façon cohérente sur six indicateurs. Mais
/// surtout : `acquerir` **n'apparaît dans aucune des deux chroniques** — ni dans les postes
/// du repos, ni dans ceux du déplacement, ni dans ceux du zoom, donc il vaut moins de
/// 0,36 ms. Il n'y avait rien à gagner : la profondeur de la chaîne ne tenait rien.
///
/// Les 19,5 ms du 21/09 au soir étaient une **attente de synchronisation**, pas une pénurie
/// d'images. En `Fifo`, `get_current_texture` attend le balayage — et c'est du repos, la
/// documentation de [`PREFERENCES`] le dit déjà. En `Mailbox`, que la RTX offre et l'Arc non,
/// il ne bloque plus. Le nombre d'images en vol n'a jamais été le sujet.
///
/// Le réglage reste, parce qu'il est un **instrument** et non un choix de production : il a
/// tranché une question en une session, et une autre machine pourra la reposer. Le défaut de
/// `wgpu` — deux — est celui qu'on garde.
pub(super) fn images_demandees() -> Option<u32> {
    let brut = std::env::var("GLUCOSE_IMAGES").ok()?;
    match brut.trim().parse::<u32>() {
        Ok(n) if (1..=8).contains(&n) => Some(n),
        _ => {
            eprintln!("[Glucose] GLUCOSE_IMAGES={brut} ignore (un entier de 1 a 8)");
            None
        }
    }
}

/// La carte graphique demandée par l'environnement : la plus économe par défaut.
///
/// # Pourquoi ce réglage existe, et pourquoi il n'est pas l'arbitre
///
/// Sur la session de terrain du 21/09, `present` a gelé huit fois entre 150 et 360 ms,
/// pendant l'édition de texte et le zoom, sur un canevas de 482 nœuds sans une photo. Rien
/// dans le code de Glucose ne s'exécute pendant `queue.present` : c'est le pilote ou le
/// compositeur qui bloque, et la fiche 18 (étape 5) le nomme « jamais élucidé » depuis trois
/// sessions.
///
/// La machine porte deux cartes — un Intel Arc intégré, que `LowPower` retient, et une RTX
/// dédiée. Sur un portable hybride, l'écran externe est souvent câblé sur la dédiée : chaque
/// image rendue sur l'intégrée traverse alors le bus pour être composée, et c'est un chemin
/// connu pour geler. **L'hypothèse se teste en une session**, pas en raisonnant :
///
/// ```text
///     GLUCOSE_CARTE=rapide    la carte la plus puissante
///     GLUCOSE_CARTE=econome   la plus économe (le défaut)
/// ```
///
/// Ce n'est pas l'arbitre de la fiche 21, qui choisira par le **débit observé** et sans
/// variable. C'est l'instrument qui dira s'il y a quelque chose à arbitrer.
pub(super) fn carte_demandee() -> wgpu::PowerPreference {
    pour_wgpu(carte_de_depart(&crate::present::souvenir::chemin()))
}

/// **La carte a ouvrir au lancement** : celle que l'environnement impose, sinon celle que
/// l'arbitre a retenue lors d'une session precedente (ARBITRE-4), sinon l'econome -- celle
/// qui gene le moins les autres logiciels.
pub fn carte_de_depart(souvenir: &std::path::Path) -> crate::present::arbitre::Preference {
    carte_imposee()
        .or_else(|| crate::present::souvenir::lire(souvenir))
        .unwrap_or(crate::present::arbitre::Preference::Econome)
}

/// La carte que l'environnement impose, s'il en impose une (ARBITRE-1).
///
/// Quand elle est posee, l'utilisateur a tranche et l'arbitre n'a plus rien a arbitrer : il
/// ne s'installe pas. C'est ce qui garde le reglage utile comme **instrument** -- forcer une
/// carte pour la mesurer -- sans qu'il redevienne le seul moyen d'avoir un logiciel fluide.
pub fn carte_imposee() -> Option<crate::present::arbitre::Preference> {
    use crate::present::arbitre::Preference;
    match std::env::var("GLUCOSE_CARTE")
        .ok()
        .as_deref()
        .map(|v| v.trim().to_lowercase())
        .as_deref()
    {
        Some("rapide") => Some(Preference::Rapide),
        Some("econome") => Some(Preference::Econome),
        None => None,
        Some(autre) => {
            eprintln!("[Glucose] GLUCOSE_CARTE={autre} inconnu (rapide, econome)");
            None
        }
    }
}

/// Ce que `wgpu` comprend de notre preference.
pub fn pour_wgpu(p: crate::present::arbitre::Preference) -> wgpu::PowerPreference {
    match p {
        crate::present::arbitre::Preference::Econome => wgpu::PowerPreference::LowPower,
        crate::present::arbitre::Preference::Rapide => wgpu::PowerPreference::HighPerformance,
    }
}

/// L'ordre dans lequel on veut que les images se succèdent, le meilleur d'abord.
///
/// # Le défaut que cette liste répare, et il courait depuis le premier jour
///
/// Ce module documentait « `Fifo`, le défaut ». Ce n'en était pas un :
/// `Surface::get_default_config` retient `present_modes.first()`, c'est-à-dire **ce que le
/// pilote annonce en premier**. Sur la machine de l'utilisateur — un Intel Arc — c'est
/// `Immediate`, et la bannière de démarrage le disait sans que personne y prête attention.
///
/// `Immediate` ne synchronise rien : l'image part au milieu d'un balayage. L'instant où elle
/// devient visible est donc arbitraire, et le mouvement calculé pour un instant se voit à un
/// autre. Aucune mesure de coût ne le montre — la cadence reste excellente pendant que le
/// contenu tressaute.
///
/// L'ordre retenu n'a rien d'un compromis :
///
/// * **`Mailbox`** — l'image la plus récente remplace celle qui attend. Pas de déchirure, pas
///   d'attente, et l'affichage tombe sur un balayage. C'est strictement le meilleur des deux.
/// * **`Fifo`** — pas de déchirure non plus, mais `get_current_texture` attend le balayage.
///   Le fil dort pendant ce temps : c'est du repos, pas du travail perdu.
/// * **`FifoRelaxed`** — comme `Fifo`, mais une image en retard part tout de suite plutôt que
///   d'attendre un balayage entier. Déchire seulement quand on a déjà raté.
/// * **`Immediate`** — en dernier, et seulement s'il ne reste que lui.
///
/// `GLUCOSE_PRESENT` garde le dernier mot : mesurer l'effet de ce choix demande de pouvoir
/// revenir à l'ancien comportement sans recompiler.
const PREFERENCES: [wgpu::PresentMode; 4] = [
    wgpu::PresentMode::Mailbox,
    wgpu::PresentMode::Fifo,
    wgpu::PresentMode::FifoRelaxed,
    wgpu::PresentMode::Immediate,
];

/// Choisit la façon dont les images se succèdent : ce qui est demandé, sinon le meilleur
/// disponible.
pub(super) fn cadencer(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
    mut config: wgpu::SurfaceConfiguration,
    cadence: Option<wgpu::PresentMode>,
) -> wgpu::SurfaceConfiguration {
    let possibles = surface.get_capabilities(adapter).present_modes;
    if let Some(voulue) = cadence {
        if possibles.contains(&voulue) {
            config.present_mode = voulue;
            return config;
        }
        eprintln!("[Glucose] cadence {voulue:?} indisponible, on prend la meilleure offerte");
    }
    config.present_mode = meilleure(&possibles).unwrap_or(config.present_mode);
    config
}

/// La première des préférences que cette surface propose.
///
/// Rend `None` si le pilote n'offre aucun des quatre modes connus — auquel cas on garde ce
/// qu'il a proposé plutôt que d'imposer un mode qu'il refuserait.
fn meilleure(possibles: &[wgpu::PresentMode]) -> Option<wgpu::PresentMode> {
    PREFERENCES.iter().copied().find(|m| possibles.contains(m))
}

/// Le nom du mode, pour que la chronique le porte.
///
/// Une chronique qui ne dit pas comment les images se succédaient ne permet pas de relire ses
/// propres chiffres : les mêmes durées ne veulent pas dire la même chose selon que la
/// présentation attendait le balayage ou non.
pub fn nom_de_la_cadence(mode: wgpu::PresentMode) -> &'static str {
    match mode {
        wgpu::PresentMode::Mailbox => "mailbox (sans attente ni dechirure)",
        wgpu::PresentMode::Fifo => "fifo (cale sur le balayage)",
        wgpu::PresentMode::FifoRelaxed => "fifo relache",
        wgpu::PresentMode::Immediate => "immediate (AUCUNE synchronisation)",
        autre => match autre {
            wgpu::PresentMode::AutoVsync => "auto avec synchronisation",
            _ => "auto sans synchronisation",
        },
    }
}
