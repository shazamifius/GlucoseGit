//! Les dialogues natifs, ancrés à la fenêtre de Glucose : **les sélecteurs de fichiers**, et
//! une seule boîte de message, qui ne s'ouvre que là où Glucose n'a pas encore de fenêtre
//! (DIAL-5). Toute question se dessine dans Glucose (POPUP-1, [`crate::ui::question`]).
//!
//! # Invariant DIAL-1 — un dialogue s'accroche toujours à la fenêtre qui l'ouvre
//!
//! Un dialogue natif ouvert **sans parent** n'appartient à aucune fenêtre. Sur Windows, le
//! gestionnaire est alors libre de le placer où il veut dans l'ordre d'empilement, et il le
//! place volontiers **derrière** la fenêtre principale. Vu de l'utilisateur, l'application
//! est figée : la croix ne fait rien, aucun bouton ne répond. Un sélecteur ancré s'affiche
//! devant son parent, le tient à l'écart des clics, et se ferme avec lui.
//!
//! # Invariant DIAL-2 — aucun dialogue ne tient la boucle (fiche 58)
//!
//! Un sélecteur ouvert sur le fil de Glucose tient sa boucle le temps que l'utilisateur
//! choisisse — dix-neuf secondes sur une session réelle : la fenêtre ne se repeint plus, et la
//! chronique lisait un « gel » qui n'en était pas un. Le sélecteur s'ouvre donc **sur un fil à
//! lui** ([`GlucoseApp::demander_un_fichier`]) — la voie asynchrone de `rfd`, attendue par
//! `pollster` —, Glucose continue de se dessiner, et le choix revient par une boîte aux
//! lettres, avec ce qu'il doit déclencher ([`crate::persist::choix::Demande`]). Un seul
//! sélecteur à la fois : un second, demandé pendant que le premier est ouvert, ne s'ouvre pas.
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
use crate::persist::choix::Demande;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

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
/// mise à jour n'y passe pas par ici.
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

/// Ce qu'on demande au sélecteur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Un fichier à ouvrir.
    Un,
    /// Des fichiers à ouvrir.
    Plusieurs,
    /// Où enregistrer.
    Enregistrer,
}

/// **Un sélecteur de fichier** : ses filtres, et le nom qu'il propose.
#[derive(Default)]
pub struct Fichier {
    filtres: Vec<(String, Vec<String>)>,
    nom: Option<String>,
}

impl Fichier {
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
}

/// Un sélecteur de fichier, sans filtre ni nom.
pub fn fichier() -> Fichier {
    Fichier::default()
}

/// **Ce qui est revenu des sélecteurs** : chaque demande, et ce qu'on y a choisi — rien si l'on
/// a renoncé. Un fil le dépose ; la boucle le relève, réveillée.
struct Boite {
    retours: Vec<(Demande, Option<Vec<PathBuf>>)>,
    // Le fil d'un sélecteur n'existe que sur un bureau, hors des épreuves : là seulement, on
    // les lit.
    #[cfg_attr(any(test, target_os = "android"), allow(dead_code))]
    ouvert: bool,
    #[cfg_attr(any(test, target_os = "android"), allow(dead_code))]
    reveil: Option<crate::plateforme::Reveil>,
}

static BOITE: Mutex<Boite> = Mutex::new(Boite {
    retours: Vec::new(),
    ouvert: false,
    reveil: None,
});

/// De quoi réveiller la boucle quand un choix revient, donné au lancement.
pub fn brancher(reveil: crate::plateforme::Reveil) {
    BOITE.lock().unwrap_or_else(PoisonError::into_inner).reveil = Some(reveil);
}

/// Ce que les sélecteurs ont rendu depuis la dernière fois.
#[cfg(not(test))]
pub fn relever() -> Vec<(Demande, Option<Vec<PathBuf>>)> {
    std::mem::take(&mut BOITE.lock().unwrap_or_else(PoisonError::into_inner).retours)
}

/// Sous les épreuves, chacune sa boîte : elles tournent en parallèle.
#[cfg(test)]
pub fn relever() -> Vec<(Demande, Option<Vec<PathBuf>>)> {
    epreuve::RETOURS.with(|r| std::mem::take(&mut *r.borrow_mut()))
}

