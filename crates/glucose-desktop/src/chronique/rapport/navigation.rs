//! Ce que la main demande, et le temps que l'écran met à le montrer — par quelle voie le
//! pavé est arrivé (fiche 53). Sorti de [`super`] quand il a dépassé six cents lignes.

use super::ms;
use crate::chronique::Chronique;

impl Chronique {
    /// Ce que la main a demandé, et le temps qu'il a fallu pour que l'écran le montre.
    ///
    /// # Pourquoi cette section existe à part
    ///
    /// Tout le reste de ce rapport mesure ce qu'une image **coûte**. Rien n'y disait le délai
    /// entre le geste et ce qu'on en voit — et c'est pourtant lui qui se ressent. Une
    /// application à cent images par seconde dont chaque image montre l'état d'il y a
    /// cinquante millisecondes paraît molle, et aucune durée d'image ne l'explique.
    pub(super) fn ecrire_la_navigation(&self, t: &mut String) {
        let c = self.navigation.comptes();
        let (mesurees, pire) = self.navigation.mesurees();
        if c.total() == 0 {
            return;
        }
        t.push_str("  Ce que la main demande, et le temps que l'ecran met a le montrer\n\n");
        t.push_str(&format!(
            "  {} pincement(s), {} zoom(s) au clavier, {} deplacement(s), {} pris pour un cran de souris\n",
            c.pincements, c.zooms, c.pans, c.crans
        ));
        // **Par quelle voie le pavé est arrivé** (fiche 53) : un déplacement qui arrive encore
        // en molette arrive par paquets, et glisse par l'élan de Glucose — pas par celui du
        // système.
        let (pave_pincements, pave_pans) = c.par_le_pave;
        if pave_pincements + pave_pans > 0 {
            t.push_str(&format!(
                "    dont par Direct Manipulation : {pave_pincements} pincement(s), {pave_pans} deplacement(s) -- le reste est venu en molette\n"
            ));
        }
        // **Autour de quoi le système zoome** (fiche 54) : Glucose applique sa similitude
        // entière, et c'est ce point qui dit si elle tient sous le curseur.
        let fixes = self.navigation.points_fixes();
        if fixes.compte() > 0 {
            t.push_str(&format!(
                "    le point que le systeme garde fixe est a {} px du curseur en median (p90 {} px), sur {} image(s) -- pres de zero : il zoome sous le curseur\n",
                fixes.centile(0.5),
                fixes.centile(0.9),
                fixes.compte()
            ));
        }
        // Le pincement est le geste que Glucose ne recevait pas du tout : tant que ce compte
        // reste nul alors qu'on a pince, le pont de plateforme ne sert pas, et c'est cela
        // qu'il faut corriger -- pas la cadence.
        if c.pincements == 0 && c.pans > 0 {
            t.push_str(
                "    aucun pincement reconnu : si on a pince, le systeme ne le marque pas ici\n",
            );
        }
        // Le pont de plateforme et la boucle d'evenements doivent voir le meme nombre de
        // pincements. Un ecart veut dire qu'un message marque n'a pas donne d'evenement -- ou
        // l'inverse -- et c'est exactement ce qui faisait dezoomer des translations pures.
        let marquees = crate::interactions::pincement::marques();
        let en_molette = c.pincements - pave_pincements;
        if marquees != en_molette {
            t.push_str(&format!(
                "    {marquees} message(s) marque(s) par le systeme pour {en_molette} pincement(s) venus en molette\n"
            ));
        }
        if c.crans > 0 && c.pans > 0 {
            t.push_str(
                "    un cran suppose au milieu d'un glissement coute un saut d'echelle visible\n",
            );
        }
        if mesurees > 0 {
            t.push_str(&format!(
                "  latence geste -> ecran : median {:.1}ms, p90 {:.1}ms, p99 {:.1}ms, pire {:.1}ms\n",
                ms(self.navigation.centile(0.50)),
                ms(self.navigation.centile(0.90)),
                ms(self.navigation.centile(0.99)),
                ms(pire),
            ));
        }
        t.push('\n');
    }
}
