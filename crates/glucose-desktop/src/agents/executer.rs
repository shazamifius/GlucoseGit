//! **Ce que fait chaque outil**, dans la boucle de Glucose — le seul fil qui touche au document.
//!
//! Chaque outil lit ses arguments, mesure ce que le noyau ne sait pas mesurer (la hauteur d'un
//! texte, les dimensions d'une capture), puis confie l'écriture au noyau en **un** geste
//! ([`glucose_core::store::Store::geste_d_agent`] : AGENT-1, AGENT-2). Ce qu'il rend est du
//! JSON que le modèle lit ; ce qui l'empêche, une phrase qu'il peut corriger.

mod chercher;
mod disposer;
mod domaines;
mod grouper;
mod partir;

use super::json::Json;
use super::lecture::{choisir, empreinte, premiere_phrase, Demande, Niveau, Rendu};
use crate::app::GlucoseApp;
use crate::renderer::card::text_card_fit_height;
use glucose_core::geometry::Rect;
use glucose_core::store::agents::{Decrit, Lien};
use glucose_core::types::{Annotation, ArrowPredicate, BoardImage, DEFAULT_TEXT_CARD_WIDTH};

/// Les arguments d'un appel, lus par leur nom.
struct Args<'a>(&'a Json);

impl<'a> Args<'a> {
    fn texte(&self, cle: &str) -> Option<&'a str> {
        self.0.get(cle).and_then(Json::as_str)
    }

    fn requis(&self, cle: &str) -> Result<&'a str, String> {
        self.texte(cle)
            .ok_or_else(|| format!("l'argument « {cle} » manque, ou n'est pas un texte"))
    }

    fn nombre(&self, cle: &str) -> Option<f64> {
        self.0
            .get(cle)
            .and_then(Json::as_f64)
            .filter(|n| n.is_finite())
    }

    fn oui(&self, cle: &str) -> bool {
        self.0.get(cle).and_then(Json::as_bool).unwrap_or(false)
    }

    /// Une couleur `#rrggbb`, ou rien ; autre chose est refusé plutôt que deviné.
    fn couleur(&self) -> Result<Option<String>, String> {
        match self.texte("couleur") {
            None => Ok(None),
            Some(c)
                if c.len() == 7
                    && c.starts_with('#')
                    && c[1..].chars().all(|h| h.is_ascii_hexdigit()) =>
            {
                Ok(Some(c.to_string()))
            }
            Some(c) => Err(format!("« {c} » n'est pas une couleur #rrggbb")),
        }
    }
}

impl GlucoseApp {
    /// **Exécute un outil** pour un agent, et rend ce qu'il dit à l'agent.
    pub(crate) fn executer_pour_un_agent(
        &mut self,
        nom: &str,
        arguments: &Json,
    ) -> Result<Json, String> {
        let a = Args(arguments);
        let board = a.texte("tableau").map_or_else(
            || self.store.project.active_board_id.clone(),
            str::to_string,
        );
        let issue = match nom {
            "glucose_lire" => self.agent_lire(&board, &a),
            "glucose_poser_texte" => self.agent_poser_texte(&board, &a),
            "glucose_poser_membrane" => self.agent_poser_membrane(&board, &a),
            "glucose_ecrire" => self.agent_ecrire(&board, &a),
            "glucose_relier" => self.agent_relier(&board, &a),
            "glucose_poser_page" => self.agent_poser_page(&board, &a),
            "glucose_deplacer" => self.agent_deplacer(&board, &a),
            "glucose_retirer" => self.agent_retirer(&board, &a),
            "glucose_ouvrir_tableau" => self.agent_ouvrir_tableau(&a),
            "glucose_poser_domaine" => self.agent_poser_domaine(&board, &a),
            "glucose_domaines" => self.agent_lire_les_domaines(&a),
            "glucose_chercher" => self.agent_chercher(&a),
            "glucose_disposer" => self.agent_disposer(&board, &a),
            "glucose_grouper" => self.agent_grouper(&board, &a),
            _ => Err(format!("Glucose n'a pas d'outil « {nom} »")),
        };
        // Ce qu'un agent pose apparaît à l'image suivante, sans attendre un geste de la main.
        self.mark_dirty();
        issue
    }

