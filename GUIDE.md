# REST Client per Zed — Guida completa

## Struttura del progetto

```
rest/
├── extension.toml          # manifest dell'estensione Zed
├── Cargo.toml              # workspace Rust
├── languages/
│   └── http/
│       ├── config.toml     # associazione file .rest / .http
│       └── highlights.scm  # syntax highlighting
└── crates/
    └── rest-runner/        # binario CLI che esegue le richieste
        ├── Cargo.toml
        └── src/
            ├── main.rs
            ├── parser.rs   # parsing del file .rest
            ├── executor.rs # client HTTP (reqwest)
            ├── env.rs      # caricamento variabili d'ambiente
            └── output.rs   # output colorato nel terminale
```

---

## 1. Prerequisiti

### Installa Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Segui le istruzioni a schermo (opzione 1 — default). Al termine, riapri il terminale oppure esegui:
```bash
source ~/.cargo/env
```
Verifica che funzioni:
```bash
rustc --version   # es. rustc 1.78.0
cargo --version   # es. cargo 1.78.0
```

---

## 2. Build e installazione del CLI

Il CLI `rest-runner` è il binario che Zed chiamerà per eseguire le richieste HTTP.

```bash
cd ~/Code/zed-plugin/rest
cargo install --path runner
```

Questo compila il progetto e copia `rest-runner` in `~/.cargo/bin/`, che è già nel tuo PATH.

Verifica:
```bash
rest-runner --version
rest-runner --help
```

---

## 3. Carica l'estensione in Zed (dev mode)

1. Apri **Zed**
2. Vai su **Extensions** (icona puzzle in basso a sinistra, oppure `Cmd+Shift+X`)
3. Clicca su **Install Dev Extension…**
4. Seleziona la cartella `~/Code/zed-plugin/rest`
5. Zed scarica automaticamente la grammatica TreeSitter da GitHub e compila il supporto al linguaggio

Da questo momento, tutti i file `.rest` e `.http` vengono riconosciuti con syntax highlighting.

---

## 4. Aggiungi il task "Run Request"

Apri (o crea) il file `~/.config/zed/tasks.json` e aggiungi:

```json
[
  {
    "label": "REST: Run Request at Cursor",
    "command": "rest-runner",
    "args": ["$ZED_FILE", "--line", "$ZED_ROW"],
    "reveal": "always"
  }
]
```

### Keybinding opzionale

In `~/.config/zed/keymap.json` aggiungi:

```json
[
  {
    "context": "Editor",
    "bindings": {
      "ctrl-alt-r": "task::Spawn"
    }
  }
]
```

Oppure, per eseguire direttamente il task REST senza passare dal menu:

```json
[
  {
    "context": "Editor && extension == rest || extension == http",
    "bindings": {
      "ctrl-alt-r": ["task::Spawn", { "task_name": "REST: Run Request at Cursor" }]
    }
  }
]
```

---

## 5. Test con un file .rest

Crea un file `test.rest` in una qualsiasi cartella e aprilo in Zed:

```http
### Ottieni un utente
GET https://jsonplaceholder.typicode.com/users/1

###

### Crea un post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "Ciao",
  "body": "Mondo",
  "userId": 1
}

###

### Test con variabili
GET {{baseUrl}}/users/{{userId}}
Authorization: Bearer {{token}}
```

Per eseguire la richiesta dove si trova il cursore:
- Palette dei comandi (`Cmd+Shift+P`) → cerca `task: spawn` → seleziona `REST: Run Request at Cursor`

Il terminale integrato si apre e mostra:

```
HTTP/1.1 200 OK

content-type: application/json; charset=utf-8
...

{
  "id": 1,
  "name": "Leanne Graham",
  ...
}

────────────────────────────────────────────────────
200 OK  42ms  1234 bytes
```

---

## 6. Variabili d'ambiente

Crea un file `.rest-client.env.json` nella stessa cartella del file `.rest`:

```json
{
  "local": {
    "baseUrl": "http://localhost:3000",
    "userId": "1",
    "token": "my-local-token"
  },
  "prod": {
    "baseUrl": "https://api.example.com",
    "userId": "42",
    "token": "prod-secret-token"
  }
}
```

Per usare un ambiente specifico, aggiungi il flag `--env`:

```json
"args": ["$ZED_FILE", "--line", "$ZED_ROW", "--env", "local"]
```

Oppure chiamalo manualmente dal terminale:

```bash
rest-runner test.rest --line 15 --env prod
rest-runner test.rest --name "Crea un post"
rest-runner test.rest --verbose   # mostra anche gli header della richiesta
```

### Variabili built-in

| Sintassi | Risultato |
|---|---|
| `{{$guid}}` | UUID v4 casuale |
| `{{$timestamp}}` | Unix timestamp in secondi |
| `{{$randomInt}}` | Numero intero casuale 0–999 |
| `{{$processEnv VAR}}` | Valore della variabile d'ambiente di sistema |

---

## 7. Release — pubblicare l'estensione

Quando l'estensione è stabile e vuoi renderla disponibile a tutti gli utenti Zed:

### 7.1 Prepara il repository

```bash
cd ~/Code/zed-plugin/rest
git init
git add .
git commit -m "feat: initial release"
```

Crea un repository pubblico su GitHub (es. `github.com/albz/zed-rest-client`) e fai push:

```bash
git remote add origin https://github.com/albz/zed-rest-client.git
git push -u origin main
```

Aggiorna `extension.toml` con il tuo repository reale:

```toml
repository = "https://github.com/albz/zed-rest-client"
```

### 7.2 Fai il fork del registry Zed

Il registry delle estensioni Zed è un repository GitHub pubblico:
```
https://github.com/zed-industries/extensions
```

Fai fork, poi clona il tuo fork:

```bash
git clone https://github.com/TUO_USERNAME/extensions
cd extensions
```

### 7.3 Aggiungi la tua estensione

Nel file `extensions.toml` del registry, aggiungi:

```toml
[rest-client]
submodule = "extensions/rest-client"
version = "0.1.0"
```

Aggiungi il tuo repo come submodule:

```bash
git submodule add https://github.com/albz/zed-rest-client extensions/rest-client
git add .
git commit -m "Add rest-client extension"
git push
```

### 7.4 Apri una Pull Request

Vai sul tuo fork su GitHub e apri una PR verso `zed-industries/extensions`. Il team Zed rivede e approva le PR solitamente in pochi giorni. Una volta mergiata, l'estensione appare nel marketplace di Zed per tutti gli utenti.

---

## 8. Aggiornamenti futuri

Per aggiornare una versione già pubblicata:

1. Bumpa la versione in `extension.toml` (es. `0.1.0` → `0.2.0`)
2. Fai commit e push del tuo repository
3. Aggiorna il submodule nel fork del registry e apri una nuova PR

---

## Troubleshooting

| Problema | Soluzione |
|---|---|
| `rest-runner: command not found` | `~/.cargo/bin` non è nel PATH — aggiungi `export PATH="$HOME/.cargo/bin:$PATH"` al tuo `.zshrc` |
| Nessun syntax highlighting | Ricarica l'estensione da dev: Extensions → tasto destro → Reload |
| La grammatica TreeSitter non si scarica | Controlla la connessione. Zed scarica da GitHub al primo caricamento |
| Errore SSL / certificati | Prova `rest-runner test.rest --verbose` nel terminale per vedere l'errore completo |
| Richiesta va in timeout | Default 30s. Puoi modificare `executor.rs` → `.timeout(Duration::from_secs(60))` |
