# Kuracare

Un'app per prendersi cura di sé, e per rendere appagante il prendersi cura di sé.

Kuracare è un diario personale da tenere sul telefono. Ricorda le cose da fare, raccoglie con
domande brevi come stai, segna i gesti che ti fanno bene, le assunzioni (terapia con cannabis,
integratori) e i pasti, e una volta a settimana ti mostra un riepilogo che comincia da ciò che ti
ha fatto bene.

> **Stato:** prototipo personale in costruzione (tappa 2 di 6: fondamenta). Per ora ci sono il
> primo avvio, la passphrase, la cifratura e il blocco; le registrazioni arrivano con la tappa 3.

## Cosa fa

- Tiene tutti i dati **solo sul dispositivo**, cifrati con una chiave che deriva dalla tua
  passphrase (Argon2id + AES-GCM 256).
- Si chiude da sola dopo qualche minuto di inattività e a ogni riapertura.
- Funziona anche offline e si installa come un'app.

## Cosa non fa

- **Non dà consigli sanitari.** Non suggerisce dosi, varietà, integratori o terapie: registra e
  riassume. Per le decisioni sulla salute c'è chi ti segue.
- Non ha account, non sincronizza, non manda dati a nessuno. Niente analytics, tracker, font o
  script esterni. I dati escono solo se li esporti tu (tappa 6), e ogni esportazione viene annotata.
- Non usa punti, serie di giorni o badge, e non ti fa sentire in colpa per un giorno vuoto.

## La passphrase

Se dimentichi la passphrase, **i dati non si possono recuperare** da nessuno. È ciò che li tiene
al sicuro. Il backup cifrato (tappa 6) sarà la rete di sicurezza.

## Installarla sul telefono (Android, Chrome)

L'app funziona solo in un contesto sicuro: HTTPS oppure `localhost`.

**Dal server (uso normale).** Apri l'indirizzo dell'app in Chrome, menu ⋮ → *Installa app*
(oppure *Aggiungi a schermata Home*).

**Dal PC, per provare, via USB** (serve il debug USB attivo sul telefono):

```sh
npm ci
npm run build && npm run preview     # serve su http://localhost:4173
adb reverse tcp:4173 tcp:4173        # il telefono vede la porta del PC come sua
```

Poi sul telefono apri `http://localhost:4173` in Chrome e installala dal menu.

## Backup

Arriva con la tappa 6: un file cifrato con la tua passphrase, da conservare dove vuoi.
Fino ad allora i dati di prova possono andare persi (per esempio cancellando i dati di Chrome).

## Ospitare l'app

Sono solo file statici (`npm run build` → `dist/`). In `deploy/` c'è una configurazione nginx
con la Content Security Policy e gli altri header di sicurezza, senza log degli accessi.
Il server non vede mai i dati personali.

## Sviluppo

```sh
npm run dev      # sviluppo (senza CSP: la CSP si applica alla build)
npm run check    # controllo dei tipi
npm test         # test unitari (Vitest)
npm run e2e      # test end-to-end (Playwright) sulla build di produzione
```

Le regole del progetto sono in [`CLAUDE.md`](CLAUDE.md).

## Licenza

[GNU AGPL-3.0](LICENSE). I testi dei questionari validati (quando ci saranno) restano soggetti
ai termini d'uso dei rispettivi titolari, non a questa licenza.