    /// **Lire par niveaux** (`lecture`) : le sommaire par défaut, le texte entier des nœuds
    /// choisis, ou la phrase qui contient ce qu'on cherche.
    fn agent_lire(&self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let decrits = self
            .store
            .decrire_pour_un_agent(board)
            .map_err(|r| r.to_string())?;
        let ids = match a.0.get("ids") {
            None => None,
            Some(l) => Some(
                l.as_liste()
                    .ok_or("l'argument « ids » est une liste d'identifiants")?
                    .iter()
                    .filter_map(Json::as_str)
                    .collect(),
            ),
        };
        let domaine = self.domaine_a_lire(a)?;
        let demande = Demande {
            domaine: domaine.as_deref(),
            dans: a.texte("dans"),
            ids,
            cherche: a.texte("cherche"),
            niveau: Niveau::lire(a.texte("niveau"))?,
        };
        let plan = demande.niveau == Niveau::Plan;
        let avis = self.renderer.sens_des_textes.avis_du_tableau(board);
        let noeuds = choisir(&decrits, &demande)
            .into_iter()
            .map(|(d, rendu)| {
                let mut j = decrit_en_json(d, &rendu);
                parente_en_json(avis.and_then(|a| a.get(&d.id)), &mut j);
                // Le plan dit ce que chaque membrane porte, sans le porter (fiche 90).
                if let (true, Json::Objet(champs)) = (plan, &mut j) {
                    let n = decrits
                        .iter()
                        .filter(|x| x.membrane.as_deref() == Some(d.id.as_str()))
                        .count();
                    champs.push(("contient".to_string(), Json::Nombre(n as f64)));
                }
                j
            })
            .collect();
        let tableaux = self
            .store
            .onglets()
            .map(|(id, nom, actif)| {
                Json::objet([
                    ("id", Json::from(id)),
                    ("nom", Json::from(nom)),
                    ("regarde", Json::Bool(actif)),
                ])
            })
            .collect();
        Ok(Json::objet([
            ("document", Json::from(self.store.identite.as_str())),
            (
                "fichier",
                Json::from(self.fichier_de_ce_document().as_str()),
            ),
            ("tableau", Json::from(board)),
            ("visible", boite_en_json(self.ce_qu_il_voit())),
            ("selection", self.ce_qu_il_designe(board)),
            ("noeuds", Json::Liste(noeuds)),
            ("tableaux", Json::Liste(tableaux)),
            ("domaines", self.catalogue_pour_un_agent()),
        ]))
    }

    /// **Ce que l'utilisateur désigne** : les nœuds qu'il a sélectionnés sur le tableau qu'il
    /// regarde — quand il écrit « ça » à l'agent, c'est cela. Rien sur un autre tableau : sa
    /// sélection n'y est pas. L'agent la lit, il n'y touche jamais (AGENT-2).
    fn ce_qu_il_designe(&self, board: &str) -> Json {
        if board != self.store.project.active_board_id {
            return Json::Liste(Vec::new());
        }
        let s = &self.store;
        Json::Liste(
            s.selected_image_ids
                .iter()
                .chain(&s.selected_annotation_ids)
                .map(|id| Json::from(id.as_str()))
                .collect(),
        )
    }

    /// La partie du canevas que l'écran montre, en unités du monde.
    fn ce_qu_il_voit(&self) -> Rect {
        let vp = self.store.viewport();
        let (l, t) = crate::canvas::screen_to_world(0.0, 0.0, &vp);
        let (w, h) = self.taille_de_la_fenetre();
        let (w, h) = (f64::from(w), f64::from(h));
        let (r, b) = crate::canvas::screen_to_world(w, h, &vp);
        Rect::new(l, t, r - l, b - t)
    }

    /// Un identifiant donné par l'agent, ou un neuf.
    fn id_d_agent(&mut self, a: &Args<'_>, prefixe: &str) -> String {
        a.texte("id")
            .map_or_else(|| self.store.generate_id(prefixe), str::to_string)
    }

