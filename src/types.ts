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

  // Webhook fields
  webhook_url?: string;
  webhook_method?: string;
  webhook_payload?: string;
}

export interface SmartProfileMapping {
  process_name: string;
  target_page: string;
}

export interface Mapping {
  actions: Action[];
  is_toggle: boolean;
  color: number;
  on_color?: number;
  state: boolean;
  label?: string;
  current_step?: number;
  is_sequence?: boolean;
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

  // Advanced features
  smart_profiles_enabled: boolean;
  smart_profiles: SmartProfileMapping[];
  web_companion_enabled: boolean;
  web_companion_port: number;
  obs_peak_meter_enabled: boolean;
  obs_peak_meter_source?: string;
  obs_peak_meter_column?: number;
  ripple_effect_enabled: boolean;

  // Spotify & Discord Integration
  media_progress_enabled: boolean;
  media_progress_row: number;
  media_control_note: number;
  discord_mute_note: number;
  discord_deafen_note: number;
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
  { value: "page", label: "Ordner: Öffnen / Seite wechseln", description: "Öffnet einen Ordner (wechselt das aktive Layout) auf eine andere konfigurierte Seite." },
  { value: "page_back", label: "Ordner: Zurück", description: "Navigiert zurück zur vorherigen Seite in deiner Ordner-Historie." },
  { value: "webhook", label: "Webhook senden", description: "Sendet einen HTTP-Request (GET/POST) an eine beliebige URL." },
  { value: "audio_panic", label: "Alle Sounds stoppen", description: "Stoppt sofort alle aktuell spielenden Soundboard-Dateien." },
  { value: "discord_mute", label: "Discord: Stumm toggeln", description: "Schaltet dein Mikrofon in Discord stumm/aktiv und färbt die Taste Rot." },
  { value: "discord_deafen", label: "Discord: Taub toggeln", description: "Schaltet den Ton in Discord taub/aktiv und färbt die Taste Gelb." },
  { value: "media_play_pause", label: "Medien: Play/Pause", description: "Startet oder pausiert deine Musik. Taste leuchtet Grün/Rot." },
  { value: "media_next", label: "Medien: Nächster Titel", description: "Springt zum nächsten Titel in deiner Wiedergabeliste." },
  { value: "media_prev", label: "Medien: Vorheriger Titel", description: "Springt zum vorherigen Titel in deiner Wiedergabeliste." },
  { value: "mouse_click", label: "Mausklick simulieren", description: "Simuliert einen Klick mit der linken, rechten, mittleren oder Doppelklick-Maustaste." },
  { value: "mouse_move", label: "Mauszeiger bewegen", description: "Bewegt den Mauszeiger relativ zur aktuellen Position (X,Y) in Pixeln." },
  { value: "mouse_scroll", label: "Mausrad scrollen", description: "Scrollt das Mausrad vertikal (z. B. positive Zahl für nach oben, negative Zahl für nach unten)." },
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
  audio_panic: { label: "Info", placeholder: "Stoppt alle Sounds sofort auf Knopfdruck." },
  text: { label: "Text", placeholder: "Nachricht oder Macro-Text" },
  system: { label: "Systembefehl", placeholder: "cmd /c ..." },
  page: { label: "Zielseite (Ordner)", placeholder: "Name der Seite" },
  page_back: { label: "Info", placeholder: "Ordner verlassen. Kehrt zur vorherigen Seite zurück." },
  webhook: { label: "Webhook URL (z. B. http://localhost:8080/api)", placeholder: "http://..." },
  discord_mute: { label: "Info", placeholder: "Toggelt Discord-Mute. Keine zusätzlichen Felder nötig." },
  discord_deafen: { label: "Info", placeholder: "Toggelt Discord-Deafen. Keine zusätzlichen Felder nötig." },
  media_play_pause: { label: "Info", placeholder: "Toggelt Wiedergabe. Keine zusätzlichen Felder nötig." },
  media_next: { label: "Info", placeholder: "Nächster Titel. Keine zusätzlichen Felder nötig." },
  media_prev: { label: "Info", placeholder: "Vorheriger Titel. Keine zusätzlichen Felder nötig." },
  mouse_click: { label: "Maustaste (Left / Right / Middle / DoubleLeft)", placeholder: "Left" },
  mouse_move: { label: "Relative Bewegung (X, Y)", placeholder: "100,-50" },
  mouse_scroll: { label: "Scroll-Intensität (z. B. 120 oder -120)", placeholder: "120" },
};

export function createEmptyMapping(): Mapping {
  return { actions: [], is_toggle: false, color: 0, state: false };
}
