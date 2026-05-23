# Auto-Updater Setup

Der Auto-Updater ist bereits im Code integriert. Folge diesen Schritten um ihn zu aktivieren.

---

## 1. GitHub Secrets setzen

Gehe zu: `https://github.com/Malionaro/apc-mini-rs-controll/settings/secrets/actions`

Füge diese zwei Secrets hinzu:

| Secret Name | Wert |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Dein minisign Private Key (der komplette Inhalt der `.key` Datei) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Das Passwort des Private Keys (leer lassen wenn keins gesetzt) |

### Private Key finden / neu generieren

Falls du den Private Key nicht mehr hast, generiere ein neues Keypair:

```bash
npm run tauri signer generate -- -w ~/.tauri/apc-mini.key
```

Das gibt dir:
- **Public Key** → in `tauri.conf.json` unter `plugins.updater.pubkey` eintragen
- **Private Key** → als `TAURI_SIGNING_PRIVATE_KEY` Secret in GitHub hinterlegen

Der aktuelle Public Key in `tauri.conf.json` ist:
```
dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDZDNDNCQUNDMEZCOEZBRUUKUldUdStyZ1B6THBEYkdCU3BoMisvb2xnWmRIVzUyWHRKWEw2eHZDZDJzQWpmbWtjUE1seGh1YWwK
```
(minisign key ID: `6C43BACC0FB8FAEE`)

---

## 2. Release erstellen (= Update auslösen)

```bash
# Version in tauri.conf.json und Cargo.toml erhöhen, dann:
git tag v0.2.0
git push origin v0.2.0
```

Der GitHub Actions Workflow läuft dann automatisch:
1. Baut die App für Windows
2. Signiert die Installer mit dem Private Key
3. Erstellt einen GitHub Release mit allen Artefakten
4. Lädt `latest.json` hoch (wird vom Updater abgefragt)

---

## 3. Wie der Updater funktioniert

- Beim App-Start prüft die App automatisch `https://github.com/Malionaro/apc-mini-rs-controll/releases/latest/download/latest.json`
- Wenn eine neue Version verfügbar ist, erscheint ein Banner oben in der UI
- Der User kann "Jetzt installieren" klicken → Download + Installation + Neustart

---

## 4. Version erhöhen vor jedem Release

Beide Dateien müssen die gleiche Version haben:

**`src-tauri/tauri.conf.json`**
```json
"version": "0.2.0"
```

**`src-tauri/Cargo.toml`**
```toml
version = "0.2.0"
```
