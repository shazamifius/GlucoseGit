"""Les épreuves du relais : ce qu'il décide, sans Glucose ni Claude Code (python3 -m unittest)."""
import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

import machine as m
import relais as r

LANCER = r.lancer  # le vrai, gardé avant que le banc du veilleur ne le remplace

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
        self.assertTrue(r.en_pause("stop\n📜 **Ta consigne au relais**"))
        self.assertFalse(r.en_pause("continue\nstop plus bas ne compte pas"))
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
        self.poser("a", 89, 89, 5000)
        self.assertIsNone(r.garder({"compte": "a"}, 1000))
        self.assertIsNone(r.garder({"compte": "b"}, 1000))

    def test_la_semaine_a_97_passe_la_main_comme_les_cinq_heures(self):
        """Sa règle du 23/11 (« on vise toujours 90 % »), la semaine portée à 97 % le 25/11 (`relais-reserve`)."""
        self.poser("a", 10, 96, 5000)
        self.assertIsNone(r.garder({"compte": "a"}, 1000))
        self.poser("a", 10, 97, 5000)
        self.assertEqual(r.garder({"compte": "a"}, 1000), "passation")

    def test_la_memoire_du_modele_a_90_demande_une_conversation_neuve(self):
        """Sa note du 23/11 (text-kpll-351) : on ne regardait jamais la limite du modèle."""
        self.poser("a", 10, 10, 5000)
        session = {"compte": "a", "depuis": 900}
        r.ecrire_json(r.ETAT / "contexte.json", {"lu": 950, "pct": 91})
        self.assertEqual(r.garder(session, 1000), "contexte")
        r.ecrire_json(r.ETAT / "contexte.json", {"lu": 950, "pct": 89})
        self.assertIsNone(r.garder(session, 1000))
        r.ecrire_json(r.ETAT / "contexte.json", {"lu": 800, "pct": 99})
        self.assertIsNone(r.garder(session, 1000), "la lecture d'une conversation d'avant")

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

    def test_faute_de_mieux_le_travail_attend_la_remise_ses_notes_vont_jusqu_au_bord(self):
        """Fiche 167 : le 24/11 à 19 h 37, b poussé jusqu'au mur faute d'autre compte, coupé en plein
        travail — il a dû l'arrêter lui-même. Ce qu'une fenêtre ne dépense pas reste à la semaine."""
        self.poser("a", 91, 77, 5000)
        self.poser("b", 4, 100, 7000)
        self.assertEqual(r.choisir("b", 1000), (None, 5000))
        self.assertEqual(r.choisir("b", 1000, notes=True), ("a", None))
        self.poser("a", 99, 77, 5000)
        self.assertEqual(r.choisir("b", 1000, notes=True), (None, 5000), "le bord : le mur moins une passation")

    def test_au_seuil_la_passation_seulement_si_un_autre_compte_peut_reprendre(self):
        self.poser("a", 92, 10, 5000)
        self.poser("b", 4, 100, 7000)
        self.assertEqual(r.garder({"compte": "a"}, 1000), "repos", "jamais au mur")
        self.assertIsNone(r.garder({"compte": "a", "passation_demandee": 900}, 1000))
        self.assertIsNone(r.garder({"compte": "a", "notes": True}, 1000), "ses notes, jusqu'au bord")
        self.poser("a", 99, 10, 5000)
        self.assertEqual(r.garder({"compte": "a", "notes": True}, 1000), "repos")
        self.poser("a", 92, 10, 5000)
        self.poser("b", 4, 10, 7000)
        self.assertEqual(r.garder({"compte": "a"}, 1000), "passation")

    def test_la_fin_de_sa_semaine_est_gardee_pour_ses_notes(self):
        """RESERVE-1 (fiche 163) : le 25/11, a à 17 % de sa semaine en trois heures, b plein jusqu'au
        30/11 — brûlée au travail, la fin de la semaine de a laissait le relais sourd quatre jours."""
        jour = 86400
        self.poser("a", 10, 98, 3 * jour)  # sa semaine se remet dans trois jours
        self.poser("b", 4, 100, 2 * jour)
        self.assertEqual(r.choisir("a", 1000), (None, 2 * jour + 1000))
        self.assertEqual(r.choisir("a", 1000, notes=True), ("a", None))
        self.poser("b", 4, 99, 2 * jour)  # b garde 1 % : ses notes vont à a, qui a le plus de marge
        self.assertEqual(r.choisir("b", 1000, notes=True), ("a", None))
        self.poser("b", 4, 100, 2 * jour)
        self.assertEqual(r.garder({"compte": "a"}, 1000), "reserve")
        self.assertIsNone(r.garder({"compte": "a", "notes": True}, 1000))
        # Dans la dernière fenêtre avant sa remise, elle se dépense au travail : sinon, perdue.
        self.poser("b", 4, 100, 4 * jour)
        self.assertEqual(r.choisir("a", 1000), (None, 3 * jour + 1000 - r.FENETRE))
        fin = 3 * jour + 1000 - r.FENETRE + 60
        self.assertEqual(r.choisir("a", fin), ("a", None))
        self.assertIsNone(r.garder({"compte": "a"}, fin), "la réserve se dépense")
        self.poser("a", 10, 99, 3 * jour)
        self.assertEqual(r.garder({"compte": "a"}, fin), "repos", "jusqu'au bord, jamais au mur")
        # Seules ses cinq heures au seuil : rien n'est gardé, elles se remettent — on les attend.
        self.poser("a", 92, 10, 3 * jour)
        self.assertEqual(r.choisir("a", 1000), (None, 3 * jour))

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

    def test_la_barre_d_etat_ecrit_l_histoire_quand_une_jauge_bouge(self):
        histoire = r.ETAT / "jauges-histoire.jsonl"
        for pct in (60, 60, 61):
            entree = {"session_id": "s1", "rate_limits": {
                "five_hour": {"used_percentage": pct, "resets_at": 99},
                "seven_day": {"used_percentage": 20}}}
            with mock.patch("sys.stdin") as stdin, mock.patch("builtins.print"), \
                    mock.patch.dict("os.environ", {"RELAIS_COMPTE": "b"}):
                stdin.read.return_value = json.dumps(entree)
                r.jauges()
        lignes = [json.loads(l) for l in histoire.read_text().splitlines()]
        self.assertEqual([(l["five_hour"], l["seven_day"]) for l in lignes], [(60, 20), (61, 20)])
        self.assertEqual((lignes[0]["compte"], lignes[0]["session"], lignes[0]["remise"]), ("b", "s1", 99))

    def test_la_barre_d_etat_d_une_session_du_relais_garde_sa_memoire_du_modele(self):
        entree = {"context_window": {"used_percentage": 42}}
        for relais, ecrit in (("0", False), ("1", True)):
            with mock.patch("sys.stdin") as stdin, mock.patch("builtins.print"), \
                    mock.patch.dict("os.environ", {"RELAIS_COMPTE": "b", "RELAIS": relais}):
                stdin.read.return_value = json.dumps(entree)
                r.jauges()
            self.assertEqual(r.lire_json(r.ETAT / "contexte.json", {}).get("pct"), 42 if ecrit else None)


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
        # Jamais le vrai serveur d'Anthropic dans une épreuve : muet, la jauge gardée reste (fiche 182).
        self.retouches.append(mock.patch.object(r.serveur, "demander", return_value=(None, "épreuve")))
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
        self.lancer.assert_called_once_with("a", [], None, [], depuis=None)
        self.assertEqual(r.lire_json(r.ETAT / "session.json", {}).get("laissee"), "compte à sa limite")

    def test_un_compte_cru_plein_est_redemande_au_serveur_a_chaque_tour(self):
        """Le 27/11 (fiche 182) : a gardé à 99 % de sa semaine, 2 % au serveur — six heures sans
        personne. Le tour du veilleur le redemande, et le relance."""
        self.jauge("b", 4, 100)
        self.jauge("a", 10, 99)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        remis = {"five_hour": {"used_percentage": 12, "resets_at": self.maintenant + 3600},
                 "seven_day": {"used_percentage": 2, "resets_at": self.maintenant + 9000}}
        reponses = {r.COMPTES["b"]: (None, "épreuve")}
        with mock.patch.object(r.serveur, "demander", side_effect=lambda d: reponses.get(d, (remis, None))):
            r.un_tour()
        self.lancer.assert_called_once_with("a", [], None, [], depuis=None)

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
        self.lancer.assert_called_once_with("a", [], None, [], depuis=None)

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
            self.lancer.assert_not_called()  # il écrit encore : un tour sans geste de lui d'abord
            r.un_tour()
        self.lancer.assert_called_once_with("a", ["note-1"], None, [], depuis=3)
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
        self.taper.assert_called_once_with(r.raison_entendre(["note-2"], 0))

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

    def endormir(self, laissee, immobile):
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": 0, **({"laissee": laissee} if laissee else {})})
        r.ecrire_json(r.ETAT / "veilleur.json", {"consigne_signee": "e", "compte": "b", "paroles": ["text-1"],
                                                 "ecran_depuis": self.maintenant - immobile})
        r.un_tour()

    def test_endormie_elle_est_close_jamais_reveillee(self):
        """Sa note du 25/11 (img-utzs-2460) : ne jamais réveiller une session qui attend depuis plus de
        50 minutes — son cache perdu, elle relirait tout au prix fort. Ses notes vont à la neuve."""
        self.endormir("à vide", r.ENDORMIE)
        self.tmux.assert_any_call("kill-session", "-t", "relais-b")
        self.taper.assert_not_called()
        self.lancer.assert_called_once_with("b", ["text-1"], None, [], depuis=None)
        self.assertEqual(r.lire_json(r.ETAT / "session.json", {}).get("laissee"), r.FERMEE_ENDORMIE)

    def test_assoupie_elle_vit(self):
        """Arrêtée depuis moins de 50 minutes, son cache tient : ses notes lui sont tapées."""
        self.endormir("à vide", r.ENDORMIE - 60)
        self.tmux.assert_not_called()
        self.lancer.assert_not_called()

    def test_une_neuve_n_herite_pas_l_immobilite_de_la_session_d_avant(self):
        """Le 28/11 à 15 h 58 : lancée à 15 h 57, close une minute après, « figée depuis plus de 50
        minutes » — l'écran gardé était celui de la session d'avant, immobile depuis des heures."""
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        r.ecrire_json(r.ETAT / "veilleur.json", {"consigne_signee": "e", "compte": "b",
                                                 "ecran_depuis": self.maintenant - 3 * r.ENDORMIE})
        r.un_tour()
        self.tmux.assert_not_called()
        self.lancer.assert_not_called()

    def test_figee_elle_est_close_comme_l_endormie(self):
        """Fiche 186 : le 27/11 à 21 h 31, figée sur un refus d'Anthropic sans que « Stop » parte (rien
        de `laissee`), une session est restée ouverte dix-huit heures. Cette épreuve disait le
        contraire (« au travail, un écran immobile est figé, pas endormi ») : elle gardait le défaut.
        Un écran immobile depuis 50 minutes ne travaille plus : close, et une neuve repart."""
        self.endormir(None, r.ENDORMIE - 60)
        self.tmux.assert_not_called()
        self.endormir(None, r.ENDORMIE)
        self.tmux.assert_any_call("kill-session", "-t", "relais-b")
        self.lancer.assert_called_once_with("b", ["text-1"], None, [], depuis=None)
        self.assertEqual(r.lire_json(r.ETAT / "session.json", {}).get("laissee"), r.FERMEE_FIGEE)

    def test_le_crochet_cede_a_sa_session(self):
        self.bail()
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b"})
        dire, session = self.crochet()
        self.assertIn(json.dumps(r.raison_passation("retrait"), ensure_ascii=False), dire.call_args.args[0])
        self.assertEqual(session.get("passation_pourquoi"), "retrait")

    def test_le_crochet_au_seuil_demande_la_passation_meme_sans_personne_pour_reprendre(self):
        """Fiche 167 : le 24/11 à 19 h 37, le crochet répétait « Ne t'arrête pas » à 90 % — b au mur."""
        self.jauge("b", 92, 60)
        self.jauge("a", 100, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b"})
        dire, session = self.crochet()
        self.assertIn(json.dumps(r.raison_passation("repos"), ensure_ascii=False), dire.call_args.args[0])
        self.assertEqual(session.get("passation_pourquoi"), "repos")
        self.jauge("a", 10, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b"})
        dire, session = self.crochet()
        self.assertIn(json.dumps(r.RAISON_PASSATION, ensure_ascii=False), dire.call_args.args[0])

    def test_la_memoire_du_modele_a_90_le_veilleur_et_le_crochet_demandent_la_passation(self):
        """Sa note du 23/11 : à 90 % du contexte, une conversation neuve — même avec les deux comptes libres."""
        self.jauge("b", 10, 10)
        self.jauge("a", 10, 10)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        r.ecrire_json(r.ETAT / "contexte.json", {"lu": self.maintenant - 5, "pct": 93})
        r.un_tour()
        self.taper.assert_called_once_with(r.raison_passation("contexte"))
        self.assertEqual(r.lire_json(r.ETAT / "session.json", {}).get("passation_pourquoi"), "contexte")
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        dire, session = self.crochet()
        self.assertIn(json.dumps(r.raison_passation("contexte"), ensure_ascii=False), dire.call_args.args[0])
        self.assertEqual(session.get("passation_pourquoi"), "contexte")

    def test_le_veilleur_lance_et_relance_avec_ses_textes(self):
        """Sa note du 23/11 : ce qu'il récrit dans son Glucose est ce que la session reçoit."""
        siens = {r.TEXTE_DEPART: "Mon départ.", r.TEXTE_RELANCE: "Ma relance."}
        with mock.patch.object(r, "ses_textes", lambda memoire, histoire: siens), \
                mock.patch.object(r, "ecran", lambda: None):
            r.un_tour()
        self.lancer.assert_called_once_with("b", [], "Mon départ.", [], depuis=None)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant, "laissee": "relancer"})
        with mock.patch.object(r, "ses_textes", lambda memoire, histoire: siens):
            r.un_tour()
        self.taper.assert_called_once_with(r.raison_continuer("continue", "Ma relance."))

    def test_au_seuil_sans_autre_compte_la_passation_puis_l_attente_jamais_le_mur(self):
        """Fiche 167 : le 24/11 à 19 h 37, poussée jusqu'au mur, la session a été coupée en plein travail."""
        self.jauge("b", 92, 60)
        self.jauge("a", 100, 60)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        r.un_tour()
        r.un_tour()
        self.taper.assert_called_once_with(r.raison_passation("repos"))
        self.tmux.assert_not_called()

    def test_une_conversation_neuve_a_sa_demande(self):
        """RENOUVELER-1 (fiche 163 § 6) : la session la demande, le veilleur et le crochet font la passation."""
        self.jauge("b", 10, 10)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 60})
        with mock.patch.dict("os.environ", {"RELAIS": "1"}):
            self.assertEqual(r.renouveler(), 0)
        with mock.patch.dict("os.environ", {"RELAIS": ""}), mock.patch("builtins.print"):
            self.assertEqual(r.renouveler(), 1)
        r.un_tour()
        self.taper.assert_called_once_with(r.raison_passation("renouveler"))
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "renouveler": 1})
        dire, session = self.crochet()
        self.assertIn(json.dumps(r.raison_passation("renouveler"), ensure_ascii=False), dire.call_args.args[0])

    def test_la_fin_de_sa_semaine_ne_se_lance_que_pour_ses_notes(self):
        """RESERVE-1 (fiche 163) : sans note de lui, le veilleur attend et le dit ; une note, et une
        session se lance pour lui répondre, puis écrit sa passation à la fin de son tour."""
        for compte in "ab":
            r.ecrire_json(r.ETAT / f"jauges-{compte}.json", {"rate_limits": {
                "five_hour": {"used_percentage": 10, "resets_at": self.maintenant + 3600},
                "seven_day": {"used_percentage": 98 if compte == "b" else 100,
                              "resets_at": self.maintenant + 3 * 86400}}})
        with mock.patch.object(r, "ecran", lambda: None):
            r.un_tour()
            self.lancer.assert_not_called()
            self.assertIn("gardée pour tes notes", r.lire_json(r.ETAT / "veilleur.json", {})["dit"])
            with mock.patch.object(r, "gestes_neufs", lambda memoire: {"total": 3, "gestes": [
                    {"auteur": "humain", "fait": ["pose le texte « sa-note »"]}]}):
                r.un_tour()
            # Il écrit encore : la session attend qu'il ait fini (le 25/11, 8 h 15 et 8 h 18).
            self.lancer.assert_not_called()
            self.assertIn("tu écris", r.lire_json(r.ETAT / "veilleur.json", {})["dit"])
            r.un_tour()
        self.lancer.assert_called_once_with("b", ["sa-note"], None, [], depuis=None)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "notes": True, "depuis": self.maintenant})
        dire, session = self.crochet()
        self.assertIn(json.dumps(r.raison_passation("reserve"), ensure_ascii=False), dire.call_args.args[0])
        self.assertEqual(session.get("passation_pourquoi"), "reserve")

    def test_figee_le_battement_ne_dit_plus_au_travail(self):
        """Le 23/11, de 16 h à 17 h 48 : figée au démarrage, le ❤️ disait « au travail » (fiche 134)."""
        self.jauge("b", 10, 10)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 7000})
        r.un_tour()
        self.assertIn("au travail", r.lire_json(r.ETAT / "veilleur.json", {})["dit"])
        memoire = r.lire_json(r.ETAT / "veilleur.json", {})
        memoire.update(bloque_signale=True, ecran_depuis=self.maintenant - 2200)
        r.ecrire_json(r.ETAT / "veilleur.json", memoire)
        r.un_tour()
        dit = r.lire_json(r.ETAT / "veilleur.json", {})["dit"]
        self.assertIn("figée", dit)
        self.assertNotIn("au travail", dit)
        self.assertIn(r.heure(self.maintenant - 2200), dit)


    # ── Un tour refusé par Anthropic (fiche 186) ──

    def refuser(self, compte="b", erreur="billing_error", fois=1):
        """Le crochet « StopFailure » de la session, `fois` fois de suite."""
        for _ in range(fois):
            with mock.patch("sys.stdin") as stdin, \
                    mock.patch.dict("os.environ", {"RELAIS": "1", "RELAIS_COMPTE": compte}):
                stdin.read.return_value = json.dumps({"error": erreur, "error_details": "payment is past due"})
                r.echec()
        return r.refus()[compte]

    def test_le_crochet_note_le_refus_et_espace_les_essais_jusqu_a_la_vie_du_cache(self):
        """Un essai refusé ne coûte rien ; Claude Code a déjà réessayé dans le tour. 5, 10, 20, 40
        minutes, puis jamais plus loin que la vie du cache : une panne longue est réessayée toutes les
        50 minutes, et le compte repart au plus tard 50 minutes après qu'elle est réparée."""
        delais = []
        for fois in range(1, 8):
            refuse = self.refuser()
            self.assertEqual(refuse["fois"], fois)
            delais.append(round((refuse["jusqu_a"] - refuse["t"]) / 60))
        self.assertEqual(delais, [5, 10, 20, 40, 50, 50, 50])
        self.assertEqual(refuse["erreur"], "billing_error")
        with mock.patch("sys.stdin") as stdin, mock.patch.dict("os.environ", {"RELAIS": "0"}):
            stdin.read.return_value = "{}"
            r.echec()  # une session à lui : son erreur est la sienne
        self.assertEqual(r.refus()["b"]["fois"], 7)

    def test_un_tour_reussi_efface_le_refus(self):
        self.refuser(fois=3)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b"})
        _, session = self.crochet()
        self.assertNotIn("b", r.refus())
        self.assertIn("tour", session)

    def test_refusee_elle_attend_son_essai_puis_la_relance_est_tapee_une_fois(self):
        """Le 27/11 à 21 h 31 : refusée, la session est restée dix-huit heures sans un essai."""
        self.jauge("a", 10, 100)  # a plein : personne pour reprendre
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 120})
        refuse = self.refuser()
        with mock.patch.object(r.time, "time", lambda: refuse["jusqu_a"] - 60):
            r.un_tour()
        self.taper.assert_not_called()
        self.tmux.assert_not_called()
        dit = r.lire_json(r.ETAT / "veilleur.json", {})["dit"]
        self.assertIn("⛔", dit)
        self.assertIn("abonnement refusé", dit)
        with mock.patch.object(r.time, "time", lambda: refuse["jusqu_a"] + 1):
            r.un_tour()
            r.un_tour()
        self.taper.assert_called_once_with(r.raison_reessayer(refuse, "continue"))
        self.tmux.assert_not_called()
        encore = self.refuser()  # refusée de nouveau : un nouvel essai, plus loin
        with mock.patch.object(r.time, "time", lambda: encore["jusqu_a"] + 1):
            r.un_tour()
        self.assertEqual(self.taper.call_count, 2)

    def test_refusee_un_autre_compte_libre_la_reprend_dans_le_meme_tour(self):
        self.jauge("a", 10, 10)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 120})
        self.refuser()
        r.un_tour()
        self.tmux.assert_any_call("kill-session", "-t", "relais-b")
        self.lancer.assert_called_once_with("a", [], None, [], depuis=None)
        self.assertEqual(r.lire_json(r.ETAT / "session.json", {}).get("laissee"), r.FERMEE_REFUSEE)

    def test_refusee_son_cache_mort_elle_est_close_pas_reveillee(self):
        """Réveiller une conversation après la vie de son cache la fait relire entière (25/11)."""
        self.jauge("a", 10, 100)
        r.ecrire_json(r.ETAT / "session.json", {"compte": "b", "depuis": self.maintenant - 2 * r.ENDORMIE,
                                                "tour": self.maintenant - 2 * r.ENDORMIE})
        refuse = self.refuser(fois=6)
        with mock.patch.object(r.time, "time", lambda: refuse["jusqu_a"] + 1):
            r.un_tour()
        self.taper.assert_not_called()
        self.tmux.assert_any_call("kill-session", "-t", "relais-b")
        self.assertIn(r.FERMEE_REFUSEE, r.FERMEES_SANS_PASSATION)

    def test_un_compte_refuse_n_est_pas_choisi_et_le_battement_dit_pourquoi(self):
        self.jauge("a", 10, 100)
        self.jauge("b", 10, 10)
        self.refuser("b")
        with mock.patch.object(r, "ecran", lambda: None):
            r.un_tour()
        self.lancer.assert_not_called()
        dit = r.lire_json(r.ETAT / "veilleur.json", {})["dit"]
        self.assertTrue(dit.startswith("⛔ compte b refusé par Anthropic"), dit)
        self.assertFalse(r.ailleurs("a", self.maintenant)[0])

    # ── La session lancée pour ses notes, au-delà du seuil (fiche 186) ──

    def test_lancee_pour_ses_notes_au_dela_des_cinq_heures_elle_lui_repond_d_abord(self):
        """Le 27/11 de 19 h 33 à 19 h 38 : a à 91 % de ses 5 h, b plein ; trois sessions lancées pour
        ses notes, chacune close une minute après, sa passation demandée avant toute réponse."""
        self.jauge("a", 91, 13)
        self.jauge("b", 0, 100)
        with mock.patch.object(r, "ses_paroles", lambda h, avant: ["sa-note"]), \
                mock.patch.object(r, "ecran", lambda: None), mock.patch.object(r, "lancer", LANCER), \
                mock.patch.object(r, "session_tmux", lambda: None), \
                mock.patch.object(r, "claude_binaire", lambda: "claude"), \
                mock.patch.object(r, "tmux", mock.Mock(
                    return_value=subprocess.CompletedProcess([], 0, "", ""))) as tmux:
            r.un_tour()
        self.assertIn(r.POURQUOI["reserve"], tmux.call_args.args[-1])
        session = r.lire_json(r.ETAT / "session.json", {})
        self.assertEqual((session["compte"], session.get("notes")), ("a", True))
        r.un_tour()  # la session vit : pas de passation en plein tour
        self.taper.assert_not_called()
        with mock.patch("sys.stdin") as stdin, mock.patch("builtins.print") as dire, \
                mock.patch.dict("os.environ", {"RELAIS": "1", "RELAIS_COMPTE": "a"}):
            stdin.read.return_value = "{}"
            r.fin_de_tour()  # sa réponse faite, la passation à la fin de son tour
        self.assertIn(json.dumps(r.raison_passation("reserve"), ensure_ascii=False), dire.call_args.args[0])

    def test_ses_notes_se_lisent_par_quoi_de_neuf_depuis_son_premier_geste(self):
        """Sa note du 27/11 (text-kix1-2714) : la liste « ids=… » s'allongeait sans fin."""
        r.ecrire_json(r.ETAT / "veilleur.json", {"consigne_signee": "e", "compte": "a", "vu": 40})
        gestes = [{"rang": 40, "auteur": "agent le terminal", "fait": ["change le texte « relais-battement »"]},
                  {"rang": 41, "auteur": "Aigle Émeraude", "fait": ["pose le texte « note-1 »"]},
                  {"rang": 42, "auteur": "humain", "fait": ["pose le texte « note-2 »"]}]
        with self.histoire(*gestes, total=43), mock.patch.object(r, "ecran", lambda: None):
            r.un_tour()
        self.assertEqual(r.lire_json(r.ETAT / "veilleur.json", {})["paroles_depuis"], 41)
        with self.histoire(total=43), mock.patch.object(r, "ecran", lambda: None):
            r.un_tour()
        self.lancer.assert_called_once_with("a", ["note-1", "note-2"], None, [], depuis=41)
        self.assertNotIn("paroles_depuis", r.lire_json(r.ETAT / "veilleur.json", {}))
        depart = r.depart("a", ["note-1", "note-2"], depuis=41)
        self.assertIn("glucose_quoi_de_neuf depuis=41", depart)
        self.assertNotIn("ids=note-1", depart)


