//! **TEXTE-UNDO-1** : annuler et rétablir *à l'intérieur* d'une saisie.
//!
//! # Ce qui manquait, et pourquoi c'était invisible
//!
//! `Ctrl+Z` existe depuis longtemps et annule une commande du **document** : déplacer un nœud,
//! en créer un, en supprimer un. Mais `handle_text_key` intercepte le clavier **avant** les
//! raccourcis dès qu'une carte est en saisie — et c'est nécessaire, sans quoi taper « n » dans
//! une carte créerait un pense-bête. Résultat : pendant qu'on écrit, `Ctrl+Z` ne faisait
//! *rien du tout*. Ni annuler le texte, ni annuler le document.
//!
//! Signalé après une session de travail réelle : *« les ctrl Z dans les texte pratique pour le
//! traintement de texte sa »*.
//!
//! # Où l'historique vit, et pourquoi pas dans la session
//!
//! Il aurait été naturel de le poser dans [`crate::renderer::TextEditSession`], avec le texte
//! qu'il concerne. **Ce serait très cher** : depuis COMPOSANT-2, la carte en saisie est une
//! texture dont l'empreinte porte une copie de la session. Un historique rangé là serait
//! cloné à chaque frappe, et la mémoire d'une saisie deviendrait quadratique en son nombre
//! d'états.
//!
//! La session porte donc l'état **courant**, et l'application porte son passé.
//!
//! # Où les entrées se coupent, et sans aucune horloge
//!
//! La fiche 05 § 3.6 fixe la règle : *« vingt caractères tapés d'affilée, c'est une entrée »*.
//! Un découpage par délai aurait demandé une constante de temps arbitraire — et il coupe au
//! mauvais endroit dès qu'on hésite au milieu d'un mot.
//!
//! La frontière se lit dans le texte lui-même : **une entrée s'arrête quand la nature de ce
//! qu'on fait change.** Écrire, puis effacer, puis réécrire fait trois entrées. Et un mot
//! commence là où le précédent s'est terminé, donc l'espace reste collé au mot qu'il suit :
//! « Bonjour le monde » s'annule en trois fois, pas en cinq ni en seize.

use glucose_core::text::Selection;

/// Ce que la saisie fait à cet instant. Deux frappes de nature différente n'appartiennent
/// jamais à la même entrée d'annulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nature {
    /// On écrit un mot — tout caractère qui n'est ni espace ni retour à la ligne.
    Mot,
    /// On écrit ce qui sépare deux mots. Il reste collé au mot qui le précède, comme dans
    /// tout traitement de texte : c'est ce qui fait qu'annuler retire un mot entier.
    Separateur,
    /// On efface.
    Effacement,
    /// Un geste qui ne se groupe avec rien : un collage, une coupe.
    Isole,
}

impl Nature {
    /// La nature d'un texte inséré, lue sur son premier caractère.
    pub fn du_texte(texte: &str) -> Self {
        match texte.chars().next() {
            Some(c) if c.is_whitespace() => Self::Separateur,
            Some(_) => Self::Mot,
            None => Self::Isole,
        }
    }

    /// Cette frappe continue-t-elle l'entrée qu'une frappe `precedente` avait ouverte ?
    ///
    /// Un séparateur prolonge le mot qu'il suit ; un mot n'en prolonge jamais un séparateur,
    /// et c'est cette asymétrie qui coupe l'entrée au bon endroit.
    fn prolonge(self, precedente: Self) -> bool {
        match (precedente, self) {
            (Self::Isole, _) | (_, Self::Isole) => false,
            (Self::Mot, Self::Separateur) => true,
            (avant, apres) => avant == apres,
        }
    }
}

/// Un instant de la saisie : ce que la carte montrait, et où le curseur se tenait.
///
/// La sélection en fait partie, et ce n'est pas un détail : annuler doit reposer le curseur
/// là où il était, sinon la frappe suivante reprend au mauvais endroit et l'annulation crée
/// un défaut au lieu d'en réparer un.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instant {
    pub texte: String,
    pub selection: Selection,
}

/// Le passé et l'avenir d'une saisie (TEXTE-UNDO-1).
#[derive(Debug, Default)]
pub struct Historique {
    passe: Vec<Instant>,
    futur: Vec<Instant>,
    derniere: Option<Nature>,
}

impl Historique {
    /// Oublie tout : une autre carte s'ouvre, et son passé n'est pas celui-ci.
    pub fn oublier(&mut self) {
        self.passe.clear();
        self.futur.clear();
        self.derniere = None;
    }

    /// Combien d'annulations sont disponibles — ce que les tests lisent.
    pub fn profondeur(&self) -> usize {
        self.passe.len()
    }

    /// **Note l'état d'avant, si cette frappe ouvre une nouvelle entrée.**
    ///
    /// Rien n'est empilé tant qu'on prolonge ce qu'on faisait déjà : c'est ce qui fait qu'un
    /// mot s'annule d'un coup et non lettre à lettre.
    pub fn noter(&mut self, avant: Instant, nature: Nature) {
        let nouvelle_entree = match self.derniere {
            Some(precedente) => !nature.prolonge(precedente),
            None => true,
        };
        self.derniere = Some(nature);
        if !nouvelle_entree {
            return;
        }
        // Une frappe referme l'avenir : on ne rétablit plus une branche qu'on vient de quitter.
        self.futur.clear();
        if self.passe.last() != Some(&avant) {
            self.passe.push(avant);
        }
    }

    /// Revient d'une entrée. `courant` est l'état présent, qui part dans l'avenir.
    pub fn annuler(&mut self, courant: Instant) -> Option<Instant> {
        let precedent = self.passe.pop()?;
        self.futur.push(courant);
        // La prochaine frappe ouvre forcément une entrée : elle ne prolonge pas ce qu'on
        // vient de défaire.
        self.derniere = None;
        Some(precedent)
    }

    /// Repart d'une entrée vers l'avant.
    pub fn retablir(&mut self, courant: Instant) -> Option<Instant> {
        let suivant = self.futur.pop()?;
        self.passe.push(courant);
        self.derniere = None;
        Some(suivant)
    }
}

#[cfg(test)]
mod tests;
