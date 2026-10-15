//! **Un accord** : des modificateurs et une touche — et la loi qui dit lesquels comptent.
//!
//! # ACCORD-1 — ce qui compte, et pourquoi (le clavier français)
//!
//! Le code d'avant ignorait Alt et Maj sur les signes et les chiffres, sans le dire : c'est ce
//! qui faisait marcher Glucose sur un clavier AZERTY. La loi l'écrit une fois :
//!
//! * **Ctrl** compte toujours ;
//! * **Alt** ne compte que sans Ctrl : sous Windows, AltGr arrive comme Ctrl+Alt, et il a déjà
//!   produit son caractère — `Ctrl+]` se tape AltGr+`)` en AZERTY, et doit rester `Ctrl+]` ;
//! * **Maj** ne compte que pour une lettre ou une touche nommée (`Ctrl+Maj+Z`, `Maj+Gauche`) :
//!   pour un signe, il l'a produit (`?` est Maj+`,`) ; pour un chiffre, en AZERTY, il est
//!   nécessaire pour l'écrire, et un raccourci numérique est **positionnel** ;
//! * **un chiffre se lit sur la touche physique** : `1` est la touche marquée 1, quel que soit
//!   le clavier (c'est à l'application de le lire ainsi).
//!
//! Un accord se construit toujours par [`Accord::de`], qui applique la loi : deux accords
//! égaux sont le même geste, au clavier comme dans un fichier.

/// Une touche nommée, celles que le clavier n'écrit pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Nommee {
    Espace,
    Echap,
    Entree,
    Tab,
    Suppr,
    Effacer,
    Gauche,
    Droite,
    Haut,
    Bas,
    Debut,
    Fin,
    PageHaut,
    PageBas,
    F(u8),
}

impl Nommee {
    /// Les touches nommées sans paramètre, pour lire et écrire leurs noms.
    const SIMPLES: [(Nommee, &'static str, &'static str); 14] = [
        (Nommee::Espace, "Espace", "Espace"),
        (Nommee::Echap, "Echap", "Échap"),
        (Nommee::Entree, "Entree", "Entrée"),
        (Nommee::Tab, "Tab", "Tab"),
        (Nommee::Suppr, "Suppr", "Suppr"),
        (Nommee::Effacer, "Effacer", "⌫"),
        (Nommee::Gauche, "Gauche", "←"),
        (Nommee::Droite, "Droite", "→"),
        (Nommee::Haut, "Haut", "↑"),
        (Nommee::Bas, "Bas", "↓"),
        (Nommee::Debut, "Debut", "Début"),
        (Nommee::Fin, "Fin", "Fin"),
        (Nommee::PageHaut, "PageHaut", "Pg↑"),
        (Nommee::PageBas, "PageBas", "Pg↓"),
    ];

    /// Son nom dans un fichier : sans accent, sans espace.
    fn nom(self) -> String {
        match self {
            Nommee::F(n) => format!("F{n}"),
            autre => Self::SIMPLES
                .iter()
                .find(|(n, _, _)| *n == autre)
                .map_or_else(String::new, |(_, nom, _)| (*nom).to_string()),
        }
    }

    /// Son nom à l'écran.
    fn dit(self) -> String {
        match self {
            Nommee::F(n) => format!("F{n}"),
            autre => Self::SIMPLES
                .iter()
                .find(|(n, _, _)| *n == autre)
                .map_or_else(String::new, |(_, _, dit)| (*dit).to_string()),
        }
    }

    fn lire(texte: &str) -> Option<Self> {
        if let Some((n, _, _)) = Self::SIMPLES
            .iter()
            .find(|(_, nom, dit)| texte.eq_ignore_ascii_case(nom) || texte == *dit)
        {
            return Some(*n);
        }
        let n: u8 = texte.strip_prefix('F')?.parse().ok()?;
        (1..=24).contains(&n).then_some(Nommee::F(n))
    }
}

/// Une touche : une lettre (en minuscule), un chiffre (la touche physique), un signe, ou une
/// touche nommée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Touche {
    Lettre(char),
    Chiffre(u8),
    Signe(char),
    Nommee(Nommee),
}

