//! Le clavier pendant une saisie : ce que chaque frappe demande, et ce qu'on en fait.
//!
//! # KEY-1 — une frappe se traduit en intention avant d'être exécutée
//!
//! Le traitement d'origine était un `match` sur `Key` dont chaque bras lisait les modificateurs
//! à sa façon, modifiait le tampon, replaçait le curseur et redemandait un dessin. Vingt bras,
//! vingt occasions d'oublier l'un des quatre gestes — et `Maj` n'y était nulle part, parce
//! qu'il aurait fallu le rajouter dans chacun.
//!
//! Ici, [`Command::of`] ne fait que **lire** : une frappe et des modificateurs donnent une
//! intention et un drapeau « étendre la sélection ». [`GlucoseApp::apply_text_command`] ne fait
//! qu'**agir**. Ajouter `Maj` à tous les mouvements est alors une ligne, et non vingt.
//!
//! # Ce que `Maj` fait, et pourquoi c'est une seule règle
//!
//! Étendre une sélection, c'est déplacer la tête **sans toucher à l'ancre** (SEL-1). Tous les
//! mouvements se ramènent donc à « calculer la nouvelle tête », puis à garder l'ancre si `Maj`
//! est enfoncé, ou à la ramener sur la tête sinon. C'est la même ligne pour `←`, `Ctrl+→`,
//! `↑`, `Début` et `Ctrl+Fin`.

use crate::app::GlucoseApp;
use crate::renderer::richtext;
use crate::renderer::richtext::hit::{line_of_offset, offset_to_x, x_to_offset};
use glucose_core::text::selection::{
    delete, line_end, line_start, move_offset, replace, Direction, Motion, Selection,
};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

/// Ce qu'une frappe demande à la saisie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Déplacer la tête d'un mouvement logique — caractère, mot, bord de document.
    Move(Motion, Direction),
    /// Monter ou descendre d'une ligne **visuelle** : cela dépend du reflux, donc de la
    /// mise en page, et non du texte seul.
    MoveLine(Direction),
    /// Aller au bord de la ligne **visuelle** (`Début` / `Fin`).
    MoveVisualEdge(Direction),
    /// Tout sélectionner.
    SelectAll,
    /// Écrire — en remplaçant la sélection s'il y en a une.
    Insert(String),
    /// Effacer la sélection, ou d'un mouvement si elle est vide.
    Delete(Motion, Direction),
    Copy,
    Cut,
    Paste,
    /// Valider la saisie et sortir.
    Commit,
}

