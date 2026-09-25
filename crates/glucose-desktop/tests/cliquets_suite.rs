//! **Les cliquets** : des nombres que le dépôt ne peut que faire descendre.
//!
//! # Pourquoi ce test existe
//!
//! Chaque dérive de ce dépôt a été constatée, écrite dans une fiche, et s'est reproduite :
//! les toasts sont passés de 24 à 46 pendant qu'un audit demandait de les réduire ;
//! `window_event` a atteint 728 lignes, a été éclaté, et sa masse s'est reposée dans
//! `handle_mouse_down` ; des modules du noyau ont été écrits, testés, et jamais appelés,
//! pendant que d'autres naissaient. La fiche 11 (§ 4, étape A) en a tiré la seule conclusion
//! qui tienne : **une règle qu'on ne mesure pas est une règle qu'on perd.**
//!
//! # La mécanique
//!
//! Chaque cliquet mesure un nombre et le compare à un plafond écrit ici. Monter fait échouer
//! le build. Descendre nettement fait échouer le build *aussi* — pour que le plafond soit
//! abaissé et que le terrain gagné ne se reperde pas. Le correctif d'un échec n'est jamais de
//! relever un plafond : c'est de faire ce que le message demande.
//!
//! # Ce qui n'est pas compté
//!
//! Les tests et les preuves. Ils ont le droit de regarder le modèle de près, d'être longs,
//! et d'appeler ce qu'ils veulent : c'est leur travail. Seul le code qui **tourne chez
//! l'utilisateur** est soumis aux cliquets.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

// ── Le balayage ─────────────────────────────────────────────────────────────

/// Un fichier de production : son chemin, et son texte **sans** son bloc de tests en ligne.
struct Source {
    chemin: PathBuf,
    texte: String,
}

/// Tous les `.rs` de production sous une racine : ni les fichiers de tests ou de preuves, ni
/// ce qu'un `#[cfg(test)]` retire de la production dans un fichier qui en contient.
fn sources(racine: &Path) -> Vec<Source> {
    let mut out = Vec::new();
    let Ok(entrees) = fs::read_dir(racine) else {
        return out;
    };
    let mut chemins: Vec<PathBuf> = entrees.flatten().map(|e| e.path()).collect();
    chemins.sort();
    for chemin in chemins {
        let nom = chemin.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if chemin.is_dir() {
            if nom != "tests" {
                out.extend(sources(&chemin));
            }
        } else if nom.ends_with(".rs") && !nom.contains("tests") && !nom.contains("proof") {
            let Ok(texte) = fs::read_to_string(&chemin) else {
                continue;
            };
            out.push(Source {
                chemin,
                texte: sans_les_tests(&texte),
            });
        }
    }
    out
}

/// Le texte d'un fichier privé de tout ce qu'un `#[cfg(test)]` en retire : l'item qui suit
/// chaque attribut, quel qu'il soit — un module, une fonction, un `use`.
///
/// # Le défaut que cette fonction corrige, et ce qu'il cachait
///
/// La première version coupait le texte au **premier** `#[cfg(test)]`, en supposant que
/// c'était le bloc `mod tests` de la fin. Un `#[cfg(test)] pub fn toast_message` à la ligne
/// 279 de `ui.rs` suffisait donc à faire disparaître les mille trois cents lignes qui
/// suivaient : le cliquet des tailles voyait un fichier de 279 lignes là où il y en avait
/// 1 650 de production, et dix fichiers étaient dans ce cas — `typography.rs` coupé à sa
/// ligne 47, `theme.rs` à sa ligne 362. **Un cliquet qui se trompe dans ce sens efface une
/// dette au lieu de la montrer.**
///
/// L'item qui suit l'attribut se termine au premier `;` s'il n'ouvre pas d'accolade avant,
/// et à l'accolade fermante équilibrée sinon — la règle que [`fonctions`] applique déjà.
fn sans_les_tests(texte: &str) -> String {
    let lignes: Vec<&str> = texte.lines().collect();
    let mut out = String::with_capacity(texte.len());
    let mut i = 0;
    while i < lignes.len() {
        if lignes[i].trim() != "#[cfg(test)]" {
            out.push_str(lignes[i]);
            out.push('\n');
            i += 1;
            continue;
        }
        // L'item retiré commence à la ligne suivante, et finit là où sa syntaxe le dit.
        let mut profondeur = 0i32;
        let mut ouverte = false;
        let mut j = i + 1;
        while j < lignes.len() {
            let propre = nue(lignes[j]);
            if !ouverte && propre.contains(';') && !propre.contains('{') {
                break;
            }
            for c in propre.chars() {
                match c {
                    '{' => {
                        profondeur += 1;
                        ouverte = true;
                    }
                    '}' => profondeur -= 1,
                    _ => {}
                }
            }
            if ouverte && profondeur <= 0 {
                break;
            }
            j += 1;
        }
        i = j + 1;
    }
    out
}

/// **Le lecteur de production ne s'arrête pas au premier `#[cfg(test)]`.** Il décide de
/// tous les cliquets qui lisent la production ; s'il se trompe, ce sont eux qui mentent.
#[test]
fn test_le_lecteur_de_production_retire_chaque_item_de_test_et_rien_d_autre() {
    let texte = "\
fn avant() {
    let s = \"{\";
}

#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
pub fn pour_les_tests(&self) -> u32 {
    if vrai {
        1
    } else {
        2
    }
}

fn apres() {
    x();
}

