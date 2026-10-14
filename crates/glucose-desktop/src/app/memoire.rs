//! **La mémoire que Glucose tient : la voir, et la rendre quand le système la demande**
//! (MEMOIRE-1 et 2, fiche 66).
//!
//! # Ce que son téléphone n'a pas pu dire
//!
//! Le 13/10, son téléphone s'est figé tout entier — le bouton d'alimentation compris — après
//! des saccades qui montaient, puis a redémarré seul. Aucune ligne de la boîte noire ne disait
//! la mémoire : ni ce que Glucose tenait, ni ce que le système laissait. Sous Android,
//! l'empreinte du processus ne se mesurait même pas (EMPREINTE-1 ne connaissait que `linux`).
//!
//! # Ce qu'Android dit, et ce qu'il ne dit plus
//!
//! `onTrimMemory` demande de rendre ce qui se reconstruit. GameActivity le transmet comme une
//! « mémoire basse » sans son niveau, et `winit` l'appelle `memory_warning` : Glucose ne
//! l'écoutait pas. Depuis Android 14, une application **au premier plan** n'en reçoit plus
//! aucun niveau de pression — seulement « l'interface est cachée » et « en arrière-plan »
//! ([ComponentCallbacks2](https://developer.android.com/reference/android/content/ComponentCallbacks2)).
//! Au premier plan, c'est donc à Glucose de ne jamais prendre plus que ce que la machine
//! laisse ; ce module ne fait qu'obéir quand le système parle, et le noter.

use super::GlucoseApp;
use crate::boite_noire::enregistrement::Occupation;
use crate::renderer::photo::Etat;

impl GlucoseApp {
    /// **Ce que Glucose occupe, et ce que la machine laisse**, à cet instant — chaque grandeur
    /// lue à sa source.
    pub(crate) fn occupation(&self) -> Occupation {
        let magasin = &self.renderer.magasin;
        Occupation {
            processus_mo: crate::plateforme::empreinte::relever()
                .map(|e| Occupation::mo(e.memoire_octets)),
            disponible_mo: crate::memoire::Memoire::du_systeme()
                .map(|m| Occupation::mo(m.disponible)),
            images_mo: Occupation::mo(magasin.octets_en(Etat::Tenu) as u64),
            offerts_mo: Occupation::mo(magasin.octets_en(Etat::Offert) as u64),
            decodages_mo: Occupation::mo(
                crate::renderer::atelier::admission::de_la_machine().en_vol(),
            ),
            carte_mo: self
                .presenter
                .as_ref()
                .and_then(|p| p.octets_sur_la_carte())
                .map(Occupation::mo),
        }
    }

    /// **Ce que le processus coûte à la machine**, relevé au rythme de la veille (EMPREINTE-1) :
    /// l'empreinte et les étages pour la chronique, et au même instant, pour la boîte noire qui
    /// voyage, un record de la mémoire (MEMOIRE-1).
    pub(super) fn veiller_sur_la_machine(&mut self) {
        let compte = crate::chronique::veille::Compte {
            sous_la_main: self.provenance.evenements_de_la_main(),
            rendues: self.chronique.rendues(),
            par_raison: self.chronique.reveils_par_bit(),
        };
        if self
            .chronique
            .veille
            .observer(std::time::Instant::now(), compte)
        {
            self.relever_les_etages();
            self.noter_la_memoire(false);
        }
    }

    /// **Pour la boîte noire** : une ligne si c'est un record, ou si on la force.
    pub(super) fn noter_la_memoire(&mut self, forcer: bool) {
        let occupation = self.occupation();
        self.chronique.noter_la_memoire(occupation, forcer);
    }

    /// **Le système demande de rendre de la mémoire** (MEMOIRE-2).
    ///
    /// L'alerte s'écrit d'abord, avec ce que Glucose tenait à cet instant — c'est elle qui dira,
    /// d'un téléphone qu'on ne possède pas, quand le système a parlé et de combien il manquait.
    /// Puis la carte rend ce qui se reconstruit ; et derrière une autre application, le magasin
    /// offre tout au système, qui reprendra ce qu'il lui faut sans rien écrire nulle part.
    pub(crate) fn rendre_la_memoire(&mut self) {
        let derriere = self.en_arriere_plan();
        self.chronique.noter_l_alerte_memoire(derriere);
        self.noter_la_memoire(true);
        if let Some(presenter) = self.presenter.as_mut() {
            presenter.rendre_la_memoire(derriere);
        }
        if derriere {
            self.renderer.magasin.offrir_tout();
        }
    }
}

#[cfg(test)]
mod tests;
