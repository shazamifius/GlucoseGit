//! Les dialogues natifs, ancrés à la fenêtre de Glucose : **les sélecteurs de fichiers**, et
//! une seule boîte de message, qui ne s'ouvre que là où Glucose n'a pas encore de fenêtre
//! (DIAL-5). Toute question se dessine dans Glucose (POPUP-1, [`crate::ui::question`]).
//!
//! # Invariant DIAL-1 — un dialogue s'accroche toujours à la fenêtre qui l'ouvre
//!
//! Un dialogue natif ouvert **sans parent** n'appartient à aucune fenêtre. Sur Windows, le
//! gestionnaire est alors libre de le placer où il veut dans l'ordre d'empilement, et il le
//! place volontiers **derrière** la fenêtre principale — qui, elle, attend la réponse sans
//! rien afficher.
//!
//! Vu de l'utilisateur, l'application est figée : la croix ne fait rien, aucun bouton ne
//! répond, et il ne reste qu'à tuer le processus depuis le gestionnaire des tâches. Le
//! symptôme ne ressemble en rien à sa cause, ce qui est le propre de ce défaut : aucun test
//! ne le voit, aucun journal ne le dit, et le programme fait exactement ce qu'on lui a
//! demandé.
//!
//! Le remède est d'une ligne par dialogue, et il vaut pour **tous** : l'ouverture,
//! l'enregistrement, l'export et l'import d'images. Un dialogue ancré s'affiche devant son
//! parent, le bloque proprement, et se ferme avec lui.
//!
//! # Invariant DIAL-2 — un dialogue passe toujours par [`GlucoseApp::sous_un_dialogue`]
//!
//! Un dialogue natif bloque la boucle dans le gestionnaire qui l'a ouvert, le temps que
//! l'utilisateur réponde — dix-neuf secondes sur une session réelle, à choisir un fichier.
//! Pendant ce temps il regarde le dialogue, pas le canevas : l'intervalle n'est ni un gel ni
//! un mouvement, et rien de ce qui précède ne décrit ce que l'œil verra ensuite. Sans le
//! dire, la chronique lisait « le pire gel : 19 836 ms à la 21,2e seconde » — vrai, et sans
//! aucun intérêt, pendant que ce chiffre cachait le vrai pire gel de la session.
//!
//! L'horloge, le rythme et le tempo repartent donc de la prochaine présentation. Et ce n'est
//! pas une convention : c'est le type [`Ancre`] qui le tient. Le sélecteur de fichiers
//! l'exige, et seule `sous_un_dialogue` sait le construire. Un sélecteur ouvert ailleurs ne
//! compile pas.
//!
//! # Invariant DIAL-4 — `rfd` ne sort jamais d'ici
//!
//! Les dialogues se demandent dans les mots de Glucose (un sélecteur [`Fichier`], une question
//! [`oui_ou_non`]) : aucun type de `rfd` ne traverse cette porte. C'est ce qui laisse Glucose
//! compiler là où `rfd` n'existe pas — Android, qui a ses propres chemins (fiches 54 et 56).
//!
//! # Invariant DIAL-5 — aucune boîte de message sur une fenêtre de Glucose
//!
//! Une boîte de message du système **tient la boucle** tant qu'elle est ouverte : la fenêtre
//! de Glucose ne se repeint plus. Posée avant la première image — la question du journal
//! technique, à l'ouverture —, elle laissait une fenêtre noire et une boîte parfois derrière :
//! *« il faut voyager dans le noir total, faire Tab puis Entrée »* (08/10, POPUP-1, fiche 58).
//! Les questions se dessinent donc dans Glucose. [`oui_ou_non`] ne reste que pour la mise à
//! jour cherchée **avant** la fenêtre, et le type le tient : elle exige un [`SansFenetre`], que
//! seul [`GlucoseApp::sans_fenetre`] construit, et seulement quand il n'y a pas de fenêtre.

use crate::app::GlucoseApp;
use winit::window::Window;

/// La fenêtre à laquelle un dialogue s'accroche — et la preuve qu'il s'ouvre sous
/// [`GlucoseApp::sous_un_dialogue`], puisque rien d'autre ne sait construire ce type.
///
/// `None` n'arrive qu'avant que la fenêtre existe — au tout début, ou en test. Le dialogue
/// s'ouvre alors sans parent, ce qui est le seul comportement possible et ne bloque personne,
/// puisqu'il n'y a pas encore de fenêtre à bloquer.
pub struct Ancre<'a>(Option<&'a Window>);

