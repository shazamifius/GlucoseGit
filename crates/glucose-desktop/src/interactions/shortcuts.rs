//! Raccourcis clavier globaux et sélecteur d'outils (PureRef-style).
//!
//! # Ordre de résolution des touches de caractère
//!
//! Trois familles se partagent les mêmes lettres selon les modificateurs (`Ctrl+V` colle,
//! `V` choisit l'outil de sélection). Elles sont donc essayées dans un ordre fixe, chacune
//! rendant `true` si elle a consommé la touche :
//!
//! 1. **Fichier** (`persist`) — `Ctrl+S`, `Ctrl+Maj+S`, `Ctrl+O`, `Ctrl+I`, `Ctrl+E`
//! 2. **Édition** — `Ctrl+V`, `Ctrl+Z`, `Ctrl+Y`, `Ctrl+D`, `Ctrl+A`
//! 3. **Outils** — les lettres nues
//!
//! Le découpage tient la règle § 1.1 : chaque famille est une fonction courte, et ajouter un
//! raccourci n'allonge plus un `match` unique de cent lignes.

use crate::app::GlucoseApp;
use crate::ui::ActiveTool;
use glucose_core::types::ArrowPredicate;

/// Ce que `Maj` fait à un déplacement au clavier : dix pas d'un coup.
///
/// La base décimale, pas une longueur choisie — l'unité fine est celle du monde, et ceci en
/// est la dizaine.
const NUDGE_DECADE: f64 = 10.0;
use glucose_core::store::StackMove;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowLevel;

impl GlucoseApp {
    /// Une touche arrive de la fenêtre. Trois preneurs, dans l'ordre : la saisie d'un nom de
    /// domaine, l'édition d'une annotation, puis les raccourcis globaux. Chacun rend `false`
    /// quand la touche ne le concerne pas ; aucun ne contient de logique (§ 1.7).
    pub fn handle_key(&mut self, event: &KeyEvent) {
        if self.handle_domain_rename_key(event) || self.handle_text_key(event) {
            return;
        }
        self.handle_keyboard_shortcut(event);
    }

    /// Traite les raccourcis clavier hors session d'édition de texte.
    ///
    /// # Invariant KEY-2 — une touche maintenue ne crée pas cinquante objets
    ///
    /// Le système répète une touche maintenue une trentaine de fois par seconde, et le dit :
    /// `KeyEvent::repeat` distingue une vraie frappe d'une répétition. Ce code l'ignorait
    /// entièrement, si bien que maintenir `Ctrl+V` collait en boucle — chaque répétition
    /// écrivant un PNG sur le disque et posant une image de plus sur le canevas.
    ///
    /// Mesuré en conditions réelles : **cinquante images créées par seconde**, trois cent
    /// cinquante-huit au total pour un seul geste, six cent cinquante mégaoctets d'images
    /// décodées, et une frame montée à 1,7 seconde. Vu de l'utilisateur, « l'application
    /// lague à l'import » ; en vérité elle dessinait fidèlement trois cent cinquante-huit
    /// images empilées sur cinquante-huit fois la surface de l'écran.
    ///
    /// C'est ce que les compteurs de la trace ont permis de voir : une durée seule disait
    /// « le rendu est lent », et aucune optimisation de rendu n'aurait touché la cause.
    pub fn handle_keyboard_shortcut(&mut self, event: &KeyEvent) {
        if event.repeat && !repetition_utile(&event.logical_key) {
            return;
        }
        self.handle_shortcut_input(&event.logical_key, event.state);
    }

