"""Les épreuves de MAJ-1 : Claude se met à jour seul, essayé avant d'être gardé — sans réseau, sans
VS Code, sans lancer Claude (python3 -m unittest)."""
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import machine as m
import mise_a_jour as maj

PRETE = """ ▐▛███▛█   Claude Code v2.1.332
❯ Try "create a util logging.py that…"
  ⏵⏵ bypass permissions on (shift+tab to cycle)"""
CONFIANCE = """ Quick safety check: Is this a project you created or one you trust?
 ❯ No, exit
   Yes, I trust this folder
 Enter to confirm · Esc to cancel"""


class Ecran(unittest.TestCase):
    def test_une_session_a_son_invite_est_prete(self):
        self.assertTrue(maj.prete(PRETE))

    def test_une_question_au_demarrage_ne_l_est_pas(self):
        self.assertFalse(maj.prete(CONFIANCE))
        self.assertFalse(maj.prete(PRETE + "\n" + CONFIANCE))


class Veiller(unittest.TestCase):
    def jouer(self, memoire, publiee="2.1.333", avant="2.1.332", essai=(True, PRETE)):
        dit, installees = [], []
        with mock.patch.object(maj, "derniere_publiee", return_value=publiee), \
                mock.patch.object(maj, "installee", side_effect=[(avant, Path("/x")), (publiee, Path("/y"))]), \
                mock.patch.object(maj, "installer", side_effect=lambda v: installees.append(v) or True), \
                mock.patch.object(maj, "essayer", return_value=essai) as essayer:
            fait = maj.veiller(memoire, 100_000.0, dit.append, lambda _: None)
        return fait, dit, installees, essayer

    def test_une_version_neuve_s_installe_et_s_essaie(self):
        memoire = {}
        fait, dit, installees, essayer = self.jouer(memoire)
        self.assertEqual(installees, ["2.1.333"])
        essayer.assert_called_once_with(Path("/y"))
        self.assertIn("installé seul", dit[0])
        self.assertEqual(memoire["maj_refusees"], [])

    def test_une_version_qui_pose_une_question_est_refusee_et_la_precedente_revient(self):
        memoire = {}
        fait, dit, installees, _ = self.jouer(memoire, essai=(False, CONFIANCE))
        self.assertEqual(installees, ["2.1.333", "2.1.332"])
        self.assertEqual(memoire["maj_refusees"], ["2.1.333"])
        self.assertIn("refusé", dit[0])
        self.assertIn("Enter to confirm", dit[0])

    def test_une_version_refusee_ne_se_retente_pas(self):
        memoire = {"maj_refusees": ["2.1.333"]}
        self.assertEqual(self.jouer(memoire)[2], [])

    def test_rien_de_neuf_rien_ne_s_installe(self):
        self.assertEqual(self.jouer({}, publiee="2.1.332")[2], [])
        self.assertEqual(self.jouer({}, publiee="2.1.99")[2], [])  # 99 < 332 : des nombres, pas des lettres
        self.assertEqual(self.jouer({}, publiee=None)[2], [])

    def test_six_heures_entre_deux_regards(self):
        with mock.patch.object(maj, "derniere_publiee") as marche:
            maj.veiller({"maj_vue": 100_000.0 - 3600}, 100_000.0, print, print)
        marche.assert_not_called()


class Binaire(unittest.TestCase):
    def test_le_relais_lance_celle_que_vs_code_dit_installee(self):
        # Une version refusée reste dans son dossier tant que VS Code ne s'ouvre pas : le plus grand
        # dossier n'est pas la version installée.
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            for v in ("2.1.331", "2.1.333"):
                b = d / f"anthropic.claude-code-{v}-linux-x64/resources/native-binary/claude"
                b.parent.mkdir(parents=True)
                b.touch()
            (d / "extensions.json").write_text(json.dumps([{
                "identifier": {"id": "anthropic.claude-code"}, "version": "2.1.331",
                "relativeLocation": "anthropic.claude-code-2.1.331-linux-x64"}]))
            with mock.patch.object(m, "EXTENSIONS", d):
                self.assertIn("2.1.331", m.claude_binaire())
                (d / "extensions.json").unlink()
                self.assertIn("2.1.333", m.claude_binaire())  # sans registre, le plus grand


if __name__ == "__main__":
    unittest.main()