class Lancement(unittest.TestCase):
    def lancer(self, *arguments):
        tmux = mock.Mock(return_value=subprocess.CompletedProcess([], 0, "", ""))
        with tempfile.TemporaryDirectory() as dossier, mock.patch.object(r, "ETAT", Path(dossier)), \
                mock.patch.object(r, "session_tmux", lambda: None), mock.patch.object(r, "tmux", tmux), \
                mock.patch.object(r, "claude_binaire", lambda: "claude"), \
                mock.patch.object(r, "journal", mock.Mock()):
            self.assertTrue(r.lancer(*arguments))
        commande = tmux.call_args.args
        return commande[commande.index("claude"):]

    def test_apres_une_endormie_la_neuve_sait_que_la_passation_n_est_pas_recrite(self):
        tmux = mock.Mock(return_value=subprocess.CompletedProcess([], 0, "", ""))
        with tempfile.TemporaryDirectory() as dossier, mock.patch.object(r, "ETAT", Path(dossier)), \
                mock.patch.object(r, "session_tmux", lambda: None), mock.patch.object(r, "tmux", tmux), \
                mock.patch.object(r, "claude_binaire", lambda: "claude"), \
                mock.patch.object(r, "journal", mock.Mock()):
            r.ecrire_json(r.ETAT / "session.json", {"compte": "a", "laissee": r.FERMEE_ENDORMIE})
            r.lancer("a")
            self.assertIn("git status", tmux.call_args.args[-1])
            r.lancer("a")
            self.assertNotIn("git status", tmux.call_args.args[-1])

    def test_en_bypass_jamais_en_mode_auto(self):
        """Sa réponse du 23/11 : « on fait tout en mode bypass, jamais de mode auto » — figé deux fois."""
        claude = self.lancer("b")
        mode = claude.index("--permission-mode")
        self.assertEqual(claude[mode + 1], "bypassPermissions")
        self.assertIn("--allow-dangerously-skip-permissions", claude)

    def test_sans_les_outils_qu_il_ne_sert_jamais(self):
        """Chaque requête relit le mode d'emploi de chaque outil : ceux qu'une session du relais ne sert
        jamais (Artifact, le plus long, et la question en menu) ne lui sont pas donnés (fiche 147)."""
        claude = self.lancer("b")
        sans = claude[claude.index("--disallowedTools") + 1].split(",")
        self.assertIn("Artifact", sans)
        self.assertIn("AskUserQuestion", sans)
        self.assertNotIn("Agent", sans)

    def test_sur_la_fin_de_sa_semaine_elle_ne_fait_que_lui_repondre(self):
        """RESERVE-1 (fiche 163) : le premier message le dit, et la session est marquée pour ses notes."""
        self.assertNotIn(r.POURQUOI["reserve"], self.lancer("b", ["sa-note"])[-1])
        with mock.patch.object(r, "gardee", lambda compte, maintenant: 1.0), \
                mock.patch.object(r, "ecrire_json") as ecrit:
            self.assertIn(r.POURQUOI["reserve"], self.lancer("b", ["sa-note"])[-1])
        self.assertTrue(ecrit.call_args.args[1].get("notes"))

    def test_sans_les_connecteurs_de_claude_ai(self):
        """Gmail, Drive, Hugging Face… : ~13 000 jetons relus à chaque réponse, jamais servis (fiche 163 § 5)."""
        tmux = mock.Mock(return_value=subprocess.CompletedProcess([], 0, "", ""))
        with tempfile.TemporaryDirectory() as dossier, mock.patch.object(r, "ETAT", Path(dossier)), \
                mock.patch.object(r, "session_tmux", lambda: None), mock.patch.object(r, "tmux", tmux), \
                mock.patch.object(r, "claude_binaire", lambda: "claude"), \
                mock.patch.object(r, "journal", mock.Mock()):
            self.assertTrue(r.lancer("b"))
        commande = tmux.call_args.args
        self.assertIn("ENABLE_CLAUDEAI_MCP_SERVERS=false", commande[:commande.index("claude")])

    def test_son_texte_sans_point_ne_se_colle_pas_a_la_suite(self):
        """Son texte finit sur « …en prioriter », sans point : « …en prioriter Depuis… » se collait."""
        depart = r.depart("a", ["note-1"], "points a faire en prioriter")
        self.assertIn("en prioriter\n\nDepuis la dernière session", depart)

    def test_son_texte_de_depart(self):
        self.assertIn("Mon départ.", self.lancer("b", [], "Mon départ.")[-1])
        self.assertIn(r.TEXTES[r.TEXTE_DEPART], self.lancer("b", [], None)[-1])

    def test_sans_la_question_de_chrome_au_demarrage(self):
        """Le compte b n'avait jamais répondu à « Claude in Chrome » : sans --no-chrome, Claude Code la
        pose avant le premier message, et personne n'est là (le 23/11, 1 h 48 figée, fiche 134)."""
        tmux = mock.Mock(return_value=subprocess.CompletedProcess([], 0, "", ""))
        with tempfile.TemporaryDirectory() as dossier, mock.patch.object(r, "ETAT", Path(dossier)), \
                mock.patch.object(r, "session_tmux", lambda: None), mock.patch.object(r, "tmux", tmux), \
                mock.patch.object(r, "claude_binaire", lambda: "claude"), \
                mock.patch.object(r, "journal", mock.Mock()):
            self.assertTrue(r.lancer("b"))
        commande = tmux.call_args.args
        self.assertIn("--no-chrome", commande[commande.index("claude"):])


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

    def test_ses_textes_ne_valent_que_de_sa_main(self):
        """Sa note du 23/11 : voir et récrire ce que le relais envoie. Une IA qui les change ne vaut rien."""
        lus = {"noeuds": [{"id": r.TEXTE_DEPART, "empreinte": "e1", "texte": " Mon départ. "},
                          {"id": r.TEXTE_RELANCE, "empreinte": "e2", "texte": "Une IA l'a écrit."}]}
        histoire = {"gestes": [{"auteur": "Aigle Émeraude", "fait": [f"change le texte « {r.TEXTE_DEPART} »"]},
                               {"auteur": "agent claude-code", "fait": [f"change le texte « {r.TEXTE_RELANCE} »"]}]}
        memoire = {}
        with mock.patch.object(r, "glucose", lambda *a, **k: lus):
            self.assertEqual(r.ses_textes(memoire, histoire), {r.TEXTE_DEPART: "Mon départ."})
            self.assertEqual(r.ses_textes(memoire, {"gestes": []}), {r.TEXTE_DEPART: "Mon départ."},
                             "vue une fois, sa version reste la sienne")
        self.assertIn(r.TEXTES[r.TEXTE_RELANCE], r.raison_continuer("c"))
        self.assertTrue(r.raison_passation("contexte", "Ma passation.").endswith("Ma passation."))

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
            {"auteur": "humain", "fait": ["change le texte « relais-bloque »"]},
            {"auteur": "Aigle Émeraude", "fait": ["retire le texte « vieille »"]},
        ]}
        self.assertEqual(r.ses_paroles(h, ["vieille", "avant"]), ["avant", "note-1", "urgent"])

    def test_un_collage_n_est_pas_une_note(self):
        """Le 25/11, ses treize nœuds rangés dans l'archive, d'un seul geste, furent donnés à lire
        comme neufs. Un nœud collé qu'il retouche redevient sa parole ; une flèche seule en est une."""
        h = {"gestes": [
            {"auteur": "Aigle Émeraude", "fait": ["retire le texte « q-12 »"]},
            {"auteur": "Aigle Émeraude", "fait": ["retire le texte « vieille »", "pose la photo « img-1 »",
                                                   "pose le texte « t-1 »", "pose le texte « t-2 »"]},
            {"auteur": "Aigle Émeraude", "fait": ["change le texte « t-2 »"]},
            {"auteur": "Aigle Émeraude", "fait": ["pose la flèche « f-1 »"]},
        ]}
        self.assertEqual(r.ses_paroles(h, ["vieille"]), ["t-2", "f-1"])

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
        self.assertIn("seulement si ton travail touche au relais", r.depart("a"))

    def test_la_passation_se_remplace_sans_recit(self):
        """Sa note du 25/11 (text-bmgs-2413) : pas de récapitulatif ; elle ne s'allonge jamais."""
        texte = r.raison_passation()
        for exige in ("remplace tout le texte", "sans jamais l'allonger", "Ni ce qui est fait"):
            self.assertIn(exige, texte)
        self.assertNotIn("ajoute seulement en tête", texte)

    def test_la_conversation_neuve_vient_au_seuil_en_jetons(self):
        """Fiche 142 : à 90 % d'un million de jetons, chaque réponse relisait 900 000 jetons."""
        s = {"depuis": 100}
        self.assertFalse(r.contexte_plein({"lu": 200, "pct": 29, "taille": 1_000_000}, s))
        self.assertTrue(r.contexte_plein({"lu": 200, "pct": 30, "taille": 1_000_000}, s))
        # Une fenêtre de 200 000 jetons n'atteint jamais 300 000 : l'ancien seuil, en pourcentage.
        self.assertFalse(r.contexte_plein({"lu": 200, "pct": 89, "taille": 200_000}, s))
        self.assertTrue(r.contexte_plein({"lu": 200, "pct": 90, "taille": 200_000}, s))
        self.assertTrue(r.contexte_plein({"lu": 200, "pct": 90}, s))
        # Une lecture d'avant son lancement est celle d'une autre session.
        self.assertFalse(r.contexte_plein({"lu": 50, "pct": 95, "taille": 1_000_000}, s))

    def test_sa_consigne_designe_par_ses_fleches(self):
        """Sa note du 24/11 (text-g420-2321) : que le démarrage regarde son bloc important."""
        noeuds = [
            {"id": "f1", "sorte": "fleche", "source": "relais-consigne", "cible": "urgent"},
            {"id": "f2", "sorte": "fleche", "source": "urgent", "cible": "note-a"},
            {"id": "f3", "sorte": "fleche", "source": "urgent", "cible": "relais-consigne"},
            {"id": "f4", "sorte": "fleche", "source": "note-a", "cible": "trop-loin"},
            {"id": "f5", "sorte": "fleche", "source": "relais-consigne", "cible": "piege"},
            {"id": "f6", "sorte": "fleche", "source": "relais-consigne", "cible": "urgent"},
        ]
        gestes = [{"auteur": a, "fait": [f"pose la flèche « {i} »"]} for a, i in (
            ("Aigle Émeraude", "f1"), ("humain", "f2"), ("humain", "f3"), ("humain", "f4"),
            ("agent claude-code", "f5"), ("humain", "f6"))]
        self.assertEqual(r.ses_fleches(noeuds, gestes), ["urgent", "note-a"])
        self.assertEqual(r.ses_fleches(noeuds, []), [])
        self.assertIn("ids=urgent ids=note-a", r.depart("a", [], None, ["urgent", "note-a"]))
        self.assertNotIn("par ses flèches", r.depart("a"))

    def test_le_depart_ne_relit_pas_ce_qui_est_charge(self):
        """Ses règles et l'index des mémoires sont chargés par Claude Code : les relire coûtait des
        jetons à chaque réponse (fiche 142)."""
        self.assertNotIn("toutes tes mémoires", r.TEXTES[r.TEXTE_DEPART])
        self.assertNotIn("~/.claude/CLAUDE.md", r.TEXTES[r.TEXTE_DEPART])
        self.assertIn("aucun compte rendu", r.REGLES)

    def test_le_depart_mene_a_son_bloc_urgent(self):
        """Son texte de démarrage, corrigé à sa demande (relais-rep-texte-depart, 27/11) : il mène à
        son bloc URGENT IMPORTANT et à ses flèches, qui ne partent pas de sa consigne (`ses_fleches`
        ne les suit pas)."""
        self.assertIn(f"ids={r.URGENT} fleches=true", r.depart("a"))

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