    /// Le corps de [`GlucoseApp::handle_keyboard_shortcut`], sans le `KeyEvent` de winit,
    /// qui ne se construit pas hors de la boucle d'événements : tout ce qui décide se teste
    /// ici, sans fenêtre (§ 7.1).
    pub fn handle_shortcut_input(&mut self, logical_key: &Key, state: ElementState) {
        if state != ElementState::Pressed {
            return;
        }

        match logical_key {
            Key::Named(NamedKey::Space) => self.enter_pan_mode(),
            Key::Named(NamedKey::Escape) => self.escape_gesture(),
            Key::Named(NamedKey::ArrowLeft) => self.nudge(-1.0, 0.0),
            Key::Named(NamedKey::ArrowRight) => self.nudge(1.0, 0.0),
            Key::Named(NamedKey::ArrowUp) => self.nudge(0.0, -1.0),
            Key::Named(NamedKey::ArrowDown) => self.nudge(0.0, 1.0),
            Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                self.delete_selection();
            }
            Key::Character(ref c) => {
                let key = c.as_str();
                if self.handle_predicate_shortcut(key) {
                    return;
                }
                if self.handle_file_shortcut(key) {
                    return;
                }
                if self.handle_edit_shortcut(key) {
                    return;
                }
                self.handle_tool_shortcut(key);
            }
            _ => {}
        }
    }

    /// `Échap` annule le geste en cours. Aujourd'hui, le seul geste annulable hors édition est
    /// un redimensionnement, qui reprend sa taille de départ ; la fiche 03 § 19.6 en attend
    /// davantage (tout geste courant), et c'est ici que les autres viendront.
    fn escape_gesture(&mut self) {
        // Un menu ouvert est ce qu'on annule en premier : c'est le geste le plus récent, et
        // celui qui attend une décision.
        if self.ui.context_menu_at.take().is_some() {
            self.mark_dirty();
            return;
        }
        // Échap rend toujours l'outil Sélection, comme dans Glucose Tauri : c'est la
        // sortie du mode Pan où la barre d'espace fait entrer, et le seul geste qui ramène
        // à un état connu quel que soit l'outil courant.
        if self.ui.active_tool != ActiveTool::Select {
            self.ui.active_tool = ActiveTool::Select;
            self.update_cursor();
            self.mark_dirty();
            return;
        }
        // Sans toast : la boîte reprend sa taille de départ sous les yeux de celui qui
        // vient d'appuyer. Un message qui décrit ce que l'œil enregistre est du bruit — la
        // même règle que pour l'ordre d'empilement.
        self.cancel_resize();
    }

    /// `Espace` bascule sur l'outil Pan — et y reste.
    ///
    /// C'est le geste de Glucose Tauri (`App.tsx` : `setActiveTool("pan")` sur la touche,
    /// aucun `keyup` pour le défaire), et c'est un mode, pas un maintien : on en sort par
    /// `V` ou `Échap`. Tenir la barre pendant qu'on déplace une carte de l'autre main est
    /// intenable sur un pavé tactile, où le pan est justement le geste le plus fréquent.
    ///
    /// Le bascule fait disparaître l'état `space_pressed` : le mode Pan était dit deux
    /// fois, par une touche tenue **et** par un outil, et deux vérités pour un seul fait
    /// finissent toujours par diverger.
    fn enter_pan_mode(&mut self) {
        self.ui.active_tool = ActiveTool::Pan;
        // Comme Tauri : entrer en navigation lâche ce qu'on tenait. Un pan n'agit sur
        // rien, garder une sélection sous le curseur n'inviterait qu'à la déplacer par
        // mégarde au retour.
        self.store.clear_selection();
        self.update_cursor();
        self.mark_dirty();
    }
    /// Les chiffres posent le **prédicat sémantique** des flèches sélectionnées (PRED-1).
    ///
    /// `1` à `6` dans l'ordre de [`ArrowPredicate::ALL`], `0` retire. Sans modificateur, et
    /// sur toute la sélection d'un coup : c'est le geste qu'il faut pour annoter un graphe,
    /// où l'on qualifie des dizaines de liens à la suite. Un menu à six entrées, ouvert et
    /// refermé à chaque flèche, coûterait trois gestes là où celui-ci en demande un.
    ///
    /// Rend `false` quand la touche n'est pas un chiffre, ou qu'aucune flèche n'est
    /// sélectionnée — le raccourci rend alors la main à ce qui suit, au lieu d'avaler la
    /// touche pour rien.
    fn handle_predicate_shortcut(&mut self, key: &str) -> bool {
        if self.modifiers.control_key() || self.modifiers.alt_key() {
            return false;
        }
        // Le rang d'un chiffre **est** le rang du prédicat (`ArrowPredicate::rank`) : la
        // touche, la couleur et le sigle désignent donc tous le même par le même nombre.
        let rang = match key.chars().next() {
            Some('0') => None,
            Some(c @ '1'..='6') => Some(ArrowPredicate::ALL[c as usize - '1' as usize]),
            _ => return false,
        };
        // Un chiffre suivi d'autre chose n'est pas un chiffre.
        if key.chars().count() != 1 {
            return false;
        }
        // Les **flèches** de la sélection, demandées au modèle, et non toute la sélection :
        // sans quoi la transaction s'ouvrirait avant de savoir s'il y a quelque chose à
        // écrire, et un chiffre tapé sur une carte marquerait le document comme modifié
        // alors que rien ne l'est.
        let fleches: Vec<String> = self
            .store
            .selected_arrows()
            .iter()
            .map(|a| a.id().to_string())
            .collect();
        if fleches.is_empty() {
            return false;
        }
        let board = self.store.project.active_board_id.clone();
        self.store.begin_live_edit();
        self.store.set_arrow_predicate(&board, &fleches, rang);
        self.store.end_live_edit();
        // **Sans toast** : le sigle apparaît, change de couleur ou disparaît sous les yeux de
        // celui qui vient d'appuyer. Un message qui décrit ce que l'œil enregistre est du
        // bruit — la même règle que pour `Échap` et pour l'ordre d'empilement.
        self.mark_dirty();
        true
    }

    /// Raccourcis d'édition du document. Rend `true` si la touche a été consommée.
    fn handle_edit_shortcut(&mut self, key: &str) -> bool {
        if !self.modifiers.control_key() {
            return false;
        }
        match key {
            "c" | "C" => self.copy_selection(false),
            "x" | "X" => self.copy_selection(true),
            "v" | "V" => self.paste_from_clipboard(),
            "z" | "Z" => {
                if self.modifiers.shift_key() {
                    let done = self.store.redo();
                    self.toast_if(done, "Rétablir");
                } else {
                    let done = self.store.undo();
                    self.toast_if(done, "Annuler");
                }
            }
            "y" | "Y" => {
                let done = self.store.redo();
                self.toast_if(done, "Rétablir");
            }
            "d" | "D" => self.duplicate_selection(),
            "a" | "A" => self.select_all(),
            "]" => self.restack(StackMove::Front),
            "[" => self.restack(StackMove::Back),
            _ => return false,
        }
        self.mark_dirty();
        true
    }

    /// Choix d'outil et recentrage : les lettres nues, plus `Alt+T`.
    fn handle_tool_shortcut(&mut self, key: &str) {
        if self.modifiers.control_key() {
            return;
        }
        let tool = match key {
            "h" | "H" => ActiveTool::Pan,
            "v" | "V" => ActiveTool::Select,
            "a" | "A" => ActiveTool::Arrow,
            "n" | "N" => ActiveTool::Sticky,
            "m" | "M" => ActiveTool::Membrane,
            "t" | "T" if self.modifiers.alt_key() => {
                self.toggle_always_on_top();
                return;
            }
            "t" | "T" => ActiveTool::Text,
            "f" | "F" => {
                self.reset_view();
                return;
            }
            "l" | "L" => {
                self.toggle_lock();
                return;
            }
            _ => return,
        };
        self.ui.active_tool = tool;
        self.update_cursor();
        self.mark_dirty();
    }

    fn toast_if(&mut self, happened: bool, message: &str) {
        if happened {
            self.ui.show_toast(message);
        }
    }

    /// Déplace la sélection d'un cran au clavier, pour l'ajustement que la souris ne sait pas
    /// faire.
    ///
    /// Le pas est **l'unité du monde**, et `Maj` le multiplie par dix : deux gestes, aucune
    /// longueur à choisir. Une image verrouillée ne bouge pas — c'est `move_selected` qui le
    /// tient, et le clavier n'a pas à le savoir.
    fn nudge(&mut self, dx: f64, dy: f64) {
        let pas = if self.modifiers.shift_key() {
            NUDGE_DECADE
        } else {
            1.0
        };
        let board = self.store.project.active_board_id.clone();
        self.store.move_selected(&board, dx * pas, dy * pas);
        self.mark_dirty();
    }

    /// Porte la sélection au premier ou au dernier plan de sa couche.
    ///
    /// Sans toast : le nœud passe devant, ou derrière, et cela **se voit**. Un message qui
    /// décrit ce que l'œil vient d'enregistrer est du bruit, pas une confirmation.
    fn restack(&mut self, mv: StackMove) {
        let board = self.store.project.active_board_id.clone();
        if self.store.move_selection_in_stack(&board, mv) > 0 {
            self.mark_dirty();
        }
    }

    /// Duplique la sélection entière.
    pub(crate) fn duplicate_selection(&mut self) {
        let board = self.store.project.active_board_id.clone();
        self.store.duplicate_selected(&board);
        self.ui.show_toast("Dupliqué");
        self.mark_dirty();
    }

    /// Supprime la sélection entière.
    ///
    /// Un geste, un endroit : la touche `Suppr` et le bouton de la barre d'action appellent
    /// la même fonction. Deux chemins vers un même geste finissent toujours par diverger —
    /// l'un oublie le toast, l'autre le `mark_dirty`.
    pub(crate) fn delete_selection(&mut self) {
        let board = self.store.project.active_board_id.clone();
        self.store.delete_selected(&board);
        self.ui.show_toast("Supprimé");
        self.mark_dirty();
    }

    /// Bascule le verrou des images sélectionnées (fiche 08 § 1.3).
    pub(crate) fn toggle_lock(&mut self) {
        let board = self.store.project.active_board_id.clone();
        let Some(locked) = self.store.toggle_lock_selection(&board) else {
            return;
        };
        // La fiche 08 § 1.3 cite le libellé de la référence, cadenas compris. Il tourne dans
        // un navigateur, qui a une police d'emoji ; le natif n'en embarque pas, et le test
        // FONT-1 refuse tout caractère qu'aucun visage ne sait dessiner. Le mot suffit — et
        // c'est le cadre rouge qui dit la chose à l'œil, pas le toast.
        self.ui.show_toast(if locked {
            "Images verrouillées"
        } else {
            "Images déverrouillées"
        });
        self.mark_dirty();
    }

    pub(crate) fn select_all(&mut self) {
        let Some(board) = self.store.active_board() else {
            return;
        };
        let img_ids = board.images.iter().map(|i| i.id.clone()).collect();
        let ann_ids = board
            .annotations
            .iter()
            .map(|a| a.id().to_string())
            .collect();
        self.store.set_selected_image_ids(img_ids);
        self.store.set_selected_annotation_ids(ann_ids);
    }

    /// Recentre la caméra PureRef sur l'origine, à l'échelle 1.
    /// `F` — montrer tout ce qu'il y a, plutôt que revenir à un point d'origine.
    ///
    /// Elle posait [`Viewport::default`], c'est-à-dire l'origine du monde à l'échelle un.
    /// L'utilisateur l'a dit sans détour : « ça ne ramène qu'au point 0 de la map, alors
    /// qu'il NE DEVRAIT PAS Y AVOIR DE POINT 0, c'est un canva infini ». Il a raison, et
    /// c'était une faute de conception, pas un réglage : dans un espace sans bord, l'origine
    /// n'est le centre de rien et ne garantit pas qu'il reste quelque chose à y voir.
    ///
    /// Ce que la touche doit répondre est « où sont mes affaires » — donc le cadrage du
    /// contenu, atteint par un vol pour qu'on voie d'où l'on vient.
    fn reset_view(&mut self) {
        let Some(fenetre) = &self.window else {
            return;
        };
        let taille = fenetre.inner_size();
        let ecran = glucose_core::membrane_focus::ScreenSize {
            width: f64::from(taille.width),
            height: f64::from(taille.height),
        };
        let bandeau = f64::from(self.ui.header_height());
        self.cadrer_sur_le_contenu(ecran, bandeau);
    }

    fn toggle_always_on_top(&mut self) {
        self.always_on_top = !self.always_on_top;
        if let Some(window) = &self.window {
            window.set_window_level(if self.always_on_top {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            });
        }
        self.ui.show_toast(if self.always_on_top {
            "Toujours au premier plan"
        } else {
            "Fenêtre normale"
        });
        self.mark_dirty();
    }
}

