"""Les épreuves du relais : ce qu'il décide, sans Glucose ni Claude Code (python3 -m unittest)."""
import json
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

import machine as m
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
        self.connecte = mock.patch.object(r, "connecte", return_value=True)
        self.connecte.start()

    def tearDown(self):
        self.connecte.stop()
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
        self.assertEqual(r.choisir("a", 1000), ("b", None))
        self.poser("b", 100, 10, 3000)
        self.poser("a", 100, 10, 5000)
        self.assertEqual(r.choisir("a", 1000), (None, 3000))

    def test_faute_de_mieux_un_compte_sous_le_mur_travaille(self):
        """Le 23/11, 10 h 08 : a à 91 %, b plein jusqu'à 16 h — trois heures sans personne."""
        self.poser("a", 91, 77, 5000)
        self.poser("b", 4, 100, 7000)
        self.assertEqual(r.choisir("b", 1000), ("a", None))

    def test_au_seuil_la_passation_seulement_si_un_autre_compte_peut_reprendre(self):
        self.poser("a", 92, 10, 5000)
        self.poser("b", 4, 100, 7000)
        self.assertEqual(r.garder({"compte": "a"}, 1000), "prevenir")
        self.assertIsNone(r.garder({"compte": "a", "prevenue": 900}, 1000))
        self.poser("b", 4, 10, 7000)
        self.assertEqual(r.garder({"compte": "a"}, 1000), "passation")

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
        self.lancer.assert_called_once_with("a", [])
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
        self.lancer.assert_called_once_with("a", [])

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

    def crochet(self, entree="{}"):
        with mock.patch("sys.stdin") as stdin, mock.patch("builtins.print") as dire, \
                mock.patch.dict("os.environ", {"RELAIS": "1", "RELAIS_COMPTE": "b"}):
            stdin.read.return_value = entree
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


    # ── Une seule session (sa parole du 23/11) ──

    def bail(self, il_y_a=0):
        r.ecrire_json(r.ETAT / "baux.json", {"s": {"transcript": None, "depuis": self.maintenant - il_y_a}})

    def test_sa_session_prend_la_main_le_relais_ecrit_sa_passation(self):
        self.bail()
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        r.un_tour()
        r.un_tour()
        self.taper.assert_called_once_with(r.raison_passation("retrait"))
        self.assertEqual(r.lire_json(r.ETAT / "session.json", {}).get("passation_pourquoi"), "retrait")
        self.tmux.assert_not_called()

    def test_en_retrait_il_ferme_sans_changer_de_compte_et_ne_relance_rien(self):
        self.bail()
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": 0, "passation_demandee": 1,
                                                "passation_pourquoi": "retrait", "laissee": "relayer"})
        r.un_tour()
        self.tmux.assert_any_call("kill-session", "-t", "relais-b")
        self.lancer.assert_not_called()
        memoire = r.lire_json(r.ETAT / "veilleur.json", {})
        self.assertEqual(memoire["compte"], "b")
        self.assertIn("en retrait", memoire["dit"])

    def test_sa_session_silencieuse_ou_rendue_le_relais_reprend(self):
        with mock.patch.object(r, "ecran", lambda: None):
            self.bail(il_y_a=r.SILENCE - 60)
            r.un_tour()
            self.lancer.assert_not_called()
            self.bail(il_y_a=r.SILENCE + 1)
            r.un_tour()
            self.lancer.assert_called_once()
            self.bail()
            r.rendre()
            r.un_tour()
            self.assertEqual(self.lancer.call_count, 2)

    def test_le_journal_de_sa_conversation_est_un_signe_de_vie(self):
        with tempfile.NamedTemporaryFile() as f:
            r.ecrire_json(r.ETAT / "baux.json", {"s": {"transcript": f.name, "depuis": 0}})
            self.assertIsNotNone(r.main_humaine(time.time()))
            self.assertIsNone(r.main_humaine(time.time() + r.SILENCE + 1))

    # ── Ce qu'il écrit ──

    def histoire(self, *gestes, total):
        def glucose(*a, **k):
            if a[:1] == ("histoire",):
                nombre = int(a[1].split("=")[1])
                return {"total": total, "gestes": list(gestes)[-nombre:] if nombre < len(gestes) else list(gestes)}
            return {}
        return mock.patch.object(r, "glucose", glucose)

    def test_ses_notes_d_un_autre_appareil_sont_donnees_a_la_session_suivante(self):
        """Le 23/11 : 127 gestes d'« Aigle Émeraude », dont une membrane URGENT, que personne n'a lus."""
        r.ecrire_json(r.ETAT / "veilleur.json", {"consigne_signee": "e", "compte": "a", "vu": 3})
        notes = [{"rang": 3, "auteur": "Aigle Émeraude", "fait": ["pose le texte « note-1 »"]},
                 {"rang": 4, "auteur": "agent claude-code", "fait": ["pose le texte « sa-reponse »"]}]
        with self.histoire(*notes, total=5), mock.patch.object(r, "ecran", lambda: None):
            r.un_tour()
        self.lancer.assert_called_once_with("a", ["note-1"])
        memoire = r.lire_json(r.ETAT / "veilleur.json", {})
        self.assertEqual((memoire["vu"], memoire["paroles"]), (5, []))

    def test_ses_notes_sont_tapees_une_fois_quand_il_a_fini_d_ecrire(self):
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        r.ecrire_json(r.ETAT / "veilleur.json", {"consigne_signee": "e", "compte": "b", "vu": 0})
        with self.histoire({"rang": 0, "auteur": "humain", "fait": ["change le texte « note-2 »"]}, total=1):
            r.un_tour()
        self.taper.assert_not_called()
        with self.histoire(total=1):
            r.un_tour()
            r.un_tour()
        self.taper.assert_called_once_with(r.raison_entendre(["note-2"]))

    # ── Le plafond des relances de Claude Code ──

    def test_le_crochet_relance_une_fois_puis_c_est_le_veilleur(self):
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b"})
        dire, session = self.crochet('{"stop_hook_active": false}')
        self.assertIn('"decision": "block"', dire.call_args.args[0])
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b"})
        dire, session = self.crochet('{"stop_hook_active": true}')
        dire.assert_not_called()
        self.assertEqual(session.get("laissee"), "relancer")
        r.un_tour()
        r.un_tour()
        self.taper.assert_called_once_with(r.raison_continuer("continue"))

    def test_le_crochet_cede_a_sa_session(self):
        self.bail()
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b"})
        dire, session = self.crochet()
        self.assertIn(json.dumps(r.raison_passation("retrait"), ensure_ascii=False), dire.call_args.args[0])
        self.assertEqual(session.get("passation_pourquoi"), "retrait")

    def test_le_crochet_au_seuil_ne_passe_la_main_que_si_quelqu_un_peut_la_prendre(self):
        self.jauge("b", 92, 60)
        self.jauge("a", 100, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b"})
        dire, session = self.crochet()
        self.assertIn(json.dumps(r.raison_continuer("continue"), ensure_ascii=False), dire.call_args.args[0])
        self.assertNotIn("passation_demandee", session)
        self.jauge("a", 10, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b"})
        dire, session = self.crochet()
        self.assertIn(json.dumps(r.RAISON_PASSATION, ensure_ascii=False), dire.call_args.args[0])

    def test_au_seuil_sans_autre_compte_elle_va_jusqu_au_mur_prevenue_une_fois(self):
        self.jauge("b", 92, 60)
        self.jauge("a", 100, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        r.un_tour()
        r.un_tour()
        self.taper.assert_called_once_with(r.raison_prevenir(self.maintenant + 3600))
        self.tmux.assert_not_called()


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
            with mock.patch.object(m, "DEPOT", depot):
                self.assertIn("· 1 pas encore sur GitHub", r.travail(0))
                # La preuve à l'envers : l'ancienne loi comptait A1, déjà parti.
                self.assertEqual(m.git("rev-list", "--count", "origin/main..main"), "2")


class Jeton(unittest.TestCase):
    def test_present_absent_et_a_renouveler_sans_jamais_sa_valeur(self):
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / "github.md"
            with mock.patch.object(m, "JETON", f):
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
            with mock.patch.object(m, "EXTENSIONS", Path(d)):
                self.assertIn("2.1.330", r.claude_binaire())


class Mains(unittest.TestCase):
    """Qui signe sa consigne : sa main, de ce Glucose ou d'un autre appareil — jamais une IA."""

    def consigne(self, auteur):
        return {"gestes": [{"auteur": auteur, "date": "23/11/2026 11:17",
                            "fait": ["change le texte « relais-consigne »"]}]}

    def test_retouchee_depuis_son_autre_appareil_elle_vaut(self):
        memoire = {"consigne_signee": "ancienne"}
        self.assertTrue(r.signee({"empreinte": "neuve"}, memoire, self.consigne("Aigle Émeraude")))
        self.assertEqual(memoire["consigne_par"], "Aigle Émeraude, le 23/11/2026 11:17")

    def test_une_ia_ne_signe_jamais_meme_depuis_l_autre_appareil(self):
        for ia in ("agent claude-code", "agent le terminal", "agent Claude (claude.ai)", "l'IA de Glucose"):
            self.assertFalse(r.signee({"empreinte": "x"}, {}, self.consigne(ia)), ia)
        self.assertFalse(r.humaine(None))

    def test_ses_paroles_dans_l_ordre_une_fois_sans_le_battement(self):
        h = {"gestes": [
            {"auteur": "humain", "fait": ["pose le texte « note-1 »"]},
            {"auteur": "agent claude-code", "fait": ["change le texte « note-2 »"]},
            {"auteur": "Aigle Émeraude", "fait": ["change le texte « note-1 »", "pose la membrane « urgent »"]},
            {"auteur": "humain", "fait": ["change le texte « relais-battement »", "déplace des nœuds"]},
            {"auteur": "humain", "fait": ["change le texte « relais-reponse »"]},
            {"auteur": "Aigle Émeraude", "fait": ["retire le texte « vieille »"]},
        ]}
        self.assertEqual(r.ses_paroles(h, ["vieille", "avant"]), ["avant", "note-1", "urgent"])

    def test_un_dossier_n_est_pas_un_identifiant(self):
        # Le 23/11, l'histoire disait « pose le dossier « Dossier » » : la session recevait ids=Dossier.
        h = {"gestes": [
            {"auteur": "Aigle Émeraude", "fait": ["pose le dossier « Dossier »", "pose ou retire un tableau"]},
            {"auteur": "Aigle Émeraude", "fait": ["change le dossier « Dossier »"]},
            {"auteur": "Aigle Émeraude", "fait": ["pose le texte « text-f5fn-357 »"]},
        ]}
        self.assertEqual(r.ses_paroles(h), ["text-f5fn-357"])
        self.assertIn("ouvre", r.depart("a", ["text-f5fn-357"]))

    def test_la_session_lit_ses_notes_avant_tout(self):
        self.assertIn("ids=note-1 ids=urgent", r.depart("a", ["note-1", "urgent"]))
        self.assertIn("avant toute autre tâche", r.depart("a", ["note-1"]))
        self.assertNotIn("Depuis la dernière session", r.depart("a"))
        self.assertIn(r.MODE_EMPLOI, r.depart("a"))

    def test_une_relance_du_crochet_passe_par_le_veilleur(self):
        self.assertEqual(r.decider({}, "x", False, True, None, 0, suite=True), ("laisser", "relancer"))
        self.assertEqual(r.decider({}, "x", False, True, None, 0, suite=False)[0], "continuer")


class Histoire(unittest.TestCase):
    def test_tous_les_gestes_neufs_meme_au_dela_de_deux_cents(self):
        demandes = []

        def glucose(*a, **k):
            demandes.append(a)
            return {"total": 500, "gestes": []}
        with mock.patch.object(r, "glucose", glucose):
            self.assertEqual(r.gestes_neufs({"vu": 213})["total"], 500)
            self.assertEqual(demandes, [("histoire", "nombre=1"), ("histoire", "nombre=287")])
            demandes.clear()
            self.assertEqual(r.gestes_neufs({}), {"gestes": [], "total": 500}, "la première fois, rien d'ancien")
            self.assertEqual(demandes, [("histoire", "nombre=1")])
        with mock.patch.object(r, "glucose", lambda *a, **k: None):
            self.assertIsNone(r.gestes_neufs({"vu": 1}))


class Parole(unittest.TestCase):
    """Le crochet « UserPromptSubmit » d'une session à lui, et le verrou du veilleur."""

    def setUp(self):
        self.dossier = tempfile.TemporaryDirectory()
        self.depot = Path(self.dossier.name) / "depot"
        (self.depot / "crates").mkdir(parents=True)
        self.retouches = [mock.patch.object(r, "ETAT", Path(self.dossier.name) / "etat"),
                          mock.patch.object(r, "DEPOT", self.depot),
                          mock.patch.object(r, "session_tmux", lambda: None)]
        for retouche in self.retouches:
            retouche.start()

    def tearDown(self):
        for retouche in reversed(self.retouches):
            retouche.stop()
        self.dossier.cleanup()

    def parler(self, cwd, relais="0", prompt="x"):
        entree = json.dumps({"session_id": "s1", "transcript_path": None, "cwd": str(cwd), "prompt": prompt})
        with mock.patch("sys.stdin") as stdin, mock.patch("builtins.print") as dire, \
                mock.patch.dict("os.environ", {"RELAIS": relais}):
            stdin.read.return_value = entree
            r.parole()
        return dire, r.lire_json(r.ETAT / "baux.json", {})

    def test_une_session_a_lui_dans_le_depot_prend_la_main(self):
        dire, baux = self.parler(self.depot / "crates")
        self.assertIn("s1", baux)
        self.assertIsNotNone(r.main_humaine(time.time()))
        dire.assert_not_called()

    def test_ailleurs_ou_session_du_relais_rien(self):
        self.assertEqual(self.parler(Path(self.dossier.name))[1], {})
        self.assertEqual(self.parler(self.depot, relais="1")[1], {})

    def test_une_notification_de_tache_ne_prend_pas_la_main(self):
        """Le 23/11, 14 h 41 : réveillée par sa propre commande, la vérification de Claude la prenait."""
        notification = "<task-notification>\n<task-id>bxicvo6es</task-id>\n<status>completed</status>"
        self.assertEqual(self.parler(self.depot, prompt=notification)[1], {})
        self.assertIn("s1", self.parler(self.depot, prompt="<b>ma</b> question")[1])

    def test_elle_apprend_qu_une_session_du_relais_vit_encore(self):
        with mock.patch.object(r, "session_tmux", lambda: "relais-a"):
            dire, _ = self.parler(self.depot)
        dire.assert_called_once_with(r.AVERTIR)

    def test_jamais_une_seconde_session(self):
        tmux = mock.Mock()
        with mock.patch.object(r, "session_tmux", lambda: "relais-a"), mock.patch.object(r, "tmux", tmux):
            self.assertFalse(r.lancer("b"))
        tmux.assert_not_called()

    def test_un_seul_veilleur(self):
        """Le verrou tenu ici, un autre processus ne le prend pas ; rendu, il le prend."""
        essai = ("import fcntl,sys; f=open(sys.argv[1],'w')\n"
                 "try: fcntl.flock(f, fcntl.LOCK_EX|fcntl.LOCK_NB)\nexcept BlockingIOError: sys.exit(1)")
        verrou = r.un_seul_veilleur()
        chemin = str(r.ETAT / "veilleur.lock")
        self.assertEqual(subprocess.run([sys.executable, "-c", essai, chemin]).returncode, 1)
        verrou.close()
        self.assertEqual(subprocess.run([sys.executable, "-c", essai, chemin]).returncode, 0)


class Reponse(unittest.TestCase):
    """Sa réponse dans relais-reponse est tapée dans la session — écrite de ce Glucose ou d'ailleurs."""

    def setUp(self):
        self.dossier = tempfile.TemporaryDirectory()
        self.reponse = {"id": r.REPONSE, "empreinte": "r2", "texte": f"{r.INVITE_REPONSE} oui, vas-y"}
        self.taper = mock.Mock()
        self.retouches = [mock.patch.object(r, "ETAT", Path(self.dossier.name)),
                          mock.patch.object(r, "ecran", lambda: "l'écran"),
                          mock.patch.object(r, "noeud", lambda ident: self.reponse),
                          mock.patch.object(r, "taper", self.taper),
                          mock.patch.object(r, "ecrire", mock.Mock()), mock.patch.object(r, "journal", mock.Mock())]
        for retouche in self.retouches:
            retouche.start()

    def tearDown(self):
        for retouche in reversed(self.retouches):
            retouche.stop()
        self.dossier.cleanup()

    def test_sa_reponse_depuis_l_autre_appareil_est_tapee_une_fois_son_geste_lu(self):
        memoire = {"reponse_vue": "r1"}
        r.surveiller(memoire, {"gestes": []}, 0)
        self.taper.assert_not_called()
        self.assertEqual(memoire["reponse_vue"], "r1", "son geste pas encore lu : on attend le tour suivant")
        geste = {"auteur": "Aigle Émeraude", "fait": [f"change le texte « {r.REPONSE} »"]}
        r.surveiller(memoire, {"gestes": [geste]}, 0)
        self.taper.assert_called_once_with("oui, vas-y")

    def test_ecrite_par_une_ia_elle_n_est_pas_tapee(self):
        memoire = {"reponse_vue": "r1"}
        geste = {"auteur": "agent claude-code", "fait": [f"change le texte « {r.REPONSE} »"]}
        r.surveiller(memoire, {"gestes": [geste]}, 0)
        self.taper.assert_not_called()
        self.assertEqual(memoire["reponse_vue"], "r2")


class Heure(unittest.TestCase):
    def test_le_jour_quand_ce_n_est_pas_aujourd_hui(self):
        self.assertNotIn("/", r.heure(time.time()))
        self.assertIn("/", r.heure(time.time() + 3 * 86400))


if __name__ == "__main__":
    unittest.main()