#[cfg(test)]
mod tests {
    fn cache() {}
}
";
    let production = sans_les_tests(texte);
    let noms: Vec<String> = fonctions(&production).into_iter().map(|(n, _)| n).collect();
    assert_eq!(
        noms,
        vec!["avant".to_string(), "apres".to_string()],
        "la production est ce qui n'est pas sous un #[cfg(test)], et tout cela : {production}"
    );
    assert!(
        !production.contains("HashMap"),
        "un `use` de test est retiré"
    );
    assert!(
        !production.contains("cache"),
        "le module de tests est retiré"
    );
}

fn racine_du_workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("le workspace est deux niveaux au-dessus du crate")
}

fn src_desktop() -> PathBuf {
    racine_du_workspace()
        .join("crates")
        .join("glucose-desktop")
        .join("src")
}

fn src_core() -> PathBuf {
    racine_du_workspace()
        .join("crates")
        .join("glucose-core")
        .join("src")
}

/// Le chemin d'une source, court et lisible dans un message d'échec.
fn court(chemin: &Path) -> String {
    let racine = racine_du_workspace();
    chemin
        .strip_prefix(&racine)
        .unwrap_or(chemin)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Une ligne de code sans ses chaînes ni son commentaire de fin : ce qu'il reste compte des
/// accolades et des appels, rien d'autre.
fn nue(ligne: &str) -> String {
    let mut out = String::with_capacity(ligne.len());
    let chars: Vec<char> = ligne.chars().collect();
    let mut i = 0;
    let mut dans_chaine = false;
    while i < chars.len() {
        let c = chars[i];
        if dans_chaine {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                dans_chaine = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => dans_chaine = true,
            '/' if chars.get(i + 1) == Some(&'/') => break,
            // Un caractère littéral — `'{'`, `'\n'` — n'est pas une accolade ni une durée de vie.
            '\'' if chars.get(i + 2) == Some(&'\'') => {
                i += 3;
                continue;
            }
            '\'' if chars.get(i + 1) == Some(&'\\') && chars.get(i + 3) == Some(&'\'') => {
                i += 4;
                continue;
            }
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

fn est_debut_de_fonction(ligne: &str) -> Option<String> {
    let l = ligne.trim_start();
    let l = l.strip_prefix("pub ").unwrap_or(l);
    let l = l.strip_prefix("pub(crate) ").unwrap_or(l);
    let l = l.strip_prefix("pub(super) ").unwrap_or(l);
    let mut l = l;
    for prefixe in ["const ", "async ", "unsafe ", "extern \"C\" "] {
        l = l.strip_prefix(prefixe).unwrap_or(l);
    }
    let reste = l.strip_prefix("fn ")?;
    let nom: String = reste
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!nom.is_empty()).then_some(nom)
}

/// Les fonctions d'un fichier avec leur longueur en lignes, signature et accolade fermante
/// comprises.
fn fonctions(texte: &str) -> Vec<(String, usize)> {
    let lignes: Vec<&str> = texte.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lignes.len() {
        let Some(nom) = est_debut_de_fonction(lignes[i]) else {
            i += 1;
            continue;
        };
        let debut = i;
        let mut profondeur = 0i32;
        let mut ouverte = false;
        let mut fin = None;
        for (j, ligne) in lignes.iter().enumerate().skip(debut) {
            let propre = nue(ligne);
            if !ouverte && propre.contains(';') && !propre.contains('{') {
                // Une déclaration de trait sans corps.
                break;
            }
            for c in propre.chars() {
                match c {
                    '{' => {
                        profondeur += 1;
                        ouverte = true;
                    }
                    '}' => profondeur -= 1,
                    _ => {}
                }
            }
            if ouverte && profondeur <= 0 {
                fin = Some(j);
                break;
            }
        }
        match fin {
            Some(f) => {
                out.push((nom, f - debut + 1));
                i = f + 1;
            }
            None => i += 1,
        }
    }
    out
}

// ── Cliquet 1 : le couplage du desktop au modèle (règle S) ──────────────────

/// Les champs de collection du modèle. Les toucher directement, c'est connaître la
/// représentation.
const CHAMPS: &[&str] = &[".annotations", ".images", ".folders", ".boards"];

/// Le plafond, relevé au commit qui a introduit ce cliquet. Il ne doit que **descendre**.
///
/// Ce n'est pas un objectif de qualité mais un cliquet : chaque fonctionnalité neuve qui passe
/// par l'API du `Store` laisse ce nombre où il est, et chaque conversion d'un site existant le
/// fait baisser. Le jour où il atteint zéro, la substitution de l'arène ne touche plus que le
/// `store` (fiche 12 § 3).
///
/// Il valait 66 quand il comptait aussi les tests en ligne des fichiers de production ; il n'en
/// compte plus que le code, et trente accès directs vivaient dans ces tests.
///
/// # Pourquoi il est passé de 36 à 51 sans qu'un seul accès ait été ajouté
///
/// Le lecteur de production coupait un fichier au **premier** `#[cfg(test)]`, en supposant que
/// c'était le `mod tests` de la fin. Dix fichiers en avaient un plus haut — une fonction de
/// test, un `use` — et tout ce qui suivait disparaissait : `ui.rs` était lu sur 279 de ses
/// 1 650 lignes de production, `typography.rs` sur 47 de ses 679. Ce plafond ne disait donc
/// pas « 36 accès dans le desktop » mais « 36 accès dans ce que le lecteur savait lire ».
///
/// Le lecteur retire désormais l'item qui suit chaque attribut, et rien d'autre ; il se prouve
/// par son propre test. La mesure vraie est **51**, et c'est elle que ce plafond porte : le
/// remonter à ce qui EXISTE n'est pas relâcher la règle, c'est cesser d'effacer une dette.
/// Un seul a été remboursé au passage — la barre d'onglets, qui lisait la collection des
/// tableaux et passe par `Store::onglets`.
///
/// Le reste est nommé : la minimap lit les trois collections d'un tableau pour les dessiner,
/// et le redimensionnement cherche un nœud par son identifiant. Les deux demandent une API de
/// lecture que le `Store` n'a pas encore, et c'est le chantier de la règle S (fiche 12 § 3).
///
/// **38 depuis le 24/09/2026 (HISTOIRE-1)** : l'enregistrement relisait chaque image du
/// document, tableau par tableau, pour la réécrire ; il écrit désormais le geste, et c'est le
/// noyau qui répond « toutes les images » et « les images qu'un geste pose ». Cinq accès
/// directs sont partis avec lui.
const PLAFOND_COUPLAGE: usize = 38;

fn compte_couplage() -> usize {
    sources(&src_desktop())
        .iter()
        .flat_map(|s| s.texte.lines())
        .filter(|ligne| !ligne.trim_start().starts_with("//"))
        .filter(|ligne| CHAMPS.iter().any(|c| touche_le_champ(ligne, c)))
        .count()
}

/// Cette ligne touche-t-elle **ce champ**, et non une méthode qui commence pareil ?
///
/// # Le faux positif que cette fonction supprime, et pourquoi il comptait
///
/// La recherche se faisait par sous-chaîne : `self.images_au_dessus(budget)` — une méthode de
/// la chronique qui ne connaît rien au modèle — était comptée comme un accès direct à
/// `.images`, et le cliquet a fait échouer la build en réclamant « passer par l'API du Store »
/// pour du code qui n'en sort jamais.
///
/// **Un cliquet qui se trompe est pire qu'un cliquet absent** : il fait faire le contraire de
/// ce qu'il défend, et la réponse tentante — relever le plafond — efface une dette réelle au
/// passage. Le nom d'un champ se termine donc là où un identifiant Rust ne continue plus.
fn touche_le_champ(ligne: &str, champ: &str) -> bool {
    let mut reste = ligne;
    while let Some(i) = reste.find(champ) {
        let apres = &reste[i + champ.len()..];
        let suivant = apres.chars().next();
        let suite_est_un_identifiant = suivant.is_some_and(|c| c.is_alphanumeric() || c == '_');
        // **Un appel de methode n'est pas un acces a un champ**, et c'est tout l'inverse :
        // `Visibles::images()` est precisement l'API par laquelle on CESSE de lire la
        // collection en direct. La compter faisait monter le plafond a chaque fois qu'on
        // passait un site a l'API -- le cliquet punissait ce qu'il existe pour encourager.
        let suite_est_un_appel = suivant == Some('(');
        if !suite_est_un_identifiant && !suite_est_un_appel {
            return true;
        }
        reste = apres;
    }
    false
}

/// **Le lecteur d'accès au modèle se prouve lui-même**, comme celui du cliquet 3.
#[test]
fn test_le_lecteur_d_acces_au_modele_ne_confond_pas_un_champ_et_une_methode() {
    for vrai in [
        "for image in &board.images {",
        "self.store.project.boards.len()",
        "b.annotations.push(a);",
        "let n = doc.folders;",
    ] {
        assert!(
            CHAMPS.iter().any(|c| touche_le_champ(vrai, c)),
            "cet acces direct doit etre compte : {vrai}"
        );
    }
    for faux in [
        "let ratees = self.images_au_dessus(BUDGET);",
        "self.images_mo = 0;",
        "compteur(\"img_n\", self.boards_count as f64)",
        "vu.annotations_us = 0;",
        // Les APPELS : l'API meme par laquelle on cesse de lire le modele en direct.
        "for img in Visibles::nouvelles(&rangs, board).images() {",
        "let n = store.annotations().len();",
    ] {
        assert!(
            !CHAMPS.iter().any(|c| touche_le_champ(faux, c)),
            "ceci n'est pas un acces au modele, mais il a ete compte : {faux}"
        );
    }
}

/// **Le couplage du desktop au modèle ne grandit pas.**
///
/// Si ce test échoue en disant que le compte a monté, c'est qu'un module neuf lit ou écrit les
/// collections du modèle en direct. Le correctif n'est jamais de relever le plafond : c'est de
/// passer par l'API du `Store`, en l'étendant si elle ne sait pas encore faire ce qu'il faut.
#[test]
fn test_cliquet_1_le_couplage_du_desktop_au_modele_ne_grandit_pas() {
    let compte = compte_couplage();
    assert!(
        compte <= PLAFOND_COUPLAGE,
        "le couplage au modèle a monté : {compte} accès directs contre {PLAFOND_COUPLAGE} \
         autorisés. Passer par l'API du Store (règle S, fiche 12 § 3) plutôt que par les \
         champs du modèle."
    );
    assert!(
        compte >= PLAFOND_COUPLAGE.saturating_sub(4),
        "le couplage est descendu à {compte} : abaisser PLAFOND_COUPLAGE à cette valeur pour \
         que le terrain gagné ne se reperde pas"
    );
}

// ── Cliquet 2 : les toasts ──────────────────────────────────────────────────

/// Le nombre de sites qui émettent un toast, relevé au commit qui a introduit ce cliquet.
///
/// Un toast n'a le droit de dire qu'une chose : ce qui **vient d'avoir lieu**. L'audit en
/// comptait 24, dont dix décrivaient une action qui n'avait pas lieu ; ils sont 45 aujourd'hui.
/// Le glisser-dépose en a ajouté deux et fait disparaître trois : chaque outil de création
/// portait son propre message, alors que c'est le même événement, et un lot déposé rend un
/// seul compte au lieu d'un par fichier.
/// Le nombre ne prouve pas la vérité de chaque message, mais il interdit la prolifération, et
/// chaque site retiré est un site de moins à relire.
///
/// L'export en a ajouté **un** — un seul pour les deux issues, parce que réussir et échouer
/// sont la même phrase à dire au même endroit : ce qui vient d'avoir lieu. Et il en a retiré
/// un : le bouton « Exporter » n'a plus à annoncer qu'il ne sait pas exporter.
///
/// Les signets de vue en ajoutent **un** pour deux gestes. Poser un signet ne se voit pas, et
/// rappeler un signet vide ne se voit pas non plus : ce sont les deux seuls cas où il faut
/// parler. Un vol réussi, lui, se regarde — donc il se tait.
///
/// # Pourquoi il est passé de 46 à 48 sans qu'un seul toast ait été ajouté
///
/// Même cause que [`PLAFOND_COUPLAGE`] : le lecteur coupait `ui.rs` à sa ligne 279, et les
/// deux sites de `handle_ui_click` n'étaient pas comptés. Ils existaient depuis toujours.
///
/// Et l'un des deux est exactement ce que ce cliquet défend : `NOT_YET_COLLAB` annonce une
/// fonctionnalité qui n'existe pas, ce que la fiche 05 § 5.4 interdit — un bouton dont
/// l'action n'existe pas est **absent ou grisé**, jamais un message qui simule. Le retirer
/// change ce que l'utilisateur voit, donc cela lui revient ; la dette est nommée ici en
/// attendant, et ce plafond descendra d'un cran ce jour-là.
const PLAFOND_TOASTS: usize = 48;

fn compte_toasts() -> usize {
    sources(&src_desktop())
        .iter()
        .flat_map(|s| s.texte.lines())
        .map(nue)
        .filter(|ligne| ligne.contains("show_toast(") && !ligne.contains("fn show_toast"))
        .count()
}

/// **Le nombre de toasts ne grandit pas.**
///
/// Un échec à la hausse veut dire qu'un nouveau message a été ajouté. Avant de relever quoi
/// que ce soit : ce message décrit-il une action qui vient d'avoir lieu ? Si oui, un autre
/// toast peut sans doute disparaître à la place ; si non, c'est un bouton qui ment (fiche 05
/// § 5.4), et il ne passe pas.
#[test]
fn test_cliquet_2_le_nombre_de_toasts_ne_grandit_pas() {
    let compte = compte_toasts();
    assert!(
        compte <= PLAFOND_TOASTS,
        "un toast de plus : {compte} sites contre {PLAFOND_TOASTS} autorisés. Un toast ne dit \
         que ce qui vient d'avoir lieu — et un de plus, c'est un de trop à relire."
    );
    assert!(
        compte >= PLAFOND_TOASTS.saturating_sub(4),
        "les toasts sont descendus à {compte} : abaisser PLAFOND_TOASTS à cette valeur"
    );
}

// ── Cliquet 3 : les modules du noyau sans appelant ──────────────────────────

/// Les modules de `glucose-core` qu'aucun chemin de production n'atteint, **admis** tels quels
/// au commit qui a introduit ce cliquet. Chacun a son chantier dans la fiche 12 ; aucun autre
/// ne doit les rejoindre, et chacun doit être retiré d'ici le jour où il est branché.
///
/// « Atteint » se calcule, il ne s'affirme pas : un module est atteint s'il est nommé par le
/// desktop en production, ou par un module du noyau lui-même atteint — la fermeture
/// transitive depuis l'application. Un module atteint n'est pas pour autant *branché* au sens
/// de la règle R1 (visible, annulable, enregistré) : ce cliquet mesure le graphe d'appel,
/// pas l'expérience de l'utilisateur.
const MODULES_SANS_APPELANT_ADMIS: &[&str] = &[
    // La fondation 10⁷ (fiche 11, étape B), en attente de sa substitution (fiche 12, vague 4).
    "arena",
    "fixed",
    // Écrits et testés, en attente de leur geste (fiche 12 § 4).
    "curtain_model",
    "curtain_panel",
    "membrane_stretch",
    "mirror_graph",
    "timeline",
    // Outillage de mesure : les documents synthétiques des bancs et de la capture témoin. Il
    // vit dans le noyau pour rester sans dépendance et se tester sans écran ; il n'a pas
    // vocation à être atteint par l'application, seulement par les exemples et les tests.
    "synth",
];

/// Les modules déclarés par `lib.rs`.
fn modules_du_noyau() -> BTreeSet<String> {
    let lib = fs::read_to_string(src_core().join("lib.rs")).expect("lib.rs du noyau");
    lib.lines()
        .filter_map(|l| l.trim().strip_prefix("pub mod "))
        .filter_map(|l| l.strip_suffix(';'))
        .map(str::to_string)
        .collect()
}

/// Les identifiants de tête qui suivent un `prefixe::` dans un texte : `prefixe::a::b` donne
/// `a`, et `prefixe::{a, b::c, d}` donne `a`, `b` et `d` — y compris sur plusieurs lignes,
/// puisque rustfmt éclate les imports longs.
/// Le texte privé de ses commentaires et de ses chaînes.
///
/// Sans cela, un lien de documentation — ``[`crate::text_anchors`]`` — compte comme un appel,
/// et le cliquet 3 annonce qu'un module mort vient d'être branché. **Un cliquet qui se trompe
/// dans ce sens est pire que pas de cliquet** : il efface une dette au lieu de la montrer, et
/// il le fait au moment précis où quelqu'un documente le module concerné.
///
/// Les commentaires de bloc s'imbriquent, comme en Rust. Les chaînes disparaissent aussi :
/// un chemin cité dans un message d'erreur n'est pas davantage un appel.
fn code_seul(texte: &str) -> String {
    let mut out = String::with_capacity(texte.len());
    let bytes = texte.as_bytes();
    let mut i = 0;
    let mut profondeur = 0usize;
    while i < bytes.len() {
        if profondeur > 0 {
            match (bytes[i], bytes.get(i + 1)) {
                (b'/', Some(b'*')) => {
                    profondeur += 1;
                    i += 2;
                }
                (b'*', Some(b'/')) => {
                    profondeur -= 1;
                    i += 2;
                }
                _ => i += 1,
            }
            continue;
        }
        match (bytes[i], bytes.get(i + 1)) {
            (b'/', Some(b'/')) => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            (b'/', Some(b'*')) => {
                profondeur = 1;
                i += 2;
            }
            (b'"', _) => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                i += 1;
            }
            (c, _) => {
                // Un chemin `crate::…` est en ASCII ; tout octet qui n'en est pas devient une
                // espace, ce qui garde le résultat en UTF-8 valide sans rien rapprocher.
                out.push(if c.is_ascii() { c as char } else { ' ' });
                i += 1;
            }
        }
    }
    out
}

/// **Le lecteur de références ne lit que du code.** Il décide du cliquet 3 ; s'il se trompe,
/// c'est le cliquet qui ment, donc il se prouve lui aussi.
#[test]
fn test_le_lecteur_de_references_ignore_les_commentaires_et_les_chaines() {
    let texte = r#"
//! Voir [`crate::documente`] pour le détail.
/// Ce lien vers [`crate::commente`] n'est pas un appel.
/* un bloc /* imbriqué */ qui parle de crate::bloc */
fn f() {
    let message = "crate::cite dans un message";
    crate::appele::vraiment();
}
"#;
    let vues = references(texte, "crate");
    assert!(
        vues.contains("appele"),
        "le seul vrai appel doit être vu : {vues:?}"
    );
    for fantome in ["documente", "commente", "bloc", "cite"] {
        assert!(
            !vues.contains(fantome),
            "« {fantome} » n'est pas un appel, mais il a été compté : {vues:?}"
        );
    }
}

fn references(texte: &str, prefixe: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let motif = format!("{prefixe}::");
    let code = code_seul(texte);
    let mut reste = code.as_str();
    while let Some(i) = reste.find(&motif) {
        let apres = &reste[i + motif.len()..];
        if let Some(groupe) = apres.strip_prefix('{') {
            let mut profondeur = 1;
            let mut fin = 0;
            for (j, c) in groupe.char_indices() {
                match c {
                    '{' => profondeur += 1,
                    '}' => {
                        profondeur -= 1;
                        if profondeur == 0 {
                            fin = j;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            for item in groupe[..fin].split(',') {
                let ident: String = item
                    .trim()
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !ident.is_empty() {
                    out.insert(ident);
                }
            }
            reste = &groupe[fin..];
        } else {
            let ident: String = apres
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !ident.is_empty() {
                out.insert(ident);
            }
            reste = apres;
        }
    }
    out
}

/// Le module du noyau auquel appartient un fichier : le premier segment de son chemin sous
/// `src/`, sans son extension. `lib.rs` n'appartient à aucun.
fn module_de(chemin: &Path) -> Option<String> {
    let relatif = chemin.strip_prefix(src_core()).ok()?;
    let premier = relatif.components().next()?.as_os_str().to_str()?;
    let nom = premier.strip_suffix(".rs").unwrap_or(premier);
    (nom != "lib").then(|| nom.to_string())
}

/// Les modules du noyau qu'aucun chemin de production n'atteint.
fn modules_sans_appelant() -> BTreeSet<String> {
    let modules = modules_du_noyau();

    // Les racines : ce que le desktop nomme en production.
    let mut atteints: BTreeSet<String> = sources(&src_desktop())
        .iter()
        .flat_map(|s| references(&s.texte, "glucose_core"))
        .filter(|m| modules.contains(m))
        .collect();

    // Les arêtes internes : ce que chaque module du noyau nomme par `crate::`.
    let mut aretes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for source in sources(&src_core()) {
        let Some(de) = module_de(&source.chemin) else {
            continue;
        };
        let vers: Vec<String> = references(&source.texte, "crate")
            .into_iter()
            .filter(|m| modules.contains(m) && *m != de)
            .collect();
        aretes.entry(de).or_default().extend(vers);
    }

    // La fermeture transitive.
    let mut frontiere: Vec<String> = atteints.iter().cloned().collect();
    while let Some(m) = frontiere.pop() {
        for suivant in aretes.get(&m).into_iter().flatten() {
            if atteints.insert(suivant.clone()) {
                frontiere.push(suivant.clone());
            }
        }
    }

    modules.difference(&atteints).cloned().collect()
}

/// **Aucun module du noyau ne rejoint les modules sans appelant, et chacun en sort un jour.**
///
/// À la hausse : un module vient d'être écrit sans son appelant. C'est R-18, la maladie
/// chronique de ce dépôt (fiche 05 § 7.6 : *aucun module n'est mergé sans son appelant*). Le
/// correctif est de brancher le module, pas de l'ajouter à la liste.
///
/// À la baisse : un module admis vient d'être atteint — le retirer de la liste, pour qu'il ne
/// puisse plus redevenir orphelin sans bruit.
#[test]
fn test_cliquet_3_aucun_module_du_noyau_ne_rejoint_les_sans_appelant() {
    let mesures = modules_sans_appelant();
    let admis: BTreeSet<String> = MODULES_SANS_APPELANT_ADMIS
        .iter()
        .map(|m| m.to_string())
        .collect();

    let nouveaux: Vec<&String> = mesures.difference(&admis).collect();
    assert!(
        nouveaux.is_empty(),
        "module(s) du noyau sans appelant depuis l'application : {nouveaux:?}. Un module \
         arrive avec son appelant (fiche 05 § 7.6) — le brancher, pas l'admettre."
    );

    let branches: Vec<&String> = admis.difference(&mesures).collect();
    assert!(
        branches.is_empty(),
        "module(s) désormais atteint(s) : {branches:?}. Les retirer de \
         MODULES_SANS_APPELANT_ADMIS pour que le terrain gagné ne se reperde pas."
    );
}

// ── Cliquet 4 : les tailles ─────────────────────────────────────────────────

/// Au-delà de cette longueur, une fonction est une dette nommée (fiche 11, A.4 : 80 lignes ;
/// la fiche 05 en demande 60 — le cliquet part de la limite la plus indulgente et descendra).
const FONCTION_MAX: usize = 80;
/// Au-delà de cette taille, un fichier est une dette nommée (fiche 11, A.4 : 600 lignes).
const FICHIER_MAX: usize = 600;

/// Les fonctions de production plus longues que [`FONCTION_MAX`], **admises** telles quelles
/// au commit qui a introduit ce cliquet, avec leur longueur d'alors. Chacune ne peut que
/// raccourcir ; aucune autre ne doit les rejoindre.
///
/// C'est la dette de structure de ce dépôt, nommée fonction par fonction. La plus lourde
/// était `handle_mouse_down`, 471 lignes — l'ancien `window_event` (fiche 01, R-19) qui avait
/// changé d'adresse ; elle est devenue une chaîne de preneurs et n'est plus dans cette liste.
/// Les suivantes sont dans le noyau : les mises en page, les encodages, et le projeteur des
/// membranes.
const FONCTIONS_LONGUES_ADMISES: &[(&str, &str, usize)] = &[
    // (fichier, fonction, longueur admise) — mesurées après le passage sous rustfmt.
    (
        "crates/glucose-core/src/arena/bridge/export.rs",
        "annotation",
        82,
    ),
    (
        "crates/glucose-core/src/arena/bridge/import.rs",
        "fill_annotation",
        111,
    ),
    ("crates/glucose-core/src/arena/doc.rs", "check", 100),
    ("crates/glucose-core/src/export.rs", "build_scene", 169),
    (
        "crates/glucose-core/src/export.rs",
        "scene_to_markdown",
        108,
    ),
    (
        "crates/glucose-core/src/membrane_space.rs",
        "project_board",
        228,
    ),
    (
        "crates/glucose-core/src/membrane_space.rs",
        "resolve_items",
        86,
    ),
    ("crates/glucose-core/src/smart_align.rs", "snap_resize", 94),
    (
        "crates/glucose-desktop/src/renderer/folder.rs",
        "draw_frame",
        94,
    ),
];

/// Les fichiers de production plus longs que [`FICHIER_MAX`], admis tels quels.
const FICHIERS_LONGS_ADMIS: &[(&str, usize)] = &[
    // (fichier, lignes de production admises) — mesurées après le passage sous rustfmt.
    ("crates/glucose-core/src/membrane_space.rs", 661),
];

fn mesure_fonctions_longues() -> BTreeMap<(String, String), usize> {
    let mut out = BTreeMap::new();
    for source in sources(&src_core())
        .into_iter()
        .chain(sources(&src_desktop()))
    {
        for (nom, longueur) in fonctions(&source.texte) {
            if longueur > FONCTION_MAX {
                out.insert((court(&source.chemin), nom), longueur);
            }
        }
    }
    out
}

fn mesure_fichiers_longs() -> BTreeMap<String, usize> {
    sources(&src_core())
        .into_iter()
        .chain(sources(&src_desktop()))
        .map(|s| (court(&s.chemin), s.texte.lines().count()))
        .filter(|(_, n)| *n > FICHIER_MAX)
        .collect()
}

/// **Aucune fonction ne dépasse 80 lignes, hormis celles nommées ici — et celles-ci ne
/// grandissent pas.**
///
/// À la hausse : soit une fonction neuve dépasse la limite, soit une fonction admise a
/// grossi. Dans les deux cas, la réponse est la règle 1.7 de la fiche 05 — extraire, traduire
/// l'événement en intention, faire une fonction courte de plus — jamais d'allonger la liste.
///
/// À la baisse : une fonction admise est repassée sous la limite, ou a nettement raccourci.
/// Mettre la liste à jour, pour que le terrain gagné ne se reperde pas.
#[test]
fn test_cliquet_4a_aucune_fonction_ne_depasse_sa_longueur_admise() {
    let mesures = mesure_fonctions_longues();
    let admises: BTreeMap<(String, String), usize> = FONCTIONS_LONGUES_ADMISES
        .iter()
        .map(|(f, n, l)| ((f.to_string(), n.to_string()), *l))
        .collect();

    let mut fautes = Vec::new();
    for ((fichier, nom), longueur) in &mesures {
        match admises.get(&(fichier.clone(), nom.clone())) {
            None => fautes.push(format!(
                "{fichier}::{nom} fait {longueur} lignes (> {FONCTION_MAX}) et n'est pas admise"
            )),
            Some(admise) if longueur > admise => fautes.push(format!(
                "{fichier}::{nom} a grossi : {longueur} lignes contre {admise} admises"
            )),
            _ => {}
        }
    }
    assert!(
        fautes.is_empty(),
        "la structure a dérivé :\n  {}\nExtraire (fiche 05 § 1.1, § 1.7), ne pas admettre.",
        fautes.join("\n  ")
    );

    let mut a_mettre_a_jour = Vec::new();
    for ((fichier, nom), admise) in &admises {
        match mesures.get(&(fichier.clone(), nom.clone())) {
            None => a_mettre_a_jour.push(format!(
                "{fichier}::{nom} est repassée sous {FONCTION_MAX} lignes : la retirer de la \
                 liste"
            )),
            Some(longueur) if *longueur + 8 <= *admise => a_mettre_a_jour.push(format!(
                "{fichier}::{nom} a raccourci : {longueur} lignes contre {admise} admises — \
                 abaisser"
            )),
            _ => {}
        }
    }
    assert!(
        a_mettre_a_jour.is_empty(),
        "du terrain gagné à consolider :\n  {}",
        a_mettre_a_jour.join("\n  ")
    );
}

/// **Aucun fichier ne dépasse 600 lignes, hormis ceux nommés ici — et ceux-ci ne grandissent
/// pas.**
#[test]
fn test_cliquet_4b_aucun_fichier_ne_depasse_sa_taille_admise() {
    let mesures = mesure_fichiers_longs();
    let admis: BTreeMap<String, usize> = FICHIERS_LONGS_ADMIS
        .iter()
        .map(|(f, l)| (f.to_string(), *l))
        .collect();

    let mut fautes = Vec::new();
    for (fichier, lignes) in &mesures {
        match admis.get(fichier) {
            None => fautes.push(format!(
                "{fichier} fait {lignes} lignes (> {FICHIER_MAX}) et n'est pas admis"
            )),
            Some(a) if lignes > a => fautes.push(format!(
                "{fichier} a grossi : {lignes} lignes contre {a} admises"
            )),
            _ => {}
        }
    }
    assert!(
        fautes.is_empty(),
        "un fichier a dérivé :\n  {}\nÉclater (fiche 05 § 1.2), ne pas admettre.",
        fautes.join("\n  ")
    );

    let mut a_mettre_a_jour = Vec::new();
    for (fichier, a) in &admis {
        match mesures.get(fichier) {
            None => a_mettre_a_jour.push(format!(
                "{fichier} est repassé sous {FICHIER_MAX} lignes : le retirer de la liste"
            )),
            Some(lignes) if *lignes + 30 <= *a => a_mettre_a_jour.push(format!(
                "{fichier} a fondu : {lignes} lignes contre {a} admises — abaisser"
            )),
            _ => {}
        }
    }
    assert!(
        a_mettre_a_jour.is_empty(),
        "du terrain gagné à consolider :\n  {}",
        a_mettre_a_jour.join("\n  ")
    );
}

// ── Cliquet 5 : les rectangles remplis sans passer par la grille ────────────

/// Les appels bruts à `fill_rect` du code de production, relevés au commit qui a
/// introduit ce cliquet — hors `scale.rs`, qui est justement là où ils ont leur place.
///
/// **tiny-skia panique** sur un `fill_rect` anti-aliasé dont un côté tombe sous le pixel :
/// `hairline_aa.rs` fait `assert!(false)`. Le crash est apparu **trois fois** dans ce
/// projet — sur un filet de séparation, sur la barre d'une citation, puis sur la barre de
/// saisie d'une étiquette de flèche — et les deux premières fois il a été corrigé *sur
/// place*. C'est exactement pour cela qu'il est revenu.
///
/// [`scale::fill_crisp`] est la réponse, et elle est meilleure que le contournement : un
/// rectangle aligné sur les axes n'a aucun bord oblique, donc l'anti-aliasing ne lui apporte
/// rien et lui coûte tout — un filet posé sur une demi-position devient deux demi-traits
/// gris, flou là où la charte demande « net à quasi 100 % » (R-46).
///
/// Ce cliquet ne réécrit pas les appels existants d'un coup : il **arrête l'hémorragie**.
/// Chaque nouveau rectangle rempli doit se demander s'il peut être fin, et la réponse est
/// presque toujours `fill_crisp`.
///
/// Dix jusqu'au 23/09 au soir ; neuf depuis que les poignées passent par la grille (ORNEMENTS-2).
const PLAFOND_FILL_RECT: usize = 9;

fn compte_fill_rect() -> usize {
    sources(&src_desktop())
        .iter()
        .filter(|s| !s.chemin.ends_with("scale.rs"))
        .flat_map(|s| s.texte.lines())
        .map(nue)
        .filter(|ligne| ligne.contains("fill_rect("))
        .count()
}

/// **Le nombre de rectangles remplis sans passer par la grille ne grandit pas.**
///
/// Un échec à la hausse veut dire qu'un `fill_rect` de plus a été écrit. Avant de relever
/// quoi que ce soit : ce rectangle peut-il être fin ? S'il le peut, il fait planter
/// l'application, et `fill_crisp` est sa place.
#[test]
fn test_cliquet_5_aucun_rectangle_ne_contourne_la_grille_de_pixels() {
    let compte = compte_fill_rect();
    assert!(
        compte <= PLAFOND_FILL_RECT,
        "un `fill_rect` de plus : {compte} contre {PLAFOND_FILL_RECT} autorisés. tiny-skia \
         panique sur un rectangle plus fin qu'un pixel — utiliser `scale::fill_crisp`, qui \
         l'aligne sur la grille et rend un dessin plus net (SCALE-3, R-46)."
    );
    assert!(
        compte >= PLAFOND_FILL_RECT.saturating_sub(4),
        "les appels bruts sont descendus à {compte} : abaisser PLAFOND_FILL_RECT à cette \
         valeur pour que le terrain gagné ne se reperde pas"
    );
}

// ── Le compteur de fonctions se vérifie lui-même ────────────────────────────

#[test]
fn test_le_compteur_de_fonctions_lit_une_fonction_a_travers_ses_chaines_et_commentaires() {
    let texte = "\
pub fn a() {
    let s = \"{\"; // }
    let c = '{';
    if s.is_empty() {
    }
}

fn b(x: i32) -> i32 { x }

trait T {
    fn sans_corps(&self);
}
";
    let f = fonctions(texte);
    assert_eq!(f, vec![("a".to_string(), 6), ("b".to_string(), 1)]);
}

// ── Cliquet 9 : un champ de la chronique qui n'est jamais rempli ─────────────

/// **Tout champ de `Instantane` doit être rempli quelque part.**
///
/// # Le défaut que ce cliquet interdit, et il a coûté une session entière
///
/// Quatre champs ont été ajoutés à l'instantané — la part servie par vignette, les périmées,
/// les prêtes, les orphelines — sans que la ligne qui les remplit soit écrite. Ils valaient
/// donc zéro dans chaque trace, **et un zéro se lit comme une mesure**. Tout un raisonnement
/// s'est bâti dessus : « aucune photo ne passe par une vignette » ne mesurait rien d'autre que
/// l'absence de ces lignes.
///
/// Un compteur qui ment est pire que pas de compteur du tout : le second se remarque.
#[test]
fn test_cliquet_9_aucun_champ_de_la_chronique_ne_reste_vide() {
    // `Instantane` a quitte `chronique.rs` le jour ou celui-ci a depasse sa taille admise :
    // une chronique AGREGE, un instantane DECRIT, et les deux ne changent pas pour les memes
    // raisons. Ce cliquet suit la structure, pas le fichier.
    let chronique = std::fs::read_to_string("src/chronique/instantane.rs").expect("instantane.rs");
    let terrain = std::fs::read_to_string("src/app/terrain.rs").expect("terrain.rs");

    // Les champs publics de `Instantane`, dans l'ordre où ils sont déclarés.
    let debut = chronique
        .find("pub struct Instantane {")
        .expect("la structure Instantane");
    let corps = &chronique[debut..];
    let fin = corps.find("\n}").expect("la fin de la structure");
    let champs: Vec<&str> = corps[..fin]
        .lines()
        // La premiere ligne est la declaration elle-meme, qui commence aussi par « pub ».
        .skip(1)
        .filter_map(|l| l.trim().strip_prefix("pub "))
        .filter_map(|l| l.split(':').next())
        .collect();
    assert!(champs.len() > 5, "la structure n'a pas été lue");

    // `instant_ms` est posé par `Chronique::enregistrer`, qui seule connaît le début de la
    // session : c'est la seule exception, et elle se vérifie ici plutôt que de se supposer.
    // Elle se lit dans l'agregat, non dans la structure — les deux vivent desormais dans deux
    // fichiers, et confondre les deux ferait passer ce cliquet pour un autre.
    let agregat = std::fs::read_to_string("src/chronique.rs").expect("chronique.rs");
    assert!(
        agregat.contains("vu.instant_ms = "),
        "instant_ms n'est plus posé par `enregistrer`"
    );

    let oublies: Vec<&str> = champs
        .iter()
        .filter(|c| c.trim() != "instant_ms" && !terrain.contains(c.trim()))
        .copied()
        .collect();
    assert!(
        oublies.is_empty(),
        "ces champs de la chronique ne sont jamais remplis, donc ils valent zéro dans chaque \
         trace : {oublies:?}\nUn compteur qui ment est pire que pas de compteur."
    );
}

// ── Cliquet 10 : les marques de mesure que la chronique ne saurait pas garder ─────────────

/// Les noms distincts passés à `perf::stage` dans le code de production.
///
/// Lus sur le texte brut, commentaires compris : une marque citée dans un commentaire est
/// comptée en trop, ce qui est le sens prudent pour une borne à ne pas dépasser.
fn marques_de_mesure() -> BTreeSet<String> {
    let mut noms = BTreeSet::new();
    for source in sources(&src_desktop()) {
        let mut depuis = source.texte.as_str();
        while let Some(i) = depuis.find("perf::stage(\"") {
            let apres = &depuis[i + "perf::stage(\"".len()..];
            if let Some(fin) = apres.find('"') {
                noms.insert(apres[..fin].to_string());
            }
            depuis = apres;
        }
    }
    noms
}

/// La borne que la chronique s'est donnée, lue dans son code.
fn borne_des_postes() -> usize {
    let texte = fs::read_to_string(src_desktop().join("chronique.rs")).expect("chronique.rs");
    let ligne = texte
        .lines()
        .find(|l| l.trim_start().starts_with("pub const POSTES: usize ="))
        .expect("la constante POSTES");
    ligne
        .split('=')
        .nth(1)
        .and_then(|v| v.trim().trim_end_matches(';').parse().ok())
        .expect("une valeur entière")
}

/// **Aucune marque de mesure n'est perdue en silence.**
///
/// La chronique garde `POSTES` noms de postes par image ; au-delà, `poste()` rend `None` et
/// la marque est ignorée -- elle vaut alors zéro dans chaque trace, et un zéro se lit comme
/// une mesure (cliquet 9). Le rendu déclarait trente marques quand la borne en admettait
/// vingt-quatre, et les six dernières à se présenter n'apparaissaient nulle part.
///
/// À la hausse : une marque de plus a été écrite. Relever `POSTES` est la réponse juste, et
/// ce test dit de combien.
#[test]
fn test_cliquet_10_aucune_marque_de_mesure_n_est_perdue() {
    let marques = marques_de_mesure();
    let borne = borne_des_postes();
    assert!(
        marques.len() <= borne,
        "{} marques de mesure distinctes pour une chronique qui n'en garde que {borne} : les \
         dernières à se déclarer seraient ignorées en silence. Relever POSTES dans \
         chronique.rs. Marques : {marques:?}",
        marques.len()
    );
}