    /// La boîte d'un nœud qui existe déjà — pour qu'une mise à jour sans position le laisse où
    /// il est.
    fn boite_existante(&self, board: &str, id: &str) -> Option<Rect> {
        self.store
            .project
            .annotation(board, id)
            .map(Annotation::bounds)
    }

    fn agent_poser_texte(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let texte = a.requis("texte")?;
        let couleur = a.couleur()?;
        let voulus = self.domaines_voulus(a)?;
        let id = self.id_d_agent(a, "text");
        let avant = self.boite_existante(board, &id);
        let largeur = a
            .nombre("largeur")
            .or(avant.map(|b| b.width))
            .unwrap_or(DEFAULT_TEXT_CARD_WIDTH);
        let operateur = partir::operateur_dit(a.texte("operateur"))?;
        // Un opérateur est un post-it : le dire suffit.
        let post_it = a.oui("post_it") || operateur.is_some_and(|o| o.is_some());
        let hauteur = text_card_fit_height(
            &self.renderer.typography,
            &self.renderer.math,
            texte,
            largeur,
        );
        let (dans, voulu) = (a.texte("dans"), position(a, avant));
        let boite = self
            .store
            .geste_d_agent(|s| {
                let (x, y) = match voulu {
                    Some(p) => p,
                    None => s.place_pour_un_agent(board, dans)?,
                };
                let boite = Rect::new(x, y, largeur, hauteur);
                if let Some(m) = dans {
                    s.agrandir_la_membrane_pour(board, m, boite)?;
                }
                let neuf = if post_it {
                    Annotation::sticky(&id, x, y, texte)
                } else {
                    Annotation::text(&id, x, y, texte)
                };
                let ann = partir::un_texte(s, board, neuf, boite);
                s.poser_pour_un_agent(board, ann, dans)?;
                // La couleur par la loi du noyau, celle de la barre de l'humain (COULEUR-2).
                if let Some(c) = couleur.as_deref() {
                    s.colorer(board, std::slice::from_ref(&id), Some(c));
                }
                domaines::poser_les_domaines(s, board, &[&id], &voulus)?;
                partir::l_operateur(s, board, &id, operateur)
            })
            .map_err(|r| r.to_string())?;
        Ok(pose(&id, boite))
    }

    fn agent_poser_membrane(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let couleur = a.couleur()?;
        let voulus = self.domaines_voulus(a)?;
        let id = self.id_d_agent(a, "membrane");
        let avant = self.boite_existante(board, &id);
        let (defaut_l, defaut_h) = crate::interactions::tools::NEW_CONTAINER_SIZE;
        let largeur = a
            .nombre("largeur")
            .or(avant.map(|b| b.width))
            .unwrap_or(defaut_l);
        let hauteur = a
            .nombre("hauteur")
            .or(avant.map(|b| b.height))
            .unwrap_or(defaut_h);
        let titre = a.texte("titre").map(str::to_string);
        let (dans, voulu) = (a.texte("dans"), position(a, avant));
        let boite = self
            .store
            .geste_d_agent(|s| {
                let (x, y) = match voulu {
                    Some(p) => p,
                    None => s.place_pour_un_agent(board, dans)?,
                };
                let boite = Rect::new(x, y, largeur, hauteur);
                if let Some(m) = dans {
                    s.agrandir_la_membrane_pour(board, m, boite)?;
                }
                let dit = (titre.as_deref(), couleur.as_deref());
                let ann = partir::une_membrane(s, board, &id, boite, dit);
                s.poser_pour_un_agent(board, ann, dans)?;
                domaines::poser_les_domaines(s, board, &[&id], &voulus)?;
                Ok(boite)
            })
            .map_err(|r| r.to_string())?;
        Ok(pose(&id, boite))
    }

    fn agent_ecrire(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let (id, texte) = (a.requis("id")?, a.requis("texte")?);
        let ajouter = a.oui("ajouter");
        let (typo, math) = (&self.renderer.typography, &self.renderer.math);
        self.store
            .geste_d_agent(|s| {
                s.ecrire_pour_un_agent(board, id, texte, ajouter)?;
                // Une carte de texte se tient à la hauteur de son contenu (TEXT-FIT-1).
                let tenue = s.project.annotation(board, id).and_then(|n| match n {
                    Annotation::Text { text, .. } => Some((text.clone(), n.bounds().width)),
                    _ => None,
                });
                if let Some((texte, largeur)) = tenue {
                    let hauteur = text_card_fit_height(typo, math, &texte, largeur);
                    s.tenir_la_hauteur_pour_un_agent(board, id, hauteur)?;
                }
                Ok(())
            })
            .map_err(|r| r.to_string())?;
        let boite = self.boite_existante(board, id).unwrap_or(Rect::ZERO);
        Ok(pose(id, boite))
    }

