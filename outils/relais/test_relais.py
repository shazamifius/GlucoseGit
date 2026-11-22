"""Les épreuves du relais : ce qu'il décide, sans Glucose ni Claude Code (python3 -m unittest)."""
import json
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

    def test_apres_la_passation_on_relaie(self):
        self.assertEqual(r.decider({"passation_demandee": 1}, "x", False, True, True, 0), ("laisser", "relayer"))

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


if __name__ == "__main__":
    unittest.main()
