import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { homeDir } from "@tauri-apps/api/path";

// dataUrl must be captured BEFORE calling this (WebGL buffer is cleared after each frame)
export async function exportCanvasPng(
  dataUrl: string,
  projectName: string,
): Promise<void> {
  let defaultPath: string;
  try {
    const home = await homeDir();
    defaultPath = `${home}/${projectName}.png`;
  } catch (_) {
    defaultPath = `${projectName}.png`;
  }

  const path = await saveDialog({
    defaultPath,
    filters: [{ name: "Image PNG", extensions: ["png"] }],
  });
  if (!path) return;

  const base64Data = dataUrl.split(",")[1];
  await invoke("write_binary_file", { path, base64Data });
}