class Alerte(unittest.TestCase):
    """L'alerte d'une session figée (relais-bloque) : sa réponse s'écrit dans l'alerte même, sous
    « ✍️ Ta réponse : », de ce Glucose ou d'ailleurs, et est tapée une fois ; l'alerte se lève quand
    l'écran repart (fiche 186 : il avait retiré relais-reponse, l'alerte renvoyait à un nœud mort)."""

    def setUp(self):
        self.dossier = tempfile.TemporaryDirectory()
        self.bloque = {"id": r.BLOQUE, "empreinte": "b2",
                       "texte": f"🆘 l'écran\n\n{r.INVITE_REPONSE} oui, vas-y"}
        self.taper, self.ecrire = mock.Mock(), mock.Mock()
        self.vu = "l'écran"
        self.retouches = [mock.patch.object(r, "ETAT", Path(self.dossier.name)),
                          mock.patch.object(r, "ecran", lambda: self.vu),
                          mock.patch.object(r, "noeud", lambda ident: self.bloque if ident == r.BLOQUE else None),
                          mock.patch.object(r, "taper", self.taper),
                          mock.patch.object(r, "ecrire", self.ecrire), mock.patch.object(r, "journal", mock.Mock())]
        for retouche in self.retouches:
            retouche.start()

    def tearDown(self):
        for retouche in reversed(self.retouches):
            retouche.stop()
        self.dossier.cleanup()

    def signalee(self):
        import hashlib
        return {"bloque_signale": True, "bloque_vu": "b1", "ecran_depuis": 0,
                "ecran": hashlib.sha256(self.vu.encode()).hexdigest()}

    def test_sa_reponse_depuis_l_autre_appareil_est_tapee_une_fois_son_geste_lu(self):
        memoire = self.signalee()
        r.surveiller(memoire, {"gestes": []}, 0)
        self.taper.assert_not_called()
        self.assertEqual(memoire["bloque_vu"], "b1", "son geste pas encore lu : on attend le tour suivant")
        geste = {"auteur": "Aigle Émeraude", "fait": [f"change le texte « {r.BLOQUE} »"]}
        r.surveiller(memoire, {"gestes": [geste]}, 0)
        self.taper.assert_called_once_with("oui, vas-y")
        self.assertIn("Tapé dans la session", self.ecrire.call_args.args[1])
        r.surveiller(memoire, {"gestes": [geste]}, 0)
        self.taper.assert_called_once()

    def test_ecrite_par_une_ia_elle_n_est_pas_tapee(self):
        memoire = self.signalee()
        geste = {"auteur": "agent claude-code", "fait": [f"change le texte « {r.BLOQUE} »"]}
        r.surveiller(memoire, {"gestes": [geste]}, 0)
        self.taper.assert_not_called()
        self.assertEqual(memoire["bloque_vu"], "b2")

    def test_l_alerte_nait_dans_la_bulle_du_relais_et_se_leve_quand_l_ecran_repart(self):
        """Le 27/11 à 22 h 07, posée sans bulle, elle est tombée hors de tout ; et son 🆘 criait
        encore le lendemain, la session close."""
        self.bloque = None
        memoire = {}
        r.surveiller(memoire, {"gestes": []}, 0)
        r.surveiller(memoire, {"gestes": []}, r.SILENCE)
        texte, place = self.ecrire.call_args.args[1], self.ecrire.call_args.kwargs
        self.assertTrue(texte.startswith("🆘") and texte.rstrip().endswith(r.INVITE_REPONSE))
        self.assertEqual(place.get("dans"), "relais")
        self.assertTrue(memoire["bloque_signale"])
        self.vu = "l'écran a bougé"
        r.surveiller(memoire, {"gestes": []}, r.SILENCE + 60)
        self.assertTrue(self.ecrire.call_args.args[1].startswith("✅"))
        self.assertFalse(memoire["bloque_signale"])

    def test_refusee_par_anthropic_pas_d_alerte(self):
        """Le ❤️ dit déjà le refus et l'heure du prochain essai : l'alerte ne le répète pas."""
        memoire = {}
        r.surveiller(memoire, {"gestes": []}, 0, refusee=True)
        r.surveiller(memoire, {"gestes": []}, r.SILENCE, refusee=True)
        self.ecrire.assert_not_called()


