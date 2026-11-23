"""Les épreuves du relais : ce qu'il décide, sans Glucose ni Claude Code (python3 -m unittest)."""
import json
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

import relais as r

H = {"gestes": [
    {"auteur": "agent le terminal", "fait": ["pose le texte « relais-consigne »"]},
    {"auteur": "humain", "fait": ["déplace des nœuds"]},
]}


class Decider(unittest.TestCase):
    def test_sa_consigne_signee_fait_continuer(self):
        self.assertEqual(r.decider({}, "continue", False, True, False, 0)[0], "continuer")

    def test_une_consigne_non_signee_laisse_s_arreter(self):
        self.assertEqual(r.decider({}, "continue", False, False, False, 0), ("laisser", "consigne non signée"))

    def test_la_pause_laisse_s_arreter(self):
        self.assertEqual(r.decider({}, "x", True, True, False, 0), ("laisser", "pause"))

    def test_le_seuil_demande_la_passation_meme_en_pause(self):
        self.assertEqual(r.decider({}, "x", True, True, True, 0)[0], "passation")

    def test_apres_la_passation_ecrite_on_relaie(self):
        self.assertEqual(r.decider({"passation_demandee": 1}, "x", False, True, True, 0, ecrite=True),
                         ("laisser", "relayer"))

    def test_une_passation_demandee_mais_pas_ecrite_se_redemande(self):
        """La demande tapée par le veilleur en plein tour : le tour qui finit ne l'a pas encore lue."""
        self.assertEqual(r.decider({"passation_demandee": 1}, "x", False, True, False, 0, ecrite=False),
                         ("passation", ""))

    def test_sans_passation_apres_trois_arrets_on_relaie_quand_meme(self):
        session = {"passation_demandee": 1, "arrets": [100, 150, 190]}
        self.assertEqual(r.decider(session, "x", False, True, True, 200, ecrite=False), ("laisser", "relayer"))

    def test_trois_arrets_en_deux_minutes_c_est_a_vide(self):
        self.assertEqual(r.decider({"arrets": [100, 150, 190]}, "x", False, True, False, 200),
                         ("laisser", "à vide"))


class Signature(unittest.TestCase):
    def test_ecrite_par_un_agent_elle_ne_vaut_rien(self):
        self.assertFalse(r.signee({"empreinte": "e1"}, {}, H))

    def test_retouchee_par_lui_elle_vaut_et_s_en_souvient(self):
        h = {"gestes": H["gestes"] + [{"auteur": "humain", "fait": ["écrit dans le texte « relais-consigne »"]}]}
        memoire = {}
        self.assertTrue(r.signee({"empreinte": "e2"}, memoire, h))
        self.assertTrue(r.signee({"empreinte": "e2"}, memoire, H), "l'empreinte gardée suffit ensuite")

    def test_un_agent_qui_la_change_lui_retire_sa_valeur(self):
        self.assertFalse(r.signee({"empreinte": "e3"}, {"consigne_signee": "e2"}, H))

    def test_la_pause_se_lit_en_premiere_ligne(self):
        self.assertTrue(r.en_pause("⏸ PAUSE\nla suite"))
        self.assertTrue(r.en_pause("pause ce soir"))
        self.assertFalse(r.en_pause("continue\n⏸ PAUSE plus bas ne compte pas"))


class Jauges(unittest.TestCase):
    def setUp(self):
        self.dossier = tempfile.TemporaryDirectory()
        self.etat = mock.patch.object(r, "ETAT", Path(self.dossier.name))
        self.etat.start()

    def tearDown(self):
        self.etat.stop()
        self.dossier.cleanup()

    def poser(self, compte, cinq, semaine, remise):
        r.ecrire_json(r.ETAT / f"jauges-{compte}.json", {"rate_limits": {
            "five_hour": {"used_percentage": cinq, "resets_at": remise},
            "seven_day": {"used_percentage": semaine, "resets_at": remise + 1000}}})

    def test_au_seuil_on_sait_quand_il_se_libere(self):
        self.poser("a", 91, 10, 5000)
        self.assertEqual(r.plein("a", 1000), (True, 5000))

    def test_deux_fenetres_pleines_libre_a_la_plus_lointaine(self):
        """Le 23/11 : cinq heures pleines jusqu'à 4 h, la semaine jusqu'à 16 h — libre à 16 h."""
        self.poser("b", 95, 100, 5000)
        self.assertEqual(r.plein("b", 1000), (True, 6000))

    def test_le_mur_ferme_la_session_meme_si_l_autre_fenetre_est_vide(self):
        self.poser("b", 4, 100, 5000)
        self.assertEqual(r.garder({"compte": "b"}, 1000), "fermer")

    def test_le_seuil_demande_la_passation_une_seule_fois(self):
        self.poser("a", 92, 10, 5000)
        self.assertEqual(r.garder({"compte": "a"}, 1000), "passation")
        self.assertIsNone(r.garder({"compte": "a", "passation_demandee": 900}, 1000))

    def test_sous_le_seuil_ou_jauge_jamais_lue_le_veilleur_laisse_faire(self):
        """Une jauge jamais lue : on lance, et la session elle-même la lit — son lancement sonde."""
        self.poser("a", 89, 94, 5000)
        self.assertIsNone(r.garder({"compte": "a"}, 1000))
        self.assertIsNone(r.garder({"compte": "b"}, 1000))

    def test_une_remise_passee_vide_la_fenetre(self):
        self.poser("a", 99, 10, 500)
        self.assertEqual(r.plein("a", 1000), (False, None))

    def test_on_passe_au_compte_libre_ou_on_attend_le_premier_libre(self):
        self.poser("a", 95, 10, 5000)
        self.poser("b", 20, 10, 7000)
        with mock.patch.object(r, "connecte", return_value=True):
            self.assertEqual(r.choisir("a", 1000), ("b", None))
            self.poser("b", 92, 10, 3000)
            self.assertEqual(r.choisir("a", 1000), (None, 3000))

    def test_un_compte_pas_connecte_n_est_jamais_choisi(self):
        with mock.patch.object(r, "connecte", side_effect=lambda c: c == "a"):
            self.assertEqual(r.choisir("b", 1000), ("a", None))

    def test_la_barre_d_etat_garde_les_jauges_du_compte(self):
        entree = {"model": {"display_name": "Opus"}, "rate_limits": {
            "five_hour": {"used_percentage": 60, "resets_at": time.time() + 99}}}
        with mock.patch("sys.stdin") as stdin, mock.patch("builtins.print") as dire, \
                mock.patch.dict("os.environ", {"RELAIS_COMPTE": "b"}):
            stdin.read.return_value = json.dumps(entree)
            r.jauges()
        self.assertEqual(r.charge("b", time.time())["five_hour"][0], 60.0)
        dire.assert_called_once_with("Opus · 5 h 60 %")


