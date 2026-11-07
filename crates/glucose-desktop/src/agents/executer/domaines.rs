//! **Les domaines, à la porte des agents** (fiche 88) : ce que `glucose_lire` en dit, l'outil qui les
//! pose, et celui qui les lit à travers tous les fichiers. Le noyau décide
//! (`store/agents/domaines.rs`, `recensement`) ; ici, on lit les arguments et on écrit le JSON.
//!
//! Sa règle du 7/11/2026 vit ici pour un agent : deux domaines ne portent jamais le même nom, et
//! Glucose **demande** au lieu de deviner. La question arrive dans la réponse à l'appel, avec un
//! exemple de ce que le nom désigne déjà, pour que l'agent sache de quel sujet on parle.

use super::{decrit_en_json, Args};
use crate::agents::json::Json;
use crate::agents::lecture::{passage, premiere_phrase, Rendu};
use crate::app::GlucoseApp;
use glucose_core::recensement::{homonymes, recenser, DocumentRecense, DomaineConnu, NoeudTrouve};
use glucose_core::store::agents::{DomaineDecrit, Naissance, Refus, Voulu};
use glucose_core::store::{lignee_de, DomainPatch, Store};
use glucose_core::types::Domain;

/// La couleur d'un domaine neuf : le même thème que ceux qu'on crée au panneau.
fn couleur(rang: usize) -> String {
    crate::dock::domains::fresh_look(rang).0.to_string()
}

fn naissance() -> Naissance {
    Naissance {
        couleur,
        instant: crate::persist::now_millis(),
    }
}

/// **Un domaine demandé par un appel**, une fois sa question réglée : ce qu'on emporte dans le geste.
pub(super) enum Choisi {
    Existant(Domain),
    Nouveau(String),
    /// Un retrait d'un domaine que personne ne porte : rien à faire.
    Rien,
}

impl Choisi {
    fn voulu(&self) -> Option<Voulu<'_>> {
        match self {
            Self::Existant(d) => Some(Voulu::Existant(d)),
            Self::Nouveau(n) => Some(Voulu::Nouveau(n)),
            Self::Rien => None,
        }
    }
}

/// **Pose sur ces nœuds les domaines qu'un appel demande** : dans son geste, pour qu'un seul `Ctrl+Z`
/// défasse le nœud et ses domaines.
pub(super) fn poser_les_domaines(
    s: &mut Store,
    board: &str,
    ids: &[&str],
    voulus: &[(Choisi, f64)],
) -> Result<(), Refus> {
    let naissance = naissance();
    for (choisi, poids) in voulus {
        if let Some(voulu) = choisi.voulu() {
            s.poser_un_domaine_pour_un_agent(board, ids, (voulu, *poids), naissance)?;
        }
    }
    Ok(())
}

/// Les domaines d'un nœud, pour l'agent : la forme qu'il peut redonner telle quelle.
pub(super) fn domaines_en_json(domaines: &[DomaineDecrit]) -> Json {
    Json::Liste(
        domaines
            .iter()
            .map(|d| {
                Json::objet([
                    ("domaine", Json::from(d.nom.as_str())),
                    ("poids", Json::Nombre(d.poids)),
                ])
            })
            .collect(),
    )
}

fn pluriel(n: usize, mot: &str) -> String {
    if n > 1 {
        format!("{n} {mot}s")
    } else {
        format!("{n} {mot}")
    }
}

/// La liste des noms connus, pour qu'une faute se corrige sans relire le tableau.
fn noms_connus(vue: &[DomaineConnu]) -> String {
    if vue.is_empty() {
        return "aucun pour l'instant".to_string();
    }
    vue.iter()
        .map(|c| format!("« {} »", c.definition.name))
        .collect::<Vec<_>>()
        .join(", ")
}

impl GlucoseApp {
    /// Ce document tel qu'il est, et tout ce qu'on connaît avec lui.
    fn vue_des_domaines(&self) -> (DocumentRecense, Vec<DomaineConnu>) {
        let courant = recenser(&self.fichier_de_ce_document(), &self.store.project);
        let vue = self.disque.recenseur.etat.vue(Some(&courant));
        (courant, vue)
    }