impl Command {
    /// L'intention d'une frappe, et si elle étend la sélection au lieu de la déplacer.
    ///
    /// `None` quand la frappe ne veut rien dire ici : elle est alors **avalée** malgré tout,
    /// parce qu'une saisie active a la priorité sur les raccourcis globaux — sans quoi taper
    /// « n » dans une carte créerait un pense-bête.
    pub fn of(key: &Key, mods: &ModifiersState, text: Option<&str>) -> Option<(Self, bool)> {
        let extend = mods.shift_key();
        let word = mods.control_key();
        let motion = |par_mot: bool| {
            if par_mot {
                Motion::Word
            } else {
                Motion::Char
            }
        };
        let command = match key {
            Key::Named(NamedKey::ArrowLeft) => Self::Move(motion(word), Direction::Backward),
            Key::Named(NamedKey::ArrowRight) => Self::Move(motion(word), Direction::Forward),
            Key::Named(NamedKey::ArrowUp) => Self::MoveLine(Direction::Backward),
            Key::Named(NamedKey::ArrowDown) => Self::MoveLine(Direction::Forward),
            Key::Named(NamedKey::Home) if word => Self::Move(Motion::DocEdge, Direction::Backward),
            Key::Named(NamedKey::End) if word => Self::Move(Motion::DocEdge, Direction::Forward),
            Key::Named(NamedKey::Home) => Self::MoveVisualEdge(Direction::Backward),
            Key::Named(NamedKey::End) => Self::MoveVisualEdge(Direction::Forward),
            Key::Named(NamedKey::Backspace) => Self::Delete(motion(word), Direction::Backward),
            Key::Named(NamedKey::Delete) => Self::Delete(motion(word), Direction::Forward),
            Key::Named(NamedKey::Escape) => Self::Commit,
            // `Entrée` va à la ligne ; `Ctrl+Entrée` et `Échap` valident.
            //
            // C'est ce que fait Glucose Tauri (`GlucoseCanvas.tsx` n'intercepte que
            // `Ctrl+Entrée` ; le reste tombe dans un `textarea`), et c'est ce que fait toute
            // zone de texte multiligne. La règle inverse rendait la saisie intenable pour
            // une raison qu'aucun test ne pouvait voir : une phrase sur deux commence par
            // une majuscule, donc `Maj` était déjà enfoncé au moment du retour à la ligne,
            // donc le saut de ligne marchait **une fois sur deux**. Un comportement qui
            // dépend d'un modificateur tenu pour une autre raison n'est pas une règle,
            // c'est un piège.
            Key::Named(NamedKey::Enter) if word => Self::Commit,
            Key::Named(NamedKey::Enter) => Self::Insert("\n".into()),
            Key::Named(NamedKey::Space) if !word && !mods.alt_key() => Self::Insert(" ".into()),
            Key::Character(c) if word => match c.as_str() {
                "a" | "A" => Self::SelectAll,
                "c" | "C" => Self::Copy,
                "x" | "X" => Self::Cut,
                "v" | "V" => Self::Paste,
                _ => return None,
            },
            // Ce que la couche de composition a produit : un caractère accentué, un idéogramme,
            // ce qu'une touche morte a fini de composer (IME).
            _ => match text.filter(|_| !mods.control_key() && !mods.alt_key()) {
                Some(t) if !t.is_empty() && !t.chars().any(|c| c.is_control()) => {
                    Self::Insert(t.to_string())
                }
                _ => return None,
            },
        };
        Some((command, extend))
    }
}

/// La frappe est-elle une commande de fichier (`Ctrl+S`, `Ctrl+Maj+S`, `Ctrl+O`) ?
///
/// Une session d'édition avale TOUTES les touches — c'est ce qui permet de taper `s` dans une
/// carte sans déclencher un raccourci. Mais `Ctrl+S` au milieu d'une phrase veut dire
/// « enregistre », pas « ignore-moi » : sans cette exception, enregistrer serait impossible
/// tant qu'un curseur clignote quelque part.
pub fn is_file_command(modifiers: &ModifiersState, key: &Key) -> bool {
    if !modifiers.control_key() {
        return false;
    }
    matches!(key, Key::Character(c) if matches!(c.as_str(), "s" | "S" | "o" | "O"))
}

impl GlucoseApp {
    /// Traite une frappe pendant une saisie. Rend `false` pour la laisser descendre aux
    /// raccourcis globaux.
    pub fn handle_text_key(&mut self, event: &KeyEvent) -> bool {
        if self.editing_session.is_none() || event.state != ElementState::Pressed {
            return self.editing_session.is_some();
        }
        // Enregistrer ou ouvrir pendant une saisie : on valide d'abord le texte en cours, puis
        // on laisse la touche descendre.
        if is_file_command(&self.modifiers, &event.logical_key) {
            self.commit_editing();
            self.mark_dirty();
            return false;
        }
        let mods = self.modifiers;
        if let Some((command, extend)) = Command::of(
            &event.logical_key,
            &mods,
            event.text.as_ref().map(|t| t.as_str()),
        ) {
            self.apply_text_command(command, extend);
        }
        // Avalée dans tous les cas : une carte en édition a la main sur le clavier.
        true
    }

