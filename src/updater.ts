import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";

export interface UpdateStatus {
  available: boolean;
  version?: string;
  body?: string;
  downloadUrl?: string;
  filename?: string;
}

const GITHUB_REPO = "Malionaro/apc-mini-rs-controll";

function isNewer(v1: string, v2: string): boolean {
  const parts1 = v1.split(".").map((p) => parseInt(p, 10));
  const parts2 = v2.split(".").map((p) => parseInt(p, 10));
  for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
    const a = isNaN(parts1[i]) ? 0 : parts1[i];
    const b = isNaN(parts2[i]) ? 0 : parts2[i];
    if (a > b) return true;
    if (a < b) return false;
  }
  return false;
}

export async function checkForUpdate(): Promise<UpdateStatus> {
  try {
    const currentVersion = await getVersion();

    const response = await fetch(
      `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`
    );
    if (!response.ok) throw new Error("GitHub API nicht erreichbar");

    const data = await response.json();
    const latestVersion = (data.tag_name as string).replace(/^v/i, "");

    if (!isNewer(latestVersion, currentVersion)) {
      return { available: false };
    }

    const assets: { name: string; browser_download_url: string }[] =
      data.assets ?? [];
    const installer = assets.find(
      (a) => a.name.endsWith(".msi") || a.name.endsWith(".exe")
    );

    return {
      available: true,
      version: latestVersion,
      body: data.body ?? undefined,
      downloadUrl: installer?.browser_download_url,
      filename: installer?.name,
    };
  } catch (error) {
    console.error("Update-Prüfung fehlgeschlagen:", error);
    return { available: false };
  }
}

export async function installUpdate(
  downloadUrl: string,
  filename: string,
  onProgress?: (percent: number) => void
): Promise<void> {
  await invoke<void>("download_and_install_update", {
    downloadUrl,
    filename,
    onProgress: onProgress ? true : false,
  });
}