class Vue(unittest.TestCase):
    """**La vue de VS Code** (`voir.sh`) : la session du relais qui vit, quel que soit le compte de ce
    VS Code — le 24/11, celui du compte a est resté vide toute la nuit pendant que le b travaillait."""
    VOIR = Path(__file__).resolve().parent / "voir.sh"

    def voir(self, sessions, dit=None):
        """Lance la vue devant un faux tmux, qui connaît ces sessions et dit à laquelle on s'attache."""
        with tempfile.TemporaryDirectory() as d:
            faux = Path(d) / "tmux"
            faux.write_text('#!/usr/bin/env bash\ncase "$1" in\n  ls) printf "%s" "$FAUX_SESSIONS" ;;\n'
                            '  attach) echo "attache $3" ;;\nesac\n')
            faux.chmod(0o755)
            if dit:
                (Path(d) / "veilleur.json").write_text(json.dumps({"dit": dit, "battu": time.time()}))
            env = {**os.environ, "PATH": f"{d}:{os.environ['PATH']}", "RELAIS_COMPTE": "a",
                   "RELAIS_ETAT": d, "FAUX_SESSIONS": "".join(f"{s}\n" for s in sessions)}
            return subprocess.run(["bash", str(self.VOIR)], env=env, capture_output=True, text=True).stdout

    def test_le_vscode_du_compte_a_montre_la_session_du_compte_b(self):
        self.assertIn("attache relais-b", self.voir(["relais-b"]))

    def test_une_autre_session_tmux_ne_se_montre_pas(self):
        self.assertNotIn("attache", self.voir(["glucose-pilote", "mon-travail"]))

    def test_sans_session_elle_dit_pourquoi_et_jusqu_a_quand(self):
        sortie = self.voir([], "les comptes sont à leur limite : reprise vers 12:00")
        self.assertIn("reprise vers 12:00", sortie)
        self.assertNotIn("attache", sortie)


class Heure(unittest.TestCase):
    def test_le_jour_quand_ce_n_est_pas_aujourd_hui(self):
        self.assertNotIn("/", r.heure(time.time()))
        self.assertIn("/", r.heure(time.time() + 3 * 86400))


if __name__ == "__main__":
    unittest.main()
