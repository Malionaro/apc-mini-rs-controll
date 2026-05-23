import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export interface UpdateStatus {
  available: boolean;
  version?: string;
  body?: string;
}

export async function checkForUpdate(): Promise<UpdateStatus> {
  try {
    const update = await check();
    if (update) {
      return {
        available: true,
        version: update.version,
        body: update.body ?? undefined,
      };
    }
    return { available: false };
  } catch (error) {
    console.error("Update check failed:", error);
    return { available: false };
  }
}

export async function installUpdate(
  onProgress?: (percent: number) => void
): Promise<void> {
  const update = await check();
  if (!update) return;

  let downloaded = 0;
  let contentLength = 0;

  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        contentLength = event.data.contentLength ?? 0;
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        if (contentLength > 0 && onProgress) {
          onProgress(Math.round((downloaded / contentLength) * 100));
        }
        break;
      case "Finished":
        break;
    }
  });

  await relaunch();
}