    fn agent_relier(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let relation = match a.texte("relation") {
            None => None,
            Some(r) => Some(
                ArrowPredicate::ALL
                    .into_iter()
                    .find(|p| p.as_str() == r)
                    .ok_or_else(|| format!("« {r} » n'est pas une relation de Glucose"))?,
            ),
        };
        let id = self.id_d_agent(a, "arrow");
        let lien = Lien {
            id: &id,
            source: a.requis("source")?,
            cible: a.requis("cible")?,
            passage_source: a.texte("passage_source"),
            passage_cible: a.texte("passage_cible"),
            relation,
            texte: a.texte("texte"),
        };
        self.store
            .geste_d_agent(|s| s.relier_pour_un_agent(board, lien))
            .map_err(|r| r.to_string())?;
        Ok(Json::objet([("id", Json::from(id))]))
    }

    fn agent_deplacer(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let id = a.requis("id")?;
        let (Some(x), Some(y)) = (a.nombre("x"), a.nombre("y")) else {
            return Err("les arguments « x » et « y » sont des nombres".to_string());
        };
        self.store
            .geste_d_agent(|s| s.deplacer_pour_un_agent(board, id, (x, y)))
            .map_err(|r| r.to_string())?;
        Ok(Json::objet([("id", Json::from(id))]))
    }

    fn agent_retirer(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let ids: Vec<&str> =
            a.0.get("ids")
                .and_then(Json::as_liste)
                .ok_or("l'argument « ids » est une liste d'identifiants")?
                .iter()
                .filter_map(Json::as_str)
                .collect();
        let n = self
            .store
            .geste_d_agent(|s| s.retirer_pour_un_agent(board, &ids))
            .map_err(|r| r.to_string())?;
        Ok(Json::objet([("retires", Json::Nombre(n as f64))]))
    }

    /// Un onglet à l'agent, par son nom : le même nom, le même onglet.
    fn agent_ouvrir_tableau(&mut self, a: &Args<'_>) -> Result<Json, String> {
        let nom = a.requis("nom")?.trim();
        if nom.is_empty() {
            return Err("un onglet a un nom".into());
        }
        let (id, cree) = self
            .store
            .ouvrir_un_tableau_pour_un_agent(nom)
            .map_err(|r| r.to_string())?;
        Ok(Json::objet([
            ("tableau", Json::from(id.as_str())),
            ("nom", Json::from(nom)),
            ("cree", Json::Bool(cree)),
        ]))
    }