    /// Exécute une intention sur la saisie en cours.
    pub(crate) fn apply_text_command(&mut self, command: Command, extend: bool) {
        match command {
            Command::Commit => {
                self.commit_editing();
                self.mark_dirty();
                return;
            }
            Command::Copy | Command::Cut => self.copy_selected_text(command == Command::Cut),
            Command::Paste => {
                if let Some(texte) = self.clipboard_text() {
                    self.write_text(&texte);
                }
            }
            Command::Insert(texte) => self.write_text(&texte),
            Command::Delete(motion, dir) => self.delete_text(motion, dir),
            Command::SelectAll => {
                if let Some(session) = &mut self.editing_session {
                    session.selection = Selection::all(&session.buffer);
                }
            }
            Command::Move(motion, dir) => self.move_head(extend, |app, sel| {
                let buffer = &app.editing_session.as_ref().expect("session").buffer;
                // Sans `Maj`, une flèche sur une sélection ne bouge pas d'un caractère : elle
                // retombe du bon côté. C'est ce que fait tout éditeur, et son absence se voit
                // aussitôt — le curseur saute un caractère de trop.
                if !extend && !sel.is_empty() && motion == Motion::Char {
                    return sel.collapsed(dir).head;
                }
                move_offset(buffer, sel.head, motion, dir)
            }),
            Command::MoveLine(dir) => {
                // COLUMN-1 : la colonne visée survit à la suite de mouvements verticaux, et
                // c'est le seul cas où elle n'est pas effacée en fin de commande.
                self.remember_goal_x();
                self.move_head(extend, |app, sel| app.line_step(sel, dir));
                self.after_text_change();
                return;
            }
            Command::MoveVisualEdge(dir) => {
                self.move_head(extend, |app, sel| app.visual_edge(sel, dir))
            }
        }
        if let Some(session) = &mut self.editing_session {
            session.goal_x = None;
        }
        self.after_text_change();
    }

    /// Pose la colonne visée si elle ne l'est pas déjà (COLUMN-1).
    fn remember_goal_x(&mut self) {
        if self
            .editing_session
            .as_ref()
            .is_some_and(|s| s.goal_x.is_some())
        {
            return;
        }
        let Some((layout, width)) = self.editing_layout() else {
            return;
        };
        let Some(session) = self.editing_session.as_ref() else {
            return;
        };
        let index = line_of_offset(&layout, session.selection.head);
        let Some(line) = layout.lines.get(index) else {
            return;
        };
        let bx = crate::renderer::card::text_box(width);
        let x = offset_to_x(
            &self.renderer.typography,
            &layout,
            line,
            &session.buffer,
            session.selection.head,
            richtext::font_of(line.kind, bx.body),
        );
        if let Some(session) = self.editing_session.as_mut() {
            session.goal_x = Some(x);
        }
    }

    /// Déplace la tête, et emmène l'ancre avec elle si l'on n'étend pas (SEL-1).
    fn move_head(&mut self, extend: bool, head: impl FnOnce(&Self, Selection) -> usize) {
        let Some(session) = self.editing_session.as_ref() else {
            return;
        };
        let selection = session.selection;
        let to = head(self, selection);
        let Some(session) = self.editing_session.as_mut() else {
            return;
        };
        session.selection.head = to;
        if !extend {
            session.selection.anchor = to;
        }
    }

    /// L'offset une ligne visuelle plus haut ou plus bas, **colonne gardée**.
    ///
    /// La colonne est celle où le curseur est dessiné, pas son rang dans le texte : passer
    /// d'une ligne de titre à une ligne de corps ne doit pas décaler le curseur, alors que les
    /// deux n'ont ni la même police ni le même nombre de caractères par ligne.
    fn line_step(&self, selection: Selection, dir: Direction) -> usize {
        let Some((layout, width)) = self.editing_layout() else {
            return self.logical_line_step(selection, dir);
        };
        let Some(session) = self.editing_session.as_ref() else {
            return selection.head;
        };
        let source = &session.buffer;
        let index = line_of_offset(&layout, selection.head);
        let cible = match dir {
            Direction::Backward if index == 0 => return 0,
            Direction::Backward => index - 1,
            Direction::Forward if index + 1 >= layout.lines.len() => return source.len(),
            Direction::Forward => index + 1,
        };
        let bx = crate::renderer::card::text_box(width);
        // COLUMN-1 : l'abscisse visée est celle du premier `↑` ou `↓` de la série, pas celle
        // de la position courante — sans quoi le curseur dériverait vers la gauche, ligne
        // après ligne, en s'arrondissant chaque fois à la frontière la plus proche.
        let depart = &layout.lines[index];
        let x = session.goal_x.unwrap_or_else(|| {
            offset_to_x(
                &self.renderer.typography,
                &layout,
                depart,
                source,
                selection.head,
                richtext::font_of(depart.kind, bx.body),
            )
        });
        let arrivee = &layout.lines[cible];
        x_to_offset(
            &self.renderer.typography,
            &layout,
            arrivee,
            source,
            x,
            richtext::font_of(arrivee.kind, bx.body),
        )
    }

