//! **La place d'un cadre** (PASSAGE-2) : un passage qui brille écarte ses voisins d'exactement
//! ce que son cadre demande, et de rien de plus.
//!
//! # Ce qu'il a vu
//!
//! Au survol d'une flèche ancrée, le cadre rose d'un passage **débordait sur les lettres
//! voisines** (sa capture du 26/09, un passage pris au milieu d'un mot) : le cadre — sa marge,
//! son liseré décalé — se posait par-dessus une mise en page qui ne savait rien de lui. Et la
//! teinte du passage se repeignait dans un rectangle découpé, un peu plus large que lui : une
//! lettre voisine sortait à moitié rose.
//!
//! Chez Tauri, le `<mark>` du passage était un élément HTML **dans** la ligne : sa marge
//! prenait de la place, et les voisins s'écartaient. Mais seulement de sa marge : son liseré,
//! qui en CSS ne prend aucune place, mordait encore sur un voisin collé.
//!
//! # La règle
//!
//! Un cadre ne couvre jamais l'encre d'une autre lettre. Il déborde de son passage de son
//! **étendue** — ce que son style dit : marge, décalage, demi-liseré — et la ligne s'ouvre de
//! ce qui manque pour cela, et de rien de plus :
//!
//! * une espace est une place libre : entre deux mots, le cadre la prend avant d'écarter
//!   quoi que ce soit ;
//! * un début ou une fin de ligne aussi : la marge de la carte est là pour ça ;
//! * deux passages voisins écartent chacun leur part.
//!
//! Le parcours est glouton, de gauche à droite : il retient où finit ce qui ne doit pas être
//! couvert — l'avance de la dernière lettre voisine, ou le bord droit du dernier cadre —, et
//! décale la suite de la ligne quand un bord gauche y tomberait. Aucune constante : l'étendue
//! vient du style, les avances de la police.
//!
//! # Ce qui ne change pas
//!
//! **La coupe des lignes.** Une ligne s'ouvre en place ; elle ne se reflue pas. Sans cela,
//! survoler une flèche pouvait faire gagner une ligne à une carte — et changer sa hauteur,
//! donc sa boîte, ses flèches, son aimant. La ligne ouverte déborde au plus dans la marge de
//! la carte.
//!
//! Le décalage se pose par les **taquets** des fragments ([`super::Fragment::tab`]), que le
//! tracé comme le clic lisent déjà : ce qui est dessiné est ce qui est visé.

use super::{font_of, TextBox, TextLayout, NO_TAB};
use crate::typography::Typography;
use glucose_core::text::BlockKind;

/// **La mise en page `mise_en_page`, ouverte pour les cadres de `plages`** : chaque fragment
/// coupé aux bords des plages, marqué éclairé s'il est dedans, et la suite de chaque ligne
/// décalée de ce que les cadres demandent — `etendue` unités du monde de chaque côté.
///
/// Une formule ne s'ouvre pas : elle est un atome, qui occupe sa ligne entière — le cadre
/// prend sa boîte dessinée, et sa ligne n'a pas de voisin à écarter.
pub(crate) fn ouvrir_la_place(
    mise_en_page: &TextLayout,
    typographie: &Typography,
    (source, bx): (&str, TextBox),
    plages: &[(usize, usize)],
    etendue: f32,
) -> TextLayout {
    let plages = fusionner(plages);
    let mut ouverte = TextLayout {
        lines: Vec::with_capacity(mise_en_page.lines.len()),
        fragments: Vec::with_capacity(mise_en_page.fragments.len() + 2 * plages.len()),
    };
    for ligne in &mise_en_page.lines {
        let debut = ouverte.fragments.len();
        let fragments = mise_en_page.fragments_of(ligne);
        let touchee = plages
            .iter()
            .any(|&(a, b)| a < ligne.end.max(ligne.start + 1) && b > ligne.start);
        if !touchee {
            ouverte.fragments.extend_from_slice(fragments);
        } else if matches!(ligne.kind, BlockKind::Math { .. }) {
            ouverte
                .fragments
                .extend(fragments.iter().map(|f| super::Fragment {
                    eclaire: true,
                    ..*f
                }));
        } else {
            let corps = font_of(ligne.kind, bx.body);
            let mut parcours = Parcours {
                typographie,
                source,
                plages: &plages,
                etendue,
                corps,
                sortie: &mut ouverte.fragments,
                naturel: 0.0,
                decalage: 0.0,
                obstacle: f32::NEG_INFINITY,
            };
            for f in fragments {
                parcours.fragment(f);
            }
        }
        ouverte.lines.push(super::VisualLine {
            fragments: debut..ouverte.fragments.len(),
            ..ligne.clone()
        });
    }
    ouverte
}