    /// **Une page web** : sa capture, qui garde son adresse — le double-clic l'ouvre
    /// (`interactions::pick`) —, et sous elle un texte qui dit son titre et son adresse, où
    /// l'agent écrira ce qu'il y a lu.
    fn agent_poser_page(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let adresse = a.requis("adresse")?;
        if !crate::interactions::links::adresse_web(adresse) {
            return Err(format!(
                "« {adresse} » n'est pas une adresse http:// ou https://"
            ));
        }
        let capture = match a.texte("capture") {
            Some(chemin) => Some(self.lire_une_capture(chemin)?),
            None => None,
        };
        let voulus = self.domaines_voulus(a)?;
        let id = self.id_d_agent(a, "page");
        let legende = format!("{id}-texte");
        let largeur = a.nombre("largeur").unwrap_or(DEFAULT_TEXT_CARD_WIDTH);
        let texte = legende_d_une_page(a.texte("titre"), adresse, a.texte("texte"));
        let hauteur_texte = text_card_fit_height(
            &self.renderer.typography,
            &self.renderer.math,
            &texte,
            largeur,
        );
        let hauteur_photo = capture
            .as_ref()
            .map_or(0.0, |(_, (w, h))| largeur * h / w.max(1.0));
        let (dans, voulu) = (
            a.texte("dans"),
            position(a, self.boite_d_une_page(board, &id)),
        );
        let boite = self
            .store
            .geste_d_agent(|s| {
                let (x, y) = match voulu {
                    Some(p) => p,
                    None => s.place_pour_un_agent(board, dans)?,
                };
                let ecart = if capture.is_some() {
                    glucose_core::store::zones::MARGE
                } else {
                    0.0
                };
                let boite = Rect::new(x, y, largeur, hauteur_photo + ecart + hauteur_texte);
                if let Some(m) = dans {
                    s.agrandir_la_membrane_pour(board, m, boite)?;
                }
                if let Some((cle, (w, h))) = &capture {
                    let mut img = BoardImage::new(
                        &id,
                        x + largeur / 2.0,
                        y + hauteur_photo / 2.0,
                        largeur,
                        hauteur_photo,
                    );
                    img.src = Some(cle.clone());
                    (img.original_width, img.original_height) = (*w, *h);
                    img.source_url = Some(adresse.to_string());
                    s.poser_une_photo_pour_un_agent(board, img, dans)?;
                }
                let mut ann = Annotation::text(&legende, x, y + hauteur_photo + ecart, texte);
                if let Annotation::Text { width, height, .. } = &mut ann {
                    (*width, *height) = (Some(largeur), Some(hauteur_texte));
                }
                s.poser_pour_un_agent(board, ann, dans)?;
                let photo = capture.is_some().then_some(id.as_str());
                let noeuds: Vec<&str> = photo.into_iter().chain([legende.as_str()]).collect();
                domaines::poser_les_domaines(s, board, &noeuds, &voulus)?;
                Ok(boite)
            })
            .map_err(|r| r.to_string())?;
        let photo = capture.is_some().then(|| id.clone());
        Ok(Json::objet([
            ("photo", Json::from(photo)),
            ("texte", Json::from(legende)),
            ("boite", boite_en_json(boite)),
        ]))
    }

    /// La boîte d'une page déjà posée — sa photo, ou son texte seul.
    fn boite_d_une_page(&self, board: &str, id: &str) -> Option<Rect> {
        self.store
            .image(board, id)
            .map(BoardImage::rect)
            .or_else(|| self.boite_existante(board, &format!("{id}-texte")))
    }

    /// **Les octets d'une capture, confiés au document** : ils s'y scellent comme ceux d'une
    /// image qu'une page livre (DEPOT-4) — le fichier de l'agent peut disparaître ensuite.
    fn lire_une_capture(&mut self, chemin: &str) -> Result<(String, (f64, f64)), String> {
        let octets = std::fs::read(chemin)
            .map_err(|e| format!("la capture « {chemin} » ne se lit pas : {e}"))?;
        let recu = crate::plateforme::moisson::Recu::nouveau("capture.png", octets)
            .ok_or_else(|| format!("la capture « {chemin} » est vide"))?;
        self.sceller_une_image(recu, 0)
            .map_err(|_| format!("« {chemin} » n'est pas une image que Glucose sait lire"))
    }
}

/// La position demandée — ou celle qu'a déjà le nœud, s'il existe et qu'on n'en donne pas.
fn position(a: &Args<'_>, avant: Option<Rect>) -> Option<(f64, f64)> {
    match (a.nombre("x"), a.nombre("y")) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => avant.map(|b| (b.left, b.top)),
    }
}

/// Ce qu'une page dit sous sa capture : son titre en gras, son adresse en lien (`Ctrl`+clic
/// l'ouvre), puis ce qu'on y a lu.
fn legende_d_une_page(titre: Option<&str>, adresse: &str, texte: Option<&str>) -> String {
    let domaine = adresse
        .split("://")
        .nth(1)
        .and_then(|reste| reste.split('/').next())
        .unwrap_or(adresse);
    let mut s = String::new();
    if let Some(t) = titre.filter(|t| !t.trim().is_empty()) {
        s.push_str(&format!("**{}**\n", t.trim()));
    }
    s.push_str(&format!("[{domaine}]({adresse})"));
    if let Some(t) = texte.filter(|t| !t.trim().is_empty()) {
        s.push_str("\n\n");
        s.push_str(t);
    }
    s
}

