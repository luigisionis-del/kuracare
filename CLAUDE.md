# Kuracare — regole per Claude Code

Kuracare è un'app per prendersi cura di sé e per rendere appagante il prendersi cura di sé.
Non è uno strumento di sorveglianza né un registro clinico. Ogni scelta di design si misura con
una domanda: fa vivere la cura di sé come un piacere o come un dovere?

Fase attuale: **prototipo personale** (un utente, dati solo sul dispositivo), pronto a un futuro
uso di ricerca (EMA) con dati standardizzati e versionati.

## Come lavorare

1. Prima pianificare, poi scrivere codice. Si lavora per tappe; a fine tappa: commit, riepilogo
   (cosa funziona, come provarlo sul telefono, cosa resta aperto) e attesa del via.
2. Prima di modificare file esistenti: commit pulito o copia di backup; altrimenti chiedere.
3. Distinguere sempre fatti verificati (con fonte e data), deduzioni e ipotesi. Dichiarare ciò
   che non si può verificare, letture parziali, documentazione inaccessibile, strumenti rotti.
4. Non inventare contenuti scientifici (testi dei questionari, soglie, traduzioni di scale
   validate): lasciare un segnaposto chiaro e segnalarlo.
5. Interfaccia in italiano; codice, identificatori e commit in inglese.
6. Commit e push solo quando Luigi lo chiede o a fine tappa.

## Vincoli non negoziabili

- **Nessun consiglio sanitario.** L'app registra e riassume; non suggerisce dosi, varietà,
  integratori o terapie (MDR, regola 11). Niente classifiche tipo "la varietà migliore".
- **Dati solo sul dispositivo.** Nessuna sincronizzazione, nessuna chiamata di rete con dati
  personali. I dati escono solo con un'azione esplicita, e ogni uscita crea un evento "Invio dati".
- **Cifratura a riposo** di tutti i dati personali (vedi Architettura). Blocco dopo inattività.
- **Nessuna terza parte**: niente analytics, tracker, font, script o CDN esterni. CSP restrittiva
  (`vite.config.ts` → `CSP`, `deploy/nginx.conf`). Il test e2e fallisce su richieste esterne.
- **Minimizzazione**: nessun campo sulla provenienza dei prodotti. Testo libero, riflessioni e
  foto non entrano mai nelle esportazioni per la ricerca (esportazione a elenco di campi ammessi).
- **Leggerezza**: ogni registrazione ≤ ~10 s, ogni domanda saltabile, modalità pausa.
  Niente serie di giorni, punteggi, badge, notifiche insistenti.
- **Open source**: AGPL-3.0-only.

### Cura e appagamento

- L'appagamento viene dal gesto di cura, non da premi esterni.
- Gesti di cura come registrazione principale (un tocco, appagamento 0-10 facoltativo).
- Dopo un check-in o un gesto, una breve frase di riconoscimento, varia, mai stucchevole, presa da
  `content/riconoscimenti.it.txt` (lo modifica Luigi).
- Promemoria come inviti ("Ti va di…?"), sempre con "non ora" senza conseguenze.
- Domanda serale facoltativa "Cosa ti ha fatto bene oggi?", solo locale.
- Il riepilogo settimanale si apre con ciò che ha fatto bene; poi gli andamenti.
- Un dato mancante non è un fallimento: l'app non segnala buchi e non fa sentire in colpa.
- Linguaggio caldo e semplice, mai clinico né da sorveglianza; forme neutre rispetto al genere
  ("Ti do il benvenuto", non "Benvenuto").

## Architettura (dalla R1, 21/11/2026: tutto in Rust)

La PWA Svelte (tappe 2-3) è conservata nel tag git `pwa-final`, solo come riferimento.

- Workspace Cargo: `crates/kuracare-core` (nucleo, nessuna interfaccia, **nessuna rete**: la CI
  lo verifica) e `crates/kuracare-app` (Dioxus 0.7.9, APK Android per il Pixel).
