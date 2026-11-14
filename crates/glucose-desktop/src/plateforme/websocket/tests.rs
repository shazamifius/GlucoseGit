use super::trames::{self, Decodeur, Trame};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

const MASQUE: [u8; 4] = [0x37, 0xfa, 0x21, 0x3d];

/// Pousse les octets un par un : un message arrive toujours par morceaux quelconques.
fn decoder_octet_par_octet(d: &mut Decodeur, octets: &[u8]) -> Vec<Trame> {
    let mut sortis = Vec::new();
    for o in octets {
        d.pousser(&[*o]);
        while let Some(t) = d.suivante().expect("trames valides") {
            sortis.push(t);
        }
    }
    sortis
}

/// Chaque longueur où l'en-tête change de forme (125, 126, 65 535, 65 536), dans les deux sens :
/// le client masque, le serveur non, et chacun relit l'autre.
#[test]
fn les_trames_traversent_dans_les_deux_sens_a_chaque_frontiere_de_longueur() {
    for n in [0usize, 1, 125, 126, 127, 65_535, 65_536, 70_000] {
        let texte: String = (0..n).map(|i| char::from(b'a' + (i % 26) as u8)).collect();
        let trame = Trame::Texte(texte);
        let du_client = trames::ecrire(&trame, Some(MASQUE));
        let mut serveur = Decodeur::d_un_client();
        serveur.pousser(&du_client);
        assert_eq!(serveur.suivante(), Ok(Some(trame.clone())), "client, {n}");
        assert_eq!(serveur.reste(), 0);
        let du_serveur = trames::ecrire(&trame, None);
        let mut client = Decodeur::d_un_serveur();
        client.pousser(&du_serveur);
        assert_eq!(client.suivante(), Ok(Some(trame)), "serveur, {n}");
    }
}

#[test]
fn un_message_arrive_entier_meme_par_morceaux_d_un_octet() {
    let mut octets = trames::ecrire(&Trame::Texte("é ü ∑ ß".into()), None);
    octets.extend(trames::ecrire(&Trame::Ping(b"x".to_vec()), None));
    octets.extend(trames::ecrire(&Trame::Fermeture(Some(1000)), None));
    let mut d = Decodeur::d_un_serveur();
    assert_eq!(
        decoder_octet_par_octet(&mut d, &octets),
        vec![
            Trame::Texte("é ü ∑ ß".into()),
            Trame::Ping(b"x".to_vec()),
            Trame::Fermeture(Some(1000)),
        ]
    );
}

/// Un message fragmenté (§ 5.4) — une trame de contrôle au milieu, qui passe d'abord.
#[test]
fn un_message_fragmente_se_recompose() {
    let mut octets = vec![0x01, 3];
    octets.extend_from_slice(b"bon");
    octets.extend(trames::ecrire(&Trame::Ping(vec![]), None));
    octets.extend_from_slice(&[0x80, 4]);
    octets.extend_from_slice(b"jour");
    let mut d = Decodeur::d_un_serveur();
    assert_eq!(
        decoder_octet_par_octet(&mut d, &octets),
        vec![Trame::Ping(vec![]), Trame::Texte("bonjour".into())]
    );
}

/// Ce qui ne suit pas le protocole ferme la connexion : un serveur qui masque, un client qui ne
/// masque pas, des bits réservés, un contrôle trop long, un texte qui n'est pas de l'UTF-8, une
/// suite sans début.
#[test]
fn ce_qui_viole_le_protocole_est_une_erreur() {
    let cas: [(&str, Vec<u8>, bool); 6] = [
        (
            "serveur masqué",
            trames::ecrire(&Trame::Texte("a".into()), Some(MASQUE)),
            false,
        ),
        (
            "client nu",
            trames::ecrire(&Trame::Texte("a".into()), None),
            true,
        ),
        ("bits réservés", vec![0xc1, 0x00], false),
        (
            "contrôle long",
            trames::ecrire(&Trame::Ping(vec![0; 126]), None),
            false,
        ),
        ("pas UTF-8", vec![0x81, 0x02, 0xc3, 0x28], false),
        ("suite orpheline", vec![0x80, 0x01, b'a'], false),
    ];
    for (nom, octets, de_client) in cas {
        let mut d = if de_client {
            Decodeur::d_un_client()
        } else {
            Decodeur::d_un_serveur()
        };
        d.pousser(&octets);
        assert!(d.suivante().is_err(), "{nom}");
    }
}

