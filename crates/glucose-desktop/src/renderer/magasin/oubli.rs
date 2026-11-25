//! **Un fichier changé sur le disque** (DISQUE-2, fiche 164 § 9) : ce que le magasin en savait
//! s'oublie, et il se redécode à sa prochaine demande — un rendu refait sous le même nom dans un
//! dossier miroir montre la nouvelle image, pas l'ancienne.

use super::Magasin;

impl Magasin {
    /// **Oublie ce fichier** : sa pyramide (rendue à la mémoire sur un ouvrier, comme une éviction),
    /// ses vignettes, son échec et son orientation. Son aperçu sur le disque n'a pas à s'oublier :
    /// il est nommé par la taille et la date du fichier (`renderer::apercu`).
    pub fn oublier(&mut self, src: &str) {
        if let Some(partie) = self.cache.remove(src) {
            self.jeter(partie.pyramide);
            self.changements += 1;
        }
        self.vignettes.oublier_la_source(src);
        self.echecs.remove(src);
        self.tournees.remove(src);
    }
}
