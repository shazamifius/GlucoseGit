//! **Comment Android a fermé la session d'avant** (SORTIE-1, fiche 72).
//!
//! Le pendant de [`super::plantage`] pour le téléphone : là où Windows note un plantage dans son
//! journal, Android garde la raison de chaque mort d'un processus de l'application, et la
//! mémoire qu'il occupait alors ([`crate::plateforme::sortie`]). C'est la seule voix qui reste
//! à une application qu'on tue : elle-même n'écrit rien.
//!
//! # Reconnaître la session, sans seuil
//!
//! Le système ne garde que les morts des processus **de Glucose**. Celle de la session d'avant
//! porte son numéro, et est arrivée **après** son début : la plus ancienne de ces morts-là est la
//! sienne — un processus plus tardif du même numéro serait né après elle, et il n'y en a pas
//! d'autre de vivant que celui qui lit.
//!
//! # Ce qui s'en écrit
//!
//! Des nombres, et la raison par un nom d'une liste fermée — celle qu'Android définit
//! (`ApplicationExitInfo.REASON_*`). Sa description, un texte du système, ne se lit pas.

use crate::plateforme::sortie::Sortie;

/// Ce qu'Android a vu de la fin d'une session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fermeture {
    /// La raison, par son nom fermé ([`raison`]).
    pub raison: &'static str,
    /// Le code de sortie, ou le signal ; absent s'il est négatif.
    pub statut: Option<u64>,
    /// L'importance du processus à sa mort : 100 au premier plan, 200 visible, 400 en cache.
    pub importance: u64,
    /// Sa mémoire à sa mort, en mébioctets.
    pub pss_mo: u64,
    pub rss_mo: u64,
}

/// L'importance au-delà de laquelle un processus n'est plus vu par l'utilisateur
/// (`IMPORTANCE_CACHED`, 400) : y mourir est la fin ordinaire d'une application de téléphone.
const EN_CACHE: u64 = 400;

/// `REASON_LOW_MEMORY`, `REASON_CRASH`, `REASON_CRASH_NATIVE`, `REASON_ANR`.
const MEMOIRE_BASSE: i32 = 3;
const PLANTAGE_JAVA: i32 = 4;
const PLANTAGE_NATIF: i32 = 5;
const GEL: i32 = 6;

/// **Le nom fermé d'une raison d'Android** — `ApplicationExitInfo.REASON_*`, dans l'ordre de
/// leurs valeurs (0 à 16) ; une raison qu'Android ajoutera se dit « inconnue ».
pub fn raison(code: i32) -> &'static str {
    match code {
        1 => "sortie_d_elle_meme",
        2 => "signal",
        MEMOIRE_BASSE => "memoire_basse",
        PLANTAGE_JAVA => "plantage_java",
        PLANTAGE_NATIF => "plantage_natif",
        GEL => "gel",
        7 => "echec_au_demarrage",
        8 => "permission_changee",
        9 => "ressources_excessives",
        10 => "demandee_par_l_utilisateur",
        11 => "arretee_par_l_utilisateur",
        12 => "dependance_morte",
        13 => "autre",
        14 => "gelee_par_le_systeme",
        15 => "etat_du_paquet",
        16 => "mise_a_jour",
        _ => "inconnue",
    }
}

/// Toutes les raisons qu'on peut écrire : les épreuves vérifient que le serveur les connaît.
pub fn toutes_les_raisons() -> Vec<&'static str> {
    (0..=17).map(raison).collect()
}

/// **La mort de la session** que ce processus a ouverte à `debut_ms`, parmi celles que le
/// système a gardées : la plus ancienne qui porte son numéro et vient après son début.
pub fn reconnaitre(sorties: &[Sortie], processus: u32, debut_ms: u64) -> Option<Fermeture> {
    sorties
        .iter()
        .filter(|s| s.processus == processus && s.instant_ms >= debut_ms)
        .min_by_key(|s| s.instant_ms)
        .map(|s| Fermeture {
            raison: raison(s.raison),
            statut: u64::try_from(s.statut).ok(),
            importance: u64::try_from(s.importance).unwrap_or(0),
            pss_mo: s.pss_ko >> 10,
            rss_mo: s.rss_ko >> 10,
        })
}

impl Fermeture {
    /// Tuée faute de mémoire pendant qu'on la regardait — et non en cache, où c'est ordinaire.
    pub fn faute_de_memoire(&self) -> bool {
        self.raison == raison(MEMOIRE_BASSE) && self.importance < EN_CACHE
    }

    /// Plantée, en Java ou dans le code natif (Rust, un pilote).
    pub fn plantee(&self) -> bool {
        self.raison == raison(PLANTAGE_JAVA) || self.raison == raison(PLANTAGE_NATIF)
    }

    /// Gelée, puis fermée par le système.
    pub fn gelee(&self) -> bool {
        self.raison == raison(GEL)
    }