impl GlucoseApp {
    /// Ouvre un dialogue natif, et dit ensuite à la boucle qu'elle a été tenue (DIAL-2).
    ///
    /// La fenêtre est clonée avant l'appel : le dialogue s'y accroche (DIAL-1) et ce qui suit
    /// a besoin de `self` en écriture.
    pub fn sous_un_dialogue<T>(&mut self, ouvrir: impl FnOnce(Ancre<'_>) -> T) -> T {
        let fenetre = self.window.clone();
        let reponse = ouvrir(Ancre(fenetre.as_deref()));
        self.horloge.oublier();
        self.chronique.rythme.oublier();
        self.chronique.entracte.oublier();
        self.tempo.oublier();
        reponse
    }
}

/// **La preuve que Glucose n'a pas de fenêtre** (DIAL-5) : rien d'autre que
/// [`GlucoseApp::sans_fenetre`] ne sait la construire.
pub struct SansFenetre(());

impl GlucoseApp {
    /// La preuve qu'aucune fenêtre n'existe encore, s'il n'en existe aucune (DIAL-5).
    pub fn sans_fenetre(&self) -> Option<SansFenetre> {
        self.window.is_none().then_some(SansFenetre(()))
    }
}

/// **Une question oui / non par le système**, là seulement où Glucose n'a pas de fenêtre où la
/// dessiner (DIAL-5) : `true` pour oui. Sous Android, rien ne se demande ainsi : « non », la
/// mise à jour n'y existe pas encore.
///
/// # DIAL-3 — une épreuve n'ouvre jamais de vraie boîte
///
/// Une épreuve qui atteindrait ce dialogue l'ouvrirait **sur son écran**, au milieu de son
/// travail, et attendrait une réponse que personne ne donnera. Sous `cfg(test)`, la réponse
/// vient donc de l'épreuve ([`epreuve::repondre`]) ; une épreuve qui n'en a pas donné tombe,
/// en nommant le dialogue, au lieu de l'afficher.
pub fn oui_ou_non(sans_fenetre: SansFenetre, titre: &str, question: &str) -> bool {
    let SansFenetre(()) = sans_fenetre;
    #[cfg(test)]
    {
        let _ = question;
        epreuve::reponse(titre)
    }
    #[cfg(all(not(test), target_os = "android"))]
    {
        let _ = (titre, question);
        false
    }
    #[cfg(all(not(test), not(target_os = "android")))]
    {
        let reponse = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Info)
            .set_title(titre)
            .set_description(question)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        matches!(
            reponse,
            rfd::MessageDialogResult::Yes | rfd::MessageDialogResult::Ok
        )
    }
}

/// **Un sélecteur de fichier**, accroché à la fenêtre (DIAL-1) : ses filtres, le nom qu'il
/// propose, puis ce qu'on lui demande — choisir un fichier, plusieurs, ou où enregistrer.
pub struct Fichier<'a> {
    ancre: Ancre<'a>,
    filtres: Vec<(String, Vec<String>)>,
    nom: Option<String>,
}

/// Un sélecteur de fichier accroché à la fenêtre.
pub fn fichier(ancre: Ancre<'_>) -> Fichier<'_> {
    Fichier {
        ancre,
        filtres: Vec::new(),
        nom: None,
    }
}

impl Fichier<'_> {
    /// N'y montre que ces extensions, sous ce nom.
    pub fn filtre(mut self, nom: &str, extensions: &[&str]) -> Self {
        let extensions = extensions.iter().map(|e| e.to_string()).collect();
        self.filtres.push((nom.to_string(), extensions));
        self
    }

    /// Le nom qu'il propose.
    pub fn nom(mut self, nom: String) -> Self {
        self.nom = Some(nom);
        self
    }

    /// Un fichier à ouvrir.
    pub fn choisir(self) -> Option<std::path::PathBuf> {
        self.systeme()?.pick_file()
    }

    /// Des fichiers à ouvrir.
    pub fn choisir_plusieurs(self) -> Option<Vec<std::path::PathBuf>> {
        self.systeme()?.pick_files()
    }

    /// Où enregistrer.
    pub fn enregistrer(self) -> Option<std::path::PathBuf> {
        self.systeme()?.save_file()
    }

    /// Le sélecteur du système.
    #[cfg(not(target_os = "android"))]
    fn systeme(self) -> Option<rfd::FileDialog> {
        let mut d = rfd::FileDialog::new();
        if let Some(fenetre) = self.ancre.0 {
            d = d.set_parent(fenetre);
        }
        for (nom, extensions) in &self.filtres {
            d = d.add_filter(nom, extensions);
        }
        if let Some(nom) = self.nom {
            d = d.set_file_name(nom);
        }
        Some(d)
    }

    /// Sous Android, aucun encore : celui du système viendra par JNI (fiche 54).
    #[cfg(target_os = "android")]
    fn systeme(self) -> Option<SansSelecteur> {
        let _ = (self.ancre.0, self.filtres, self.nom);
        None
    }
}

/// Le sélecteur qui n'existe pas encore sous Android : un type sans valeur, que rien ne
/// construit.
#[cfg(target_os = "android")]
enum SansSelecteur {}

#[cfg(target_os = "android")]
impl SansSelecteur {
    fn pick_file(self) -> Option<std::path::PathBuf> {
        match self {}
    }
    fn pick_files(self) -> Option<Vec<std::path::PathBuf>> {
        match self {}
    }
    fn save_file(self) -> Option<std::path::PathBuf> {
        match self {}
    }
}

/// Les réponses que les épreuves donnent aux dialogues (DIAL-3).
#[cfg(test)]
pub mod epreuve {
    use std::cell::Cell;

    thread_local! {
        static REPONSE: Cell<Option<bool>> = const { Cell::new(None) };
    }

    /// La réponse au prochain dialogue oui / non de ce fil.
    pub fn repondre(oui: bool) {
        REPONSE.with(|r| r.set(Some(oui)));
    }

    pub(super) fn reponse(titre: &str) -> bool {
        REPONSE.with(Cell::take).unwrap_or_else(|| {
            panic!("une épreuve a ouvert « {titre} » sans réponse : il serait apparu à l'écran")
        })
    }
}
