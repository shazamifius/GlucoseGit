//! Les raisons qu'a l'application de se réveiller, et dans combien de temps.
//!
//! # Une seule question, posée cinq fois
//!
//! Le curseur qui clignote, le toast qui s'efface, la caméra qui vole, les images qui se
//! décodent, le minuteur qui tourne : ces cinq-là n'ont rien en commun sauf l'essentiel — ils
//! veulent tous que l'application repasse **bientôt**, et chacun sait dire dans combien de
//! temps.
//!
//! Ils vivaient en cinq blocs accumulés dans `about_to_wait`, chacun mettant à jour à la main
//! un minimum courant et un drapeau « il y a un minuteur ». Le schéma se répétait, donc il
//! s'oubliait : ajouter une sixième raison demandait de se souvenir des deux gestes.
//!
//! Ici chaque raison rend simplement **le délai qu'elle demande**, ou rien si elle n'a rien à
//! demander. La plus pressée l'emporte, et c'est `min` qui le dit — pas un accumulateur.
//! Aucune borne arbitraire ne subsiste : l'ancien plafond d'une seconde n'était que la valeur
//! initiale de ce minimum, et il disparaît avec lui.

use super::GlucoseApp;
use crate::ui::ToastRepaint;

/// Ce qui empêche l'application de dormir, à une image donnée.
///
/// # Pourquoi cette énumération a fallu être écrite
///
/// La chronique disait ce qu'une image coûte, jamais **pourquoi elle a été demandée**. Sur une
/// session où l'utilisateur regardait surtout son canevas sans y toucher, elle comptait trois
/// mille six cents images « au repos » — et rien ne permettait de savoir si elles étaient
/// dues, ou si quelque chose réveillait l'application pour rien.
///
/// Une image inutile est la plus chère de toutes : elle coûte son prix entier et ne montre
/// rien de neuf. Avant de rendre les images moins chères, il faut savoir combien ne devraient
/// pas exister.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Raison {
    Curseur,
    Toast,
    Animation,
    Elan,
    Vol,
    Decodage,
    Chantier,
    Pomodoro,
}

impl Raison {
    /// Dans l'ordre des bits du masque. L'ordre est celui de [`GlucoseApp::prochain_reveil`],
    /// pour qu'une raison ajoutée là se retrouve ici sans réfléchir.
    pub const TOUTES: [Self; 8] = [
        Self::Curseur,
        Self::Toast,
        Self::Animation,
        Self::Elan,
        Self::Vol,
        Self::Decodage,
        Self::Chantier,
        Self::Pomodoro,
    ];

    pub fn nom(self) -> &'static str {
        match self {
            Self::Curseur => "curseur qui clignote",
            Self::Toast => "message a l'ecran",
            Self::Animation => "animation en cours",
            Self::Elan => "la vue glisse encore",
            Self::Vol => "vol de camera",
            Self::Decodage => "images en decodage",
            Self::Chantier => "vignettes a construire",
            Self::Pomodoro => "minuteur",
        }
    }

    /// Le bit de cette raison dans le masque d'une image.
    pub fn bit(self) -> u16 {
        1 << Self::TOUTES.iter().position(|r| *r == self).unwrap_or(0)
    }
}

impl GlucoseApp {
    /// Dans combien de temps demander la prochaine image d'une animation.
    ///
    /// # La rétroaction que cette version supprime, et elle gelait le démarrage
    ///
    /// La version précédente dormait **aussi longtemps que la dernière image avait coûté**,
    /// borné entre la période de l'écran et un quart de seconde. La justification était « ne
    /// pas remplir la file d'événements plus vite qu'elle ne se vide ». La conséquence était
    /// une rétroaction : une image de 229 ms — la première soumission à la carte graphique —
    /// faisait dormir la boucle 229 ms de plus, et le premier geste de l'utilisateur tombait
    /// dans un gel d'une seconde et quart, dont un bon tiers à ne rien faire du tout.
    ///
    /// La chronique du rythme l'a montré au premier lancement : « le pire gel : 1261 ms à la
    /// 1,3e seconde, dont 1032 ms à ne pas dessiner ». Aucune durée d'image ne pouvait le
    /// dire, puisque ce temps n'était dans aucune image.
    ///
    /// # Ce qui remplace le sommeil : viser le prochain balayage
    ///
    /// L'écran bat à sa période, et l'image précédente a été présentée à un instant connu. La
    /// prochaine image est due au balayage suivant — pas avant, parce que rien ne serait
    /// montré ; pas après, parce que ce serait rater un balayage pour rien. Si le rendu prend
    /// plus d'une période, c'est la présentation qui rate le balayage et attend le suivant,
    /// ce que `Fifo` fait de lui-même.
    ///
    /// Aucune constante : la période est lue sur l'écran, l'instant de présentation est
    /// observé. Et si on est déjà en retard, la réponse est zéro — tout de suite.
    ///
    /// # Et depuis le tempo, tout de suite
    ///
    /// Le tempo attend APRES le rendu, jusqu'a l'heure de soumettre : le rendu peut donc
    /// commencer des la presentation precedente, et l'attente absorbe ce qu'il reste. Viser
    /// le prochain balayage ici ferait rater la cible des que le rendu depasse une periode --
    /// exactement le cas qui a fait naitre le tempo.
    pub(super) fn animation_interval_ms(&self) -> u64 {
        0
    }

