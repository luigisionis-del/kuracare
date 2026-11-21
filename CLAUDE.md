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

## Architettura

- Svelte 5 + Vite + TypeScript, una sola app (niente SvelteKit). PWA con vite-plugin-pwa.
- `src/core/crypto`: Argon2id (hash-wasm, WASM) → KEK; DEK AES-GCM-256 casuale, avvolta con
  AES-KW. In memoria la DEK è non estraibile. AAD = id del record (i record non si scambiano).
- `src/core/store`: Dexie/IndexedDB. In chiaro solo `id` del record e intestazione del vault
  (parametri KDF + chiave avvolta). Tipi, orari e valori sono cifrati. Le ricerche si fanno in
  memoria dopo lo sblocco.
- `src/core/model`: ogni registrazione è un evento (`KuraEvent`): ts ISO 8601 con offset, fuso
  IANA, kind, origin (`prompted`/`spontaneous`), `schemaVersion`, payload. I mancanti sono `null`.
- **Due versioni distinte**: `schemaVersion` (forma del payload, migrata in lettura da
  `src/core/migrations`, mai riscrivendo lo storico) e versione dello strumento (es. `core-v0.1`,
  `gad7@…`) dentro il payload, mai migrata.
- Promemoria (tappa 4): interfaccia `ReminderScheduler` sostituibile. Ora opzione A (in app +
  `.ics`), poi C (Capacitor, notifiche locali). Niente Web Push (passerebbe da FCM/Apple).
- Questionari: GAD-7 e PHQ-8 con testi **segnaposto**; i testi ufficiali li fornisce Luigi.
  Mai PHQ-9 (item sul suicidio: serve un protocollo con comitato etico).

## Decisioni prese (21/11/2026)

- Telefono: Pixel 8 Pro (Android, Chrome). Promemoria: A ora, C dopo. Licenza AGPL-3.0-only.
- Hosting: server nuovo dedicato con nginx (`deploy/nginx.conf`), dominio da definire.
- Repository su GitHub, CI con GitHub Actions. Finestre dei check-in scelte al primo avvio.
- Preset cannabis (tappa 3), percentuali THC/CBD da etichetta, non inventate:
  THC-dominanti dalla collaborazione Sherbinskis × Silent Seeds; CBD-dominanti di High Alpine
  (es. "Mandarine Flokies", "Lemon Cockies" — nomi da confermare con Luigi).

## Comandi

- `npm run dev` · `npm run build` · `npm run preview`
- `npm test` (Vitest, unità) · `npm run e2e` (Playwright su build di produzione, viewport Pixel)
- `npm run check` (svelte-check)