- `crypto`: Argon2id (passphrase normalizzata NFC) → KEK; DEK casuale AES-256-GCM, avvolta con
  AES-KW. Chiavi in `Zeroizing`: al blocco vengono cancellate dalla memoria. AAD = id del record.
- `store`: SQLite (`rusqlite` bundled, `secure_delete`) nella cartella privata dell'app. In chiaro
  solo l'id dei record e l'intestazione del vault. `MemStore` per i test.
- `event`: `RawEvent` (forma salvata, payload JSON) ↔ `Event` (tipizzato). ts ISO 8601 con offset
  (`jiff`), fuso IANA, origin `prompted`/`spontaneous`, `schemaVersion`, mancanti = null.
  Stessa forma JSON della PWA (c'è un test).
- **Due versioni distinte**: `schemaVersion` (forma del payload, migrata in lettura da
  `migrations`, mai riscrivendo lo storico) e versione dello strumento (es. `core-v0.1`) dentro
  il payload, mai migrata.
- `kinds`: checkin, thought, care, home (stanze), reflection, cannabis, supplement, meal.
  `research_fields` = elenco dei campi ammessi; `export::research_view` costruisce per inclusione.
- `enrich` ("Se ti va"): solo dettagli fattuali, mai risposte del check-in.
- **Rete (verificato il 21/11/2026 sul Pixel)**: Dioxus su Android parla con la WebView tramite un
  websocket su 127.0.0.1, e Android vuole il permesso INTERNET anche per i socket locali: senza,
  l'app resta bianca. Quindi l'APK ha INTERNET, con queste difese:
  - nucleo senza librerie di rete (CI);
  - CSP della pagina (`main.rs`, `CSP`): connessioni solo a 127.0.0.1;
  - `network_security_config`: niente traffico in chiaro tranne 127.0.0.1, **nessuna CA
    attendibile** (ogni connessione TLS fallisce);
  - WebView con Safe Browsing e metriche spenti (meta-data nel manifest).
  Fatto aperto: la WebView tenta comunque connessioni HTTPS verso server Google (172.217.x.x,
  2001:4860:…), tutte rifiutate dalla regola sulle CA (nessun contenuto esce). Da identificare con
  un net-log della WebView su build debug in emulatore. Controllo: `scripts/check-network.sh`
  (contatori Android, cumulativi) e lettura di /proc/net/tcp* per l'UID dell'app.
- **APK**: `scripts/apk-release.sh` (dx release + Gradle release + manifest e risorse di
  `crates/kuracare-app/android/` + firma con la chiave in `~/.config/kuracare`, mai nel repo).
  Non debuggabile, `allowBackup=false`, regole che escludono backup e trasferimento dati.
  Argon2id 64 MiB: 199 ms sul Pixel in release.
- **Foto dei pasti: sospese** (21/11, scelta di Luigi). La fotocamera via `<input capture>` non ha
  funzionato nell'APK di rilascio; wry lascerebbe la foto originale in chiaro nella memoria
  esterna dell'app (`platform::clear_capture_dir` la svuota a ogni avvio). `photo::shrink` resta.
- **Inviti (tappa 4, verificata sul Pixel il 21/11)**: `kuracare-core/src/reminders.rs` calcola il
  piano dei prossimi 14 giorni (un invito per finestra in un momento casuale ma stabile, dal sale
  `invite_salt`; niente inviti a finestra già con check-in, in pausa, o finita; cose da fare con
  orario). Il piano contiene solo istanti, chiavi e testi generici, mai titoli: Kotlin
  (`crates/kuracare-app/android/kotlin/`, `Bridge.kt`) lo tiene in chiaro nelle preferenze private
  perché funzioni ad app bloccata e dopo il riavvio. Rust → Kotlin via JNI (`src/notify.rs`,
  regole R8 in `android/kuracare-rules.pro`). Una sveglia alla volta con `setWindow` da 10 minuti
  (misurato: `setAndAllowWhileIdle` dava +1 ora). Controllo: `adb shell dumpsys alarm | grep -A2 kuracare`.
  Toccare un invito apre il check-in con origine `prompted`. Permessi: INTERNET, POST_NOTIFICATIONS,
  RECEIVE_BOOT_COMPLETED (lo script di firma verifica che siano esattamente questi).
- Cose da fare: tipo `task` (titolo solo locale; sforzo 1-5, scadenza, stato da fare/fatta/lasciata
  andare). Niente rosso né "in ritardo".
- **Aspetto (21/11)**: `src/theme.rs` definisce i temi (Zafferano, Salvia, Alba, e le stagioni
  della Sardegna centrale: Primavera, Estate, Autunno, Inverno; "Le stagioni" cambia il 1/3, 1/6,
  1/9, 1/12) come gettoni CSS su `:root[data-theme][data-mode]`; `assets/app.css` usa solo quelli.
  Un test verifica i contrasti WCAG AA di ogni tema, giorno e sera: ogni colore nuovo deve
  passarlo. Caratteri OFL in `assets/fonts/` (con licenze), inclusi come data: URI (CSP
  `font-src 'self' data:`). Scelta del tema in chiaro in `look.json` (solo nome e modalità), così
  lo sblocco ha già il tema. Barre di Android colorate via `Bridge.setBars`. Icone disegnate a
  mano in `views/parts.rs` (`Icon`, `Emblem`).
- Ipotesi (21/11, non verificata): i tentativi HTTPS della WebView potrebbero essere il download
  del seed delle "variations" (in logcat: `variations_seed_loader … Seed missing signature`).
- Grafici (tappa 5): SVG disegnati a mano, calcoli nel nucleo con test.
- Questionari: GAD-7 e PHQ-8 con testi **segnaposto**; i testi ufficiali li fornisce Luigi.
  Mai PHQ-9 (item sul suicidio: serve un protocollo con comitato etico).

## Decisioni prese (21/11/2026)

- Telefono: Pixel 8 Pro (Android 17). Licenza AGPL-3.0-only.
- 21/11: riscrittura completa in Rust con Dioxus (scelta di Luigi), APK. Ordine: R1 nucleo,
  R2 interfaccia, R3 APK firmato; poi tappa 4 (promemoria nativi) e 5 (riepilogo e grafici).
- 21/11: inviti in un momento casuale della finestra; inviti delle cose da fare con testo generico.
  Niente file `.ics` (superfluo con le notifiche native).
- Casa: stanze Bagno, Cucina, Camera da letto (modificabili), un tocco + dettagli facoltativi.
- Repository pubblico su GitHub, CI con GitHub Actions. Finestre dei check-in scelte al primo avvio.
- Preset cannabis (tappa 3), percentuali THC/CBD da etichetta, non inventate:
  THC-dominanti dalla collaborazione Sherbinskis × Silent Seeds; CBD-dominanti di High Alpine
  (es. "Mandarine Flokies", "Lemon Cockies" — nomi da confermare con Luigi).

## Comandi

- `cargo test -p kuracare-core` · `cargo clippy --all-targets -- -D warnings` · `cargo fmt`
- App su PC: `cargo run -p kuracare-app --features desktop`
- APK di rilascio: `scripts/apk-release.sh` → `target/kuracare-<versione>.apk`, poi
  `adb install -r` (stessa firma: i dati restano). Icone: `scripts/make-android-icons.sh`.
- Un APK debug (`dx build` senza lo script) non contiene il codice Kotlin degli inviti: l'app
  funziona, gli inviti no.
- APK debug (firma diversa: per installarlo va disinstallato quello firmato, e si perdono i dati):
  `dx build --platform android` con `GRADLE_USER_HOME=~/.cache/kuracare-gradle`.
- Build pesanti con `systemd-run --user --scope -p MemoryMax=5G` (PC con 7 GB di RAM)