class Veilleur(unittest.TestCase):
    """Un tour du veilleur, Glucose et tmux remplacés : ce qu'il fait d'une session vivante."""

    def setUp(self):
        self.dossier = tempfile.TemporaryDirectory()
        self.maintenant = time.time()
        self.retouches = [mock.patch.object(r, "ETAT", Path(self.dossier.name))]
        consigne = {"id": r.CONSIGNE, "empreinte": "e", "texte": "continue"}
        self.passation = {"id": r.PASSATION, "empreinte": "p1"}
        noeuds = {r.CONSIGNE: consigne, r.PASSATION: self.passation}
        self.tmux, self.lancer, self.taper = mock.Mock(), mock.Mock(), mock.Mock()
        for nom, valeur in (("noeud", lambda ident: noeuds.get(ident)), ("glucose", lambda *a, **k: {}),
                            ("ecran", lambda: "l'écran"), ("session_tmux", lambda: "relais-b"),
                            ("connecte", lambda c: True), ("tmux", self.tmux), ("lancer", self.lancer),
                            ("taper", self.taper), ("battre", mock.Mock()), ("surveiller", mock.Mock()),
                            ("journal", mock.Mock())):
            self.retouches.append(mock.patch.object(r, nom, valeur))
        for retouche in self.retouches:
            retouche.start()
        r.ecrire_json(r.ETAT / "veilleur.json", {"consigne_signee": "e", "compte": "b"})

    def tearDown(self):
        for retouche in reversed(self.retouches):
            retouche.stop()
        self.dossier.cleanup()

    def jauge(self, compte, cinq, semaine):
        r.ecrire_json(r.ETAT / f"jauges-{compte}.json", {"rate_limits": {
            "five_hour": {"used_percentage": cinq, "resets_at": self.maintenant + 3600},
            "seven_day": {"used_percentage": semaine, "resets_at": self.maintenant + 9000}}})

    def test_au_mur_il_ferme_et_lance_l_autre_compte_dans_le_meme_tour(self):
        """La nuit du 23/11 : la session b ouverte sur une semaine pleine passait pour vivante."""
        self.jauge("b", 4, 100)
        self.jauge("a", 10, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        r.un_tour()
        self.tmux.assert_any_call("kill-session", "-t", "relais-b")
        self.lancer.assert_called_once_with("a")
        self.assertEqual(r.lire_json(r.ETAT / "session.json", {}).get("laissee"), "compte à sa limite")

    def test_au_mur_des_deux_comptes_il_attend_la_plus_lointaine_remise(self):
        self.jauge("b", 95, 100)
        self.jauge("a", 100, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        r.un_tour()
        self.lancer.assert_not_called()
        self.assertIn(r.heure(self.maintenant + 3600), r.lire_json(r.ETAT / "veilleur.json", {})["dit"])

    def test_le_seuil_passe_en_plein_tour_la_passation_est_tapee_une_fois(self):
        self.jauge("b", 92, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        r.un_tour()
        r.un_tour()
        self.taper.assert_called_once_with(r.RAISON_PASSATION)
        session = r.lire_json(r.ETAT / "session.json", {})
        self.assertEqual(session.get("passation_avant"), "p1")
        self.tmux.assert_not_called()

    def test_relayee_il_ferme_et_dit_si_la_passation_a_ete_ecrite(self):
        self.jauge("b", 92, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": 0, "passation_demandee": 1,
                                                "passation_avant": "p1", "laissee": "relayer"})
        self.passation["empreinte"] = "p2"
        r.un_tour()
        self.tmux.assert_any_call("kill-session", "-t", "relais-b")
        dits = [appel.args[0] for appel in r.journal.call_args_list]
        self.assertTrue(any("passation écrite" in dit for dit in dits), dits)
        self.lancer.assert_called_once_with("a")

    def test_relayee_sans_passation_neuve_il_le_dit(self):
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": 0, "passation_demandee": 1,
                                                "passation_avant": "p1", "laissee": "relayer"})
        r.un_tour()
        dits = [appel.args[0] for appel in r.journal.call_args_list]
        self.assertTrue(any("PAS récrite" in dit for dit in dits), dits)

    def test_la_reference_est_la_passation_d_avant_la_premiere_demande(self):
        session = {}
        r.demander_passation(session, 1)
        self.passation["empreinte"] = "p2"
        r.demander_passation(session, 2)
        self.assertEqual((session["passation_demandee"], session["passation_avant"]), (1, "p1"))
        self.assertTrue(r.passation_ecrite(session))

    def crochet(self):
        with mock.patch("sys.stdin") as stdin, mock.patch("builtins.print") as dire, \
                mock.patch.dict("os.environ", {"RELAIS": "1", "RELAIS_COMPTE": "b"}):
            stdin.read.return_value = "{}"
            r.fin_de_tour()
        return dire, r.lire_json(r.ETAT / "session.json", {})

    def test_le_crochet_laisse_partir_une_fois_la_passation_ecrite(self):
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "passation_demandee": 1, "passation_avant": "p1"})
        dire, session = self.crochet()
        self.assertNotEqual(session.get("laissee"), "relayer")
        self.assertIn(json.dumps(r.RAISON_PASSATION, ensure_ascii=False), dire.call_args.args[0])
        self.passation["empreinte"] = "p2"
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "passation_demandee": 1, "passation_avant": "p1"})
        dire, session = self.crochet()
        self.assertEqual(session.get("laissee"), "relayer")
        dire.assert_not_called()