    /// Le délai avant le prochain réveil, ou `None` si rien n'est attendu.
    ///
    /// Chaque raison salit la vue elle-même si elle a besoin d'être redessinée : demander un
    /// rafraîchissement et demander un réveil sont deux choses distinctes — un toast au
    /// plateau attend sans rien redessiner, un décodage redessine sans rien animer.
    pub(super) fn prochain_reveil(&mut self) -> Option<u64> {
        let attentes = [
            self.attente_du_curseur(),
            self.attente_du_toast(),
            self.attente_de_l_animation(),
            self.attente_de_l_elan(),
            self.attente_du_vol(),
            self.attente_du_decodage(),
            self.attente_du_chantier(),
            self.attente_du_pomodoro(),
        ];
        // Toutes les raisons actives, et non la seule qui l'emporte : savoir laquelle est la
        // plus pressée ne dit pas laquelle il faudrait supprimer.
        let masque = attentes
            .iter()
            .zip(Raison::TOUTES)
            .filter(|(attente, _)| attente.is_some())
            .fold(0u16, |masque, (_, raison)| masque | raison.bit());
        crate::perf::compteur("reveil_masque", f64::from(masque));
        attentes.into_iter().flatten().min()
    }

    /// Le curseur d'édition de texte clignote à la demi-seconde.
    ///
    /// Le réveil est calé sur **ce qui reste** de la demi-seconde en cours, et non sur une
    /// demi-seconde pleine : sans cela le clignotement dériverait à chaque image lente.
    fn attente_du_curseur(&mut self) -> Option<u64> {
        let Some(session) = self.editing_session.as_mut() else {
            self.last_blink_phase = true;
            return None;
        };
        let elapsed = session.blink_timer.elapsed().as_millis();
        let phase = (elapsed / 500) % 2 == 0;
        // **C'est ici que la phase se décide, et nulle part ailleurs** (BLINK-1). Cette
        // fonction la calculait déjà pour savoir quand se réveiller ; le dessin la
        // recalculait de son côté, sur une horloge qui avait avancé entre-temps.
        session.curseur_visible = phase;
        if phase != self.last_blink_phase {
            self.last_blink_phase = phase;
            self.mark_dirty();
        }
        Some((500 - (elapsed % 500)).max(1) as u64)
    }

    /// Le toast sait lui-même s'il doit se redessiner, attendre, ou disparaître.
    fn attente_du_toast(&mut self) -> Option<u64> {
        match self.ui.current_toast.as_ref()?.repaint_need() {
            ToastRepaint::Gone => {
                self.ui.current_toast = None;
                self.mark_dirty();
                None
            }
            ToastRepaint::Redraw => {
                self.mark_dirty();
                Some(self.animation_interval_ms())
            }
            // Le plateau : l'opacité ne bouge plus, donc rien à redessiner — mais il faudra
            // revenir quand le fondu de sortie commencera.
            ToastRepaint::Sleep(wait_ms) => Some(wait_ms),
        }
    }

    /// Le vol de la caméra avance d'une image, et dit quand la suivante est due.
    fn attente_de_l_animation(&mut self) -> Option<u64> {
        let reste_ms = self.animator.tick(&mut self.store)?;
        self.mark_dirty();
        Some(reste_ms.max(1).min(self.animation_interval_ms()))
    }

    /// La caméra glisse encore : la main a lâché, mais l'élan n'est pas éteint.
    ///
    /// C'est la seule raison de réveil qui **finit d'elle-même** sans horloge extérieure :
    /// l'élan s'éteint quand ce qui reste à parcourir tient sous le demi-pixel, donc sous ce
    /// qu'un écran peut montrer. Rien n'a été choisi pour l'arrêter.
    fn attente_de_l_elan(&mut self) -> Option<u64> {
        if !self.elan.en_cours() {
            return None;
        }
        self.mark_dirty();
        Some(self.animation_interval_ms())
    }

    /// Un vol de caméra est en route vers une destination.
    ///
    /// Même raison que l'élan, et même forme : rien ne se passe à l'écran tant que personne
    /// ne demande l'image qui montrerait le pas suivant.
    fn attente_du_vol(&mut self) -> Option<u64> {
        if !self.vol.en_cours() {
            return None;
        }
        self.mark_dirty();
        Some(self.animation_interval_ms())
    }

    /// Des images se décodent sur les fils de fond (DECODE-1).
    ///
    /// Tant qu'il en reste, l'application a une raison de se réveiller même si personne ne
    /// touche à rien : c'est ce qui fait paraître une photo dès qu'elle est prête, sans que
    /// le rendu ait eu à l'attendre.
    fn attente_du_decodage(&mut self) -> Option<u64> {
        if self.renderer.magasin.en_travail() == 0 {
            return None;
        }
        self.mark_dirty();
        Some(self.animation_interval_ms())
    }

    /// Des vignettes attendent leur tour au chantier (CASCADE-1).
    ///
    /// **Rien n'est sali, et c'est voulu** : une vignette donne exactement les mêmes pixels que
    /// le chemin général, donc l'image à l'écran est déjà juste. Elle ne la rend pas plus
    /// belle, elle la rend beaucoup moins chère — il faut donc seulement revenir, pour que
    /// l'atelier dispose d'une nouvelle tranche de temps libre.
    ///
    /// Sans ce réveil, une application immobile s'endormirait avec son chantier en plan, et le
    /// premier geste suivant paierait tout ce qui n'a pas été fait.
    fn attente_du_chantier(&mut self) -> Option<u64> {
        if self.renderer.magasin.vignettes.en_chantier() == 0 {
            return None;
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        Some(self.animation_interval_ms())
    }

    /// Le minuteur Pomodoro du dock, qui compte en secondes pleines.
    fn attente_du_pomodoro(&mut self) -> Option<u64> {
        if !self.dock_manager.pomodoro.running {
            return None;
        }
        if self.dock_manager.tick_pomodoro() {
            self.mark_dirty();
        }
        let elapsed_ms = self.dock_manager.pomodoro.last_tick.elapsed().as_millis();
        Some(1000_u128.saturating_sub(elapsed_ms).max(1) as u64)
    }
}
