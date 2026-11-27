"""Les épreuves de la jauge demandée au serveur (fiche 182), sans réseau : python3 -m unittest."""
import io
import json
import tempfile
import unittest
import urllib.error
from pathlib import Path
from unittest import mock

import relais as r
import serveur as s

# Sa réponse du 27/11 à 17 h 27, réduite à ce qui compte : le compte a remis à 2 % de sa semaine.
REMIS = {"five_hour": {"utilization": 12.0, "resets_at": "2026-11-27T20:30:00.397065+00:00"},
         "seven_day": {"utilization": 2.0, "resets_at": "2026-12-01T23:00:00.397086+00:00"},
         "extra_usage": None}
SEMAINE = 1796166000.0  # 02/12 00:00 à Paris, la remise de sa semaine


class Traduire(unittest.TestCase):
    def test_la_reponse_du_serveur_prend_la_forme_de_la_barre_d_etat(self):
        lu = s.traduire(REMIS)
        self.assertEqual(lu["seven_day"], {"used_percentage": 2.0, "resets_at": 1796166000.397086})
        self.assertEqual(lu["five_hour"]["used_percentage"], 12.0)
        self.assertNotIn("credits", lu)

    def test_une_fenetre_sans_remise_dit_zero(self):
        self.assertEqual(s.traduire({"five_hour": {"utilization": 0.0, "resets_at": None}})["five_hour"],
                         {"used_percentage": 0.0, "resets_at": 0.0})

    def test_des_credits_avec_un_plafond_comptent(self):
        lu = s.traduire({"extra_usage": {"is_enabled": True, "monthly_limit": 2000, "used_credits": 500,
                                         "utilization": None}})
        self.assertEqual(lu["credits"], {"used_percentage": 25.0, "resets_at": 0})
        lu = s.traduire({"extra_usage": {"is_enabled": True, "monthly_limit": 2000, "utilization": 40.0}})
        self.assertEqual(lu["credits"]["used_percentage"], 40.0)

    def test_sans_plafond_ou_eteints_les_credits_ne_comptent_pas(self):
        """Un relais qui tourne jour et nuit ne dépense pas sans borne."""
        for extra in ({"is_enabled": True, "monthly_limit": None, "used_credits": 3},
                      {"is_enabled": False, "monthly_limit": 2000, "used_credits": 3}, None):
            self.assertNotIn("credits", s.traduire({"extra_usage": extra}), extra)


class Demander(unittest.TestCase):
    def setUp(self):
        self.dossier = tempfile.TemporaryDirectory()
        self.chemin = Path(self.dossier.name)
        (self.chemin / ".credentials.json").write_text(json.dumps({"claudeAiOauth": {"accessToken": "k"}}))

    def tearDown(self):
        self.dossier.cleanup()

    def test_la_connexion_du_compte_est_portee_et_la_reponse_traduite(self):
        vu = {}

        def ouvrir(requete, timeout):
            vu["cle"] = requete.get_header("Authorization")
            return io.BytesIO(json.dumps(REMIS).encode())
        lu, pourquoi = s.demander(self.chemin, ouvrir)
        self.assertEqual((lu["seven_day"]["used_percentage"], pourquoi, vu["cle"]), (2.0, None, "Bearer k"))

    def test_une_connexion_expiree_se_dit(self):
        def ouvrir(requete, timeout):
            raise urllib.error.HTTPError(s.ADRESSE, 401, "", {}, None)
        self.assertEqual(s.demander(self.chemin, ouvrir)[0], None)
        self.assertIn("expirée", s.demander(self.chemin, ouvrir)[1])

    def test_sans_connexion_rien_n_est_demande(self):
        (self.chemin / ".credentials.json").unlink()
        self.assertEqual(s.demander(self.chemin, mock.Mock(side_effect=AssertionError)),
                         (None, "pas de connexion lisible"))