fn pose(id: &str, boite: Rect) -> Json {
    Json::objet([("id", Json::from(id)), ("boite", boite_en_json(boite))])
}

fn boite_en_json(b: Rect) -> Json {
    Json::objet([
        ("x", coordonnee(b.left)),
        ("y", coordonnee(b.top)),
        ("largeur", coordonnee(b.width)),
        ("hauteur", coordonnee(b.height)),
    ])
}

/// **COORD-1 — une coordonnée, au plus court sans rien perdre** (fiche 90) : le document place tout à
/// 1/2⁸ d'unité près (`glucose_core::fixed`) ; avec d décimales, 10^d ≥ 2⁸, la valeur qu'un agent
/// relit et renvoie retombe sur le même point. d = 3, déduit et non choisi. Une hauteur mesurée en
/// `f32` s'écrivait « 51.599998474121094 » : des milliers de caractères perdus sur un grand tableau.
fn coordonnee(v: f64) -> Json {
    let decimales = (f64::from(glucose_core::fixed::FRAC_BITS) * 2f64.log10()).ceil();
    let pas = 10f64.powf(decimales);
    Json::Nombre((v * pas).round() / pas)
}

/// **PARENTE-3 — ce que l'IA de Glucose dit d'un texte**, comme l'humain le voit à son halo (fiche
/// 106 ; VISION § 1 : *« ce qu'un humain voit, une IA doit pouvoir le lire »*) : le numéro de sa
/// famille — des textes de la même famille se ressemblent sûrement, leur halo a la même couleur —,
/// rien s'il est seul ; et la sûreté qu'il a un parent, de 0 à 1, au centième.
fn parente_en_json(avis: Option<&crate::parente::AvisDUnTexte>, noeud: &mut Json) {
    let (Some(a), Json::Objet(champs)) = (avis, noeud) else {
        return;
    };
    let famille = a.famille.map_or(Json::Nul, |f| Json::Nombre(f as f64));
    let surete = Json::Nombre((f64::from(a.surete) * 100.0).round() / 100.0);
    let parente = Json::objet([("famille", famille), ("surete", surete)]);
    champs.push(("parente".to_string(), parente));
}

/// Un nœud pour l'agent : son texte entier, ou sa première phrase et sa taille (le sommaire), ou
/// le passage trouvé.
fn decrit_en_json(d: &Decrit, rendu: &Rendu) -> Json {
    let mut champs = vec![
        ("id", Json::from(d.id.as_str())),
        ("sorte", Json::from(d.sorte.nom())),
        ("boite", boite_en_json(d.boite)),
    ];
    if let Some(t) = d.texte.as_deref() {
        champs.push(("empreinte", Json::from(empreinte(t).as_str())));
    }
    if let (Rendu::Sommaire | Rendu::Passage(_), Some(t)) = (rendu, d.texte.as_deref()) {
        champs.push(("taille", Json::Nombre(t.chars().count() as f64)));
        if let Some(p) = premiere_phrase(t) {
            champs.push(("debut", Json::from(p)));
        }
    }
    if let Rendu::Passage(p) = rendu {
        champs.push(("passage", Json::from(p.as_str())));
    }
    let entier = (*rendu == Rendu::Entier)
        .then_some(d.texte.as_deref())
        .flatten();
    let facultatifs = [
        ("texte", entier),
        ("membrane", d.membrane.as_deref()),
        ("source", d.source.as_deref()),
        ("cible", d.cible.as_deref()),
        ("relation", d.relation.map(|r| r.as_str())),
        ("operateur", d.operateur.map(|o| o.id())),
        ("couleur", d.couleur.as_deref()),
        ("adresse", d.adresse.as_deref()),
    ];
    champs.extend(
        facultatifs
            .into_iter()
            .filter_map(|(cle, v)| v.map(|v| (cle, Json::from(v)))),
    );
    if !d.domaines.is_empty() {
        champs.push(("domaines", domaines::domaines_en_json(&d.domaines)));
    }
    Json::objet(champs)
}

#[cfg(test)]
mod tests;