    /// Ce qu'on en dit, en une proposition.
    pub fn dire(&self) -> String {
        let quoi = match self.raison {
            "memoire_basse" => "tuée par Android faute de mémoire",
            "plantage_java" | "plantage_natif" => "plantée",
            "gel" => "gelée, puis fermée par Android",
            "demandee_par_l_utilisateur" | "arretee_par_l_utilisateur" => {
                "fermée par l'utilisateur"
            }
            _ => "fermée par Android",
        };
        format!("{quoi} ({} Mo occupés)", self.rss_mo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mort(processus: u32, instant_ms: u64, raison: i32, importance: i32) -> Sortie {
        Sortie {
            processus,
            instant_ms,
            raison,
            statut: 9,
            importance,
            pss_ko: 1_500 << 10,
            rss_ko: 2_000 << 10,
        }
    }

    /// **La session reconnaît sa mort, et seulement elle** : même numéro, après son début, la
    /// plus ancienne — ni celle d'un autre processus, ni celle d'avant son début.
    #[test]
    fn test_sortie_1_la_session_reconnait_sa_mort() {
        let sorties = [
            mort(7, 5_000, PLANTAGE_JAVA, 100), // un autre processus
            mort(42, 900, GEL, 100),            // ce numéro, avant le début : un autre
            mort(42, 3_000, MEMOIRE_BASSE, 100),
            mort(42, 4_000, PLANTAGE_NATIF, 100), // plus tard : né après elle
        ];
        let f = reconnaitre(&sorties, 42, 1_000).expect("sa mort");
        assert_eq!(f.raison, "memoire_basse");
        assert_eq!((f.pss_mo, f.rss_mo, f.statut), (1_500, 2_000, Some(9)));
        assert!(f.faute_de_memoire());
        assert_eq!(reconnaitre(&sorties, 43, 1_000), None);
    }

    /// **Mourir en cache faute de mémoire est ordinaire** ; au premier plan, non.
    #[test]
    fn test_sortie_1_la_memoire_basse_en_cache_est_ordinaire() {
        let au_premier_plan = reconnaitre(&[mort(1, 10, MEMOIRE_BASSE, 100)], 1, 0).unwrap();
        let en_cache = reconnaitre(&[mort(1, 10, MEMOIRE_BASSE, 400)], 1, 0).unwrap();
        assert!(au_premier_plan.faute_de_memoire());
        assert!(!en_cache.faute_de_memoire());
    }

    /// Une session ouverte puis lâchée sans sa fin — ce que laisse un processus qu'on tue — et
    /// la mort qu'Android en aurait gardée, avec cette raison et cette importance.
    fn session_tuee(
        d: &crate::epreuve::Dossier,
        raison: i32,
        importance: i32,
    ) -> (std::path::PathBuf, Sortie) {
        let (b, _) =
            super::super::BoiteNoire::ouvrir_avec(d, |_| Vec::new(), Vec::new).expect("ouverte");
        b.synchroniser();
        let tombee = b.chemin().to_path_buf();
        drop(b);
        let nom = tombee.file_name().unwrap().to_str().unwrap();
        let processus = super::super::fichiers::processus_de(nom).expect("une session");
        let debut = super::super::fichiers::debut_du_fichier(&tombee).expect("son début");
        (tombee, mort(processus, debut + 60_000, raison, importance))
    }

    /// **Tuée faute de mémoire sous les yeux, elle se dit telle au lancement suivant** — et la
    /// session suivante écrit sa ligne `sortie`, pour que ce bilan voyage (SORTIE-1).
    #[test]
    fn test_sortie_1_une_session_tuee_faute_de_memoire_le_dit() {
        let d = crate::epreuve::Dossier::nouveau("sortie-memoire");
        let (_, sa_mort) = session_tuee(&d, MEMOIRE_BASSE, 100);
        let (seconde, precedente) =
            super::super::BoiteNoire::ouvrir_avec(&d, |_| Vec::new(), move || vec![sa_mort])
                .expect("rouverte");
        let p = precedente.expect("la session d'avant");
        assert_eq!(p.fin, super::super::bilan::Fin::FauteDeMemoire);
        assert!(
            p.dire()
                .starts_with("tuée par Android faute de mémoire (2000 Mo"),
            "{}",
            p.dire()
        );
        seconde.synchroniser();
        let texte = std::fs::read_to_string(seconde.chemin()).expect("lisible");
        let ligne = texte
            .lines()
            .find(|l| l.contains("\"type\":\"sortie\""))
            .expect("une ligne sortie");
        assert!(ligne.contains("\"raison\":\"memoire_basse\""), "{ligne}");
        assert!(ligne.contains("\"rss_mo\":2000"), "{ligne}");
        assert!(texte.contains("\"fin\":\"faute_de_memoire\""), "{texte}");
    }

    /// **Plantée derrière une autre application, elle est plantée** ; tuée en cache faute de
    /// mémoire, elle garde sa fin — Android y tue qui il veut.
    #[test]
    fn test_sortie_1_ce_qu_android_dit_change_la_fin_au_besoin() {
        let d = crate::epreuve::Dossier::nouveau("sortie-plantee");
        let (_, plantee) = session_tuee(&d, PLANTAGE_NATIF, 400);
        let (_, p) =
            super::super::BoiteNoire::ouvrir_avec(&d, |_| Vec::new(), move || vec![plantee])
                .expect("rouverte");
        assert_eq!(p.expect("bilan").fin, super::super::bilan::Fin::Plantee);

        let d = crate::epreuve::Dossier::nouveau("sortie-cache");
        let (_, en_cache) = session_tuee(&d, MEMOIRE_BASSE, 400);
        let (_, p) =
            super::super::BoiteNoire::ouvrir_avec(&d, |_| Vec::new(), move || vec![en_cache])
                .expect("rouverte");
        let p = p.expect("bilan");
        assert_eq!(p.fin, super::super::bilan::Fin::Interrompue);
        assert_eq!(p.sortie.map(|f| f.raison), Some("memoire_basse"));
    }

    /// **Chaque raison d'Android a son nom**, distinct ; une raison inconnue se dit telle.
    #[test]
    fn test_sortie_1_chaque_raison_a_son_nom() {
        let mut noms: Vec<&str> = (0..=16).map(raison).collect();
        noms.sort_unstable();
        noms.dedup();
        assert_eq!(noms.len(), 17);
        assert_eq!(raison(99), "inconnue");
        assert_eq!(raison(0), "inconnue");
    }
}
