//! **Changer les codes** (sa note du 26/11, `text-c80i-2481`, et `relais-rep-admin-grades` :
//! *« Changer les codes »* ; fiche 194). Un code donné se reprend : celui qui partage ouvre une
//! salle neuve, sous deux codes neufs, avec le canevas tel qu'il est ; l'ancienne salle reste à
//! ceux qui y sont — ils n'y voient plus rien venir de lui. Il redonne les nouveaux codes à qui doit
//! rester. Un lecteur ne le peut pas : son code ne donne rien.

use crate::app::GlucoseApp;
use crate::ui::question::{Question, Reponse, Suite};

/// « Changer les codes… », dans la fenêtre de Collaborer.
pub(super) const CHANGER: Reponse = Reponse::Choix(7);
/// Sa confirmation.
pub(super) const CHANGER_VRAIMENT: Reponse = Reponse::Choix(9);
const RETOUR: Reponse = Reponse::Choix(15);

impl GlucoseApp {
    /// **Ce que changer les codes fera**, avant de le faire : ceux qui ont les codes d'aujourd'hui
    /// restent dehors.
    pub(super) fn demander_de_changer_les_codes(&mut self) {
        let question = Question {
            titre: "Collaborer › Changer les codes".into(),
            texte:
                "Ce canevas passe dans une salle neuve, sous deux codes neufs. Ceux qui ont les \
                    codes d'aujourd'hui n'y entrent plus, et ne voient plus tes changements : \
                    redonne les nouveaux codes à qui doit rester."
                    .into(),
            choix: vec![
                ("Changer les codes".into(), CHANGER_VRAIMENT),
                ("Retour".into(), RETOUR),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::Collaborer);
    }

    /// **Change les codes** : la salle neuve se prépare d'abord — sans réseau, rien ne change et il
    /// reste dans la sienne ; puis il quitte l'ancienne, et le canevas part dans la neuve, son code
    /// copié. Le souvenir de sa salle (fiche 148) devient celui de la neuve : « Partager », plus
    /// tard, n'y ramène jamais l'ancienne.
    pub(super) fn changer_les_codes(&mut self) {
        let peut = (self.lancement.agents.partage.as_ref()).is_some_and(|cp| cp.code.peut_ecrire());
        if !peut {
            return self.montrer_collaborer();
        }
        let salle = match self.preparer_une_salle() {
            Ok(s) => s,
            Err(e) => return self.montrer_collaborer_avec(Some(e)),
        };
        self.consigner();
        self.quitter_le_canevas_partage();
        self.ouvrir_la_salle(salle);
    }
}
