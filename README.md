# Kuracare

Un'app per prendersi cura di sé, e per rendere appagante il prendersi cura di sé.

Kuracare è un diario personale da tenere sul telefono. Ricorda le cose da fare, raccoglie con
domande brevi come stai, segna i gesti che ti fanno bene, le assunzioni (terapia con cannabis,
integratori) e i pasti, e una volta a settimana ti mostra un riepilogo che comincia da ciò che ti
ha fatto bene.

> **Stato:** riscrittura in Rust in corso (R1: nucleo). La prima versione, una web app in Svelte,
> è conservata nel tag `pwa-final`.

## Cosa fa

- Tiene tutti i dati **solo sul telefono**, cifrati con una chiave che deriva dalla tua
  passphrase (Argon2id + AES-256-GCM). Al blocco la chiave viene cancellata dalla memoria.
- Registra con un tocco gesti di cura, la cura della casa (bagno, cucina, camera da letto),
  assunzioni e pasti; un check-in breve e facoltativo; i pensieri che tornano.
- Funziona senza connessione: la versione per il telefono non ha nemmeno il permesso di usare
  internet.

## Cosa non fa

- **Non dà consigli sanitari.** Non suggerisce dosi, varietà, integratori o terapie: registra e
  riassume. Per le decisioni sulla salute c'è chi ti segue.
- Non ha account, non sincronizza, non manda dati a nessuno. I dati escono solo se li esporti tu,
  e ogni esportazione viene annotata.
- Non usa punti, serie di giorni o badge, e non ti fa sentire in colpa per un giorno vuoto.

## La passphrase

Se dimentichi la passphrase, **i dati non si possono recuperare** da nessuno. È ciò che li tiene
al sicuro. Il backup cifrato sarà la rete di sicurezza.

## Struttura

- `crates/kuracare-core`: il nucleo in Rust (cifratura, archivio, modello dei dati, migrazioni,
  esportazione per la ricerca). Nessuna interfaccia e nessuna libreria di rete.
- `crates/kuracare-app`: l'app (Dioxus) per Android e, per lo sviluppo, per il PC.
- `content/riconoscimenti.it.txt`: le frasi di riconoscimento, modificabili.

## Sviluppo

```sh
cargo test -p kuracare-core                    # test del nucleo
cargo run -p kuracare-app --features desktop   # l'app sul PC
dx build --platform android                    # l'APK (servono Android SDK, NDK e JDK 17)
```

Le regole del progetto sono in [`CLAUDE.md`](CLAUDE.md).

## Licenza

[GNU AGPL-3.0](LICENSE). I testi dei questionari validati (quando ci saranno) restano soggetti
ai termini d'uso dei rispettivi titolari, non a questa licenza.
