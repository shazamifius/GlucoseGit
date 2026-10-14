//! **Plusieurs images lâchées remplissent les cases vides à la suite** (P2, proposition 1,
//! fiche 66).
//!
//! # Ce qui se passait
//!
//! Une image lâchée dans une case s'y ajustait (STORYBOARD-1) ; dix images lâchées ensemble
//! sur une case s'y ajustaient **toutes**, empilées dans le même cadre. Boords fait l'inverse :
//! un lot d'images y devient une case par image, dans l'ordre du choix
//! ([la documentation](https://boords.com/docs/adding-your-own-images)). Sa demande, validée le
//! 13/10 : *« en lâcher dix sur la première case les range une par case, et le storyboard
//! s'allonge s'il le faut »*.
//!
//! # Une seule loi
//!
//! Les images d'un même lâcher, prises **dans l'ordre de lecture** de leurs boîtes
//! ([`super::ordre_de_lecture`], celui des cases) : chacune va dans la case sous son centre —
//! comme avant, même si cette case porte déjà une image. Si une image **de ce lâcher** y est
//! déjà arrivée, elle passe à la **case vide suivante** dans la suite — une case dont aucune
//! image n'est d'ailleurs que de ce lâcher —, et quand il n'en reste plus, le storyboard
//! s'allonge d'une case. Une image lâchée seule fait donc exactement ce qu'elle faisait.

use super::super::Store;
use super::{ordre_de_lecture, Boite, Grille};
use std::collections::HashSet;

impl Store {
    /// **Les images lâchées dans le storyboard le remplissent** : parmi `ids`, chaque image non
    /// verrouillée dont le centre tombe dans le cadre d'une case y est contenue, centrée, à ses
    /// proportions — une par case, à la suite. Dans le geste ouvert, s'il y en a un.
    pub fn ajuster_dans_les_cases(&mut self, board_id: &str, ids: &[String]) {
        let lacher: Vec<(String, Boite)> = ids
            .iter()
            .filter_map(|id| {
                let i = self.image(board_id, id)?;
                let ajustable = !i.locked && i.width > 0.0 && i.height > 0.0;
                ajustable.then(|| {
                    let boite = (i.x - i.width / 2.0, i.y - i.height / 2.0, i.width, i.height);
                    (id.clone(), boite)
                })
            })
            .collect();
        let boites: Vec<Boite> = lacher.iter().map(|(_, b)| *b).collect();
        let du_lacher: HashSet<&str> = lacher.iter().map(|(id, _)| id.as_str()).collect();
        let mut remplies: HashSet<String> = HashSet::new();
        for rang in ordre_de_lecture(&boites) {
            let (id, (gauche, haut, largeur, hauteur)) = &lacher[rang];
            let centre = (gauche + largeur / 2.0, haut + hauteur / 2.0);
            let Some(sous) = self.case_sous(board_id, centre).map(|c| c.id.clone()) else {
                continue;
            };
            let case = if remplies.contains(&sous) {
                match self.case_vide_apres(board_id, &sous, (&du_lacher, &remplies)) {
                    Some(c) => c,
                    None => match self.ajouter_une_case(board_id, Grille::default()) {
                        Some(c) => c,
                        None => continue,
                    },
                }
            } else {
                sous
            };
            self.ajuster_dans_la_case(board_id, id, &case);
            remplies.insert(case);
        }
    }

    /// **La première case vide après `apres`** dans la suite : aucune image ne la remplit déjà
    /// pendant ce lâcher, et toutes celles qu'elle porte sont de ce lâcher — elles s'en vont.
    fn case_vide_apres(
        &self,
        board_id: &str,
        apres: &str,
        (du_lacher, remplies): (&HashSet<&str>, &HashSet<String>),
    ) -> Option<String> {
        let cases = self.cases(board_id);
        let depuis = cases.iter().position(|c| c.id == apres)? + 1;
        cases[depuis..]
            .iter()
            .find(|c| {
                !remplies.contains(&c.id)
                    && self
                        .images_de_la_case(board_id, &c.id)
                        .iter()
                        .all(|i| du_lacher.contains(i.as_str()))
            })
            .map(|c| c.id.clone())
    }

    /// Une image contenue dans le cadre de cette case, centrée, à ses proportions.
    fn ajuster_dans_la_case(&mut self, board_id: &str, id: &str, case_id: &str) {
        let Some(case) = self
            .cases(board_id)
            .into_iter()
            .find(|c| c.id == case_id)
            .cloned()
        else {
            return;
        };
        let Some((largeur, hauteur)) = self.image(board_id, id).map(|i| (i.width, i.height)) else {
            return;
        };
        let s = (case.width / largeur).min(case.height / hauteur);
        self.update_image(board_id, id, |img| {
            (img.width, img.height) = (img.width * s, img.height * s);
            img.x = case.x + case.width / 2.0;
            img.y = case.y + case.height / 2.0;
        });
    }
}
