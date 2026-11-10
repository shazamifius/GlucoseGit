//! **Ce que la carte reconnaît** : une photo par son numéro, tout le reste par son nom (PHOTO-1,
//! fiche 87).
//!
//! Une photo se désignait par une chaîne fabriquée à chaque image (`format!("{src}@{facteur}")`),
//! rangée, clonée, hachée et comparée lettre à lettre par chaque étape : près des deux tiers du fil
//! qui dessine sur huit mille images, alors que l'envoi à la carte n'en coûtait pas 4 %. Elle est
//! désormais deux entiers, que tout le monde copie et compare sans lire un seul chemin.
//!
//! Ce qui n'a pas de numéro garde un nom : un composant dit ce qu'il montre par une empreinte, et
//! il y en a peu à l'écran, puisqu'il se dessine lui-même.

use crate::renderer::magasin::Numero;

/// La clé d'une texture ou son identité : ce qu'elle montre, ou ce qu'elle est.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cle {
    /// **Ce niveau de cette photo** — ce qu'elle montre : le fichier, réduit tant de fois.
    Niveau(Numero, u32),
    /// **Cette photo** — ce qu'elle est : ses niveaux sont le même objet à deux finesses, et
    /// l'ancien se pose pendant que le nouveau se téléverse.
    Photo(Numero),
    /// **Le repli de cette photo** : une identité à lui, qui ne remplace jamais une texture nette
    /// gardée sous celle de la photo (ETAGES-1).
    Repli(Numero),
    /// Tout le reste : un composant, la chrome, un panneau.
    Nom(String),
}

impl Cle {
    /// **Ce que cette clé montre est-il une photo ?** Un niveau prêté par le magasin, celui de la
    /// photo ou de son repli (PHOTO-2). Ce qui arrive tout prêt sans en être un — la chrome — n'en
    /// est pas : la minimap se comptait parmi les photos posées (ETAGES-3), parce que ses pixels
    /// arrivaient prêts comme ceux du magasin.
    pub fn est_une_photo(&self) -> bool {
        matches!(self, Self::Niveau(..))
    }

    /// Le nom de ce qui en a un.
    pub fn nom(&self) -> Option<&str> {
        match self {
            Self::Nom(nom) => Some(nom),
            _ => None,
        }
    }
}

impl From<&str> for Cle {
    fn from(nom: &str) -> Self {
        Self::Nom(nom.to_string())
    }
}

impl From<&String> for Cle {
    fn from(nom: &String) -> Self {
        Self::Nom(nom.clone())
    }
}

impl From<String> for Cle {
    fn from(nom: String) -> Self {
        Self::Nom(nom)
    }
}

impl PartialEq<str> for Cle {
    fn eq(&self, autre: &str) -> bool {
        self.nom() == Some(autre)
    }
}

impl PartialEq<&str> for Cle {
    fn eq(&self, autre: &&str) -> bool {
        self.nom() == Some(*autre)
    }
}
