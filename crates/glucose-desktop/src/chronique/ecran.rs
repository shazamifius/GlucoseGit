//! **Ce que l'écran a montré** (PARU-1, fiche 112) — le compositeur le dit, image par image.
//!
//! # Le défaut que ce module corrige
//!
//! Sa session du 12/11 à 18 h 16 : la chronique disait « 516 images ont raté leur balayage
//! (32,1 % des images en mouvement) », et la fiche 111 en a conclu que le rythme entre Glucose et
//! niri faisait son lag. Ce chiffre était celui du **tempo** : une image prête après l'heure que
//! visait SA grille, qui n'est pas celle de l'écran. Rien ici ne savait ce que l'écran montrait :
//! le rythme (RYTHME-1) prend l'instant où la présentation rend la main — et sous Wayland, en
//! `mailbox`, elle la rend tout de suite ; l'image paraît deux battements plus tard, ou jamais.
//! Mesuré à son écran le 12/11 au soir, par ce module : 93 à 99 % des battements montraient une
//! image neuve.
//!
//! # Ce que le compositeur dit
//!
//! `wp_presentation`, sous Wayland : pour chaque image confiée, il répond plus tard — **montrée**,
//! à tel instant, au battement numéro tant ; ou **jetée**, remplacée avant d'avoir paru. Une
//! plateforme qui sait l'entendre branche une [`Sonde`] ; sans elle, la section se tait plutôt que
//! d'inventer.
//!
//! # Ce qui se compte
//!
//! * les images montrées et jetées — une jetée est un travail perdu, et un saut du contenu ;
//! * **les battements sautés** : un battement où l'écran a remontré l'image d'avant alors que la
//!   suivante était déjà demandée — son échéance (GEL-1) précède l'instant où la précédente a
//!   paru. Hors de ce cas rien n'était dû, et un battement sans image neuve est le repos : il n'y a
//!   aucun seuil à choisir ;
//! * le temps de l'envoi à l'écran, au centile : la latence que la présentation ajoute, et sa
//!   **gigue** — une image montrée un battement plus tard que sa voisine se voit comme un tressaut.

use super::histogramme::Histogramme;
use std::collections::VecDeque;

/// **Ce que le compositeur a fait d'une image confiée.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retour {
    /// L'instant de l'envoi, en microsecondes, sur l'horloge du compositeur.
    pub envoi_us: u64,
    /// L'image a paru — ou `None` : elle a été jetée.
    pub vue: Option<Vue>,
}

/// **L'instant où une image a paru.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vue {
    /// Le début de son balayage, même horloge que l'envoi.
    pub us: u64,
    /// Le numéro du battement — zéro quand l'écran ne les compte pas.
    pub battement: u64,
    /// La période de l'écran en nanosecondes — zéro quand elle varie.
    pub periode_ns: u32,
}

/// **Ce qui demande au compositeur ce qu'il fera d'une image** : `wp_presentation` sous Wayland.
pub trait Sonde: Send {
    /// Juste avant de confier une image : la demande, et l'instant de l'envoi sur l'horloge du
    /// compositeur.
    fn envoyer(&self) -> u64;
    /// Les verdicts arrivés depuis la dernière relève.
    fn relever(&self) -> Vec<Retour>;
}

/// **Ce que l'écran a montré**, cumulé sur la session.
#[derive(Default)]
pub struct Ecran {
    sonde: Option<Box<dyn Sonde>>,
    /// Les envois qui attendent leur verdict, dans l'ordre : l'instant de l'envoi, et
    /// l'échéance de l'image si elle était due — sur la même horloge.
    en_attente: VecDeque<(u64, Option<u64>)>,
    /// La dernière image montrée.
    derniere: Option<Vue>,
    montrees: u64,
    jetees: u64,
    /// Les battements où une image neuve était due, et ceux où l'écran a remontré l'ancienne.
    dus: u64,
    sautes: u64,
    /// De l'envoi à l'écran, en microsecondes.
    latence: Histogramme,
}

impl Ecran {
    /// Branche la sonde de cette fenêtre, s'il y en a une.
    pub fn brancher(&mut self, sonde: Option<Box<dyn Sonde>>) {
        self.sonde = sonde;
    }

