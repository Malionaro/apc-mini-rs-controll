# APC Mini Mk2 Controller (apc-mini-rs)

Ein leistungsstarkes und hochgradig anpassbares Tool zur Steuerung von Programmen, Hotkeys, Sounds, MIDI-Befehlen und OBS Studio direkt über dein **Akai APC Mini Mk2**. 

Das Tool nutzt die RGB-Matrix und Fader des APC Mini, um es in ein vielseitiges Control Deck (ähnlich einem Stream Deck) mit dynamischem Feedback zu verwandeln.

---

## 🚀 Features

### 🕹️ Pad- & Fader-Mapping
- **RGB-Pads**: Konfiguriere jedes der 64 Hauptpads sowie die seitlichen und unteren Funktionstasten individuell.
- **Mehrere Seiten (Layouts)**: Erstelle unbegrenzt viele Seiten und wechsle zwischen ihnen direkt über ein zugewiesenes Pad.
- **Fader-Zuweisung**: Map die Fader deines APC Mini, um z. B. die Windows-Systemlautstärke oder OBS-Lautstärken stufenlos zu regeln.
- **Auto-Select**: Wenn aktiviert, wählt das Tool im Inspector-Interface automatisch das Pad aus, das du gerade physisch auf dem Controller drückst.

### 🎥 Vollständige OBS Studio Integration
Steuere dein OBS direkt über die RGB-Pads mit Echtzeit-Statusanzeige und 4 dedizierten Unterkategorien:
- 🎬 **Szenen**: Szenenwechsel (Live & Preview), Übergänge (Transitions) ausführen und Studio-Modus umschalten.
- 🔊 **Audio**: Stummschalten (Mute toggeln) von Audioquellen und präzises Setzen der Lautstärke.
- 🖼️ **Quellen**: Sichtbarkeit (Auge-Icon) von Quellen in Szenen toggeln oder explizit an/aus schalten sowie Filter umschalten.
- 📡 **Output**: Stream starten/stoppen, Aufnahme starten/stoppen sowie Replay-Buffer steuern und Highlights speichern.

### ⚡ Vielseitige Aktionen & Ketten (Macros)
Jedes Pad kann eine Kette aus mehreren nacheinander folgenden Aktionen ausführen:
- 🚀 **App starten**: Startet jede beliebige Anwendung oder Datei per Pfadangabe.
- 🌐 **Webseite öffnen**: Öffnet URLs in deinem Standardbrowser.
- ⌨️ **Hotkey**: Simuliert Tastatur-Shortcuts (z. B. `CTRL+C`, `WIN+D`) systemweit.
- ⏱️ **Warten**: Fügt zeitliche Verzögerungen (Delays) in Millisekunden in deine Aktionskette ein.
- 🎹 **MIDI senden**: Sendet Note-On/Off und Control-Change (CC) Signale an andere MIDI-Geräte.
- 🔹 **Medientasten**: Play, Pause, Nächster/Vorheriger Track und Systemlautstärke steuern.
- 🔊 **Sound abspielen**: Nutze das APC Mini als vollwertiges Soundboard für MP3- und WAV-Dateien mit eigener Lautstärkensteuerung.
- ✍️ **Text senden**: Tippt vordefinierte Texte/Makros wie eine Tastatur ein.
- 💻 **Systembefehl**: Führt komplexe CMD-/PowerShell-Befehle im Hintergrund aus.

---

## 🛠️ Technologie-Stack

- **Frontend**: React + TypeScript + Vite + Vanilla CSS (modernes Glassmorphism-Design mit HSL-Farbpaletten und Micro-Animations)
- **Backend**: Rust mit Tauri v2 (für performante, native MIDI-Interaktion, Systembefehle, Audio-Wiedergabe und OBS-WebSocket-Verbindung)

---

## 📦 Installation & Start

### Voraussetzungen
1. **Rust & Cargo**: Zum Kompilieren des Backends.
2. **Node.js & npm**: Für die Frontend-Abhängigkeiten.

### Entwicklungsumgebung starten
1. Klone das Repository.
2. Installiere die Node-Pakete:
   ```bash
   npm install
   ```
3. Starte die Tauri-Anwendung im Entwicklungsmodus:
   ```bash
   npm run tauri dev
   ```

---

## 🎨 Design-Philosophie
Das UI des Managers ist auf maximale Übersicht und Ästhetik ausgelegt:
- **Dark Mode**: Ein augenschonendes, dunkles Theme mit harmonischen Farbakzenten.
- **Live-Feedback**: Das virtuelle APC-Grid im UI spiegelt das physische Leuchten der RGB-Pads in Echtzeit wider.
- **Echtzeit-Verbindung**: Eine dynamische Status-Pill zeigt sofort an, ob die Verbindung zu OBS Studio oder dem MIDI-Controller online ist.