    /// Le bord de la ligne **visuelle** où se trouve la tête.
    ///
    /// `Début` et `Fin` parlent de ce qu'on voit : sur un paragraphe reflué en quatre lignes,
    /// `Début` va au début de la ligne sous le curseur, pas au début du paragraphe. C'est
    /// pourquoi ce mouvement ne peut pas vivre dans le noyau, qui ne connaît pas le reflux.
    fn visual_edge(&self, selection: Selection, dir: Direction) -> usize {
        let Some((layout, _)) = self.editing_layout() else {
            // Sans mise en page refluée, la ligne visuelle est la ligne logique — exact tant
            // que le paragraphe tient sur une ligne, prévisible sinon.
            return self.editing_session.as_ref().map_or(selection.head, |s| {
                move_offset(&s.buffer, selection.head, Motion::LineEdge, dir)
            });
        };
        let index = line_of_offset(&layout, selection.head);
        match layout.lines.get(index) {
            Some(line) => match dir {
                Direction::Backward => line.start,
                Direction::Forward => line.end,
            },
            None => selection.head,
        }
    }

    /// `↑` et `↓` **sans mise en page refluée** : d'un paragraphe à l'autre, colonne gardée en
    /// caractères.
    ///
    /// Ce repli sert les pense-bêtes et les membranes, dont le texte se reflue encore par un
    /// autre chemin que [`crate::renderer::richtext`]. Il est moins juste qu'un vrai pas de
    /// ligne visuelle — sur un paragraphe coupé en trois, il les saute d'un coup — mais il est
    /// prévisible, et infiniment préférable à une touche qui ne fait rien.
    fn logical_line_step(&self, selection: Selection, dir: Direction) -> usize {
        let Some(session) = self.editing_session.as_ref() else {
            return selection.head;
        };
        let texte = &session.buffer;
        let debut = line_start(texte, selection.head);
        let colonne = texte[debut..selection.head].chars().count();
        let voisin = match dir {
            Direction::Backward if debut == 0 => return 0,
            Direction::Backward => line_start(texte, debut - 1),
            Direction::Forward => {
                let fin = line_end(texte, selection.head);
                if fin >= texte.len() {
                    return texte.len();
                }
                fin + 1
            }
        };
        let fin_voisin = line_end(texte, voisin);
        texte[voisin..fin_voisin]
            .char_indices()
            .nth(colonne)
            .map_or(fin_voisin, |(i, _)| voisin + i)
    }

    /// Écrit `texte` à la place de la sélection.
    fn write_text(&mut self, texte: &str) {
        if let Some(session) = &mut self.editing_session {
            session.selection = replace(&mut session.buffer, session.selection, texte);
        }
    }

    /// Efface la sélection, ou d'un mouvement si elle est vide.
    fn delete_text(&mut self, motion: Motion, dir: Direction) {
        if let Some(session) = &mut self.editing_session {
            session.selection = delete(&mut session.buffer, session.selection, motion, dir);
        }
    }

    /// Le curseur se rallume à chaque geste : il ne doit pas être éteint au moment précis où
    /// l'on vient de taper, sinon on croit que la frappe n'a pas été prise.
    fn after_text_change(&mut self) {
        if let Some(session) = &mut self.editing_session {
            session.blink_timer = std::time::Instant::now();
        }
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