class Relire(unittest.TestCase):
    def setUp(self):
        self.dossier = tempfile.TemporaryDirectory()
        self.etat = mock.patch.object(r, "ETAT", Path(self.dossier.name))
        self.etat.start()
        self.connecte = mock.patch.object(r, "connecte", side_effect=lambda c: c == "a")
        self.connecte.start()
        self.maintenant = 1795796800.0  # 27/11, 17 h 26

    def tearDown(self):
        self.connecte.stop()
        self.etat.stop()
        self.dossier.cleanup()

    def garder(self, cinq, semaine, credits=None):
        limites = {"five_hour": {"used_percentage": cinq, "resets_at": self.maintenant + 3600},
                   "seven_day": {"used_percentage": semaine, "resets_at": SEMAINE}}
        if credits is not None:
            limites["credits"] = {"used_percentage": credits, "resets_at": 0}
        r.ecrire_json(r.ETAT / "jauges-a.json", {"lu": self.maintenant - 20000, "rate_limits": limites})

    def test_le_27_11_un_compte_remis_n_est_plus_cru_plein(self):
        """99 % gardé depuis 11 h 49, 2 % au serveur : le relais dormait six heures."""
        self.garder(2, 99)
        self.assertEqual(r.choisir("a", self.maintenant)[0], None)
        with mock.patch.object(r.serveur, "demander", return_value=(s.traduire(REMIS), None)) as demande:
            r.relire(self.maintenant)
        demande.assert_called_once()
        self.assertEqual(r.plein("a", self.maintenant), (False, None))
        self.assertEqual(r.choisir("a", self.maintenant)[0], "a")

    def test_une_jauge_sous_ses_seuils_ne_coute_rien_au_serveur(self):
        self.garder(10, 50)
        with mock.patch.object(r.serveur, "demander", side_effect=AssertionError("demandé pour rien")):
            r.relire(self.maintenant)

    def test_au_plus_une_question_par_dix_minutes(self):
        self.garder(2, 99)
        plein = (s.traduire({"seven_day": {"utilization": 100.0, "resets_at": None}}), None)
        with mock.patch.object(r.serveur, "demander", return_value=plein) as demande:
            r.relire(self.maintenant)
            r.relire(self.maintenant + s.INTERVALLE - 1)
            self.assertEqual(demande.call_count, 1)
            r.relire(self.maintenant + s.INTERVALLE)
            self.assertEqual(demande.call_count, 2)

    def test_le_serveur_muet_laisse_la_jauge_gardee_et_le_dit(self):
        self.garder(2, 99)
        with mock.patch.object(r.serveur, "demander", return_value=(None, "connexion expirée")):
            r.relire(self.maintenant)
        self.assertTrue(r.plein("a", self.maintenant)[0])
        self.assertIn("serveur : connexion expirée", r.jauge_en_mots("a", self.maintenant))

    def test_ses_credits_achetes_font_travailler_au_dela_de_l_abonnement(self):
        self.garder(10, 100, credits=40)
        self.assertEqual(r.plein("a", self.maintenant), (False, None))
        self.assertIsNone(r.gardee("a", self.maintenant), "la semaine n'est plus la fin")
        self.assertEqual(r.choisir("a", self.maintenant)[0], "a")

    def test_des_credits_a_leur_seuil_rendent_le_compte_a_sa_semaine(self):
        self.garder(10, 100, credits=97)
        self.assertEqual(r.plein("a", self.maintenant), (True, SEMAINE))

    def test_sans_credits_l_abonnement_au_bout_reste_plein(self):
        self.garder(10, 100)
        self.assertEqual(r.plein("a", self.maintenant), (True, SEMAINE))

    def test_la_barre_d_etat_garde_les_credits_que_le_serveur_a_dits(self):
        self.garder(10, 100, credits=40)
        barre = {"rate_limits": {"five_hour": {"used_percentage": 11, "resets_at": self.maintenant + 3600},
                                 "seven_day": {"used_percentage": 100, "resets_at": SEMAINE}}}
        with mock.patch("sys.stdin", io.StringIO(json.dumps(barre))), mock.patch("builtins.print"), \
                mock.patch.dict("os.environ", {"RELAIS_COMPTE": "a"}):
            r.jauges()
        self.assertEqual(r.charge("a", self.maintenant)["credits"], (40.0, 0.0))
        self.assertEqual(r.charge("a", self.maintenant)["five_hour"][0], 11.0)


class Battement(unittest.TestCase):
    def test_l_heure_seule_ne_change_pas_l_etat(self):
        """Le 27/11 : le battement récrit chaque minute pendant qu'une session à lui travaillait."""
        retrait = "en retrait : une session de Claude à toi travaille (son dernier signe : {})"
        self.assertTrue(r.meme_etat(retrait.format("17:52"), retrait.format("17:53")))
        self.assertTrue(r.meme_etat(retrait.format("9:05"), retrait.format("10:41")))
        self.assertFalse(r.meme_etat(retrait.format("17:52"), "session lancée sur le compte a"))
        self.assertFalse(r.meme_etat("session lancée sur le compte a", "session lancée sur le compte b"))
        self.assertFalse(r.meme_etat("x", None))


if __name__ == "__main__":
    unittest.main()