impl Touche {
    /// La touche d'un caractère : une lettre se range en minuscule, un chiffre est un chiffre.
    pub fn du_caractere(c: char) -> Self {
        if let Some(n) = c.to_digit(10) {
            Touche::Chiffre(n as u8)
        } else if c.is_alphabetic() {
            Touche::Lettre(c.to_lowercase().next().unwrap_or(c))
        } else {
            Touche::Signe(c)
        }
    }

    /// Maj compte-t-il pour elle (ACCORD-1) ?
    fn prend_maj(self) -> bool {
        matches!(self, Touche::Lettre(_) | Touche::Nommee(_))
    }
}

/// **Un accord** : Ctrl, Alt, Maj, et une touche — toujours construit par [`Accord::de`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Accord {
    ctrl: bool,
    alt: bool,
    maj: bool,
    touche: Touche,
}

impl Accord {
    /// **L'accord de ce geste**, la loi ACCORD-1 appliquée : ce qui ne compte pas tombe.
    pub fn de(ctrl: bool, alt: bool, maj: bool, touche: Touche) -> Self {
        Self {
            ctrl,
            alt: alt && !ctrl,
            maj: maj && touche.prend_maj(),
            touche,
        }
    }

    pub fn ctrl(&self) -> bool {
        self.ctrl
    }

    pub fn alt(&self) -> bool {
        self.alt
    }

    pub fn maj(&self) -> bool {
        self.maj
    }

    pub fn touche(&self) -> Touche {
        self.touche
    }

    /// **Lit un accord écrit** — `Ctrl+Maj+Z`, `Alt+F`, `1`, `Ctrl+]`, `Maj+Gauche`. Ce que la
    /// loi ferait tomber est refusé, avec sa raison : un raccourci qui ne peut jamais se
    /// déclencher ne s'écrit pas en silence.
    pub fn lire(texte: &str) -> Result<Self, String> {
        let texte = texte.trim();
        // La touche est le dernier morceau ; `Ctrl++` et `+` nomment la touche `+`.
        let (mods, touche) = match texte.strip_suffix('+') {
            Some(m) => (m, "+"),
            None => texte.rsplit_once('+').unwrap_or(("", texte)),
        };
        let (mut ctrl, mut alt, mut maj) = (false, false, false);
        for m in mods.split('+').filter(|m| !m.is_empty()) {
            match m.to_lowercase().as_str() {
                "ctrl" | "control" => ctrl = true,
                "alt" => alt = true,
                "maj" | "shift" => maj = true,
                autre => {
                    return Err(format!(
                        "« {autre} » n'est pas un modificateur (Ctrl, Alt, Maj)"
                    ))
                }
            }
        }
        let mut lettres = touche.chars();
        let touche = match (lettres.next(), lettres.next()) {
            (Some(c), None) => Touche::du_caractere(c),
            (Some(_), Some(_)) => Touche::Nommee(
                Nommee::lire(touche).ok_or_else(|| format!("« {touche} » n'est pas une touche"))?,
            ),
            (None, _) => return Err("un accord sans touche".to_string()),
        };
        let accord = Self::de(ctrl, alt, maj, touche);
        if alt && ctrl {
            return Err("Ctrl+Alt est AltGr sous Windows : il écrit un caractère".to_string());
        }
        if maj && !accord.maj {
            return Err("Maj ne compte pas pour un signe ou un chiffre : il les écrit".to_string());
        }
        Ok(accord)
    }

    /// **L'accord écrit**, tel qu'un fichier le porte et que [`Self::lire`] le relit.
    pub fn ecrire(&self) -> String {
        self.avec(match self.touche {
            Touche::Lettre(c) => c.to_uppercase().collect(),
            Touche::Chiffre(n) => n.to_string(),
            Touche::Signe(c) => c.to_string(),
            Touche::Nommee(n) => n.nom(),
        })
    }

    /// **L'accord dit à l'écran** — `Ctrl+Z`, `Maj+←`, `Suppr` : ce que le menu et la palette
    /// montrent à côté d'une commande.
    pub fn dire(&self) -> String {
        self.avec(match self.touche {
            Touche::Nommee(n) => n.dit(),
            _ => return self.ecrire(),
        })
    }

    fn avec(&self, touche: String) -> String {
        let mut t = String::new();
        for (oui, nom) in [(self.ctrl, "Ctrl+"), (self.alt, "Alt+"), (self.maj, "Maj+")] {
            if oui {
                t.push_str(nom);
            }
        }
        t + &touche
    }
}