/// Un choix revient : on le dépose, le sélecteur est fermé, et la boucle se réveille.
#[cfg(not(any(test, target_os = "android")))]
fn deposer(demande: Demande, choisi: Option<Vec<PathBuf>>) {
    let mut boite = BOITE.lock().unwrap_or_else(PoisonError::into_inner);
    boite.retours.push((demande, choisi));
    boite.ouvert = false;
    if let Some(reveil) = &boite.reveil {
        reveil();
    }
}

impl GlucoseApp {
    /// **Ouvre un sélecteur, sans tenir la boucle** (DIAL-2) : sur un fil à lui, accroché à la
    /// fenêtre (DIAL-1) ; `demande` dit ce que le choix déclenchera quand il reviendra
    /// ([`crate::persist::choix`]). Sous Android, il n'y en a pas : rien ne s'ouvre.
    pub(crate) fn demander_un_fichier(&mut self, fichier: Fichier, mode: Mode, demande: Demande) {
        // Comme en vrai, le choix revient **plus tard**, par la boîte : l'épreuve tourne la boucle
        // pour le relever ([`Self::suivre_les_fichiers_choisis`]).
        #[cfg(test)]
        {
            let _ = (fichier, mode);
            let choisi = epreuve::choix();
            epreuve::RETOURS.with(|r| r.borrow_mut().push((demande, choisi)));
        }
        #[cfg(all(not(test), target_os = "android"))]
        {
            let _ = (fichier, mode, demande);
        }
        #[cfg(all(not(test), not(target_os = "android")))]
        {
            {
                let mut boite = BOITE.lock().unwrap_or_else(PoisonError::into_inner);
                if boite.ouvert {
                    return;
                }
                boite.ouvert = true;
            }
            let mut d = rfd::AsyncFileDialog::new();
            if let Some(fenetre) = self.window.as_deref() {
                d = d.set_parent(fenetre);
            }
            for (nom, extensions) in &fichier.filtres {
                d = d.add_filter(nom, extensions);
            }
            if let Some(nom) = fichier.nom {
                d = d.set_file_name(nom);
            }
            // Le futur se crée ici, sur le fil de la fenêtre — macOS le demande — et s'attend
            // ailleurs.
            type Futur<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
            let futur: Futur<Option<Vec<rfd::FileHandle>>> = match mode {
                Mode::Un => {
                    let f = d.pick_file();
                    Box::pin(async move { f.await.map(|h| vec![h]) })
                }
                Mode::Plusieurs => Box::pin(d.pick_files()),
                Mode::Enregistrer => {
                    let f = d.save_file();
                    Box::pin(async move { f.await.map(|h| vec![h]) })
                }
            };
            let fil = std::thread::Builder::new()
                .name("selecteur".into())
                .spawn(move || {
                    let choisi = pollster::block_on(futur)
                        .map(|v| v.iter().map(|h| h.path().to_path_buf()).collect());
                    deposer(demande, choisi);
                });
            if fil.is_err() {
                BOITE.lock().unwrap_or_else(PoisonError::into_inner).ouvert = false;
            }
        }
    }

    /// **Ce que les sélecteurs ont rendu**, suivi — à chaque tour de boucle.
    pub(crate) fn suivre_les_fichiers_choisis(&mut self) {
        for (demande, choisi) in relever() {
            self.suivre_le_choix(demande, choisi);
        }
    }
}

/// Les réponses que les épreuves donnent aux dialogues (DIAL-3).
#[cfg(test)]
pub mod epreuve {
    use std::cell::{Cell, RefCell};
    use std::path::PathBuf;

    /// Ce que les sélecteurs d'une épreuve ont rendu, et que sa boucle relève.
    pub(super) type Retours = Vec<(crate::persist::choix::Demande, Option<Vec<PathBuf>>)>;

    thread_local! {
        static REPONSE: Cell<Option<bool>> = const { Cell::new(None) };
        static CHOIX: RefCell<Option<Option<Vec<PathBuf>>>> = const { RefCell::new(None) };
        pub(super) static RETOURS: RefCell<Retours> = const { RefCell::new(Vec::new()) };
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

    /// Ce que le prochain sélecteur de ce fil rendra — `None` : on y renonce.
    pub fn choisir(choisi: Option<Vec<PathBuf>>) {
        CHOIX.with(|c| *c.borrow_mut() = Some(choisi));
    }

    pub(super) fn choix() -> Option<Vec<PathBuf>> {
        CHOIX.with(|c| c.borrow_mut().take()).unwrap_or_else(|| {
            panic!("une épreuve a ouvert un sélecteur sans choix : il serait apparu à l'écran")
        })
    }
}
