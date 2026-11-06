//! **L'échelle des moteurs** (MOTEUR-1, fiche 84) : un moteur qui a fait tomber Glucose à son
//! ouverture ne s'ouvre plus.
//!
//! # Ce qui l'a demandée
//!
//! La télémétrie du 29/10 : une machine Windows, quatre lancements en dix-huit secondes, aucune
//! image, et chaque fois `igvk64.dll` — le pilote Vulkan d'Intel — au même octet. Aucun `Result`
//! ne remonte d'un pilote qui écrit où il ne doit pas : le processus meurt, et le repli vers le
//! moteur suivant ne s'exécute jamais. Glucose excluait cette machine, ce que la charte interdit.
//!
//! # Observer, sans table de pilotes ni seuil
//!
//! Avant d'ouvrir un moteur, Glucose écrit son nom ; à la première image présentée, il l'efface.
//! **Un nom encore écrit au lancement suivant dit que ce moteur n'a pas survécu à son ouverture**
//! — quel que soit le pilote, le système ou l'étape. Le moteur suivant prend sa place, et
//! l'échelle se termine : le dernier barreau est le processeur, qui n'a pas de pilote.
//!
//! # Une nouvelle version repose la question, sans risque
//!
//! Le pilote a pu être réparé, ou l'arrêt n'être qu'une coupure de courant pendant l'ouverture.
//! Une nouvelle version sonde donc chaque moteur tombé **dans un processus à part**
//! (`--sonder-moteur`), en fond : s'il meurt, personne ne le voit ; s'il vit, le moteur revient
//! au lancement suivant. Android n'a pas d'exécutable à relancer : il réessaie directement.
//!
//! La télémétrie n'a pas besoin d'une ligne de plus : la ligne `plantage` nomme le pilote, et la
//! ligne `carte` de la session suivante dit le moteur qui a pris le relais.

use crate::present::moteurs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

/// Le nom du fichier, dans le dossier de l'application.
pub const FICHIER: &str = "moteurs.txt";

/// La variable d'instrument : le moteur à l'ouverture duquel ce processus doit tomber. C'est ce
/// qui permet d'éprouver l'échelle sans le pilote fautif — comme `GLUCOSE_CARTE`, un instrument
/// et non un choix de production.
const TOMBER: &str = "GLUCOSE_TOMBER";

/// Ce que `moteurs.txt` retient d'une machine.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Echelle {
    version: String,
    tombes: Vec<String>,
    ouverture: Option<String>,
}

/// Ce que le lancement apprend en relisant le fichier.
#[derive(Debug, PartialEq, Eq)]
pub struct Lecture {
    pub echelle: Echelle,
    /// Le moteur que la session d'avant, de cette même version, n'a pas survécu à ouvrir.
    pub vient_de_tomber: Option<String>,
    /// Les moteurs tombés sous une autre version : la question se repose.
    pub a_sonder: Vec<String>,
}

impl Echelle {
    /// **Relit le fichier** pour cette version. Ce qui ne se reconnaît pas n'est pas une erreur :
    /// c'est l'absence de souvenir, et l'échelle repart du premier barreau.
    pub fn lire(texte: &str, version: &str) -> Lecture {
        let valeurs = |cle: &'static str| {
            texte
                .lines()
                .filter_map(move |l| l.trim().strip_prefix(cle)?.strip_prefix('='))
                .map(str::trim)
                .filter(|v| !v.is_empty())
        };
        let meme_version = valeurs("version").next() == Some(version);
        let ouverture = valeurs("ouverture").next().map(str::to_string);
        let mut tombes: Vec<String> = valeurs("tombe").map(str::to_string).collect();
        if let Some(o) = &ouverture {
            if !tombes.contains(o) {
                tombes.push(o.clone());
            }
        }
        Lecture {
            vient_de_tomber: ouverture.filter(|_| meme_version),
            a_sonder: if meme_version {
                Vec::new()
            } else {
                tombes.clone()
            },
            echelle: Self {
                version: version.to_string(),
                tombes,
                ouverture: None,
            },
        }
    }

    /// Le texte du fichier.
    pub fn texte(&self) -> String {
        let mut t = format!("version={}\n", self.version);
        for moteur in &self.tombes {
            t.push_str(&format!("tombe={moteur}\n"));
        }
        if let Some(moteur) = &self.ouverture {
            t.push_str(&format!("ouverture={moteur}\n"));
        }
        t
    }

    /// Ce moteur peut-il s'ouvrir ?
    pub fn permet(&self, moteur: &str) -> bool {
        !self.tombes.iter().any(|t| t == moteur)
    }

    /// **La sonde a survécu** : le moteur revient au lancement suivant.
    pub fn oublier(&mut self, moteur: &str) {
        self.tombes.retain(|t| t != moteur);
    }
}

/// L'échelle et le fichier qui la retient. Écrire **avant** d'ouvrir est toute la garantie : le
/// pilote peut tuer le processus à l'instruction suivante.
struct Habitee {
    chemin: PathBuf,
    echelle: Echelle,
}

impl Habitee {
    /// Relit ce dossier et retient aussitôt ce qui s'en déduit : un moteur tombé le reste, même
    /// si le suivant tombe à son tour.
    fn ouvrir(dossier: &Path, version: &str) -> (Self, Option<String>, Vec<String>) {
        let chemin = dossier.join(FICHIER);
        let texte = std::fs::read_to_string(&chemin).unwrap_or_default();
        let lecture = Echelle::lire(&texte, version);
        let ici = Self {
            chemin,
            echelle: lecture.echelle,
        };
        ici.retenir();
        (ici, lecture.vient_de_tomber, lecture.a_sonder)
    }

