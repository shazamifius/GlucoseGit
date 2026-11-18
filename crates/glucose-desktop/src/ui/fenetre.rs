//! **FENETRE-1 — les trois modes de la fenêtre** (fiche 120).
//!
//! Sa capture du 17/11 : *« ce qui serait bien, ce serait d'avoir un mode fenêtre classique, qui
//! est l'état actuel de Glucose sur Linux ; un mode qui force le plein écran ; et un mode qui force
//! le fait d'être au-dessus de toutes les applications et permet de faire exactement comme
//! Windows »*. Trois modes, **exclusifs** — une seule valeur, jamais deux drapeaux qui pourraient
//! se contredire —, retenus d'une session à l'autre :
//!
//! * **normale** : la fenêtre du système ;
//! * **plein écran** (`F11`, la touche de tous les logiciels de bureau) : la fenêtre couvre l'écran,
//!   l'interface reste ; le mode zen, qui cache l'interface sans toucher à la fenêtre, passe à
//!   `Maj+F11` — les deux se combinent ;
//! * **référence** (`Ctrl+Maj+A`, le geste de PureRef) : sans cadre, sans interface, toujours
//!   devant les autres applications — sous Wayland, par la voie de chaque bureau (DEVANT-1,
//!   `plateforme/devant.rs`).
//!
//! Le même geste défait son mode ; un autre geste passe d'un mode à l'autre.

/// Le mode de la fenêtre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModeDeFenetre {
    #[default]
    Normale,
    PleinEcran,
    Reference,
}

impl ModeDeFenetre {
    /// **Le mode après le geste qui demande `voulu`** : le même geste le défait.
    pub fn apres(self, voulu: Self) -> Self {
        if self == voulu {
            Self::Normale
        } else {
            voulu
        }
    }

    /// Ce que le souvenir écrit ; rien pour la fenêtre normale.
    pub fn souvenir(self) -> Option<&'static str> {
        match self {
            Self::Normale => None,
            Self::PleinEcran => Some("plein-ecran"),
            Self::Reference => Some("reference"),
        }
    }

    /// **Ce qu'un souvenir dit** — `1`, ce qu'écrivait le mode référence avant FENETRE-1.
    pub fn lire(souvenir: &str) -> Self {
        match souvenir.trim() {
            "plein-ecran" => Self::PleinEcran,
            "reference" | "1" => Self::Reference,
            _ => Self::Normale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ModeDeFenetre::*;
    use super::*;

    #[test]
    fn test_fenetre_1_le_meme_geste_defait_un_autre_change() {
        assert_eq!(Normale.apres(PleinEcran), PleinEcran);
        assert_eq!(PleinEcran.apres(PleinEcran), Normale);
        assert_eq!(PleinEcran.apres(Reference), Reference);
        assert_eq!(Reference.apres(Reference), Normale);
    }

    #[test]
    fn test_fenetre_1_le_souvenir_se_relit_et_lit_celui_d_avant() {
        for mode in [Normale, PleinEcran, Reference] {
            assert_eq!(ModeDeFenetre::lire(mode.souvenir().unwrap_or("")), mode);
        }
        assert_eq!(
            ModeDeFenetre::lire("1"),
            Reference,
            "le souvenir du mode référence d'avant"
        );
        assert_eq!(ModeDeFenetre::lire("n'importe quoi"), Normale);
    }
}