/// Les plages triées, celles qui se recouvrent réunies. Deux plages **qui se touchent** restent
/// deux : ce sont deux cadres, et chacun demande sa place.
fn fusionner(plages: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut triees: Vec<(usize, usize)> = plages.iter().copied().filter(|p| p.0 < p.1).collect();
    triees.sort_unstable();
    let mut sortie: Vec<(usize, usize)> = Vec::with_capacity(triees.len());
    for (a, b) in triees {
        match sortie.last_mut() {
            Some(derniere) if a < derniere.1 => derniere.1 = derniere.1.max(b),
            _ => sortie.push((a, b)),
        }
    }
    sortie
}

/// Le parcours glouton d'une ligne : où l'on en est sans cadre (`naturel`), de combien la
/// suite est décalée, et jusqu'où va ce qu'un cadre ne doit pas couvrir (`obstacle`).
///
/// Tout est en unités du monde ; les taquets écrits sont en corps de la ligne (TABLE-2).
struct Parcours<'a> {
    typographie: &'a Typography,
    source: &'a str,
    plages: &'a [(usize, usize)],
    etendue: f32,
    corps: f32,
    sortie: &'a mut Vec<super::Fragment>,
    naturel: f32,
    decalage: f32,
    obstacle: f32,
}

impl Parcours<'_> {
    fn dans_une_plage(&self, o: usize) -> bool {
        self.plages.iter().any(|&(a, b)| a <= o && o < b)
    }

    /// Un fragment de la ligne, coupé là où un cadre commence, finit, ou décale la suite.
    fn fragment(&mut self, f: &super::Fragment) {
        if f.tab >= 0.0 {
            self.naturel = f.tab * self.corps;
        }
        // Le morceau en cours : son début, son taquet, s'il est éclairé.
        let mut morceau = super::Fragment {
            tab: if f.tab >= 0.0 {
                (self.naturel + self.decalage) / self.corps
            } else {
                NO_TAB
            },
            eclaire: self.dans_une_plage(f.start),
            ..*f
        };
        let face = f.face();
        for (i, ch) in self.source[f.start..f.end].char_indices() {
            let o = f.start + i;
            let dedans = self.dans_une_plage(o);
            // Tout bord de plage coupe : l'éclairage ne change qu'à un bord, et un fragment qui
            // commence dans une plage — la suite d'un passage coupé par la ligne — l'est dès
            // son début.
            if self.avant(o, ch, dedans) {
                if o > morceau.start {
                    self.sortie.push(super::Fragment { end: o, ..morceau });
                }
                morceau = super::Fragment {
                    start: o,
                    tab: (self.naturel + self.decalage) / self.corps,
                    eclaire: dedans,
                    ..morceau
                };
            }
            let avance = self.typographie.advance(ch, self.corps, face);
            // L'encre d'une voisine est un obstacle ; une espace n'en est pas un, et une
            // lettre du passage est dans son propre cadre.
            if !dedans && !ch.is_whitespace() {
                self.obstacle = self.naturel + self.decalage + avance;
            }
            self.naturel += avance;
        }
        if f.end > morceau.start {
            self.sortie.push(super::Fragment {
                end: f.end,
                ..morceau
            });
        }
    }

    /// **Ce qui se passe devant le caractère `ch`, à l'octet `o`** : un cadre qui se ferme y
    /// pose son bord droit, un cadre qui s'ouvre y écarte la ligne, une voisine qui toucherait
    /// un bord y est poussée. Rend vrai si la suite commence un nouveau morceau.
    fn avant(&mut self, o: usize, ch: char, dedans: bool) -> bool {
        let x = self.naturel + self.decalage;
        let finit = self.plages.iter().any(|&(_, b)| b == o);
        let commence = self.plages.iter().any(|&(a, _)| a == o);
        let mut coupe = false;
        if finit {
            self.obstacle = x + self.etendue;
            coupe = true;
        }
        if commence {
            let gauche = x - self.etendue;
            if gauche < self.obstacle {
                self.decalage += self.obstacle - gauche;
            }
            coupe = true;
        } else if !dedans && !ch.is_whitespace() && x < self.obstacle {
            self.decalage += self.obstacle - x;
            coupe = true;
        }
        coupe
    }
}

#[cfg(test)]
mod tests;
