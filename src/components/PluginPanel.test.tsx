import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import PluginPanel from "./PluginPanel";

// RÉGRESSION vécue : le panneau n'affichait « Aucun plugin installé » et le
// bouton « Lancer » restait gris À VIE, sans jamais dire pourquoi — le moteur
// vivait dans un binaire externe que personne n'avait. Ces tests verrouillent
// les trois garanties qui en découlent :
//   1. un moteur est TOUJOURS proposé (l'intégré) ;
//   2. un bouton grisé dit toujours ce qui lui manque ;
//   3. l'installation d'un plugin externe est une option de fin de parcours,
//      pas une étape obligatoire au milieu.
//
// En jsdom, `invoke` n'existe pas : `listPlugins` échoue exactement comme sur
// une machine sans dossier `plugins/` — soit le cas de l'utilisateur.

describe("PluginPanel", () => {
  // `globals: false` dans la config Vitest : le nettoyage entre tests n'est pas
  // automatique, sans lui les rendus s'empilent et tout devient ambigu.
  afterEach(cleanup);

  it("propose le moteur intégré même quand aucun plugin externe n'est installé", async () => {
    render(<PluginPanel docked />);
    await waitFor(() => {
      expect(screen.getByText("Cours magistral (intégré)")).toBeInTheDocument();
    });
    expect(screen.queryByText(/Aucun plugin installé/)).toBeNull();
  });

  it("dit ce qui manque au lieu de laisser un bouton gris muet", async () => {
    render(<PluginPanel docked />);
    const launch = await screen.findByRole("button", { name: /Lancer/ }, { timeout: 3000 });
    // Sans texte source, le lancement reste impossible — mais la raison est écrite.
    expect(launch).toBeDisabled();
    await waitFor(() => {
      expect(screen.getByText(/Choisis un texte source/)).toBeInTheDocument();
    }, { timeout: 3000 });
  });

  it("laisse l'installation d'un plugin externe APRÈS le bouton Lancer", async () => {
    const { container } = render(<PluginPanel docked />);
    const launch = await screen.findByRole("button", { name: /Lancer/ }, { timeout: 3000 });
    const install = screen.getByRole("button", { name: /Installer un plugin externe/ });
    // Node.DOCUMENT_POSITION_FOLLOWING = 4 : « install » vient après « Lancer ».
    expect(launch.compareDocumentPosition(install) & 4).toBeTruthy();
    expect(container.textContent).toContain("Avancé");
  });

  it("expose les réglages du moteur intégré (recette auto-câblée)", async () => {
    render(<PluginPanel docked />);
    await waitFor(() => {
      expect(screen.getByText("Densité")).toBeInTheDocument();
    });
    expect(screen.getByText("Disposition")).toBeInTheDocument();
  });
});