    /// **Juste avant de confier une image** : relever ce qui est arrivé, puis demander ce que le
    /// compositeur fera de celle-ci. `due` est son échéance, si elle en avait une (GEL-1).
    pub fn avant_d_envoyer(&mut self, due: Option<std::time::Instant>) {
        let Some(sonde) = self.sonde.as_ref() else {
            return;
        };
        let retours = sonde.relever();
        let envoi = sonde.envoyer();
        // L'échéance passe sur l'horloge du compositeur par son âge : une durée ne dépend
        // d'aucune horloge.
        let du = due.map(|d| envoi.saturating_sub(micro(d.elapsed())));
        for r in retours {
            self.recevoir(r);
        }
        self.en_attente.push_back((envoi, du));
    }

    /// **La surface a refusé l'image** : rien n'est parti, et la demande déjà faite vaudra pour
    /// l'envoi suivant — son verdict ne retrouvera pas son envoi, et ne comptera pas.
    pub fn rien_n_est_parti(&mut self) {
        self.en_attente.pop_back();
    }

    /// **Un verdict.** Ils arrivent dans l'ordre des envois : ceux qui précèdent un verdict sans
    /// en avoir eu ne l'auront plus, et sortent de l'attente — elle ne grandit donc jamais.
    pub fn recevoir(&mut self, retour: Retour) {
        while self
            .en_attente
            .front()
            .is_some_and(|(e, _)| *e < retour.envoi_us)
        {
            self.en_attente.pop_front();
        }
        let Some((_, du)) = self
            .en_attente
            .front()
            .copied()
            .filter(|(e, _)| *e == retour.envoi_us)
        else {
            return;
        };
        self.en_attente.pop_front();
        let Some(vue) = retour.vue else {
            self.jetees += 1;
            return;
        };
        self.montrees += 1;
        self.latence
            .ajouter(sature(vue.us.saturating_sub(retour.envoi_us)));
        if let (Some(avant), Some(du)) = (self.derniere, du) {
            if du <= avant.us {
                let ecart = battements_entre(avant, vue);
                self.dus += ecart;
                self.sautes += ecart.saturating_sub(1);
            }
        }
        self.derniere = Some(vue);
    }

    /// **La section du rapport** — rien quand aucun verdict n'est arrivé.
    pub fn ecrire(&self, t: &mut String) {
        let confiees = self.montrees + self.jetees;
        if confiees == 0 {
            return;
        }
        t.push_str(
            "  Ce que l'ecran a montre -- le compositeur l'a dit, image par image (PARU-1)\n\n",
        );
        t.push_str(&format!(
            "  {confiees} image(s) confiee(s) : {} montree(s), {} jetee(s) ({:.1} %) -- une \
             jetee est remplacee avant d'avoir paru\n",
            self.montrees,
            self.jetees,
            100.0 * self.jetees as f64 / confiees as f64
        ));
        if self.dus > 0 {
            t.push_str(&format!(
                "  battements sautes alors qu'une image etait due : {} sur {} ({:.1} %) -- \
                 l'ecran a remontre l'image d'avant\n",
                self.sautes,
                self.dus,
                100.0 * self.sautes as f64 / self.dus as f64
            ));
        }
        t.push_str(&format!(
            "  de l'envoi a l'ecran : median {:.2}ms, p90 {:.2}ms, p99 {:.2}ms -- l'ecart du \
             median au p90 est la gigue, qui se voit comme un tressaut\n",
            super::rapport::ms(self.latence.centile(0.5)),
            super::rapport::ms(self.latence.centile(0.9)),
            super::rapport::ms(self.latence.centile(0.99)),
        ));
        t.push('\n');
    }
}

/// **Combien de battements séparent deux images montrées** : le compte de l'écran quand il le
/// tient, sinon le temps rapporté à sa période — et un seul quand il ne dit ni l'un ni l'autre.
fn battements_entre(avant: Vue, apres: Vue) -> u64 {
    if avant.battement > 0 && apres.battement > avant.battement {
        return apres.battement - avant.battement;
    }
    let periode = u64::from(apres.periode_ns);
    if periode == 0 {
        return 1;
    }
    let ecart_ns = apres.us.saturating_sub(avant.us) * 1000;
    ((ecart_ns + periode / 2) / periode).max(1)
}

fn micro(d: std::time::Duration) -> u64 {
    u64::try_from(d.as_micros()).unwrap_or(u64::MAX)
}

fn sature(us: u64) -> u32 {
    u32::try_from(us).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