    /// Un exemple de ce qu'un domaine désigne : la première phrase de son nœud le plus lourd.
    fn exemple_de(&self, courant: &DocumentRecense, connu: &DomaineConnu) -> String {
        self.disque
            .recenseur
            .etat
            .noeuds_du_domaine(Some(courant), &connu.lignee)
            .iter()
            .find_map(|t| t.decrit.texte.as_deref().and_then(premiere_phrase))
            .map_or_else(String::new, |p| format!(", par exemple « {p} »"))
    }

    /// **La question d'un nom déjà pris** (sa règle) : ce qu'il désigne, et les deux réponses.
    fn question_du_nom_pris(&self, courant: &DocumentRecense, connu: &DomaineConnu) -> String {
        let nom = &connu.definition.name;
        format!(
            "« {nom} » existe déjà : {} dans {}{}. Si c'est le même sujet, redonne-le dans « domaine ». \
             Sinon, précise le mot dans « nouveau » : « {nom} … ».",
            pluriel(connu.noeuds(), "nœud"),
            pluriel(connu.fichiers.len(), "fichier"),
            self.exemple_de(courant, connu),
        )
    }

    /// **La question de deux créations séparées** : Glucose ne les réunit pas en silence.
    fn question_des_homonymes(&self, courant: &DocumentRecense, h: &[&DomaineConnu]) -> String {
        let chacun: Vec<String> = h
            .iter()
            .map(|c| {
                format!(
                    "{} dans {}{}",
                    pluriel(c.noeuds(), "nœud"),
                    c.fichiers
                        .iter()
                        .map(|f| f.chemin.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.exemple_de(courant, c)
                )
            })
            .collect();
        format!(
            "« {} » désigne {} domaines créés séparément : {}. Est-ce le même sujet ? Si oui, appelle \
             glucose_poser_domaine avec « domaine » et « meme_sujet » : true. Sinon, précise le mot \
             avec « domaine » et « renommer ».",
            h[0].definition.name,
            h.len(),
            chacun.join(" ; "),
        )
    }

    /// **Ce que `domaine` désigne**, ou la question à poser. `nouveau` passe par la règle du nom pris.
    fn choisir(
        &self,
        (courant, vue): (&DocumentRecense, &[DomaineConnu]),
        (domaine, nouveau): (Option<&str>, Option<&str>),
        poids: f64,
    ) -> Result<Choisi, String> {
        match (domaine, nouveau) {
            (Some(_), Some(_)) => {
                Err("donne « domaine » (il existe) ou « nouveau » (à créer), pas les deux".into())
            }
            (None, None) => {
                Err("il manque « domaine » (il existe) ou « nouveau » (à créer)".into())
            }
            (None, Some(nom)) => match homonymes(vue, nom).first() {
                Some(connu) => Err(self.question_du_nom_pris(courant, connu)),
                None => Ok(Choisi::Nouveau(nom.to_string())),
            },
            (Some(nom), None) => {
                let h = homonymes(vue, nom);
                match h.as_slice() {
                    [] if poids == 0.0 => Ok(Choisi::Rien),
                    [] => Err(format!(
                        "aucun domaine « {nom} ». Ceux qui existent : {}. Pour en créer un, donne son \
                         nom dans « nouveau ».",
                        noms_connus(vue)
                    )),
                    [connu] => Ok(Choisi::Existant(connu.definition.clone())),
                    plusieurs => Err(self.question_des_homonymes(courant, plusieurs)),
                }
            }
        }
    }

    /// **Ce que `domaines` demande de poser**, avec un nœud : `[{"domaine":"eau","poids":0.8}]`, la
    /// forme même qu'un agent lit, ou `{"nouveau":"eau chaude"}` ; un simple nom vaut un poids de 1.
    pub(super) fn domaines_voulus(&self, a: &Args<'_>) -> Result<Vec<(Choisi, f64)>, String> {
        let Some(valeur) = a.0.get("domaines") else {
            return Ok(Vec::new());
        };
        let faute = "l'argument « domaines » est une liste : [{\"domaine\":\"eau\",\"poids\":0.8}]";
        let (courant, vue) = self.vue_des_domaines();
        let mut voulus = Vec::new();
        for d in valeur.as_liste().ok_or(faute)? {
            let (domaine, nouveau, poids) = match d {
                Json::Texte(nom) => (Some(nom.as_str()), None, 1.0),
                Json::Objet(_) => (
                    d.get("domaine").and_then(Json::as_str),
                    d.get("nouveau").and_then(Json::as_str),
                    d.get("poids").and_then(Json::as_f64).unwrap_or(1.0),
                ),
                _ => return Err(faute.to_string()),
            };
            let choisi = self.choisir((&courant, &vue), (domaine, nouveau), poids)?;
            voulus.push((choisi, poids));
        }
        Ok(voulus)
    }

    /// **Le catalogue de ce document** : chaque domaine, combien de nœuds le portent ici, et s'il faut
    /// préciser son nom — ce que le panneau Domaines montre à l'humain.
    pub(super) fn catalogue_pour_un_agent(&self) -> Json {
        let (_, vue) = self.vue_des_domaines();
        let porteurs = self.store.porteurs_des_domaines();
        let domaines = self.store.project.domains.iter().zip(porteurs);
        Json::Liste(
            domaines
                .map(|(d, noeuds)| {
                    let mut champs = vec![
                        ("nom", Json::from(d.name.as_str())),
                        ("couleur", Json::from(d.color.as_str())),
                        ("sigle", Json::from(d.icon.as_str())),
                        ("noeuds", Json::Nombre(noeuds as f64)),
                    ];
                    if homonymes(&vue, &d.name).len() > 1 {
                        champs.push(("a_preciser", Json::Bool(true)));
                    }
                    Json::objet(champs)
                })
                .collect(),
        )
    }

    /// **Le domaine que `domaine` désigne, pour lire ce tableau** : son identifiant dans ce document,
    /// ou la faute qui dit ceux qu'il a. Il ne se crée jamais à la lecture.
    pub(super) fn domaine_a_lire(&self, a: &Args<'_>) -> Result<Option<String>, String> {
        let Some(designe) = a.texte("domaine") else {
            return Ok(None);
        };
        match self.store.trouver_un_domaine(designe) {
            Some(d) => Ok(Some(d.id.clone())),
            None => {
                let (_, vue) = self.vue_des_domaines();
                Err(format!(
                    "ce document n'a pas de domaine « {designe} ». Ceux qu'on connaît : {} ; \
                     glucose_domaines les lit dans tous les fichiers.",
                    noms_connus(&vue)
                ))
            }
        }
    }

    /// **`glucose_poser_domaine`** : prendre ou créer un domaine, le poser sur des nœuds ; ou le
    /// renommer, ou dire qu'il est le même sujet qu'un homonyme. En un geste.
    pub(super) fn agent_poser_domaine(
        &mut self,
        board: &str,
        a: &Args<'_>,
    ) -> Result<Json, String> {
        if let Some(nom) = a.texte("renommer") {
            return self.agent_renommer_un_domaine(a, nom);
        }
        if a.oui("meme_sujet") {
            return self.agent_reunir_un_domaine(a);
        }
        let poids = a.nombre("poids").unwrap_or(1.0);
        let couleur = a.couleur()?;
        let ids: Vec<&str> = match a.0.get("ids") {
            None => Vec::new(),
            Some(l) => l
                .as_liste()
                .ok_or("l'argument « ids » est une liste d'identifiants")?
                .iter()
                .filter_map(Json::as_str)
                .collect(),
        };
        let choisi = {
            let (courant, vue) = self.vue_des_domaines();
            self.choisir(
                (&courant, &vue),
                (a.texte("domaine"), a.texte("nouveau")),
                poids,
            )?
        };
        let Some(voulu) = choisi.voulu() else {
            return Ok(Json::objet([("cree", Json::Bool(false))]));
        };
        let naissance = naissance();
        let (id, cree) = self
            .store
            .geste_d_agent(|s| {
                let issue =
                    s.poser_un_domaine_pour_un_agent(board, &ids, (voulu, poids), naissance)?;
                if let (Some(id), Some(c)) = (&issue.0, couleur) {
                    let patch = DomainPatch::new().with_color(c).at(naissance.instant);
                    s.try_update_domain(id, patch)
                        .map_err(|e| Refus::Domaine(e.to_string()))?;
                }
                Ok(issue)
            })
            .map_err(|r| r.to_string())?;
        let nom = id
            .as_deref()
            .and_then(|i| self.store.domain(i))
            .map_or("", |d| d.name.as_str());
        Ok(Json::objet([
            ("domaine", Json::from(nom)),
            ("cree", Json::Bool(cree)),
            ("noeuds", Json::Nombre(ids.len() as f64)),
        ]))
    }

    /// **Renommer** : le nom neuf doit être libre partout ; un domaine connu seulement d'un autre
    /// fichier entre d'abord dans celui-ci. L'identité ne bouge pas (DOM-5).
    fn agent_renommer_un_domaine(&mut self, a: &Args<'_>, nom: &str) -> Result<Json, String> {
        let designe = a.requis("domaine")?;
        let (courant, vue) = self.vue_des_domaines();
        let ici = self.store.trouver_un_domaine(designe).cloned();
        let h = homonymes(&vue, designe);
        let definition = match (ici, h.as_slice()) {
            (Some(d), _) => d,
            (None, [connu]) => connu.definition.clone(),
            (None, []) => return Err(format!("aucun domaine « {designe} »")),
            (None, plusieurs) => return Err(self.question_des_homonymes(&courant, plusieurs)),
        };
        let lignee = lignee_de(&definition);
        if let Some(pris) = homonymes(&vue, nom).iter().find(|c| c.lignee != lignee) {
            return Err(self.question_du_nom_pris(&courant, pris));
        }
        let instant = crate::persist::now_millis();
        self.store
            .geste_d_agent(|s| {
                let refus = |e: glucose_core::error::CoreError| Refus::Domaine(e.to_string());
                let id = s.adopter_un_domaine(&definition).map_err(refus)?;
                let patch = DomainPatch::new().with_name(nom).at(instant);
                s.try_update_domain(&id, patch).map_err(refus)
            })
            .map_err(|r| r.to_string())?;
        Ok(Json::objet([("domaine", Json::from(nom))]))
    }

    /// **« C'est le même sujet »** : le domaine de ce document prend l'identité de son homonyme.
    fn agent_reunir_un_domaine(&mut self, a: &Args<'_>) -> Result<Json, String> {
        let designe = a.requis("domaine")?;
        let Some(ici) = self.store.trouver_un_domaine(designe).cloned() else {
            return Err(format!(
                "ce document ne porte pas « {designe} » : ouvre un des fichiers qui le portent"
            ));
        };
        let (_, vue) = self.vue_des_domaines();
        let lignee = lignee_de(&ici);
        let autre = homonymes(&vue, designe)
            .into_iter()
            .find(|c| c.lignee != lignee)
            .map(|c| c.definition.clone())
            .ok_or_else(|| format!("« {designe} » n'a pas d'homonyme : rien à réunir"))?;
        let suivis = self
            .store
            .geste_d_agent(|s| {
                s.reunir_un_domaine(&ici.id, &autre)
                    .map_err(|e| Refus::Domaine(e.to_string()))
            })
            .map_err(|r| r.to_string())?;
        Ok(Json::objet([
            ("domaine", Json::from(autre.name.as_str())),
            ("noeuds", Json::Nombre(suivis as f64)),
        ]))
    }

    /// **`glucose_domaines`** : la table des matières de tout ce que Glucose connaît, ou les nœuds
    /// d'un domaine, dans tous les fichiers, sans en ouvrir aucun à l'écran.
    pub(super) fn agent_lire_les_domaines(&mut self, a: &Args<'_>) -> Result<Json, String> {
        // Ce que le disque a déjà rendu d'abord ; « a_jour » dit si une lecture restait en route.
        // Puis une passe, pour ce qui aurait changé depuis : elle ne bloque rien.
        self.disque.recenseur.relever();
        let a_jour = !self.disque.recenseur.en_cours;
        self.disque.recenseur.relancer();
        let (courant, vue) = self.vue_des_domaines();
        let tete = [
            ("fichier_ouvert", Json::from(courant.chemin.as_str())),
            ("a_jour", Json::Bool(a_jour)),
        ];
        let Some(designe) = a.texte("domaine") else {
            let domaines = vue.iter().map(|c| connu_en_json(c, &vue)).collect();
            let champs = tete
                .into_iter()
                .chain([("domaines", Json::Liste(domaines))]);
            return Ok(Json::objet(champs));
        };
        let h = homonymes(&vue, designe);
        if h.is_empty() {
            return Err(format!(
                "aucun domaine « {designe} ». Ceux qui existent : {}.",
                noms_connus(&vue)
            ));
        }
        let rendu = |t: &NoeudTrouve<'_>| -> Option<Rendu> {
            match a.texte("cherche") {
                Some(c) => Some(Rendu::Passage(passage(t.decrit.texte.as_deref()?, c)?)),
                None if a.oui("entier") => Some(Rendu::Entier),
                None => Some(Rendu::Sommaire),
            }
        };
        let mut noeuds = Vec::new();
        for connu in &h {
            for t in self
                .disque
                .recenseur
                .etat
                .noeuds_du_domaine(Some(&courant), &connu.lignee)
            {
                if let Some(r) = rendu(&t) {
                    noeuds.push(trouve_en_json(&t, &r));
                }
            }
        }
        let mut champs: Vec<(&str, Json)> = tete.into_iter().collect();
        champs.push(("domaine", connu_en_json(h[0], &vue)));
        if h.len() > 1 {
            champs.push((
                "question",
                Json::from(self.question_des_homonymes(&courant, &h)),
            ));
        }
        champs.push(("noeuds", Json::Liste(noeuds)));
        Ok(Json::objet(champs))
    }
}