class Travail(unittest.TestCase):
    """Ce que GitHub n'a pas encore, dans un vrai dépôt Git jetable (son « GitHub » : un dépôt nu)."""

    def git(self, *a, dans):
        subprocess.run(["git", "-C", str(dans), *a], check=True, capture_output=True)

    def test_compte_contre_github_pas_contre_origin_main(self):
        identite = {f"GIT_{q}_{c}": v for q in ("AUTHOR", "COMMITTER")
                    for c, v in (("NAME", "relais"), ("EMAIL", "relais@exemple.invalid"))}
        with tempfile.TemporaryDirectory() as d, mock.patch.dict("os.environ", identite):
            d = Path(d)
            distant, depot = d / "distant.git", d / "depot"
            subprocess.run(["git", "init", "-q", "--bare", "-b", "main", str(distant)], check=True)
            subprocess.run(["git", "clone", "-q", str(distant), str(depot)], check=True, capture_output=True)
            self.git("checkout", "-q", "-b", "main", dans=depot)
            self.git("commit", "-q", "--allow-empty", "-m", "A0", dans=depot)
            self.git("push", "-q", "origin", "main", dans=depot)
            self.git("commit", "-q", "--allow-empty", "-m", "A1", dans=depot)
            # Comme le relais envoie : par l'adresse (le jeton), qui ne touche pas origin/main.
            self.git("push", "-q", str(distant), "main:main", dans=depot)
            self.git("commit", "-q", "--allow-empty", "-m", "A2", dans=depot)
            with mock.patch.object(r, "DEPOT", depot):
                self.assertIn("· 1 pas encore sur GitHub", r.travail(0))
                # La preuve à l'envers : l'ancienne loi comptait A1, déjà parti.
                self.assertEqual(r.git("rev-list", "--count", "origin/main..main"), "2")


class Jeton(unittest.TestCase):
    def test_present_absent_et_a_renouveler_sans_jamais_sa_valeur(self):
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / "github.md"
            with mock.patch.object(r, "JETON", f):
                self.assertTrue(r.jeton_github(0).startswith("absent"))
                f.write_text("# commentaire\n\n")
                self.assertTrue(r.jeton_github(0).startswith("absent"))
                f.write_text("# commentaire\nghp_secret\n")
                dit = r.jeton_github(f.stat().st_mtime + 4 * 86400)
                self.assertIn("à renouveler", dit)
                self.assertNotIn("ghp_secret", dit)


class Binaire(unittest.TestCase):
    def test_le_claude_de_la_derniere_extension_par_numero_pas_par_texte(self):
        with tempfile.TemporaryDirectory() as d:
            for v in ("2.1.99", "2.1.330", "2.1.4"):
                b = Path(d) / f"anthropic.claude-code-{v}-linux-x64/resources/native-binary/claude"
                b.parent.mkdir(parents=True)
                b.write_text("")
            with mock.patch.object(r, "EXTENSIONS", Path(d)):
                self.assertIn("2.1.330", r.claude_binaire())


if __name__ == "__main__":
    unittest.main()