    fn retenir(&self) {
        if let Some(dossier) = self.chemin.parent() {
            let _ = std::fs::create_dir_all(dossier);
        }
        if let Err(e) = std::fs::write(&self.chemin, self.echelle.texte()) {
            println!("[Glucose] l'échelle des moteurs ne s'écrit pas : {e}");
        }
    }

    fn j_ouvre(&mut self, moteur: &str) {
        self.echelle.ouverture = Some(moteur.to_string());
        self.retenir();
    }

    fn survecu(&mut self) {
        self.echelle.ouverture = None;
        self.retenir();
    }

    fn oublier(&mut self, moteur: &str) {
        self.echelle.oublier(moteur);
        self.retenir();
    }
}

/// L'échelle de ce lancement. Vide dans les épreuves et les bancs, qui ne passent pas par
/// [`crate::demarrage`] : aucun n'écrit chez l'utilisateur, et tous les moteurs leur sont permis.
static ICI: OnceLock<Mutex<Habitee>> = OnceLock::new();

/// Un moteur attend-il encore sa première image ? Lu à chaque image : un seul atome, sans verrou.
static EN_OUVERTURE: AtomicBool = AtomicBool::new(false);

/// **Habite ce dossier, pour ce lancement** : rend le moteur que la session d'avant n'a pas
/// survécu à ouvrir, et lance en fond la sonde de ceux qu'une nouvelle version peut retrouver.
pub fn habiter(dossier: &Path, version: &str) -> Option<String> {
    let (ici, vient_de_tomber, a_sonder) = Habitee::ouvrir(dossier, version);
    if ICI.set(Mutex::new(ici)).is_err() {
        return None;
    }
    if cfg!(target_os = "android") {
        a_sonder
            .iter()
            .for_each(|m| avec_l_echelle(|e| e.oublier(m)));
    } else if !a_sonder.is_empty() {
        sonder_en_fond(a_sonder);
    }
    vient_de_tomber
}

fn avec_l_echelle(faire: impl FnOnce(&mut Habitee)) {
    if let Some(ici) = ICI.get() {
        // Un verrou empoisonné protège quand même un état juste : chaque écriture est entière.
        faire(
            &mut ici
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
    }
}

/// Chaque moteur tombé, dans son propre processus : son succès le rend au lancement suivant.
fn sonder_en_fond(moteurs: Vec<String>) {
    let _ = std::thread::Builder::new()
        .name("sonde des moteurs".into())
        .spawn(move || {
            let Ok(moi) = std::env::current_exe() else {
                return;
            };
            for moteur in moteurs {
                let survit = std::process::Command::new(&moi)
                    .args(["--sonder-moteur", &moteur])
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .is_ok_and(|s| s.success());
                println!("[Glucose] sonde du moteur {moteur} : {}", dire(survit));
                if survit {
                    avec_l_echelle(|e| e.oublier(&moteur));
                }
            }
        });
}

const fn dire(survit: bool) -> &'static str {
    if survit {
        "il reviendra au prochain lancement"
    } else {
        "il reste écarté"
    }
}

/// **Les moteurs à essayer, dans l'ordre**, sans ceux qui sont tombés.
pub fn barreaux() -> Vec<(wgpu::Backends, &'static str)> {
    let permet = |moteur: &str| {
        ICI.get().is_none_or(|ici| {
            ici.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .echelle
                .permet(moteur)
        })
    };
    moteurs::ici().filter(|(_, nom)| permet(nom)).collect()
}

/// **Ce moteur va s'ouvrir.** À appeler avant tout appel qui peut atteindre son pilote.
pub fn j_ouvre(moteur: &str) {
    avec_l_echelle(|e| e.j_ouvre(moteur));
    EN_OUVERTURE.store(true, Ordering::Release);
    tomber_si_demande(moteur);
}

/// **Il a survécu** : une image est partie à l'écran, ou Glucose se ferme normalement.
pub fn survecu() {
    if EN_OUVERTURE.swap(false, Ordering::AcqRel) {
        avec_l_echelle(Habitee::survecu);
    }
}

fn tomber_si_demande(moteur: &str) {
    if std::env::var(TOMBER).as_deref() == Ok(moteur) {
        std::process::abort();
    }
}

/// **`--sonder-moteur <nom>`** : ouvre ce moteur jusqu'à sa carte, sans fenêtre ni fichier. Le
/// code de sortie suffit : 0 s'il a survécu, 2 si le nom n'existe pas, et le système dira le
/// reste s'il tombe.
pub fn sonder(moteur: Option<&str>) -> i32 {
    // La sonde peut mourir : c'est son travail, et aucune fenêtre de Windows ne doit le signaler.
    #[cfg(windows)]
    // SAFETY : ce mode de ce processus seul, sans pointeur ni état partagé.
    unsafe {
        use windows::Win32::System::Diagnostics::Debug::{SetErrorMode, SEM_NOGPFAULTERRORBOX};
        SetErrorMode(SEM_NOGPFAULTERRORBOX);
    }
    let Some((backends, nom)) = moteurs::ici().find(|(_, nom)| Some(*nom) == moteur) else {
        return 2;
    };
    tomber_si_demande(nom);
    let instance = moteurs::instance(backends);
    let carte =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()));
    drop(carte);
    0
}

#[cfg(test)]
mod tests;