/// Les touches dont la **répétition automatique** rend un service, et elles seules (KEY-2).
///
/// La question n'est pas « quelle touche est dangereuse » mais « quelle action a un effet
/// cumulatif que l'utilisateur demande en maintenant la touche ». Deux familles répondent oui :
///
/// * les **flèches**, où chaque répétition avance d'un pas de plus — c'est le geste même du
///   déplacement fin au clavier, et l'interrompre le rendrait inutilisable ;
/// * l'**annulation** et le **rétablissement**, où chaque répétition remonte d'un cran dans
///   une pile — remonter dix fois est une intention courante, et rien n'est créé.
///
/// Tout le reste a un effet **ponctuel** : coller crée un objet, dupliquer aussi, enregistrer
/// écrit un fichier, ouvrir montre un dialogue, un outil se choisit une fois. Les répéter n'est
/// jamais ce qu'on demande en gardant le doigt appuyé.
///
/// La saisie de texte n'a pas à figurer ici : elle est prise plus tôt, par `handle_text_key`,
/// et c'est justement le lieu où la répétition est la raison d'être du mécanisme.
fn repetition_utile(logical_key: &Key) -> bool {
    match logical_key {
        Key::Named(
            NamedKey::ArrowLeft | NamedKey::ArrowRight | NamedKey::ArrowUp | NamedKey::ArrowDown,
        ) => true,
        Key::Character(c) => matches!(c.as_str(), "z" | "Z" | "y" | "Y"),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
