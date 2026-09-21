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
    match std::env::var("GLUCOSE_CARTE")
        .ok()
        .as_deref()
        .map(|v| v.trim().to_lowercase())
        .as_deref()
    {
        Some("rapide") => wgpu::PowerPreference::HighPerformance,
        Some("econome") | None => wgpu::PowerPreference::LowPower,
        Some(autre) => {
            eprintln!("[Glucose] GLUCOSE_CARTE={autre} inconnu (rapide, econome)");
            wgpu::PowerPreference::LowPower
        }
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