#[test]
fn la_reponse_d_ouverture_se_lit_et_se_juge() {
    let cle = "dGhlIHNhbXBsZSBub25jZQ==";
    let bonne = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
                 Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
    let mut avec_suite = bonne.as_bytes().to_vec();
    avec_suite.extend_from_slice(&[0x81, 0x00]);
    assert_eq!(
        trames::lire_la_reponse(&avec_suite, cle),
        Ok(Some(bonne.len()))
    );
    assert_eq!(
        trames::lire_la_reponse(&bonne.as_bytes()[..40], cle),
        Ok(None)
    );
    let refus = "HTTP/1.1 401 Unauthorized\r\n\r\n";
    assert!(trames::lire_la_reponse(refus.as_bytes(), cle).is_err());
    let fausse = bonne.replace(
        "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAA=",
    );
    assert!(trames::lire_la_reponse(fausse.as_bytes(), cle).is_err());
}

/// **Un petit relais** sur cette machine, en clair : il exige l'autorisation, renvoie chaque
/// message, pingue une fois (le client doit répondre seul), puis ferme sur « fin ».
fn relais_local() -> (String, std::thread::JoinHandle<Vec<String>>) {
    let ecoute = TcpListener::bind("127.0.0.1:0").expect("un port local");
    let adresse = format!(
        "ws://{}/g/abc/glucose",
        ecoute.local_addr().expect("son adresse")
    );
    let fil = std::thread::spawn(move || {
        let (mut tcp, _) = ecoute.accept().expect("le client vient");
        tcp.set_read_timeout(Some(Duration::from_secs(10))).ok();
        let demande = lire_les_entetes(&mut tcp);
        assert!(
            demande.starts_with("GET /g/abc/glucose HTTP/1.1"),
            "{demande}"
        );
        assert!(
            demande.contains("Authorization: Bearer secret"),
            "{demande}"
        );
        let cle = demande
            .lines()
            .find_map(|l| l.strip_prefix("Sec-WebSocket-Key: "))
            .expect("une clé")
            .trim()
            .to_string();
        let reponse = format!(
            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            trames::acceptation(&cle)
        );
        tcp.write_all(reponse.as_bytes()).expect("réponse");
        tcp.write_all(&trames::ecrire(&Trame::Ping(b"vif".to_vec()), None))
            .expect("ping");
        let mut d = Decodeur::d_un_client();
        let mut vus = Vec::new();
        let mut tampon = [0u8; 4096];
        loop {
            while let Some(t) = d.suivante().expect("le client parle le protocole") {
                match t {
                    Trame::Texte(s) if s == "fin" => {
                        tcp.write_all(&trames::ecrire(&Trame::Fermeture(Some(1000)), None))
                            .ok();
                        return vus;
                    }
                    Trame::Texte(s) => {
                        tcp.write_all(&trames::ecrire(&Trame::Texte(s.clone()), None))
                            .expect("écho");
                        vus.push(s);
                    }
                    Trame::Pong(p) => vus.push(format!("pong:{}", String::from_utf8_lossy(&p))),
                    autre => vus.push(format!("{autre:?}")),
                }
            }
            let n = tcp.read(&mut tampon).expect("lecture");
            assert!(n > 0, "le client est parti avant la fin");
            d.pousser(&tampon[..n]);
        }
    });
    (adresse, fil)
}

fn lire_les_entetes(tcp: &mut TcpStream) -> String {
    let mut lus = Vec::new();
    let mut octet = [0u8; 1];
    while !lus.ends_with(b"\r\n\r\n") {
        tcp.read_exact(&mut octet).expect("en-têtes");
        lus.push(octet[0]);
    }
    String::from_utf8_lossy(&lus).into_owned()
}

/// **Le vrai chemin** — la voie du système (WinHTTP sous Windows, la nôtre ailleurs) contre un
/// relais local : l'ouverture autorisée, un message court et un long, le ping répondu seul, la
/// fermeture du relais vue comme une erreur.
#[test]
fn un_websocket_s_ouvre_parle_et_se_ferme_contre_un_relais_local() {
    let (adresse, relais) = relais_local();
    let mut canal = super::ouvrir(&adresse, &[("Authorization", "Bearer secret")]).expect("ouvert");
    let long: String = "Glucose ".repeat(40_000);
    for message in ["bonjour", long.as_str()] {
        canal.envoyer(message).expect("envoyé");
        let recu = canal
            .recevoir(Duration::from_secs(10))
            .expect("la connexion vit");
        assert_eq!(recu.as_deref(), Some(message));
    }
    assert_eq!(canal.recevoir(Duration::from_millis(50)), Ok(None));
    canal.envoyer("fin").expect("envoyé");
    assert!(canal.recevoir(Duration::from_secs(10)).is_err());
    let vus = relais.join().expect("le relais finit");
    // Le pong part quand le client lit : avant ou après « bonjour », selon la voie.
    assert!(vus.contains(&"pong:vif".to_string()), "{:?}", &vus[..1]);
    let textes: Vec<&String> = vus.iter().filter(|v| !v.starts_with("pong:")).collect();
    assert_eq!(textes, vec![&"bonjour".to_string(), &long]);
}
