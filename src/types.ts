export interface Action {
  type: string;
  path?: string;
  url?: string;
  keys?: string[];
  delay_ms?: number;
  midi_type?: string;
  midi_note?: number;
  midi_value?: number;
  midi_channel?: number;
  midi_device?: string;
  media_key?: string;
  obs_action?: string;
  obs_target?: string;
  audio_path?: string;
  audio_volume?: number;
  text_content?: string;
  system_command?: string;
  target_page?: string;
}

export interface Mapping {
  actions: Action[];
  is_toggle: boolean;
  color: number;
  on_color?: number;
  state: boolean;
  label?: string;
}

export interface Page {
  name: string;
  mappings: Record<string, Mapping>;
}

export interface AppConfig {
  device_name: string;
  output_device_name: string;
  pages: Page[];
  active_page: string;
  fader_mappings: Record<string, { type: string; target?: string }>;
  obs: { host: string; port: number; password?: string; auto_connect: boolean };
  config_url: string;
}

export const padRows = [7, 6, 5, 4, 3, 2, 1, 0];
export const padCols = [0, 1, 2, 3, 4, 5, 6, 7];
export const trackKeys = Array.from({ length: 8 }, (_, i) => 100 + i);
export const sideKeys = Array.from({ length: 8 }, (_, i) => 112 + i);

export const ACTION_OPTIONS = [
  { value: "app", label: "App starten", description: "Startet eine Anwendung oder öffnet eine Datei. Tipp: Nutze den kompletten Pfad zur .exe Datei." },
  { value: "url", label: "Webseite öffnen", description: "Öffnet eine URL in deinem Standard-Browser. Beispiel: https://google.de" },
  { value: "hotkey", label: "Hotkey", description: "Simuliert Tastenkombinationen wie 'CTRL+C' oder 'WIN+D'. Funktioniert systemweit." },
  { value: "wait", label: "Warten", description: "Pausiert die Aktionskette für X Millisekunden (1000ms = 1 Sekunde)." },
  { value: "midi", label: "MIDI senden", description: "Sendet Note- oder Control-Change Befehle an ein ausgewähltes MIDI-Gerät." },
  { value: "media", label: "Medientaste", description: "Steuert Wiedergabe, Pause oder Lautstärke deines Betriebssystems." },
  { value: "obs", label: "OBS steuern", description: "Schaltet Szenen um, toggelt Mute oder startet/stoppt deinen Stream in OBS." },
  { value: "audio", label: "Sound abspielen", description: "Spielt eine lokale Audiodatei (MP3/WAV) ab. Ideal für eigene Soundboards." },
  { value: "text", label: "Text senden", description: "Schreibt automatisch einen Text, als hättest du ihn gerade getippt." },
  { value: "system", label: "Systembefehl", description: "Führt einen Befehl in der Windows-CMD aus. Nur für Fortgeschrittene!" },
  { value: "page", label: "Seite wechseln", description: "Wechselt das aktive Layout deiner Pads auf eine andere konfigurierte Seite." },
] as const;

export const ACTION_FIELD_MAP: Record<string, { label: string; placeholder: string }> = {
  app: { label: "Programm / Pfad", placeholder: "C:\\Program Files\\..." },
  url: { label: "URL", placeholder: "https://..." },
  hotkey: { label: "Tastenkombination", placeholder: "CTRL+ALT+L" },
  wait: { label: "Dauer in ms", placeholder: "500" },
  midi: { label: "MIDI Typ", placeholder: "note_on / cc / note_off" },
  media: { label: "Media-Key", placeholder: "play_pause / next / previous" },
  obs: { label: "OBS Ziel", placeholder: "scene / source / filter" },
  obs_vol: { label: "Quelle & Volume (Quelle|Volume%)", placeholder: "Mikrofon|80" },
  obs_toggle: { label: "Szene & Quelle (Szene|Quelle)", placeholder: "Gaming|Overlay" },
  obs_filter: { label: "Quelle & Filter (Quelle|Filter)", placeholder: "Mikrofon|Rauschunterdrückung" },
  obs_visible: { label: "Szene & Quelle (Szene|Quelle|1/0)", placeholder: "Gaming|Overlay|1" },
  obs_replay: { label: "Modus (toggle/save)", placeholder: "toggle" },
  audio: { label: "Audio Datei", placeholder: "C:\\Sounds\\..." },
  text: { label: "Text", placeholder: "Nachricht oder Macro-Text" },
  system: { label: "Systembefehl", placeholder: "cmd /c ..." },
  page: { label: "Zielseite", placeholder: "Page name" },
};

export function createEmptyMapping(): Mapping {
  return { actions: [], is_toggle: false, color: 0, state: false };
}