/// Un domaine connu : ce qu'il est, combien de nœuds, et dans quels fichiers.
fn connu_en_json(c: &DomaineConnu, vue: &[DomaineConnu]) -> Json {
    let fichiers = c
        .fichiers
        .iter()
        .map(|f| {
            Json::objet([
                ("fichier", Json::from(f.chemin.as_str())),
                ("noeuds", Json::Nombre(f.noeuds as f64)),
            ])
        })
        .collect();
    let mut champs = vec![
        ("nom", Json::from(c.definition.name.as_str())),
        ("couleur", Json::from(c.definition.color.as_str())),
        ("sigle", Json::from(c.definition.icon.as_str())),
        ("noeuds", Json::Nombre(c.noeuds() as f64)),
        ("fichiers", Json::Liste(fichiers)),
    ];
    if homonymes(vue, &c.definition.name).len() > 1 {
        champs.push(("a_preciser", Json::Bool(true)));
    }
    Json::objet(champs)
}

/// Un nœud trouvé par son domaine : ce que `glucose_lire` en dirait, et d'où il vient.
fn trouve_en_json(t: &NoeudTrouve<'_>, rendu: &Rendu) -> Json {
    let mut champs = vec![
        ("fichier".to_string(), Json::from(t.chemin)),
        ("tableau".to_string(), Json::from(t.tableau)),
        ("poids".to_string(), Json::Nombre(t.poids)),
    ];
    if let Json::Objet(reste) = decrit_en_json(t.decrit, rendu) {
        champs.extend(reste);
    }
    Json::Objet(champs)
}

#[cfg(test)]
mod tests;
