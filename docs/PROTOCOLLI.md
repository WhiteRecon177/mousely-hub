> **Notice (English).** This is a research log written in Italian with the help of AI tools. Everything in it is **unverified**: protocol bytes, VID:PID and the "licence" of each source were NOT checked at the source or on hardware. Treat it as a list of leads, not as a specification. Before using anything, open the cited source yourself and check its licence (see CONTRIBUTING.md). Product and brand names belong to their owners and are used only to say which devices are meant. Mousely Hub is not affiliated with any of them.

# DeviceDeck - protocolli e cose da inserire per i mouse non Logitech
Compilato il 31/10/2026 dai file di ricerca di ChatGPT (README, indice, CSV per marchio, schede protocollo). **Nessun dato è stato verificato da me alla fonte** (le fonti non erano raggiungibili da qui) e nessun driver è stato provato su hardware. Tutto ciò che è "non trovato" non va indovinato.
> **Aggiornamento 01/11/2026:** in fondo al file ci sono l'ottavo giro (Attack Shark X6/X11 lettura, Maya X, Pulsar, Glorious, ASUS) e il nono giro (descrittore Pulsar `5402`, prove Windows, audit HaloBattery, lacune aperte).

## Regole per scrivere i driver
- Un driver alla volta, prima in **sola lettura** (batteria, DPI, polling). Le scritture solo dopo la conferma di un utente con quel mouse.
- Fonti **GPL/AGPL**: solo documentazione, mai copiare codice. Sono escluse le sequenze ottenute da decompilazione del software ufficiale.
- Non inviare mai: pairing, DFU, reset di fabbrica, flash o comandi sconosciuti. Salvataggi persistenti solo con backup e conferma.
- Un frame "prefisso di comando" non è un pacchetto completo: servono report ID, lunghezza, padding, checksum e ACK verificati.
- Separare cavo, dongle, Bluetooth e firmware: ID e interfaccia cambiano.
- Aggiungere sempre il blocco globale (un solo discorso HID alla volta) e il passaggio a modalità host se il protocollo lo richiede.
- Ogni driver nuovo implementa `MouseDriver` in `src-tauri/src/mouse/drivers/<marchio>.rs`, va registrato in `drivers/mod.rs` e restituisce `SupportLevel::Tested` solo dopo un test reale.
## Stato per marchio

| Marchio | Licenza fonti | Modelli provati da utenti | Cosa si può fare |
|---|---|---|---|
| Logitech | - | PRO X SUPERLIGHT 2 SE (maintainer) | già fatto (lettura/scrittura reale) |
| Attack Shark | permissiva | 3 | DPI, polling, batteria (X3); X6/X11: lettura DPI/polling + batteria via grant A0 (ottavo giro, MIT, provato solo su Linux) |
| Pulsar | permissiva | 4 | batteria; DPI/polling parziali |
| HyperX | permissiva | 1 | DPI, polling, batteria (Pulsefire Dart) |
| SteelSeries | permissiva | 8 | DPI, polling, batteria (Aerox/Prime) |
| VAXEE | permissiva | 0 | batteria, carica, polling; DPI no |
| Lamzu | MIT (HaloBattery, fonti MIT) | 2 | batteria (Maya X), frame completo ricostruito nell'ottavo giro |
| Razer | GPL/AGPL | 8 | solo documentazione |
| Corsair | GPL/AGPL | 2 | solo documentazione |
| Xtrfy / CHERRY XTRFY | GPL/AGPL | 2 | solo documentazione |
| Keychron | GPL/AGPL | 1 | solo documentazione (batteria) |


---

## Attack Shark

Licenza delle fonti: **permissiva**.

### Famiglia e apertura del dispositivo
Attack Shark X3 e X11. X3 1D57:FA61 cavo, 1D57:FA60 dongle; MI_02 COL04 config, COL03 batteria; feature report 04/05/06/08/0C.
Fonte: https://github.com/prodmutant/Attack-Shark-X3/blob/main/docs/PROTOCOL.md; licenza: MIT.

### Lettura/scrittura DPI, polling, batteria e modalità host
X3 SET polling report completo 06 09 01 01 FE 00 00 00 00 (1000 Hz), VV=08/04/02/01, checksum/complemento=FF-VV. DPI X3 report 04: raw=DPI/50-1, LSB byte 8-15 MSB 16-23, stage byte 24. Batteria asincrona 03 10 VV 01 XX (VV tensione, non percentuale). X11 report 0B puo disassociare dongle: NON inviare.

Fonte: https://github.com/prodmutant/Attack-Shark-X3/blob/main/docs/PROTOCOL.md; licenza: MIT.

### Modelli con comandi documentati (2)

**Attack Shark X3** - TESTATO da utenti
- VID:PID cavo: 1D57:FA61
- dongle: 1D57:FA60
- Interfaccia: 2
- Report ID: 0x04/0x05/0x06/0x08/0x0C (config); 0x03 (batteria)
- Dimensione: 9–59 byte (secondo report)
- lettura DPI: NON SUPPORTATO: HidD_GetFeature nessun dato; stato locale
- scrittura DPI: Feature report 0x04 56 byte: [04 38 01 ...]; DPI raw = DPI/50-1, low [8..15], high [16..23], stage [24], checksum BE [50..51] somma [3:50]
- lettura polling: NON SUPPORTATO: nessun GET della configurazione
- scrittura polling: Feature report 0x06 9 byte: 06 09 01 VV (FF-VV) 00 00 00 00; VV 08/04/02/01 per 125/250/500/1000 Hz
- batteria: INPUT ASINCRONO MI_02 COL03: 03 10 VV 01 XX (5 byte); VV/16 volt, % solo stimata
- persistenza: memoria mouse (scrittura config, read-back non disponibile)
- fonte: https://github.com/prodmutant/Attack-Shark-X3/blob/main/docs/PROTOCOL.md
- licenza: MIT
- note: Configurazione scrittura soltanto; feature report 0x04/0x05/0x08 checksum BE sum payload; test hardware macro; batteria tensione solo ipotesi e % NON certa. Scritture flash: backup. Stato DPI/polling non leggibile. Non usare MI_02 COL03 per scritture.

**Attack Shark X11** - TESTATO da utenti
- VID:PID cavo: 1D57:FA55 (documentato per X11)
- dongle: 1D57:FA60 (condiviso con X3; identificare model id 0x55)
- Interfaccia: 2
- Report ID: 0x0B
- lettura DPI: Report feature 0xA0 come richiesta per leggere config (vedi doc X11); risposta da validare per revisione
- scrittura DPI: Feature report 0x04 56 byte, layout X11 in PROTOCOL_EN; checksum BE somma [3..49] in [50..51]
- lettura polling: Feature report 0xA0 (richiesta di lettura nel driver X11; risposta per firmware da verificare)
- scrittura polling: Feature report 0x06, 9 byte: 06 09 01 VV (FF-VV) 00 00 00 00; VV 08/04/02/01 per 125/250/500/1000 Hz
- batteria: Input heartbeat dal ricevitore; report di status documentato nel driver X11, payload completo per firmware da verificare
- fonte: https://github.com/clevim/OpenSharkX11/blob/main/docs/protocol/PROTOCOL_EN.md
- licenza: MIT
- note: DPI/polling testati via dongle, BLE limitato (polling BLE non funziona); in collegamento via cavo max 125 Hz secondo test OpenSharkX11. NON inviare report 0x0B: ha causato unpairing RF. La richiesta 0xA0 non va confusa con funzione GET del X3.

### Da inserire nell'app
Implementare X3 (cavo 1D57:FA61, dongle 1D57:FA60, interfaccia MI_02: COL04 configurazione, COL03 batteria). Batteria asincrona = tensione, non percentuale: serve una tabella di conversione. **Non** inviare il report 0B del X11 (scollega il dongle).

---

## Pulsar

Licenza delle fonti: **permissiva**.

### Famiglia e apertura del dispositivo
Nordic/Sonix; X2 V2 e X3. Distinguere report Nordic a 17 byte da Sonix Feature a 64 byte; interfaccia e report variano per modello.
Fonte: https://github.com/packerlschupfer/pulsar-mouse-linux; licenza: MIT.

### Lettura/scrittura DPI, polling, batteria e modalità host
Batteria Nordic: report ID 08, comando 04, risposta byte 6 percentuale; 7 cavo; 8–9 mV; 16 checksum. DPI/polling frame completi specifici non trovato. X3 polling write segnalato non funzionante.

Fonte: https://github.com/packerlschupfer/pulsar-mouse-linux; licenza: MIT.

### Modelli con comandi documentati (12)

**X2V2** - DOCUMENTATO
- VID:PID cavo: 3554:F507 (famiglia Nordic; singolo modello da verificare)
- dongle: 3554:F508 (famiglia Nordic; singolo modello da verificare)
- Report ID: 0x08
- lettura DPI: Feature report 64 byte IF3: cat 05 reg 84 sub 21/15 (bulk preset; layout completo da driver)
- scrittura DPI: Feature report 64 byte IF3: cat 05 reg 04 sub 21/15; checksum LE sum bytes 0..61
- lettura polling: Feature report 64 byte IF3: cat 01 reg 89 sub 02
- scrittura polling: Feature report 64 byte IF3: cat 01 reg 09 sub 02
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT
- note: 4K solo X2V2 X2H X2A Xlite V3 ufficialmente compatibili; protocollo Nordic condiviso: interrogare identificazione reale prima di scrivere. eS non implicitamente identico.

**X2V2 Mini** - TESTATO da utenti
- VID:PID cavo: 3554:F507 (famiglia condivisa; attribuzione singolo modello da verificare)
- dongle: 3554:F508 (famiglia condivisa)
- Report ID: 0x08
- lettura DPI: Feature report 64 byte IF3: cat 05 reg 84 sub 21/15 (bulk preset; layout completo da driver)
- scrittura DPI: Feature report 64 byte IF3: cat 05 reg 04 sub 21/15; checksum LE sum bytes 0..61
- lettura polling: Feature report 64 byte IF3: cat 01 reg 89 sub 02
- scrittura polling: Feature report 64 byte IF3: cat 01 reg 09 sub 02
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/andrewrabert/python-pulsar-mouse-tool | https://github.com/packerlschupfer/pulsar-mouse-linux | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility
- licenza: MIT
- note: Test reale X2 V2 Mini (andrewrabert), lettura/scrittura DPI polling e batteria; PID condivisi tra più modelli, non identificazione univoca.

**X2H** - DOCUMENTATO
- VID:PID cavo: 3554:F507 (famiglia Nordic; singolo modello da verificare)
- dongle: 3554:F508 (famiglia Nordic; singolo modello da verificare)
- Report ID: 0x08
- lettura DPI: Feature report 64 byte IF3: cat 05 reg 84 sub 21/15 (bulk preset; layout completo da driver)
- scrittura DPI: Feature report 64 byte IF3: cat 05 reg 04 sub 21/15; checksum LE sum bytes 0..61
- lettura polling: Feature report 64 byte IF3: cat 01 reg 89 sub 02
- scrittura polling: Feature report 64 byte IF3: cat 01 reg 09 sub 02
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT
- note: 4K solo X2V2 X2H X2A Xlite V3 ufficialmente compatibili; protocollo Nordic condiviso: interrogare identificazione reale prima di scrivere. eS non implicitamente identico.

**X2H Mini** - DOCUMENTATO
- VID:PID cavo: 3554:F507 (famiglia Nordic; singolo modello da verificare)
- dongle: 3554:F508 (famiglia Nordic; singolo modello da verificare)
- Report ID: 0x08
- lettura DPI: Feature report 64 byte IF3: cat 05 reg 84 sub 21/15 (bulk preset; layout completo da driver)
- scrittura DPI: Feature report 64 byte IF3: cat 05 reg 04 sub 21/15; checksum LE sum bytes 0..61
- lettura polling: Feature report 64 byte IF3: cat 01 reg 89 sub 02
- scrittura polling: Feature report 64 byte IF3: cat 01 reg 09 sub 02
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT
- note: 4K solo X2V2 X2H X2A Xlite V3 ufficialmente compatibili; protocollo Nordic condiviso: interrogare identificazione reale prima di scrivere. eS non implicitamente identico.

**X2A** - DOCUMENTATO
- VID:PID cavo: 3554:F507 (famiglia Nordic; singolo modello da verificare)
- dongle: 3554:F508 (famiglia Nordic; singolo modello da verificare)
- Report ID: 0x08
- lettura DPI: Feature report 64 byte IF3: cat 05 reg 84 sub 21/15 (bulk preset; layout completo da driver)
- scrittura DPI: Feature report 64 byte IF3: cat 05 reg 04 sub 21/15; checksum LE sum bytes 0..61
- lettura polling: Feature report 64 byte IF3: cat 01 reg 89 sub 02
- scrittura polling: Feature report 64 byte IF3: cat 01 reg 09 sub 02
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT
- note: 4K solo X2V2 X2H X2A Xlite V3 ufficialmente compatibili; protocollo Nordic condiviso: interrogare identificazione reale prima di scrivere. eS non implicitamente identico.

**X2A Mini** - DOCUMENTATO
- VID:PID cavo: 3554:F507 (famiglia Nordic; singolo modello da verificare)
- dongle: 3554:F508 (famiglia Nordic; singolo modello da verificare)
- Report ID: 0x08
- lettura DPI: Feature report 64 byte IF3: cat 05 reg 84 sub 21/15 (bulk preset; layout completo da driver)
- scrittura DPI: Feature report 64 byte IF3: cat 05 reg 04 sub 21/15; checksum LE sum bytes 0..61
- lettura polling: Feature report 64 byte IF3: cat 01 reg 89 sub 02
- scrittura polling: Feature report 64 byte IF3: cat 01 reg 09 sub 02
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT
- note: 4K solo X2V2 X2H X2A Xlite V3 ufficialmente compatibili; protocollo Nordic condiviso: interrogare identificazione reale prima di scrivere. eS non implicitamente identico.

**X2 CrazyLight** - TESTATO da utenti
- VID:PID cavo: 3710:3414
- dongle: 3710:5406
- Report ID: 0x08
- lettura DPI: report 17 byte: registro DPI (offset completi non verificati)
- scrittura DPI: report 17 byte: impostazioni DPI; encoder nel driver
- lettura polling: report 17 byte: lettura polling implementata
- scrittura polling: report 17 byte: scrittura polling implementata
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://github.com/darthsoup/PulsarBattery | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT
- note: Confermati lettura e scrittura hardware nei collegamenti wireless/cavo; non generalizzare a tutte le edizioni CrazyLight.

**Xlite V3** - DOCUMENTATO
- VID:PID cavo: 3554:F507 (famiglia Nordic; singolo modello da verificare)
- dongle: 3554:F508 (famiglia Nordic; singolo modello da verificare)
- Report ID: 0x08
- lettura DPI: Feature report 64 byte IF3: cat 05 reg 84 sub 21/15 (bulk preset; layout completo da driver)
- scrittura DPI: Feature report 64 byte IF3: cat 05 reg 04 sub 21/15; checksum LE sum bytes 0..61
- lettura polling: Feature report 64 byte IF3: cat 01 reg 89 sub 02
- scrittura polling: Feature report 64 byte IF3: cat 01 reg 09 sub 02
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT
- note: 4K solo X2V2 X2H X2A Xlite V3 ufficialmente compatibili; protocollo Nordic condiviso: interrogare identificazione reale prima di scrivere. eS non implicitamente identico.

**Xlite V3 Mini** - DOCUMENTATO
- VID:PID cavo: 3554:F507 (famiglia Nordic; singolo modello da verificare)
- dongle: 3554:F508 (famiglia Nordic; singolo modello da verificare)
- Report ID: 0x08
- lettura DPI: Feature report 64 byte IF3: cat 05 reg 84 sub 21/15 (bulk preset; layout completo da driver)
- scrittura DPI: Feature report 64 byte IF3: cat 05 reg 04 sub 21/15; checksum LE sum bytes 0..61
- lettura polling: Feature report 64 byte IF3: cat 01 reg 89 sub 02
- scrittura polling: Feature report 64 byte IF3: cat 01 reg 09 sub 02
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT
- note: 4K solo X2V2 X2H X2A Xlite V3 ufficialmente compatibili; protocollo Nordic condiviso: interrogare identificazione reale prima di scrivere. eS non implicitamente identico.

**Xlite V3 Large** - DOCUMENTATO
- VID:PID cavo: 3554:F507 (famiglia Nordic; singolo modello da verificare)
- dongle: 3554:F508 (famiglia Nordic; singolo modello da verificare)
- Report ID: 0x08
- lettura DPI: Feature report 64 byte IF3: cat 05 reg 84 sub 21/15 (bulk preset; layout completo da driver)
- scrittura DPI: Feature report 64 byte IF3: cat 05 reg 04 sub 21/15; checksum LE sum bytes 0..61
- lettura polling: Feature report 64 byte IF3: cat 01 reg 89 sub 02
- scrittura polling: Feature report 64 byte IF3: cat 01 reg 09 sub 02
- batteria: Report 0x08: comando 0x04 (frame 17 byte); risposta byte6 percentuale byte7 cavo byte8-9 mV, checksum byte16
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT
- note: 4K solo X2V2 X2H X2A Xlite V3 ufficialmente compatibili; protocollo Nordic condiviso: interrogare identificazione reale prima di scrivere. eS non implicitamente identico.

**X3** - TESTATO da utenti
- VID:PID cavo: 3710:3409 / 3710:3410 (revisioni riportate)
- dongle: 3710:5402 (1K condiviso) / 3710:5403 (8K condiviso)
- lettura DPI: report Sonix 64 byte IF3 - lettura DPI documentata
- scrittura DPI: report Sonix 64 byte IF3 - scrittura DPI documentata
- lettura polling: lettura polling inattendibile nei test jonkristian
- scrittura polling: scrittura polling non funzionante nei test jonkristian
- batteria: report Sonix 64 byte IF3 - lettura batteria confermata
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://github.com/jonkristian/pulsar-x3-python | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility
- licenza: MIT
- note: Differenze hardware/firmware fra revisioni, X3 appare due volte nell’inventario originale; dati riferiti alla famiglia X3, determinare PID e connessione realmente enumerati.

**X3** - TESTATO da utenti
- VID:PID cavo: 3710:3409 / 3710:3410 (revisioni riportate)
- dongle: 3710:5402 (1K condiviso) / 3710:5403 (8K condiviso)
- lettura DPI: report Sonix 64 byte IF3 - lettura DPI documentata
- scrittura DPI: report Sonix 64 byte IF3 - scrittura DPI documentata
- lettura polling: lettura polling inattendibile nei test jonkristian
- scrittura polling: scrittura polling non funzionante nei test jonkristian
- batteria: report Sonix 64 byte IF3 - lettura batteria confermata
- fonte: https://github.com/packerlschupfer/pulsar-mouse-linux | https://github.com/jonkristian/pulsar-x3-python | https://support.pulsar.gg/hc/en-us/articles/58413468467353-4k-and-8k-Dongle-Compatibility
- licenza: MIT
- note: Differenze hardware/firmware fra revisioni, X3 appare due volte nell’inventario originale; dati riferiti alla famiglia X3, determinare PID e connessione realmente enumerati.

### Da inserire nell'app
Distinguere report Nordic a 17 byte da Sonix feature a 64 byte. Batteria Nordic: report ID 08, comando 04. Polling X3: scrittura non funzionante, tenerla disattivata.

---

## HyperX

Licenza delle fonti: **permissiva**.

### Famiglia e apertura del dispositivo
Pulsefire Dart. USB 0951:16E2, dongle 0951:16E1; interrupt IN/OUT, frame 64 byte (non feature).
Fonte: https://github.com/santeri3700/hyperx_pulsefire_dart_reverse_engineering/blob/main/protocol/index.md; licenza: CC0-1.0.

### Lettura/scrittura DPI, polling, batteria e modalità host
SET DPI D3 02 PP 02 LL HH 00... , DPI = u16_le*50; SET polling D0 00 00 01 RR 00... (RR 00/01/02/03 per 125/250/500/1000 Hz). Heartbeat 0x51, risposta byte 4 batteria %, 5 in carica. Persistenza separata: prefisso DE FF 00 00, NON inviare senza verifica completa.

Fonte: https://github.com/santeri3700/hyperx_pulsefire_dart_reverse_engineering/blob/main/protocol/index.md; licenza: CC0-1.0.

### Modelli con comandi documentati (3)

**Pulsefire Dart** - TESTATO da utenti
- VID:PID cavo: 0951:16E2
- dongle: 0951:16E1
- Interfaccia: 1
- scrittura DPI: 64-byte interrupt output: D3 02 [profilo] 02 [DPI/50 u16LE] + padding 00
- scrittura polling: 64-byte interrupt output: D0 00 00 01 [00=125;01=250;02=500;03=1000] + padding 00
- batteria: Richiesta 51 heartbeat; risposta byte 4=0..100% byte 5=0 non in carica / 1 in carica
- fonte: https://github.com/santeri3700/hyperx_pulsefire_dart_reverse_engineering/blob/main/protocol/index.md
- licenza: CC0-1.0
- note: Test reale su Pulsefire Dart; URB_INTERRUPT 64 byte; salva solo con DE FF 00 00; non inviare commit senza backup; non è Feature Report; Specifiche commerciali ereditate da inventario precedente, non tutte rivalidate; Non inviare DE FF 00 00 automaticamente: salva in NVM, modalità direct è volatile

**Pulsefire Haste** - DOCUMENTATO
- VID:PID cavo: 0951:1727
- lettura DPI: codec OpenMouse Haste: dettagli byte non verificati
- scrittura DPI: codec OpenMouse Haste: dettagli byte non verificati
- lettura polling: codec OpenMouse Haste: dettagli byte non verificati
- scrittura polling: codec OpenMouse Haste: dettagli byte non verificati
- fonte: https://openmouse.app/supported | https://github.com/hyperb1iss/hypercolor/blob/main/docs/content/hardware/compatibility.md
- licenza: AGPL-3.0
- note: PID documentato in catalogo RGB; non prova del protocollo DPI/polling; OpenMouse stato Test Needed; non testato su hardware dal progetto; DPI e polling per profilo; Specifiche commerciali ereditate da inventario precedente, non tutte rivalidate

**Pulsefire Haste** - DOCUMENTATO
- VID:PID cavo: 0951:1727
- lettura DPI: codec OpenMouse Haste: dettagli byte non verificati
- scrittura DPI: codec OpenMouse Haste: dettagli byte non verificati
- lettura polling: codec OpenMouse Haste: dettagli byte non verificati
- scrittura polling: codec OpenMouse Haste: dettagli byte non verificati
- fonte: https://openmouse.app/supported | https://github.com/hyperb1iss/hypercolor/blob/main/docs/content/hardware/compatibility.md
- licenza: AGPL-3.0
- note: PID documentato in catalogo RGB; non prova del protocollo DPI/polling; OpenMouse stato Test Needed; non testato su hardware dal progetto; DPI e polling per profilo; duplicato originale mantenuto; Specifiche commerciali ereditate da inventario precedente, non tutte rivalidate

### Da inserire nell'app
Pulsefire Dart: interrupt IN/OUT da 64 byte, non feature report. Non inviare il prefisso di persistenza DE FF 00 00 senza verifica completa.

---

## SteelSeries

Licenza delle fonti: **permissiva**.

### Famiglia e apertura del dispositivo
Rival/Aerox e ricevitore wireless. Per Aerox 5 cablato, interfaccia 3; frame completo e usage da verificare.
Fonte: https://github.com/flozz/rivalcfg/blob/master/rivalcfg/devices/aerox5.py; licenza: WTFPL.

### Lettura/scrittura DPI, polling, batteria e modalità host
Aerox 5: output 0x2D (preset DPI), output 0x2B (polling); 125/250/500/1000 Hz corrispondono a 04/03/02/01. Wireless batteria: 00 D2 00 00... via dongle in alcune varianti; non universalizzare.

Fonte: https://github.com/flozz/rivalcfg/blob/master/rivalcfg/devices/aerox5.py; licenza: WTFPL.

### Modelli con comandi documentati (13)

**Rival 3 Wireless** - TESTATO da utenti
- dongle: 1038:1830
- batteria: 00 AA 01 ... sul ricevitore 2.4 GHz - decodifica variante Rival 3
- fonte: https://github.com/yurtemre7/steel-mouse | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md | https://rivalcfg.flozz.org/devices.html | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- licenza: MIT (SteelMouse/steelseriesgg-rs) / WTFPL (rivalcfg)
- note: TESTATO su hardware per LETTURA BATTERIA secondo SteelMouse; DPI/polling non automaticamente testati - PID e report vanno confermati sulla specifica revisione

**Rival 3 Wireless Gen 2** - TESTATO da utenti
- dongle: 1038:1872
- batteria: 00 AA 01 ... sul ricevitore 2.4 GHz - decodifica variante Rival 3
- fonte: https://github.com/yurtemre7/steel-mouse | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md | https://rivalcfg.flozz.org/devices.html | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md | https://rivalcfg.flozz.org/devices.html
- licenza: MIT (SteelMouse/steelseriesgg-rs) / WTFPL (rivalcfg)
- note: TESTATO su hardware per LETTURA BATTERIA secondo SteelMouse; DPI/polling non automaticamente testati - PID e report vanno confermati sulla specifica revisione; protocollo Gen2 da non equiparare alla Gen1; report AA della Gen1 NON confermato sulla Gen2

**Rival 5** - DOCUMENTATO
- VID:PID cavo: 1038:183C
- scrittura DPI: output report 0x2D con codifica preset specifica del modello (non pacchetto completo)
- scrittura polling: output report 0x2B - codici dipendono dal modello (non pacchetto completo)
- fonte: https://rivalcfg.flozz.org/devices.html | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md
- licenza: WTFPL (rivalcfg) / MIT (steelseriesgg-rs)
- note: I comandi non sono automaticamente validi per ogni firmware - modalità Bluetooth non verificata

**Aerox 3** - DOCUMENTATO
- VID:PID cavo: 1038:1836
- scrittura DPI: output report 0x2D con codifica preset specifica del modello (non pacchetto completo)
- scrittura polling: output report 0x2B - codici dipendono dal modello (non pacchetto completo)
- fonte: https://rivalcfg.flozz.org/devices.html | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md
- licenza: WTFPL (rivalcfg) / MIT (steelseriesgg-rs)
- note: I comandi non sono automaticamente validi per ogni firmware - modalità Bluetooth non verificata

**Aerox 3 Wireless** - TESTATO da utenti
- VID:PID cavo: 1038:183A
- dongle: 1038:1838
- scrittura DPI: output report 0x2D con codifica preset specifica del modello (non pacchetto completo)
- scrittura polling: output report 0x2B - codici dipendono dal modello (non pacchetto completo)
- batteria: output 00 92 (cavo) / 00 D2 (dongle) - risposta e decode secondo famiglia; non universalmente testato
- fonte: https://github.com/yurtemre7/steel-mouse | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md | https://rivalcfg.flozz.org/devices.html
- licenza: MIT (SteelMouse/steelseriesgg-rs) / WTFPL (rivalcfg)
- note: TESTATO su hardware per LETTURA BATTERIA secondo SteelMouse; DPI/polling non automaticamente testati - PID e report vanno confermati sulla specifica revisione

**Aerox 3 Wireless (2022)** - DOCUMENTATO
- VID:PID cavo: 1038:183A
- dongle: 1038:1838
- scrittura DPI: output report 0x2D con codifica preset specifica del modello (non pacchetto completo)
- scrittura polling: output report 0x2B - codici dipendono dal modello (non pacchetto completo)
- batteria: output 00 92 (cavo) / 00 D2 (dongle) - risposta e decode secondo famiglia; non universalmente testato
- fonte: https://github.com/yurtemre7/steel-mouse | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md | https://rivalcfg.flozz.org/devices.html
- licenza: MIT (SteelMouse/steelseriesgg-rs) / WTFPL (rivalcfg)
- note: Possibile stesso VID:PID della revisione precedente (verificare sul dispositivo); batteria testata su Aerox 3 Wireless ma non specificamente revisione 2022

**Aerox 3 Wireless Gen 2** - TESTATO da utenti
- VID:PID cavo: 1038:1892
- dongle: 1038:1890
- scrittura DPI: output report 0x2D con codifica preset specifica del modello (non pacchetto completo)
- scrittura polling: output report 0x2B - codici dipendono dal modello (non pacchetto completo)
- batteria: output 00 92 (cavo) / 00 D2 (dongle) - risposta e decode secondo famiglia; non universalmente testato
- fonte: https://github.com/yurtemre7/steel-mouse | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md | https://rivalcfg.flozz.org/devices.html | https://rivalcfg.flozz.org/devices.html
- licenza: MIT (SteelMouse/steelseriesgg-rs) / WTFPL (rivalcfg)
- note: TESTATO su hardware per LETTURA BATTERIA secondo SteelMouse; DPI/polling non automaticamente testati - PID e report vanno confermati sulla specifica revisione; protocollo Gen2 da non equiparare alla Gen1

**Aerox 5** - DOCUMENTATO
- VID:PID cavo: 1038:1850
- Interfaccia: 3
- DPI: 100 - 18000 (step 100)
- scrittura DPI: HID output 0x2D - preset 100..18000 DPI passo 100, serializzazione in rivalcfg (non byte completi)
- scrittura polling: HID output 0x2B - 125:04 / 250:03 / 500:02 / 1000:01
- fonte: https://github.com/flozz/rivalcfg/blob/master/rivalcfg/devices/aerox5.py
- licenza: WTFPL
- note: I comandi non sono automaticamente validi per ogni firmware - modalità Bluetooth non verificata

**Aerox 5 Wireless** - TESTATO da utenti
- VID:PID cavo: 1038:1854
- dongle: 1038:1852
- scrittura DPI: output report 0x2D con codifica preset specifica del modello (non pacchetto completo)
- scrittura polling: output report 0x2B - codici dipendono dal modello (non pacchetto completo)
- batteria: output 00 92 (cavo) / 00 D2 (dongle) - risposta e decode secondo famiglia; non universalmente testato
- fonte: https://github.com/yurtemre7/steel-mouse | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md | https://rivalcfg.flozz.org/devices.html
- licenza: MIT (SteelMouse/steelseriesgg-rs) / WTFPL (rivalcfg)
- note: TESTATO su hardware per LETTURA BATTERIA secondo SteelMouse; DPI/polling non automaticamente testati - PID e report vanno confermati sulla specifica revisione

**Aerox 9 Wireless** - TESTATO da utenti
- VID:PID cavo: 1038:185A
- dongle: 1038:1858
- scrittura DPI: output report 0x2D con codifica preset specifica del modello (non pacchetto completo)
- scrittura polling: output report 0x2B - codici dipendono dal modello (non pacchetto completo)
- batteria: output 00 92 (cavo) / 00 D2 (dongle) - risposta e decode secondo famiglia; non universalmente testato
- fonte: https://github.com/yurtemre7/steel-mouse | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md | https://rivalcfg.flozz.org/devices.html
- licenza: MIT (SteelMouse/steelseriesgg-rs) / WTFPL (rivalcfg)
- note: TESTATO su hardware per LETTURA BATTERIA secondo SteelMouse; DPI/polling non automaticamente testati - PID e report vanno confermati sulla specifica revisione

**Prime Wireless** - TESTATO da utenti
- VID:PID cavo: 1038:1842
- dongle: 1038:1840
- batteria: output 00 92 (cavo) / 00 D2 (dongle) - risposta e decode secondo famiglia; non universalmente testato
- fonte: https://github.com/yurtemre7/steel-mouse | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md | https://rivalcfg.flozz.org/devices.html
- licenza: MIT (SteelMouse/steelseriesgg-rs) / WTFPL (rivalcfg)
- note: TESTATO su hardware per LETTURA BATTERIA secondo SteelMouse; DPI/polling non automaticamente testati - PID e report vanno confermati sulla specifica revisione

**Prime Mini** - DOCUMENTATO
- VID:PID cavo: 1038:184D
- scrittura DPI: output report 0x2D con codifica preset specifica del modello (non pacchetto completo)
- scrittura polling: output report 0x2B - codici dipendono dal modello (non pacchetto completo)
- fonte: https://rivalcfg.flozz.org/devices.html | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md
- licenza: WTFPL (rivalcfg) / MIT (steelseriesgg-rs)
- note: I comandi non sono automaticamente validi per ogni firmware - modalità Bluetooth non verificata

**Prime Mini Wireless** - TESTATO da utenti
- VID:PID cavo: 1038:184A
- dongle: 1038:1848
- scrittura DPI: output report 0x2D con codifica preset specifica del modello (non pacchetto completo)
- scrittura polling: output report 0x2B - codici dipendono dal modello (non pacchetto completo)
- batteria: output 00 92 (cavo) / 00 D2 (dongle) - risposta e decode secondo famiglia; non universalmente testato
- fonte: https://github.com/yurtemre7/steel-mouse | https://github.com/Ven0m0/steelseriesgg-rs/blob/main/docs/development/devices.md | https://rivalcfg.flozz.org/devices.html
- licenza: MIT (SteelMouse/steelseriesgg-rs) / WTFPL (rivalcfg)
- note: TESTATO su hardware per LETTURA BATTERIA secondo SteelMouse; DPI/polling non automaticamente testati - PID e report vanno confermati sulla specifica revisione

### Da inserire nell'app
Riferimento rivalcfg (WTFPL, utilizzabile). Frame completo e usage da verificare per modello. La batteria wireless varia per dongle: non universalizzare.

---

## VAXEE

Licenza delle fonti: **permissiva**.

### Famiglia e apertura del dispositivo
Protocollo VAXEE Feature. VID 3057; usage 0xFF05/0x0001; Feature report ID 0E, 64 byte.
Fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md; licenza: MIT.

### Lettura/scrittura DPI, polling, batteria e modalità host
Batteria TX 0E A5 0B 01 01 00... (64 byte); RX byte 5 * 5 percentuale; carica TX 0E A5 10 01 01 00..., RX byte 5. Polling GET 0E A5 07 01 01 00..., SET 0E A5 07 02 01 02 00... (1000 Hz), indici 01/02/03/04/05 per 500/1000/2000/4000/8000; test XE-S-L. DPI non trovato.

Fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md; licenza: MIT.

### Modelli con comandi documentati (9)

**ZYGEN NP-01 Wireless** - DOCUMENTATO
- VID:PID cavo: 3057:1006
- dongle: 3057:1001 / 3057:1002 / 3057:0005 / 3057:2001 (ricevitori VAXEE condivisi; associazione specifica non trovata)
- Usage: 0xFF05/0x0001 (report 0x0E; verificare collection del PID specifico)
- Report ID: 0x0E
- Dimensione: 64 byte
- DPI: 400 - 3200 (step preset 400/800/1600/3200 - passo non uniforme)
- lettura polling: FEATURE 0E A5 07 01 01 + padding 00; risposta [5] indice (1=500,2=1000,3=2000,4=4000,5=8000) - testato XE-S-L, altri da verificare
- scrittura polling: FEATURE 0E A5 07 02 01 [indice] + padding 00; verificare ACK e limite link/firmware; testato XE-S-L, altri da verificare
- batteria: FEATURE report 0x0E, 64 byte: 0E A5 0B 01 01 + padding 00; risposta [5]*5 percentuale; ricarica 0E A5 10 01 01 + padding, risposta [5]!=0
- fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md | https://github.com/ryanlewis/mousebatt/blob/main/src/protocol.rs | https://eu.vaxee.co/feature.php?c=usermanual
- licenza: MIT (stuffz; ryanlewis); documentazione ufficiale - licenza non specificata
- note: PID diretto documentato nel registro MIT; ricevitore multi-modello non attribuito univocamente; batteria e polling implementati per famiglia, NON testati individualmente su questo modello; nessuna scrittura DPI confermata

**ZYGEN NP-01S Wireless** - DOCUMENTATO
- VID:PID cavo: 3057:1004
- dongle: 3057:1001 / 3057:1002 / 3057:0005 / 3057:2001 (ricevitori VAXEE condivisi; associazione specifica non trovata)
- Usage: 0xFF05/0x0001 (report 0x0E; verificare collection del PID specifico)
- Report ID: 0x0E
- Dimensione: 64 byte
- DPI: 400 - 3200 (step preset 400/800/1600/3200 - passo non uniforme)
- lettura polling: FEATURE 0E A5 07 01 01 + padding 00; risposta [5] indice (1=500,2=1000,3=2000,4=4000,5=8000) - testato XE-S-L, altri da verificare
- scrittura polling: FEATURE 0E A5 07 02 01 [indice] + padding 00; verificare ACK e limite link/firmware; testato XE-S-L, altri da verificare
- batteria: FEATURE report 0x0E, 64 byte: 0E A5 0B 01 01 + padding 00; risposta [5]*5 percentuale; ricarica 0E A5 10 01 01 + padding, risposta [5]!=0
- fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md | https://github.com/ryanlewis/mousebatt/blob/main/src/protocol.rs | https://eu.vaxee.co/feature.php?c=usermanual
- licenza: MIT (stuffz; ryanlewis); documentazione ufficiale - licenza non specificata
- note: PID diretto documentato nel registro MIT; ricevitore multi-modello non attribuito univocamente; batteria e polling implementati per famiglia, NON testati individualmente su questo modello; nessuna scrittura DPI confermata

**ZYGEN NP-01S V2 Wireless** - DOCUMENTATO
- VID:PID cavo: 3057:1011
- dongle: 3057:1001 / 3057:1002 / 3057:0005 / 3057:2001 (ricevitori VAXEE condivisi; associazione specifica non trovata)
- Usage: 0xFF05/0x0001 (report 0x0E; verificare collection del PID specifico)
- Report ID: 0x0E
- Dimensione: 64 byte
- DPI: 400 - 3200 (step preset 400/800/1600/3200 - passo non uniforme)
- lettura polling: FEATURE 0E A5 07 01 01 + padding 00; risposta [5] indice (1=500,2=1000,3=2000,4=4000,5=8000) - testato XE-S-L, altri da verificare
- scrittura polling: FEATURE 0E A5 07 02 01 [indice] + padding 00; verificare ACK e limite link/firmware; testato XE-S-L, altri da verificare
- batteria: FEATURE report 0x0E, 64 byte: 0E A5 0B 01 01 + padding 00; risposta [5]*5 percentuale; ricarica 0E A5 10 01 01 + padding, risposta [5]!=0
- fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md | https://github.com/ryanlewis/mousebatt/blob/main/src/protocol.rs | https://eu.vaxee.co/feature.php?c=usermanual
- licenza: MIT (stuffz; ryanlewis); documentazione ufficiale - licenza non specificata
- note: PID diretto documentato nel registro MIT; ricevitore multi-modello non attribuito univocamente; batteria e polling implementati per famiglia, NON testati individualmente su questo modello; nessuna scrittura DPI confermata

**OUTSET AX Wireless** - DOCUMENTATO
- VID:PID cavo: 3057:1005
- dongle: 3057:1001 / 3057:1002 / 3057:0005 / 3057:2001 (ricevitori VAXEE condivisi; associazione specifica non trovata)
- Usage: 0xFF05/0x0001 (report 0x0E; verificare collection del PID specifico)
- Report ID: 0x0E
- Dimensione: 64 byte
- DPI: 400 - 3200 (step preset 400/800/1600/3200 - passo non uniforme)
- lettura polling: FEATURE 0E A5 07 01 01 + padding 00; risposta [5] indice (1=500,2=1000,3=2000,4=4000,5=8000) - testato XE-S-L, altri da verificare
- scrittura polling: FEATURE 0E A5 07 02 01 [indice] + padding 00; verificare ACK e limite link/firmware; testato XE-S-L, altri da verificare
- batteria: FEATURE report 0x0E, 64 byte: 0E A5 0B 01 01 + padding 00; risposta [5]*5 percentuale; ricarica 0E A5 10 01 01 + padding, risposta [5]!=0
- fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md | https://github.com/ryanlewis/mousebatt/blob/main/src/protocol.rs | https://eu.vaxee.co/feature.php?c=usermanual
- licenza: MIT (stuffz; ryanlewis); documentazione ufficiale - licenza non specificata
- note: PID diretto documentato nel registro MIT; ricevitore multi-modello non attribuito univocamente; batteria e polling implementati per famiglia, NON testati individualmente su questo modello; nessuna scrittura DPI confermata

**OUTSET AX V2 Wireless** - DOCUMENTATO
- dongle: 3057:1001 / 3057:1002 / 3057:0005 / 3057:2001 (ricevitori VAXEE condivisi; associazione specifica non trovata)
- Usage: 0xFF05/0x0001 (report 0x0E; verificare collection del PID specifico)
- Report ID: 0x0E
- Dimensione: 64 byte
- lettura polling: FEATURE 0E A5 07 01 01 + padding 00; risposta [5] indice (1=500,2=1000,3=2000,4=4000,5=8000) - testato XE-S-L, altri da verificare
- scrittura polling: FEATURE 0E A5 07 02 01 [indice] + padding 00; verificare ACK e limite link/firmware; testato XE-S-L, altri da verificare
- batteria: FEATURE report 0x0E, 64 byte: 0E A5 0B 01 01 + padding 00; risposta [5]*5 percentuale; ricarica 0E A5 10 01 01 + padding, risposta [5]!=0
- fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md | https://github.com/ryanlewis/mousebatt/blob/main/src/protocol.rs | https://eu.vaxee.co/feature.php?c=usermanual
- licenza: MIT (stuffz; ryanlewis); documentazione ufficiale - licenza non specificata
- note: Variante V2 non associata a PID univoco nella fonte; non copiare il PID della prima generazione; batteria e polling da testare

**XE Wireless** - DOCUMENTATO
- VID:PID cavo: 3057:1003
- dongle: 3057:1001 / 3057:1002 / 3057:0005 / 3057:2001 (ricevitori VAXEE condivisi; associazione specifica non trovata)
- Usage: 0xFF05/0x0001 (report 0x0E; verificare collection del PID specifico)
- Report ID: 0x0E
- Dimensione: 64 byte
- DPI: 400 - 3200 (step preset 400/800/1600/3200 - passo non uniforme)
- lettura polling: FEATURE 0E A5 07 01 01 + padding 00; risposta [5] indice (1=500,2=1000,3=2000,4=4000,5=8000) - testato XE-S-L, altri da verificare
- scrittura polling: FEATURE 0E A5 07 02 01 [indice] + padding 00; verificare ACK e limite link/firmware; testato XE-S-L, altri da verificare
- batteria: FEATURE report 0x0E, 64 byte: 0E A5 0B 01 01 + padding 00; risposta [5]*5 percentuale; ricarica 0E A5 10 01 01 + padding, risposta [5]!=0
- fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md | https://github.com/ryanlewis/mousebatt/blob/main/src/protocol.rs | https://eu.vaxee.co/feature.php?c=usermanual
- licenza: MIT (stuffz; ryanlewis); documentazione ufficiale - licenza non specificata
- note: PID diretto documentato nel registro MIT; ricevitore multi-modello non attribuito univocamente; batteria e polling implementati per famiglia, NON testati individualmente su questo modello; nessuna scrittura DPI confermata

**XE-S Wireless** - DOCUMENTATO
- VID:PID cavo: 3057:1007
- dongle: 3057:1001 / 3057:1002 / 3057:0005 / 3057:2001 (ricevitori VAXEE condivisi; associazione specifica non trovata)
- Usage: 0xFF05/0x0001 (report 0x0E; verificare collection del PID specifico)
- Report ID: 0x0E
- Dimensione: 64 byte
- DPI: 400 - 3200 (step preset 400/800/1600/3200 - passo non uniforme)
- lettura polling: FEATURE 0E A5 07 01 01 + padding 00; risposta [5] indice (1=500,2=1000,3=2000,4=4000,5=8000) - testato XE-S-L, altri da verificare
- scrittura polling: FEATURE 0E A5 07 02 01 [indice] + padding 00; verificare ACK e limite link/firmware; testato XE-S-L, altri da verificare
- batteria: FEATURE report 0x0E, 64 byte: 0E A5 0B 01 01 + padding 00; risposta [5]*5 percentuale; ricarica 0E A5 10 01 01 + padding, risposta [5]!=0
- fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md | https://github.com/ryanlewis/mousebatt/blob/main/src/protocol.rs | https://eu.vaxee.co/feature.php?c=usermanual
- licenza: MIT (stuffz; ryanlewis); documentazione ufficiale - licenza non specificata
- note: PID diretto documentato nel registro MIT; ricevitore multi-modello non attribuito univocamente; batteria e polling implementati per famiglia, NON testati individualmente su questo modello; nessuna scrittura DPI confermata; test reale disponibile per XE-S-L (variante mancina), non prova diretta XE-S

**E1 Wireless** - DOCUMENTATO
- VID:PID cavo: 3057:1010
- dongle: 3057:1001 / 3057:1002 / 3057:0005 / 3057:2001 (ricevitori VAXEE condivisi; associazione specifica non trovata)
- Usage: 0xFF05/0x0001 (report 0x0E; verificare collection del PID specifico)
- Report ID: 0x0E
- Dimensione: 64 byte
- DPI: 400 - 3200 (step preset 400/800/1600/3200 - passo non uniforme)
- lettura polling: FEATURE 0E A5 07 01 01 + padding 00; risposta [5] indice (1=500,2=1000,3=2000,4=4000,5=8000) - testato XE-S-L, altri da verificare
- scrittura polling: FEATURE 0E A5 07 02 01 [indice] + padding 00; verificare ACK e limite link/firmware; testato XE-S-L, altri da verificare
- batteria: FEATURE report 0x0E, 64 byte: 0E A5 0B 01 01 + padding 00; risposta [5]*5 percentuale; ricarica 0E A5 10 01 01 + padding, risposta [5]!=0
- fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md | https://github.com/ryanlewis/mousebatt/blob/main/src/protocol.rs | https://eu.vaxee.co/feature.php?c=usermanual
- licenza: MIT (stuffz; ryanlewis); documentazione ufficiale - licenza non specificata
- note: PID diretto documentato nel registro MIT; ricevitore multi-modello non attribuito univocamente; batteria e polling implementati per famiglia, NON testati individualmente su questo modello; nessuna scrittura DPI confermata

**E1 V2 Wireless** - DOCUMENTATO
- dongle: 3057:1001 / 3057:1002 / 3057:0005 / 3057:2001 (ricevitori VAXEE condivisi; associazione specifica non trovata)
- Usage: 0xFF05/0x0001 (report 0x0E; verificare collection del PID specifico)
- Report ID: 0x0E
- Dimensione: 64 byte
- lettura polling: FEATURE 0E A5 07 01 01 + padding 00; risposta [5] indice (1=500,2=1000,3=2000,4=4000,5=8000) - testato XE-S-L, altri da verificare
- scrittura polling: FEATURE 0E A5 07 02 01 [indice] + padding 00; verificare ACK e limite link/firmware; testato XE-S-L, altri da verificare
- batteria: FEATURE report 0x0E, 64 byte: 0E A5 0B 01 01 + padding 00; risposta [5]*5 percentuale; ricarica 0E A5 10 01 01 + padding, risposta [5]!=0
- fonte: https://github.com/stuffz/mouse-battery-monitor/blob/main/docs/VAXEE.md | https://github.com/ryanlewis/mousebatt/blob/main/src/protocol.rs | https://eu.vaxee.co/feature.php?c=usermanual
- licenza: MIT (stuffz; ryanlewis); documentazione ufficiale - licenza non specificata
- note: Variante V2 non associata a PID univoco nella fonte; non copiare il PID della prima generazione; batteria e polling da testare

### Da inserire nell'app
VID 3057, usage 0xFF05/0x0001, feature report 0E da 64 byte. Test su XE-S-L con VXD02. DPI non documentato.

---

## Lamzu

Licenza delle fonti: **da verificare**.

### Famiglia e apertura del dispositivo
Atlantis/Thorn; Maya X batteria. Maya X cavo 373E:001C, dongle 373E:001E, interface 2, usage 0xFFFF/0x0000, Feature Report ID 00.
Fonte: https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md; licenza: non trovato.

### Lettura/scrittura DPI, polling, batteria e modalità host
Batteria Maya X: TX significativo 00 00 02 02 00 83; RX A1 00 02 02 00 83 CC PP (CC stato di carica, PP percentuale). Padding completo non trovato. DPI/polling Atlantis/Thorn: implementazione Rust, byte completi non verificati.

Fonte: https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md; licenza: non trovato.

### Modelli con comandi documentati (0)

### Da inserire nell'app
Maya X: interfaccia 2, usage 0xFFFF/0x0000, feature report ID 00. Padding completo non trovato. Licenza HaloBattery non verificata: attendere o contattare l'autore.

---

## Razer

Licenza delle fonti: **GPL/AGPL**.

### Famiglia e apertura del dispositivo
Razer HID++-independent vendor protocol. Report di configurazione tradizionale da 90 byte; identificare collection vendor corretta prima di usare feature/output.
Fonte: https://github.com/openrazer/openrazer; licenza: GPL-2.0-or-later.

### Lettura/scrittura DPI, polling, batteria e modalità host
GET DPI: classe 0x04 comando 0x85; SET DPI 0x04/0x05; GET/SET polling legacy 0x00/0x85 e 0x00/0x05; batteria 0x07/0x80. Sono identificatori classe/comando, non frame completi.

Fonte: https://github.com/openrazer/openrazer; licenza: GPL-2.0-or-later.

### Modelli con comandi documentati (7)

**Razer Diamondback Chroma** - TESTATO da utenti
- VID:PID cavo: 1532:004C
- Usage: 0x0001/0x0002 (interfaccia controllo MI_00 nel test)
- Interfaccia: 0
- lettura DPI: Feature class=04 id=85; risposta X/Y uint16 BE (offset da verificare sul modello)
- scrittura DPI: Feature class=04 id=05; args [store 01; X uint16 BE; Y uint16 BE]
- lettura polling: Feature class=00 id=85; divisore 1000 (125=08;500=02;1000=01)
- scrittura polling: Feature class=00 id=05; args [08/02/01] per 125/500/1000 Hz
- fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/docs/razer-testing.md
- licenza: AGPL-3.0 (OpenMouse; solo documentazione)
- note: Report Razer 90 byte; transazione e indice USB dipendono da modello e comando; verificare checksum e response status. Non dedurre persistenza dal solo read-back. Test effettuato tramite bridge nativo Windows, non solo codice.

**Razer Viper Mini** - TESTATO da utenti
- VID:PID cavo: 1532:008A
- Usage: 0x0001/0x0002 (interfaccia controllo MI_00 nel test)
- Interfaccia: 0
- lettura DPI: Feature class=04 id=85; risposta X/Y uint16 BE (offset da verificare sul modello)
- scrittura DPI: Feature class=04 id=05; args [store 01; X uint16 BE; Y uint16 BE]
- lettura polling: Feature class=00 id=85; divisore 1000 (125=08;500=02;1000=01)
- scrittura polling: Feature class=00 id=05; args [08/02/01] per 125/500/1000 Hz
- fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/docs/razer-testing.md
- licenza: AGPL-3.0 (OpenMouse; solo documentazione)
- note: Report Razer 90 byte; transazione e indice USB dipendono da modello e comando; verificare checksum e response status. Non dedurre persistenza dal solo read-back.

**Razer Cobra** - TESTATO da utenti
- VID:PID cavo: 1532:00A3
- Usage: 0x0001/0x0002 (interfaccia controllo MI_00 nel test)
- Interfaccia: 0
- lettura DPI: Feature class=04 id=85; risposta X/Y uint16 BE (offset da verificare sul modello)
- scrittura DPI: Feature class=04 id=05; args [store 01; X uint16 BE; Y uint16 BE]
- lettura polling: Feature class=00 id=85; divisore 1000 (125=08;500=02;1000=01)
- scrittura polling: Feature class=00 id=05; args [08/02/01] per 125/500/1000 Hz
- fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/docs/razer-testing.md
- licenza: AGPL-3.0 (OpenMouse; solo documentazione)
- note: Report Razer 90 byte; transazione e indice USB dipendono da modello e comando; verificare checksum e response status. Non dedurre persistenza dal solo read-back. 100–8500 DPI; polling 125/500/1000 testati; illuminazione class 0F/02.

**Razer Viper V2 Pro** - TESTATO da utenti
- VID:PID cavo: 1532:00A5
- dongle: 1532:00A6
- Usage: 0x0001/0x0002 (interfaccia controllo MI_00 nel test)
- Interfaccia: 0
- lettura DPI: Feature class=04 id=85; risposta X/Y uint16 BE (offset da verificare sul modello)
- scrittura DPI: Feature class=04 id=05; args [store 01; X uint16 BE; Y uint16 BE]
- lettura polling: Feature class=00 id=85; divisore 1000 (125=08;500=02;1000=01) | esteso class=00 id=C0, divisore 8000 su ricevitore compatibile
- scrittura polling: Feature class=00 id=05; args [08/02/01] per 125/500/1000 Hz | esteso class=00 id=40, args [00; divisore 8000]
- batteria: Feature class=07 id=80; livello uint8/255 *100; ricarica class=07 id=84 solo se implementata
- fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/docs/razer-testing.md
- licenza: AGPL-3.0 (OpenMouse; solo documentazione)
- note: Report Razer 90 byte; transazione e indice USB dipendono da modello e comando; verificare checksum e response status. Non dedurre persistenza dal solo read-back.

**Razer Naga V2 Pro** - TESTATO da utenti
- VID:PID cavo: 1532:00A7
- dongle: 1532:00A8
- Usage: 0x0001/0x0002 (interfaccia controllo MI_00 nel test)
- Interfaccia: 0
- lettura DPI: Feature class=04 id=85; risposta X/Y uint16 BE (offset da verificare sul modello)
- scrittura DPI: Feature class=04 id=05; args [store 01; X uint16 BE; Y uint16 BE]
- lettura polling: Feature class=00 id=85; divisore 1000 (125=08;500=02;1000=01)
- scrittura polling: Feature class=00 id=05; args [08/02/01] per 125/500/1000 Hz
- batteria: Feature class=07 id=80; livello uint8/255 *100; ricarica class=07 id=84 solo se implementata
- fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/docs/razer-testing.md
- licenza: AGPL-3.0 (OpenMouse; solo documentazione)
- note: Report Razer 90 byte; transazione e indice USB dipendono da modello e comando; verificare checksum e response status. Non dedurre persistenza dal solo read-back. Polling stock 125/500/1000 Hz; non assumere 8K via Dock Pro.

**Razer DeathAdder V3 Pro** - TESTATO da utenti
- VID:PID cavo: 1532:00B6
- dongle: 1532:00B7
- Usage: 0x0001/0x0002 (interfaccia controllo MI_00 nel test)
- Interfaccia: 0
- lettura DPI: Feature class=04 id=85; risposta X/Y uint16 BE (offset da verificare sul modello)
- scrittura DPI: Feature class=04 id=05; args [store 01; X uint16 BE; Y uint16 BE]
- lettura polling: Feature class=00 id=85; divisore 1000 (125=08;500=02;1000=01)
- scrittura polling: Feature class=00 id=05; args [08/02/01] per 125/500/1000 Hz
- batteria: Feature class=07 id=80; livello uint8/255 *100; ricarica class=07 id=84 solo se implementata
- fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/docs/razer-testing.md
- licenza: AGPL-3.0 (OpenMouse; solo documentazione)
- note: Report Razer 90 byte; transazione e indice USB dipendono da modello e comando; verificare checksum e response status. Non dedurre persistenza dal solo read-back. Test con Node hidapi su firmware 2.1: il polling legacy modifica la frequenza misurata; DPI e polling non sempre persistono dopo spegnimento.

**Razer Viper V3 Pro** - TESTATO da utenti
- VID:PID cavo: 1532:00C0
- dongle: 1532:00C1
- Usage: 0x0001/0x0002 (interfaccia controllo MI_00 nel test)
- Interfaccia: 0
- lettura DPI: Feature class=04 id=85; risposta X/Y uint16 BE (offset da verificare sul modello)
- scrittura DPI: Feature class=04 id=05; args [store 01; X uint16 BE; Y uint16 BE]
- lettura polling: Feature class=00 id=85; divisore 1000 (125=08;500=02;1000=01) | esteso class=00 id=C0, divisore 8000 su ricevitore compatibile
- scrittura polling: Feature class=00 id=05; args [08/02/01] per 125/500/1000 Hz | esteso class=00 id=40, args [00; divisore 8000]
- batteria: Feature class=07 id=80; livello uint8/255 *100; ricarica class=07 id=84 solo se implementata
- fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/docs/razer-testing.md
- licenza: AGPL-3.0 (OpenMouse; solo documentazione)
- note: Report Razer 90 byte; transazione e indice USB dipendono da modello e comando; verificare checksum e response status. Non dedurre persistenza dal solo read-back.

### Da inserire nell'app
Nessun driver: la fonte è GPL/AGPL. L'app riconosce il modello (database) e mostra "recognized".

---

## Corsair

Licenza delle fonti: **GPL/AGPL**.

### Famiglia e apertura del dispositivo
NIGHTSWORD RGB e SLIPSTREAM/Bragi. NIGHTSWORD 1B1C:1B5C, MI_01, usage 0xFFC2/0x04, feature report ID 0, payload 64 byte.
Fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/captures/corsair-nightsword/PROTOCOL.md; licenza: AGPL-3.0.

### Lettura/scrittura DPI, polling, batteria e modalità host
GET DPI TX 0E 13 02 00; RX 0E 13 02 00 02 09 60 09 60... (2400 DPI; big endian). SET DPI 07 13 [D0|stage] 00 [independent] [X_LO X_HI Y_LO Y_HI] [R G B]; little endian. GET polling 0E 01 00, dato al byte 16; SET 07 0A 00 00 [interval_ms] (scrittura NON testata).

Fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/captures/corsair-nightsword/PROTOCOL.md; licenza: AGPL-3.0.

### Modelli con comandi documentati (2)

**NIGHTSWORD RGB** - TESTATO da utenti
- VID:PID cavo: 1B1C:1B5C
- Usage: 0xFFC2/0x04 - MI_01 - feature 64 B - ID 0
- Interfaccia: 1
- Report ID: 0x00
- Dimensione: 64 byte
- lettura DPI: 0E 13 02 00 -> RX[4]=stage RX[5:7]=X BE RX[7:9]=Y BE; 0E 13 Dn 00 stage preset
- scrittura DPI: 07 13 Dn 00 xy_independent X_LO X_HI Y_LO Y_HI R G B (LE, n=0..5); 07 13 02 00 stage (TESTATO)
- lettura polling: 0E 01 00 -> RX[16]=intervallo ms (1000/RX[16] Hz)
- scrittura polling: 07 0A 00 00 interval_ms (1/2/4/8); DOCUMENTATO, non verificato su hardware
- batteria: non applicabile - mouse cablato
- fonte: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/captures/corsair-nightsword/PROTOCOL.md | https://github.com/ckb-next/ckb-next/wiki/Supported-Mice
- licenza: AGPL-3.0 (OpenMouse, solo documentazione) | GPL-2.0 (ckb-next, solo documentazione)
- note: Firmware 3.41 Windows 11 test 2026-09-05/06; DPI GET BE, SET LE; letture + scritture DPI testate, polling write non testato; nessuna risposta SET, confermare con GET; stato live volatile; report ID 0 e 64 byte di payload; evitare comandi di bootloader | Specifiche DPI/polling dall’inventario preesistente ove presenti: non tutte riconfermate in questa ricerca

**DARK CORE RGB PRO SE** - TESTATO da utenti
- VID:PID cavo: 1B1C:1B7E
- dongle: 1B1C:1B7F
- batteria: SLIPSTREAM route 0x09 cmd 0x02 property 0x0F -> RX 01 02 00 [u16 LE decimi %]; test sul dongle 1B1C:1B7F
- fonte: https://github.com/ckb-next/ckb-next/wiki/Supported-Mice | https://github.com/HeyOkay/HaloBattery/blob/main/docs/protocols.md
- note: Lettura batteria TESTATA solo su dongle; frame 64 byte report ID 0; percentuale=u16/10; comandi DPI/polling per questo PID non verificati; confermare risposta due volte per collisione con altri software | Specifiche DPI/polling dall’inventario preesistente ove presenti: non tutte riconfermate in questa ricerca

### Da inserire nell'app
Nessun driver: la fonte è GPL/AGPL. L'app riconosce il modello (database) e mostra "recognized".

---

## Xtrfy / CHERRY XTRFY

Licenza delle fonti: **GPL/AGPL**.

### Famiglia e apertura del dispositivo
CHERRY XTRFY M64 Wireless. USB 046A:0332, dongle 046A:0330, interface 1 usage 0xFF02; Output/Input report ID 08, 16 byte payload (17 incluso ID).
Fonte: https://github.com/commitblyat/compx-mouse-tool/blob/main/PROTOCOL.md; licenza: GPL-3.0-or-later.

### Lettura/scrittura DPI, polling, batteria e modalità host
Payload [CMD STATUS ADDR_HI ADDR_LO LEN DATA0..9 CHECKSUM], checksum=(0x55-0x08-somma primi 15 byte) mod 256. Batteria cmd 04, RX payload[5]=%, [6]=carica. Polling flash addr 0000 GET 08 SET 07, codici 08/04/02/01 per 125/250/500/1000. Host cmd 03, write DPI NON testata.

Fonte: https://github.com/commitblyat/compx-mouse-tool/blob/main/PROTOCOL.md; licenza: GPL-3.0-or-later.

### Modelli con comandi documentati (1)

**M64 Wireless** - TESTATO da utenti
- VID:PID cavo: 046A:0332
- dongle: 046A:0330
- Interfaccia: 1
- Report ID: 0x08
- Dimensione: 17 byte, ID incluso
- lettura DPI: Output report 0x08, cmd 0x08 read flash addr 0x000C, 8x4 byte stage; vedi PROTOCOL.md; il valore stage cambiato fisicamente può non essere sincronizzato
- scrittura DPI: Output report 0x08 cmd 0x07 write flash addr 0x000C (8x4 byte), cmd 0x03 host control; scrittura valori DPI NON testata
- lettura polling: Output report 0x08 cmd 0x08 read flash indirizzo 0x0000, valore 1=1000 2=500 4=250 8=125
- scrittura polling: Output report 0x08 cmd 0x03 host on; cmd 0x07 write flash addr 0x0000, 2 byte value + complemento 0x55-value; host off; TESTATO firmware 2.00
- batteria: Output report 0x08 cmd 0x04; Input report 0x08 byte5 percentuale byte6 flag in carica; TESTATO firmware 2.00
- modalità host: si
- persistenza: memoria mouse
- fonte: https://github.com/commitblyat/compx-mouse-tool/blob/main/PROTOCOL.md | https://www.cherry.de/it-it/prodotto/m64-wireless
- licenza: GPL-3.0-or-later (repository); documentazione produttore proprietaria
- note: TESTATO: letture, polling read/write, batteria, debounce, LOD, motion sync, bottoni e stage. NON TESTATO: scrittura valori DPI. USB IF 1; 16-byte payload + report ID 0x08 = 17-byte buffer per HID API. Checksum 0x55-(0x08+sum(payload[0..14])) mod 256. Firmware 2.00; scritture su flash, backup prima e read-back. Non inviare pair 05/reset 09/DFU 0D. DPI fisico può non aggiornare lo stage memorizzato.

### Da inserire nell'app
Nessun driver: la fonte è GPL/AGPL. L'app riconosce il modello (database) e mostra "recognized".

---

## Keychron

Licenza delle fonti: **GPL/AGPL**.

### Famiglia e apertura del dispositivo
M5 8K — batteria. 3434:D048 USB, 3434:D028 dongle, interface 4, 64 byte, request report B3, response B4.
Fonte: https://github.com/csutcliff/keychron-battery-dkms; licenza: GPL-2.0-only.

### Lettura/scrittura DPI, polling, batteria e modalità host
GET batteria: report B3, comando 06, risposta report B4, comando 06, batteria byte 20 secondo driver; frame completo non trovato. DPI e polling non trovati da fonti ammissibili.

Fonte: https://github.com/csutcliff/keychron-battery-dkms; licenza: GPL-2.0-only.

### Modelli con comandi documentati (1)

**Keychron M5 Wireless** - TESTATO da utenti
- VID:PID cavo: 3434:D048 (M5 8K, non tutte le M5)
- dongle: 3434:D028 (Ultra-Link 8K, condiviso)
- Interfaccia: 4
- Report ID: 0xB3 richiesta / 0xB4 risposta
- Dimensione: 64 byte
- batteria: Feature Report ID B3, CMD 06, 64 byte; risposta Input Report ID B4, CMD 06, 64 byte, byte 20 livello %; padding/byte restanti non verificati
- fonte: https://github.com/csutcliff/keychron-battery-dkms | https://github.com/SPDG/inputscout
- licenza: GPL-2.0-only (keychron-battery-dkms); licenza InputScout da verificare
- note: M5 8K + Ultra-Link; battery test/support in DKMS e output reale InputScout. Non compatibilità garantita per M5 1K. Lettura/scrittura DPI/polling non trovata da fonte ammessa. Interface 4.

### Da inserire nell'app
Nessun driver: la fonte è GPL/AGPL. L'app riconosce il modello (database) e mostra "recognized".

---

## Marchi senza protocollo utilizzabile (solo riconoscimento)
Zowie, MSI, Gigabyte / Aorus, Microsoft, Endgame Gear, Cooler Master, ASUS ROG, Glorious, Finalmouse, Roccat. Per Glorious, ASUS e Roccat esistono frame parziali (gloryctl MIT, libratbag MIT) ma non completi: servono altre fonti prima di scrivere un driver.

---

## VERIFICA ALLA FONTE (31/10/2026, repository clonati e letti)

Controllati: stuffz/mouse-battery-monitor (VAXEE), ryanlewis/mousebatt (VAXEE, Pulsar), santeri3700/hyperx_pulsefire_dart_reverse_engineering, prodmutant/attack-shark-x3. **Non ancora letti:** rivalcfg (SteelSeries), Lamzu, Pulsar completo, OpenMouse.

### VAXEE - CONFERMATO (MIT)
- VID 0x3057, usage page 0xFF05, usage 0x01, **feature report** ID 0x0E, buffer 64 byte (ID + 63).
- Invio: `0E A5 <cmd> <rw> <len> <dati...>` con rw 1 = lettura, 2 = scrittura. Risposta: stesso formato, byte 2 = eco del comando (deve essere diverso da 0), dati dal byte 5.
- Batteria: cmd `0x0B` lettura, `resp[5] * 5` = percentuale (0-20). Carica: cmd `0x10`, `resp[5] != 0` = in carica.
- Polling: cmd `0x07`, `resp[5]` = indice 1..5 = 500/1000/2000/4000/8000 Hz. Dopo la scrittura il collegamento si rinegozia: ignora richieste per circa 300 ms. Cmd `0x08` = modalità di tracciamento (le modalità "Standard" limitano a 1 kHz). Tempi: invia, attendi 100 ms, leggi.
- PID dongle: 0x1001, 0x1002 (4K), 0x0005, 0x2001 (4K dual-track). PID mouse USB: XE Wireless 0x1003, ZYGEN NP-01S 0x1004, AX Wireless 0x1005, ZYGEN NP-01 0x1006, XE-S 0x1007, XE-S-L 0x1008, x NINJUTSO Sora 0x1009, E1 0x1010, ZYGEN NP-01S V2 0x1011, XE V2 0x1012, ZYGEN NP-01S Ergo 0x1013.
- Provato su XE-S-L con dongle VXD02 (PID 0x2001). DPI non documentato.

### Pulsar - CONFERMATO IN PARTE (MIT, mousebatt)
- Famiglia X3, cavo o dongle 8K: **output report da 17 byte** `08 <cmd> ... <ck>`, con `ck = 0x55 - somma(byte 0..15)`. Risposta = input report con eco del comando.
- Cmd `0x04` (energia): risposta byte 6 = batteria %, byte 7 = carica, byte 8-9 = tensione in mV (big endian). Cmd `0x01` info: byte 12 = tipo di collegamento.
- Cmd `0x07`/`0x08` = scrittura/lettura memoria: byte 3-4 = indirizzo, byte 5 = lunghezza, dati dal 6. Un valore a un byte si salva come coppia `(v, 0x55 - v)`. Polling all'indirizzo 0x0000 come periodo (0x01 = 1 kHz ... 0x40 = 8 kHz).
- Pulsar X2A v3 / XS-1: feature report non numerati da 64 byte su usage 0xFFFF/0x01, interfaccia 3 (la 2 non risponde). Checksum a 16 bit little endian. Batteria `08 81 01` (byte 6), carica `08 83 01`, polling `01 89 02` (byte 7, bitmask 0x01 = 125 Hz ... 0x40 = 8 kHz), scrittura `01 09 02`.
- Provato su X3 LHD CrazyLight (cavo e dongle 8K) e su X2A v3 eS Mini (PID cavo 0x3408, dongle 0x5403). Via dongle ogni richiesta è un giro radio: il mouse in sleep non risponde.

### HyperX Pulsefire Dart - CONFERMATO (CC0)
- VID 0x0951: 0x16E1 wireless, 0x16E2 cavo. Usage page 0xFF13 (cavo) / 0xFF00 (wireless), interfaccia 1 (cavo) / 2 (wireless).
- Polling: `D0 00 00 01 <v>` con v = 00/01/02/03 per 125/250/500/1000 Hz.
- DPI: `D3 02 <profilo> 02 <lo> <hi>`, valore = DPI / 50 in 16 bit little endian (16000 DPI = 0x0140 -> `40 01`). Profili 0-4. Maschera dei profili attivi `D3 01 ...` a 5 bit.
- Modello vecchio: più utile come esempio che come mercato.

### Attack Shark X3 - CORRETTO (MIT) - **diverso da quanto scritto da ChatGPT**
- VID 0x1D57, PID 0xFA60 (ricevitore) / 0xFA61 (cavo). Si configura solo su MI_02 COL04 (usage page 0x000B, feature report da 262 byte), con `HidD_SetFeature`.
- **La lettura NON è supportata**: il driver ufficiale non rilegge mai le impostazioni, le salva in un file locale e le riscrive ogni volta. Quindi per l'X3 non si può mostrare il DPI o il polling attuale: l'app dovrebbe tenere il proprio stato.
- Batteria: report **di input** da 5 byte sulla collezione usage page 0x000A: `03 10 <livello> 01 <?>`. Il byte è probabilmente una tensione in sedicesimi di volt (0x40 = 4,00 V), ma il documento stesso dice che l'interpretazione non è certa (potrebbe essere una percentuale grezza). **Non mostrare una percentuale sicura.**
- Polling: report 0x06 da 9 byte, valore come coppia `(v, 0xFF - v)`. DPI: report 0x04 da 56 byte con `raw = dpi/50 - 1` e checksum a 16 bit big endian.
- Il protocollo è stato ricavato da catture del programma ufficiale (strumentando le sue chiamate HID). La licenza del repository è MIT, ma la regola di ChatGPT escludeva sequenze ricavate dal software ufficiale: valutare prima di usare.
- Report 0x0B del X11 può scollegare il dongle: non inviarlo.

### Cosa cambia per i driver
- **VAXEE** è il candidato migliore: protocollo chiaro, MIT, batteria e polling con byte completi.
- **Pulsar X3** va bene per batteria (e polling con cautela).
- **HyperX Dart**: documentato, ma il mouse è vecchio.
- **Attack Shark X3**: solo scrittura, niente lettura, batteria incerta. Non è il primo da fare.

---

## SECONDO FILE DI RICERCA (31/10/2026) - cosa contiene davvero

- **Frame completi: solo 11**, di cui 4 Attack Shark X3 (scrittura polling, verificati alla fonte), 2 VAXEE (batteria e carica, verificati alla fonte), 2 Xtrfy (GPL, solo riferimento) e 3 Glorious (solo selettori di lettura, non scrivono). SteelSeries, Lamzu, Pulsar, ASUS e Roccat restano senza frame completi: ChatGPT stesso dice di non averli inventati.
- **Attack Shark X3**: i valori di polling `VV` 08/04/02/01 per 125/250/500/1000 Hz e il checksum `0xFF - VV` sono confermati. Sono parte di una sequenza (0C, 04, 05, 06, 08): non vanno inviati da soli.
- **ID USB nuovi: 11** (Pulsar 3710:xxxx, Glorious Model I 2 093A:821D, Roccat Kone AIMO 16K 1E7D:2E2C, Endgame OP1 8K Purple Frost 3367:1976, ASUS Impact III 0B05:1ACE). Alcuni sono ricevitori condivisi: non identificano un mouse da soli. Aggiunti al database.
- **Comandi da non inviare**: Glorious (banco macro 0x30 e scrittura 0x04 da 520 byte), SteelSeries Aerox 5 (`11 00` = salvataggio), Xtrfy (0x05/0x09/0x0D/0x13 pairing/reset/firmware, 0x07 flash), ASUS (03 50, 51 31 0A), Attack Shark X11 (report 0x0B), Corsair (07 02 AA = bootloader).
- **Prove di test (26)**: le più solide sono Pulsar X2 CrazyLight (letture e scritture confermate su hardware), Feinmann (cattura USB + video), Lamzu Maya X (batteria, release 1.12.0), VAXEE XE-S-L (polling e batteria).


---

# Terzo giro di ricerca (31/10/2026) - aggiunte e correzioni

**Stato di verifica:** ho controllato struttura e coerenza interna dei file (lunghezze dei frame, URL presenti, licenze dichiarate, regola SteelSeries Rival: 1000/500/250/125 Hz = 01/02/03/04). Le pagine GitHub **non sono raggiungibili dalla mia sessione**, quindi i dati non sono verificati alla fonte: ogni frame resta DOCUMENTATO finche' non lo prova un utente con quel mouse.

## Frame TX completi (7)

| Marchio | Modello | Funzione | Report | Byte | Stato | Licenza |
|---|---|---|---|---|---|---|
| SteelSeries | Aerox 5 cablato 1038:1850 | Scrittura polling 125 Hz | ID 00, 3 B | `00 2B 04` | DOCUMENTATO | WTFPL |
| SteelSeries | Aerox 5 cablato 1038:1850 | Scrittura polling 250 Hz | ID 00, 3 B | `00 2B 03` | DOCUMENTATO | WTFPL |
| SteelSeries | Aerox 5 cablato 1038:1850 | Scrittura polling 500 Hz | ID 00, 3 B | `00 2B 02` | DOCUMENTATO | WTFPL |
| SteelSeries | Aerox 5 cablato 1038:1850 | Scrittura polling 1000 Hz | ID 00, 3 B | `00 2B 01` | DOCUMENTATO | WTFPL |
| Glorious | Model O/O-/D/D- cablati protocollo SinoWealth legacy | Selezionare lettura profili DPI e polling | ID 05, 6 B | `05 11 00 00 00 00` | DOCUMENTATO | MIT |
| Roccat | Kone Pure e Kone EMP protocollo legacy libratbag | Selezionare lettura profilo DPI/polling 0 | ID 04, 3 B | `04 00 80` | DOCUMENTATO | MIT |
| SteelSeries | Rival 3 Wireless dongle 1038:1830 | Lettura batteria | ID 00, 64 B | `00 AA 01 00 00 00 00 00 00 00 00 00 00 00 00 00 ...` | DOCUMENTATO | MIT |

Note: SteelSeries Aerox 5 **solo cablato** (`1038:1850`), 4 scritture polling `00 2B 04/03/02/01` (125/250/500/1000), senza read-back documentato. Glorious (`05 11 00 00 00 00` poi get feature `04`, 520 B) e Roccat (`04 00 80` poi get `06`, 43 B) sono **sola lettura**: non scrivere mai il report di configurazione. Rival 3 Wireless: le fonti si contraddicono sugli offset, da non usare senza test.

## VID:PID nuovi (16)

| Marchio | Modello | Cavo | Dongle |
|---|---|---|---|
| SteelSeries | Aerox 3 Wireless CS2 Dragon Lore | 1038:187A | 1038:1878 |
| SteelSeries | Aerox 5 Wireless Destiny Edition | 1038:185E | 1038:185C |
| SteelSeries | Aerox 5 Wireless Diablo IV Edition | 1038:1862 | 1038:1860 |
| SteelSeries | Aerox 9 Wireless variante PID 1876/1874 | 1038:1876 | 1038:1874 |
| SteelSeries | Rival 5 Destiny Edition | 1038:183E | non trovato |
| SteelSeries | Rival 3, revisione con PID 184C | 1038:184C | non trovato |
| Roccat | Kone Pure (revisione USB 1E7D:2DC2; non attribuire a Owl-Eye o Ultra) | 1E7D:2DC2 | non trovato |
| Roccat | Kone Pure (revisione USB 1E7D:2DBE; non attribuire a Owl-Eye o Ultra) | 1E7D:2DBE | non trovato |
| Roccat | Kone Pure (revisione USB 1E7D:2DCB; non attribuire a Owl-Eye o Ultra) | 1E7D:2DCB | non trovato |
| Microsoft | IntelliMouse Explorer 3.0 | 045E:0047 | non trovato |
| Microsoft | IntelliMouse Explorer 4.0 | 045E:0095 | non trovato |
| Microsoft | Trackball Explorer | 045E:0024 | non trovato |

Non attribuire un PID a un modello dal solo nome simile.

## Prove su hardware reale dichiarate dalle fonti (19)

| Marchio | Modello | Funzione | Stato | Licenza |
|---|---|---|---|---|
| SteelSeries | Aerox 3 Wireless 1038:1838 | Batteria (lettura) + charging, risposta D2 | TESTATO | GPL-2.0-only |
| SteelSeries | Aerox 9 Wireless 1038:1858 | Batteria (lettura) 15% come SteelSeries GG | TESTATO | MIT |
| SteelSeries | Aerox 5 cablato 1038:1850 | Scrittura polling configurata nel driver | DOCUMENTATO | WTFPL |
| SteelSeries | Rival 3 Wireless 1038:1830 | Batteria lettura: varianti decode | DOCUMENTATO | MIT |
| SteelSeries | Rival 5 1038:183C / Rival 5 Destiny 183E | Configurazione dichiarata rivalcfg | DOCUMENTATO | WTFPL |
| Lamzu | Atlantis e Thorn | Lettura/scrittura profili DPI e polling dichiarate | TESTATO | MIT OR Apache-2.0 |
| Lamzu | Maya X dongle 8K 373E:001E | Batteria e indicatore carica | TESTATO | MIT |
| Lamzu | Maya X | Batteria e ricarica dal Windows tray | TESTATO | MIT |
| Pulsar | X2 CrazyLight Wireless 3710:5406 | DPI e polling lettura/scrittura | TESTATO | MIT |
| Pulsar | X2 CrazyLight Wired 3710:3414 | DPI e polling lettura/scrittura | TESTATO | MIT |
| Pulsar | X2 Wireless (Areson) 25A7:FA7B/FA7C | DPI/polling lettura/scrittura | TESTATO | MIT |
| Pulsar | Feinmann F01 Noctua 3710:7507/5504 | DPI/polling e letture | TESTATO | MIT |
| Pulsar | X3 | DPI e batteria; polling write fallisce | TESTATO | non trovato |
| Pulsar | X2 V2 Mini | DPI polling batteria | TESTATO | MIT |
| Glorious | Model O cablato (protocollo legacy) | Lettura Feature 04 config DPI/poll | TESTATO | MIT |
| Roccat | Kone Pure / Kone EMP legacy | Lettura dei profili DPI/polling | DOCUMENTATO | MIT |
| ASUS ROG | Gladius III Wireless AimPoint 0B05:1A72 | DPI read/write e polling read/write, batteria | TESTATO | GPL-3.0 |
| Corsair | NIGHTSWORD RGB 1B1C:1B5C | DPI read/write e polling read | TESTATO | AGPL-3.0 |
| Razer | DeathAdder V4 Pro dongle 1532:00BF | Batteria read (0xAB => 67%) | TESTATO | MIT |

## Da NON inviare mai (10)

| Marchio | Famiglia | Comando | Motivo |
|---|---|---|---|
| SteelSeries | Aerox 3 WL e altri | Save/commit alla flash; dongle pairing | Modifica persistente e associazione: escludere dai frame operativi; solo riferimento GPL |
| SteelSeries | Rival 3 Gen2 Wireless | Scrittura DPI su firmware non testato | Issue 273 riferisce il mouse reso inutilizzabile dopo modifica sensibilità; fix successivo |
| Lamzu | Atlantis/Thorn | Scrittura profili e impostazioni persistenti senza backup | Possibilità di sovrascrivere configurazioni |
| Pulsar | X3/CrazyLight/Feinmann | DFU; firmware update; profili non identificati | Rischio brick o cambio identificativo durante la scrittura |
| Glorious | Model O/D | Macro bank write fuori limite | Autore gloryctl dichiara di aver brickato un mouse cercando il limite |
| Roccat | Kone Pure/EMP | Feature report settings 0x06 in scrittura; macro 0x08; commit profilo | Scrittura persistente di dati hardware/profili, non equivalente al read-only |
| ASUS ROG | Omni e dongle | Pairing/firmware update e commit profili | Re-enumerazione o pairing perso; GPL solo riferimento |
| Razer | Tutte le generazioni | Bootloader/DFU e update firmware, reset | Non devono comparire nei frame di configurazione |
| Corsair | NXP e Bragi | Bootloader e flash save | Esclusi; note sulla memoria e modalità software |
| Endgame Gear | OP1/XM2 | Protocollo derivato esclusivamente da decompilazione software ufficiale | Fonte vietata: non usarne byte/report/payload |

## Cosa implementare per primo (consigliato)
1. **Pulsar** (MIT, provato su 4 modelli): DPI e polling in lettura, poi scrittura dopo test utente. Famiglie Sonix 64 B e Nordic 17 B distinte dal PID; i frame completi vanno ricavati dal repository `packerlschupfer/pulsar-mouse-linux`.
2. **SteelSeries Aerox 5 cablato**: polling con i 4 frame sopra; batteria Aerox 9 `1038:1858` e Rival 3 solo dopo test.
3. **Glorious Model O/D legacy** e **Roccat Kone Pure/EMP**: solo lettura (DPI, polling) con i selettori sopra.
4. **Lamzu Maya X** (`373E:001E`): solo batteria; il frame completo non e' certificato.
5. Razer, Corsair, ASUS ROG, Xtrfy, Keychron: **solo documentazione** (fonti GPL/AGPL), nessun codice copiato.

## Lacune ancora aperte
Frame DPI/polling completi di Lamzu, Pulsar e ASUS ROG; read-back delle SteelSeries wireless; Roccat recenti; protocolli pubblici verificati per Zowie, MSI, Gigabyte/AORUS, Microsoft e Cooler Master. Endgame Gear escluso (fonti da decompilazione). L'elenco di ~770 modelli non era stato incollato nella richiesta: il confronto dei mancanti e' provvisorio.


---

# Quarto giro di ricerca (31/10/2026)

**Verifica:** i 3 frame Pulsar hanno lunghezza 17 B e checksum corretto (`byte16 = (0x55 - somma byte 0..15) mod 256`): controllato io OK. Il resto non e' verificabile alla fonte da qui. Il giro conferma che per Lamzu, ASUS ROG, SteelSeries wireless, Roccat e Glorious recenti **non esistono ancora frame completi verificabili**, e che Zowie, MSI, Gigabyte/AORUS, Microsoft e Cooler Master **non hanno una fonte pubblica verificata**.

## Pulsar X2 V2 Mini (famiglia Nordic) - sola lettura, fonte MIT (andrewrabert/python-pulsar-mouse-tool)

Protocollo diverso dai Sonix a 64 B. La fonte usa USB SET_REPORT Output `wValue=0x0208` con risposta Input da 17 B: **non e' provato che `hidapi.write()` funzioni su Windows**, serve test su un mouse reale. Nessuna pausa documentata: usare timeout in lettura.

| Funzione | Frame (17 B) | Risposta |
|---|---|---|
| Lettura polling e indice DPI attivo | `08 08 00 00 00 0A 00 00 00 00 00 00 00 00 00 00 3B` | RX[6] = registro 0x00 polling: 01=1000 Hz, 02=500, 04=250, 08=125; RX[10] = registro 0x04 stage DPI attivo; risposta 17 byte con eco indirizzo in RX[4], lunghezza in RX[5] |
| Lettura DPI preset 0/1 | `08 08 00 00 0A 0A 00 00 00 00 00 00 00 00 00 00 31` | RX[8..10] = registri 0x0C..0x0E stage0; RX[12..14] = registri 0x10..0x12 stage1; per stage raw[0]=raw[1], DPI=50*(raw[1]+1)+12800*((raw[2]&0x0F)>>2); RX[4] echo=0A, RX[5]=0A |
| Lettura DPI preset 2/3 | `08 08 00 00 14 0A 00 00 00 00 00 00 00 00 00 00 27` | RX[6..8]=registri 0x14..0x16 stage2; RX[10..12]=registri 0x18..0x1A stage3; stessa formula DPI; RX[4] echo=14, RX[5]=0A |

Non inviare: scrittura MEM_SET e restore/factory reset (presenti nel codice, esclusi).

## VID:PID (13 righe, quasi tutte conferme)

| Marchio | Modello | Cavo | Dongle | Stato | Nota |
|---|---|---|---|---|---|
| Attack Shark | X3 | 1D57:FA61 | 1D57:FA60 | TESTATO | già noto; riconfermato; Non estendere alle versioni Pro/Max |
| Attack Shark | X11 | 1D57:FA55 | 1D57:FA60 | TESTATO | già noto; riconfermato; Ricevitore con PID condiviso con X3; non identificare il modello dal solo dongle |
| HyperX | Pulsefire Dart | 0951:16E2 | 0951:16E1 | TESTATO | già noto; riconfermato; Kingston-era; USB wired / dongle distinti |
| VAXEE | XE-S-L Wireless | 3057:1008 | 3057:2001 | DOCUMENTATO | modello non incluso nelle 16 righe del catalogo precedente; Dongle VXD02 4K condivisibile; test polling/batteria sul mouse XE-S-L nel progetto mousebatt |
| VAXEE | XE Wireless | 3057:1003 | non trovato | DOCUMENTATO | già noto; riconfermato; Solo ID mouse diretto; non assegnare automaticamente un PID ricevitore |
| VAXEE | ZYGEN NP-01S Wireless | 3057:1004 | non trovato | DOCUMENTATO | già noto; riconfermato; Solo ID mouse diretto; non assegnare automaticamente un PID ricevitore |
| VAXEE | OUTSET AX Wireless | 3057:1005 | non trovato | DOCUMENTATO | già noto; riconfermato; Solo ID mouse diretto; non assegnare automaticamente un PID ricevitore |
| VAXEE | ZYGEN NP-01 Wireless | 3057:1006 | non trovato | DOCUMENTATO | già noto; riconfermato; Solo ID mouse diretto; non assegnare automaticamente un PID ricevitore |
| VAXEE | XE-S Wireless | 3057:1007 | non trovato | DOCUMENTATO | già noto; riconfermato; Solo ID mouse diretto; non assegnare automaticamente un PID ricevitore |
| VAXEE | E1 Wireless | 3057:1010 | non trovato | DOCUMENTATO | già noto; riconfermato; Solo ID mouse diretto; non assegnare automaticamente un PID ricevitore |
| VAXEE | ZYGEN NP-01S V2 Wireless | 3057:1011 | non trovato | DOCUMENTATO | già noto; riconfermato; Solo ID mouse diretto; non assegnare automaticamente un PID ricevitore |
| VAXEE | XE V2 Wireless | 3057:1012 | non trovato | DOCUMENTATO | già noto; riconfermato; Solo ID mouse diretto; non assegnare automaticamente un PID ricevitore |
| Xtrfy | CHERRY XTRFY M64 Wireless | 046A:0332 | 046A:0330 | TESTATO | già noto; riconfermato; Firmware 2.00, interfaccia 1; protocollo soggetto a GPL, frame non copiati |

Novita' reale: VAXEE XE-S-L Wireless (`3057:1008`, dongle `3057:2001`). I PID dei ricevitori sono condivisi (X3/X11, VAXEE 1K/4K): **non identificare il modello dal solo dongle**. Xtrfy M64 e' GPL: solo riferimento.

## Prove (16)

| Marchio | Modello | Funzione | Stato |
|---|---|---|---|
| Pulsar | X2 V2 Mini | DPI / polling lettura e scrittura (profilo), batteria | TESTATO |
| Pulsar | X2 CrazyLight, via 3710:5406 e 3710:3414 | DPI/polling lettura e scrittura | TESTATO |
| Pulsar | Feinmann F01 Noctua (3710:5504 / 3710:7507) | DPI/polling lettura e scrittura | TESTATO |
| Pulsar | X3 (3710:3409 via cavo, 3710:5402 ricevitore 1K) | Batteria, DPI, polling | TESTATO |
| Pulsar | X2A Medium Wired (3710:1404) | Profili, DPI, polling | TESTATO |
| Lamzu | Atlantis / Atlantis Mini Pro | Lettura/scrittura profili DPI e polling | TESTATO |
| Lamzu | Thorn | Profili DPI e polling | TESTATO |
| Lamzu | Maya X 8K, ricevitore 373E:001E | Batteria e carica | TESTATO |
| ASUS ROG | Keris Wireless AimPoint 0B05:1A66 (nuova revisione) | Identificazione e tentativo di lettura profili | TESTATO |
| Glorious | Model I 2 | Riconoscimento DPI/polling | TESTATO |
| Roccat | Kone Pure (vecchia generazione) | Lettura dati profilo, DPI e polling | DOCUMENTATO |
| SteelSeries | Aerox 5 cablato 1038:1850 | Scrittura DPI e polling | DOCUMENTATO |
| Attack Shark | X3 via cavo/ricevitore | Scrittura DPI e polling; batteria asincrona | TESTATO |
| HyperX | Pulsefire Dart 0951:16E2 / 16E1 | DPI/polling write, batteria e carica | TESTATO |
| VAXEE | XE-S-L / VXD02 4K | Batteria e polling read/write | TESTATO |
| Xtrfy | CHERRY XTRFY M64 Wireless | Batteria, polling e profili | TESTATO |

## Da NON inviare (12 categorie)

| Marchio | Famiglia | Categoria | Motivo |
|---|---|---|---|
| Pulsar | X2 V2 Mini | Scrittura MEM_SET della configurazione | Tocca parametri conservati sul mouse; esclusa anche se il comando è documentato |
| Pulsar | X2 V2 Mini | Restore/factory reset | Funzione di ripristino nel codice: fuori ambito |
| Pulsar | X3 / Xlite V4 | Memoria profili e commit senza prova di persistenza | Non inviare finché non esiste test read-back e procedura di recovery |
| Lamzu | Atlantis / Thorn | Scrittura diretta di profili memorizzati | Non usare i setter finché persistenza e firmware non sono verificati sulla revisione |
| ASUS ROG | Keris / Omni | Salvataggio profilo hardware e pairing ricevitore | Il ricevitore è condiviso; persistenza e pairing esclusi |
| SteelSeries | Aerox Wireless / Rival | Reset, pairing e save/commit | Operazioni non di sola lettura, rischi di disassociazione o configurazione non recuperabile |
| Glorious | Model O/D e generazioni nuove | Scrittura Feature Report di configurazione e flash | Il tool classic gestisce memoria persistente; non inviare con frame incompleto |
| Roccat | Kone Pure e recenti | Scrittura di profili e macro | Il backend storico usa report persistenti con checksum, non usabili su moderni |
| Attack Shark | X3/X11 | Pairing, DFU, reset e macro persistenti | Canali vendor con comandi potenzialmente irreversibili o disassociante |
| HyperX | Pulsefire Dart | Commit di memoria e update firmware | Impatto persistente; escluso per regola richiesta |
| VAXEE | Ricevitori 1K/4K | Firmware e pairing | Non inviare comandi non espressamente documentati di lettura |
| Xtrfy | M64 Wireless | Pairing/factory reset/DFU/scrittura flash | Il progetto vieta operazioni rischiose; fonte GPL solo riferimento |

## Mouse senza ID completi (411, confronto provvisorio)

La lista dei ~770 modelli non e' mai arrivata (segnaposto rimasto): il confronto e' fatto solo sui cataloghi storici. Elenco per marchio:

| Marchio | Voci |
|---|---|
| Zowie | 45 |
| ASUS ROG | 39 |
| SteelSeries | 37 |
| Glorious | 33 |
| Pulsar | 31 |
| Roccat | 26 |
| Finalmouse | 24 |
| Cooler Master | 22 |
| Lamzu | 21 |
| Microsoft | 19 |
| Gigabyte / AORUS | 18 |
| HyperX | 18 |
| MSI | 17 |
| Xtrfy | 17 |
| Endgame Gear | 16 |
| VAXEE | 15 |
| Attack Shark | 13 |

Elenco completo dei 411 nel file `mouse_mancanti_411.csv`.


---

# Quinto giro di ricerca (31/10/2026)

**Verifica:** controllati da me solo lunghezze e coerenza dei buffer; le pagine GitHub non sono raggiungibili dalla mia sessione. 6 frame **di sola lettura**: 2 Lamzu Maya X 8K e 4 SteelSeries batteria. Nessun VID:PID nuovo. Il confronto con la lista dei mouse non e' stato fatto (segnaposto rimasto di nuovo).

## Frame completi

### Lamzu Maya X 8K (dongle 8K) - Lettura batteria + stato carica
- Tipo: Feature TX e Feature RX; report ID 0x00; buffer hidapi **65 B** (verificati 65 B); stato **TESTATO**; licenza MIT
- TX: `00 00 00 02 02 00 83 00 00 00 00 00 00 00 00 00 00 00 00 00  ...` (resto zeri)
- Risposta: get_feature_report: 65 byte, RX[0]=00 report ID; verificare RX[1]=A1 e RX[6]=83; RX[7]==01 significa in carica, RX[8] percentuale (0..100); altri offset non interpretati.
- Tempi: 100 ms tra send_feature_report e get_feature_report (Rust); alternativa HaloBattery: fino a 10 tentativi con 50 ms ciascuno
- Fonte: https://github.com/Sheroune/lamzu-battery-monitory/blob/master/src/device.rs#L4-L65

### Lamzu Maya X 8K (USB cablato) - Lettura batteria + stato carica
- Tipo: Feature TX e Feature RX; report ID 0x00; buffer hidapi **65 B** (verificati 65 B); stato **DOCUMENTATO**; licenza MIT
- TX: `00 00 00 02 02 00 83 00 00 00 00 00 00 00 00 00 00 00 00 00  ...` (resto zeri)
- Risposta: get_feature_report: 65 byte, RX[0]=00 report ID; verificare RX[1]=A1 e RX[6]=83; RX[7]==01 significa in carica, RX[8] percentuale (0..100); altri offset non interpretati.
- Tempi: 100 ms tra send_feature_report e get_feature_report (Rust); alternativa HaloBattery: fino a 10 tentativi con 50 ms ciascuno
- Fonte: https://github.com/Sheroune/lamzu-battery-monitory/blob/master/src/device.rs#L4-L65

### SteelSeries Aerox 3 Wireless (dongle 2,4 GHz) - Lettura batteria + flag carica
- Tipo: Output TX, Input RX; report ID 0x00; buffer hidapi **64 B** (verificati 64 B); stato **DOCUMENTATO**; licenza MIT
- TX: `00 D2 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ...` (resto zeri)
- Risposta: Leggere fino a 64 byte via Input. Eliminare eventuale RX[0]==00 se presente; imporre eco D2; successivo byte b: carica=(b & 80)!=0, v=b & 7F; v=0 => dormiente/off; 1..21 => (v-1)*5%; v>21 => min(v,100)%.
- Tempi: Nessuna attesa fissa; timeout per lettura 100 ms, fino a 6 letture per 3 trasmissioni.
- Fonte: https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py#L140-L231; https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py#L394-L432

### SteelSeries Aerox 5 Wireless (dongle 2,4 GHz) - Lettura batteria + flag carica
- Tipo: Output TX, Input RX; report ID 0x00; buffer hidapi **64 B** (verificati 64 B); stato **DOCUMENTATO**; licenza MIT
- TX: `00 D2 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ...` (resto zeri)
- Risposta: Leggere fino a 64 byte via Input. Eliminare eventuale RX[0]==00 se presente; imporre eco D2; successivo byte b: carica=(b & 80)!=0, v=b & 7F; v=0 => dormiente/off; 1..21 => (v-1)*5%; v>21 => min(v,100)%.
- Tempi: Nessuna attesa fissa; timeout per lettura 100 ms, fino a 6 letture per 3 trasmissioni.
- Fonte: https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py#L140-L231; https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py#L394-L432

### SteelSeries Aerox 9 Wireless (dongle 2,4 GHz) - Lettura batteria + flag carica
- Tipo: Output TX, Input RX; report ID 0x00; buffer hidapi **64 B** (verificati 64 B); stato **TESTATO**; licenza MIT
- TX: `00 D2 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ...` (resto zeri)
- Risposta: Leggere fino a 64 byte via Input. Eliminare eventuale RX[0]==00 se presente; imporre eco D2; successivo byte b: carica=(b & 80)!=0, v=b & 7F; v=0 => dormiente/off; 1..21 => (v-1)*5%; v>21 => min(v,100)%.
- Tempi: Nessuna attesa fissa; timeout per lettura 100 ms, fino a 6 letture per 3 trasmissioni.
- Fonte: https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py#L140-L231; https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py#L394-L432

### SteelSeries Rival 3 Wireless (dongle 2,4 GHz) - Lettura batteria e carica (layout risposta controverso)
- Tipo: Output TX, Input RX; report ID 0x00; buffer hidapi **64 B** (verificati 64 B); stato **DOCUMENTATO**; licenza MIT
- TX: `00 AA 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ...` (resto zeri)
- Risposta: Leggere massimo 64 byte. Eventuale report ID 00 iniziale rimosso. Due layout NON risolti: [AA,livello,?,carica] -> livello RX[1],carica RX[3]; oppure [livello,?,carica] -> livello RX[0],carica RX[2]. Verificare 0<=livello<=100 e carica in 0..1 per layout senza eco. Nessun layout confermato su hardware dalle fonti.
- Tempi: 100 ms per lettura; 6 tentativi RX e massimo 3 richieste TX senza pausa fissa dichiarata.
- Fonte: https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py#L140-L231; https://github.com/HeyOkay/HaloBattery/blob/main/providers/steelseries.py#L394-L432

## Come usarli su Windows
- **Lamzu Maya X 8K** (`373E:001E` dongle, `373E:001C` USB): interfaccia 2, usage page `0xFFFF`. `send_feature_report` con 65 B (`00 00 00 02 02 00 83` + 58 zeri), attendi 100 ms, `get_feature_report` 65 B. Controlla `RX[1]=A1`, `RX[6]=83`; `RX[7]==01` = in carica, `RX[8]` = percentuale 0-100. E' il frame migliore del giro: sorgente Rust MIT con test su mouse reale.
- **SteelSeries Aerox 3/5/9 Wireless**: `write` di 64 B (`00 D2` + 62 zeri), leggi fino a 6 volte con timeout 100 ms, fino a 3 invii. Risposta con eco `D2`; byte successivo: bit `0x80` = in carica, resto = livello (0 = dormiente, 1..21 -> (v-1)*5 %, oltre 21 = percentuale diretta). Testato solo su Aerox 9.
- **SteelSeries Rival 3 Wireless**: `00 AA 01` + 61 zeri. Il layout della risposta e' **ancora controverso** (livello al byte 1 e carica al byte 3, oppure livello al byte 0 e carica al byte 2): non abilitare senza test su un Rival 3 vero.
- Trasporti dei frame dei giri precedenti: Aerox 5 cablato = Output report; Glorious Model O/D e Roccat Kone Pure = Feature report; Pulsar X2 V2 Mini = USB control transfer SET_REPORT (non equivalente a `hid_write` su Windows: da provare).

## Da NON inviare (aggiunte)

| Marchio | Dispositivo | Operazione | Motivo |
|---|---|---|---|
| Lamzu | Atlantis/Thorn | Scritture di profilo su memoria persistente | Possono modificare valori persistenti; non comprese in frame di lettura sicuri |
| SteelSeries | Aerox 5 | Comandi di salvataggio memoria e modifiche profili | Non inoltrare il commit firmware/memoria come comando DPI ordinario |
| ASUS ROG | ROG tutti | Scritture di memoria profilo / DFU / firmware | Non inclusi in frame operativi senza verifica per PID, firmware, risposte |
| Pulsar | X2/X3/Feinmann | Flash, commit profilo, reset, pairing e scritture ai registri ignoti | Potrebbero modificare configurazione persistente o rendere instabile il dispositivo |
| Roccat | Kone Pure e recenti | Scritture di profili/firmware | È un protocollo con memoria hardware e diversi report; nessuna prova su generazioni recenti |
| Glorious | Model O/D | Commit configurazione e scritture a flash | I Feature Report lunghi possono modificare configurazione persistente |
| SteelSeries | Wireless generici | Pairing/DFU/reset/firmware | Non confondere le richieste batteria con i report di gestione del ricevitore |
| Lamzu | Maya X 8K | Probing di comandi Feature diversi dal comando batteria documentato | Il solo Feature Report 0x83 è sufficientemente documentato per batteria; altri byte vietati |

## Lacune ancora aperte (37 righe in non_trovato.csv)
DPI/polling completi per Lamzu Atlantis/Thorn, ASUS ROG (Gladius III, Keris, Chakram, Harpe), Pulsar Sonix 64 B e X2 CrazyLight/Feinmann/X3, Roccat recenti, Glorious recenti; read-back delle SteelSeries wireless; nessuna fonte verificata per Zowie, MSI, Gigabyte/AORUS, Microsoft, Cooler Master. Dettaglio in `non_trovato_5.csv`.


---

# Sesto giro di ricerca (31/10/2026)

**Verifica:** ho letto io i sorgenti che mi hai incollato (lamzu-cfg `hid.rs`, libratbag `asus.c`/`asus.h`, patch kernel Rival 3). I 14 frame hanno tutti 17 B e checksum `(0x55 - somma byte 0..15) mod 256` corretto: **OK**. Le pagine GitHub restano non raggiungibili dalla mia sessione. Nessun VID:PID nuovo. Lista mouse ancora mancante (segnaposto).

## Protocollo Nordic a 17 byte: Lamzu Atlantis/Thorn e Pulsar X2 CrazyLight/Feinmann Noctua

Stesso protocollo per tutti: Output report, ID `08`, 17 byte, risposta Input da 17 byte con stessa struttura e stesso checksum (`0x55 - somma`). Comandi: `04` = tensione batteria, `08` = lettura memoria (indirizzo big-endian in byte 3-4, lunghezza in byte 5, risposta con eco di indirizzo e lunghezza). Fonte: `LeadSun/lamzu-cfg` (MIT o Apache-2.0), `packerlschupfer/pulsar-mouse-linux` (MIT). In `hidapi`: `write()` di 17 B, poi `read()` con timeout; il codice originale salta fino a 3 risposte non corrispondenti. **Solo lettura: non inviare i comandi `07` (scrittura flash), `0F` (cambio profilo).**

| Marchio | Modello | Funzione | Frame (17 B) |
|---|---|---|---|
| Lamzu | Atlantis / Atlantis Mini Pro | LEGGERE polling corrente | `08 08 00 00 00 02 00 00 00 00 00 00 00 00 00 00 43` |
| Lamzu | Atlantis / Atlantis Mini Pro | LEGGERE indice DPI corrente | `08 08 00 00 04 02 00 00 00 00 00 00 00 00 00 00 3F` |
| Lamzu | Atlantis / Atlantis Mini Pro | LEGGERE preset DPI #0 (X/Y) | `08 08 00 00 0C 04 00 00 00 00 00 00 00 00 00 00 35` |
| Lamzu | Atlantis / Atlantis Mini Pro | LEGGERE tensione batteria | `08 04 00 00 00 00 00 00 00 00 00 00 00 00 00 00 49` |
| Lamzu | Thorn | LEGGERE polling corrente | `08 08 00 00 00 02 00 00 00 00 00 00 00 00 00 00 43` |
| Lamzu | Thorn | LEGGERE indice DPI corrente | `08 08 00 00 04 02 00 00 00 00 00 00 00 00 00 00 3F` |
| Lamzu | Thorn | LEGGERE preset DPI #0 (X/Y) | `08 08 00 00 0C 04 00 00 00 00 00 00 00 00 00 00 35` |
| Lamzu | Thorn | LEGGERE tensione batteria | `08 04 00 00 00 00 00 00 00 00 00 00 00 00 00 00 49` |
| Pulsar | X2 CrazyLight Wireless | LEGGERE polling | `08 08 00 00 00 02 00 00 00 00 00 00 00 00 00 00 43` |
| Pulsar | X2 CrazyLight Wireless | LEGGERE indice DPI | `08 08 00 00 04 02 00 00 00 00 00 00 00 00 00 00 3F` |
| Pulsar | X2 CrazyLight Wireless | LEGGERE preset DPI #0 | `08 08 00 00 0C 04 00 00 00 00 00 00 00 00 00 00 35` |
| Pulsar | Feinmann F01 Noctua Edition | LEGGERE polling | `08 08 00 00 00 02 00 00 00 00 00 00 00 00 00 00 43` |
| Pulsar | Feinmann F01 Noctua Edition | LEGGERE indice DPI | `08 08 00 00 04 02 00 00 00 00 00 00 00 00 00 00 3F` |
| Pulsar | Feinmann F01 Noctua Edition | LEGGERE preset DPI #0 | `08 08 00 1B 00 06 00 00 00 00 00 00 00 00 00 00 24` |

Note:
- Risposta (stessa struttura per tutti): byte 0 = `08`, byte 1 = comando, byte 2 = stato, byte 5 = lunghezza, dati dal byte 6. **Polling** (dato byte 6): `01`=1000, `02`=500, `04`=250, `08`=125, `10`=2000, `20`=4000, `40`=8000 Hz (limitare a cio' che il modello supporta). **Indice DPI** (byte 6): preset attivo, base 0. **Preset DPI**: byte 6 = X, byte 7 = Y, `DPI = (valore+1)*50`. **Batteria**: tensione in mV = `u16` big-endian ai byte 8-9; la percentuale va ricavata con una tabella. Lamzu provato dall'autore su Atlantis e Thorn.
- Pulsar: la tabella DPI sta a `0x000C` (4 B) su X2 CrazyLight e a `0x1B00` (6 B) su F01 Noctua. PID: CrazyLight dongle `3710:5406`, cavo `3710:3414`; Noctua dongle `3710:5504`, cavo `3710:7507`.
- I tre frame Lamzu/CrazyLight per polling, indice DPI e preset sono **byte per byte identici**: stesso chip, stesso protocollo. Un solo driver "Nordic 17 B" copre Lamzu e Pulsar (X2 V2 Mini incluso, con frame del quarto giro).
- Windows: usare `hid_write()` (Output report); `send_feature_report` e' sbagliato. Alcune interfacce non sono raggiungibili da `hidapi`: in quel caso serve WinUSB/libusb (control transfer `0x21 / 0x09 / wValue 0x0208`, timeout 2000 ms). Non testato su Windows.

## SteelSeries Rival 3 Wireless: layout batteria risolto
Il layout della risposta e' **livello % al byte 0, byte 1 = 0, carica al byte 2, poi zeri** (richiesta `AA 01` su interfaccia 3, risposta dopo circa 80 ms; se il mouse e' spento o dormiente il ricevitore **non risponde**). Fonte: patch kernel Linux di Martin Oberleiter (ottobre 2026), **provata su un Rival 3 Wireless reale** con ricevitore `1038:1830`; il kernel e' GPL, quindi solo descrizione (stesso protocollo di `rivalcfg`, WTFPL). Vecchia ipotesi con eco `AA` al byte 0: scartata. Accetta una risposta solo se `livello <= 100`, byte 1 = 0 e i byte dopo il terzo sono zero.

## ASUS ROG: protocollo da libratbag (MIT, trasporto hidraw)
Trasporto: Output report da **64 byte**, risposta Input da 64 byte letta subito dopo. I comandi sono `u16` little-endian, quindi il primo byte e' la meta' bassa: la richiesta di impostazioni `0x0412` va sul filo come `12 04 00 ...`. **CORREZIONE (settimo giro):** `12` e' solo il byte basso del comando `0x0412` (primo campo `uint16_t` della struct), **non** la prova di un report ID `12`. Il report ID del modello classico su Windows e la lunghezza del buffer hidapi (64 o 65) sono **non trovati** senza un dump del descrittore HID. Solo il ricevitore Omni dichiara esplicitamente report ID `03`. Non anteporre `00` e non leggere `12` come ID in produzione.
| Funzione | Comando (u16) | Sul filo (inizio) | Note |
|---|---|---|---|
| Info profilo/firmware | `0x0012` | `12 00 ...` | profilo attivo `results[8]` (`[7]` su Strix), preset DPI `results[9]-1` |
| Impostazioni (DPI, polling, response, snapping) | `0x0412` | `12 04 00 ...` | parametro 0 = DPI unico, **2 = X/Y separati** |
| Pulsanti | `0x0512` | `12 05 ...` | non serve |
| Errore/dormiente | risposta `0xaaff` | `ff aa` | mouse spento, scollegato o addormentato |
- DPI = `valore*50 + 50` (raddoppiato con il quirk DOUBLE_DPI). Polling: indice `0..3` = 125/250/500/1000 Hz. Layout vecchio: pad 4 B poi `dpi[n]`, rate, response, snapping come `u16`. Layout `SETTINGS_V2` (Gladius III e nuovi): dati da offset 4, preset da 2 B, **rate a offset 12 (`& 0x07`)**, response a 14, snapping a 16.
- Ricevitore **Omni** (`0B05:1ACE`, condiviso da tutti i mouse): il frame cambia, report ID `03` e payload spostato di 1 byte; il modello si riconosce con la richiesta `03 12 12 02` e il seriale ASCII a 12 caratteri. Non assegnare il modello dal solo PID.
- **Da non inviare:** `0x0350` (salva in flash), `0x0250` (cambia profilo), `0x3151` (scrive impostazioni), `0x2151` / `0x2851` (pulsanti, LED).
- Stato: DOCUMENTATO (codice), nessun test su Windows. Per Gladius III il PID del dongle e' `0B05:1A72`; la lettura con V2 va verificata sul modello vero prima di mostrare i numeri.

## Da NON inviare (6 classi nuove)

| Marchio | Famiglia | Classe | Motivo |
|---|---|---|---|
| Lamzu | WriteFlash, cambio profilo e write-active | NON INVIARE | Le letture di un profilo diverso possono commutare stato; mai write_flash via diagnostica automatica |
| Pulsar | Nordic CMD_MEM_SET e CMD_ACTIVE_PROFILE_SET | NON INVIARE | Comandi di scrittura e cambio profilo non necessari per sola lettura |
| Pulsar | Factory reset, pairing, DFU e scrittura configurazioni | NON INVIARE | Potenzialmente disconnette o corrompe la configurazione |
| ASUS ROG | Save profiles / firmware / pairing via Omni | NON INVIARE | Non usare sequenze di commit e update senza conferma model-specific; ricevitore condiviso |
| SteelSeries | Commit/salvataggio profili / firmware | NON INVIARE | Assenti da frame sola lettura, non dedurre dai setter di rivalcfg |
| Roccat | Salvataggio NVM e profili legacy | NON INVIARE | Nessun trasferimento automatico su Kone Pro/XP/Burst/Kain |

## Ancora senza dati (16 righe in non_trovato_6.csv)
Sonix a 64 B (Pulsar X3/Xlite V4), Roccat recenti, Glorious recenti, read-back delle SteelSeries wireless per DPI e polling; nessuna fonte verificata per Zowie, MSI, Gigabyte/AORUS, Microsoft, Cooler Master.


---

# Settimo giro di ricerca (31/10/2026)

**Verifica:** ho ricalcolato il checksum di tutti i frame Sonix (64 B, somma dei byte 0..61 come u16 little-endian nei byte 62-63): **OK**. Le pagine GitHub restano non raggiungibili dalla mia sessione, quindi le fonti non sono verificate alla radice. Lista mouse non fornita di nuovo (segnaposto).

## Pulsar Sonix 64 byte (X3, X3 LHD, Xlite V4, ricevitore Fusion2 1K) - sola lettura, fonte MIT (`packerlschupfer/pulsar-mouse-linux`, driver `x2a.py`)

Trasporto della fonte: USB control transfer HID, **SET_REPORT** `21 09 0300 wIndex=3` con 64 byte, poi **GET_REPORT** `A1 01 0300 wIndex=3` con 64 byte. `wValue=0300` = Feature report ID 0. Il byte 0 del payload e' anche la *direzione*. **Non e' provato** che su Windows `hid_send_feature_report` voglia 64 o 65 byte (descrittore e prove Windows mancanti): non inventare il byte in piu', non usare `hid_write`. Se `hidapi` non raggiunge l'interfaccia serve WinUSB/libusb.

| Funzione | Frame (inizio ... checksum) | Note |
|---|---|---|
| Polling | `00 01 89 02 00 00 00 00 ... 8c 00` | byte 62-63 = somma dei byte 0..61 (u16 LE) |
| Indice DPI attivo (profilo 1) | `00 05 81 02 00 00 01 00 ... 89 00` | byte 62-63 = somma dei byte 0..61 (u16 LE) |
| Preset DPI (profilo 1) | `00 05 84 15 00 00 01 00 ... 9f 00` | byte 62-63 = somma dei byte 0..61 (u16 LE) |
| Batteria | `00 08 81 01 00 00 00 00 ... 8a 00` | byte 62-63 = somma dei byte 0..61 (u16 LE) |
| Flag in carica | `00 08 83 01 00 00 00 00 ... 8c 00` | byte 62-63 = somma dei byte 0..61 (u16 LE) |

Risposta: byte 0 = `01` conferma (`02` = ancora in corso: aspetta e rileggi, non accodare altri comandi); **polling** nel byte 7: `1`=125, `2`=250, `4`=500, `8`=1000 Hz; **indice DPI** byte 7; **preset** `0x15` byte: numero stadi in byte 8, poi blocchi da 5 byte da byte 9 con DPI X = `LE16 [base+1..base+2]`, DPI Y = `LE16 [base+3..base+4]`; **batteria** percentuale in byte 6 (`FF` = non disponibile); **carica** `bool(byte 6)`. I profili 2-6 si leggono cambiando il byte 6 della richiesta. Tempi: fino a 8 poll a 20 ms su cavo; dongle ogni circa 10 ms; massimo 6 rinvii della stessa lettura a 100 ms.

PID: X3 cavo `3710:3409`, X3 LHD cavo `3710:3410` (testati dall'autore); Xlite V4 cavo `3710:3401` (letture di profilo confermate, **DPI e polling solo SUPPOSTI**); ricevitore Sonix Fusion2 1K `3710:5402` **condiviso**: il modello si riconosce dalla risposta, mai dal PID; provato solo con X3. Il frame batteria/polling e' lo stesso per tutti i modelli Sonix elencati.

## Glorious Model O/D classico (SinoWealth, `258A:0036`): conversione DPI e polling
Fonte MIT `Mason363/MouseFlash` (`hid.js`, `protocol.js`, `docs/PROTOCOL.md`). Richiesta Feature `05 11 00 00 00 00` (6 B, report ID 05), poi `get_feature_report` report `04` con buffer da **520 B** (alcune varianti long usano `06`). Offset della risposta con report ID incluso: **9** sensore, **10** nibble basso = polling (`1`=125, `2`=250, `3`=500, `4`=1000 Hz), **11** nibble alto = stadio DPI attivo tra gli abilitati (base 1), **12** maschera stadi disabilitati, **13..28** DPI grezzi (1 byte per preset con X/Y comuni, 2 con X/Y indipendenti). **Conversione:** sensore `06` (PMW3360) e `0E` (PMW3327): `(raw+1)*100`; sensore `08` (PMW3212) e `0F` (PMW3389): `raw*100`. Con sensore non riconosciuto non usare una formula di default. Non scrivere mai il report 520 (un autore ha danneggiato un mouse).

## Esiti negativi del settimo giro
- **ASUS ROG** (Gladius III, Keris, Chakram, Harpe): report ID e lunghezza hidapi su Windows non trovati (vedi correzione sopra). Ricevitore Omni: report ID `03`.
- **SteelSeries wireless** (Aerox 3/5/9, Rival 3, Prime): `rivalcfg` documenta solo i *setter* di DPI e polling; **nessuna lettura** di DPI o polling con risposta e offset. Vale solo la batteria.
- **Roccat recenti** e **Glorious recenti** (O 2, I 2, D 2, O Eternal `3794:A000`): nessun frame di lettura permissivo; solo supposti.
- **Lamzu Maya X 8K:** DPI e polling in lettura non trovati (solo batteria).
- **Zowie, MSI, Gigabyte/AORUS, Microsoft, Cooler Master:** nessuna fonte verificata.
- **Endgame Gear XM2we:** esiste un repository MIT ma il protocollo nasce da analisi statica del software ufficiale: **escluso**. **Ninjutso Sora:** GPL-3.0, solo descrizione. **Redragon G49/M916:** repository MIT ma senza getter completi e con funzioni firmware/reset da evitare. **WLMouse Beast X:** nessuna licenza permissiva trovata. **Logitech G102/G203:** usano HID++ 2.0 (`046D:C084`, `C092`, `C09D`), quindi il nostro driver Logitech li copre.

## Da NON inviare (nuovi)

| Marchio | Famiglia | Classe | Motivo |
|---|---|---|---|
| scrittura configurazione | SteelSeries Rival 3 Wireless / Aerox | SET DPI, SET polling e SAVE | comandi SET non leggono, possono alterare parametri; non riportati i byte nei frame |
| salvataggio | ASUS ROG | SAVE impostazioni/SET profilo/SET DPI | scrittura persistente o cambia configurazione |
| firmware | Pulsar Sonix | DFU, firmware update, flash, reset o pairing | fuori ambito e potenziale rischio irreversibile |
| provenienza | Endgame Gear XM2we | qualsiasi frame derivato da tool ufficiale disassemblato | la fonte MIT dichiara esplicitamente provenienza da disassemblaggio |
| copyleft | Roccat Eruption, Ninjutso Sora_tray, Razer OpenRazer, Corsair ckb-next | frame derivati da codice GPL/AGPL | la regola limita queste fonti a descrizioni |
| licenza | WLMouse Beast X | frame operativi non autorizzati | repository pubblico senza licenza permissiva individuata |
| identificazione | Pulsar 3710:5402 | assumere che il PID dongle identifichi Xlite V4 o X3 | ricevitore condiviso; chiedere identità al protocollo prima di associare |
| conversione | ASUS ROG tradizionale | inviare 12 04 come report ID 12 sotto Windows | 12 è byte basso del comando 0x0412, descriptor ID non trovato |
| conversione | Pulsar Sonix Windows | invio frame 64B raw USB direttamente via hid_write | frame origine è SET_REPORT Feature su control; differenza di API/padding non testata |
| persistenza | Redragon G49/M916 Pro | DFU, factory reset, save permanente | repository include funzionalità di rischio, non fanno parte della ricerca di lettura |

## Ancora aperto (39 righe in non_trovato_7.csv)
Descrittori HID reali (ASUS, Sonix), letture DPI/polling SteelSeries wireless, Roccat e Glorious recenti, Maya X 8K, cinque marchi senza fonte.

---

# OTTAVO GIRO (31/10/2026): cosa è cambiato

Questa sezione l'ho scritta dopo aver letto io stesso il codice sorgente allegato (ATKsoft, MouseFlash, HaloBattery `lamzu.py`, pulsar-mouse-linux, log di libratbag e rivalcfg). Dove sotto scrivo "verificato" intendo: riletto sul sorgente e ricalcolato, **non** provato su un mouse.

## Risultato in sintesi
- **Descrittori HID reali (ASUS, Pulsar Sonix): ancora non trovati.** ChatGPT lo dice onestamente nel README. Però per i Pulsar Sonix il report ID si ricava da un altro dato (vedi sotto) e per ASUS c'è una riconciliazione utile.
- **Nuovo e usabile:** protocollo di lettura Attack Shark X6/X11 (MIT, ATKsoft), frame batteria Lamzu Maya X completo, elenco ID Pulsar confermato.
- **SteelSeries wireless DPI/polling: non esiste lettura.** Solo batteria.
- Nessun test Windows hidapi trovato per nessun frame. L'unica prova Windows è la batteria di HaloBattery (Python `hid`).

## Attack Shark X6 / X11: lettura DPI, polling, batteria (MIT, `grivia0/ATKsoft`)
Provato dall'autore su **X6 con ricevitore `1D57:FA60`, Linux hidraw**. Cavo X11 = `1D57:FA55` (non provato). Il cavo dell'X6 non è noto. L'X6 ha lo stesso firmware dell'X11 (stessi descrittori).

**Interfaccia:** la 2 (config). Si riconosce dal descrittore perché espone insieme i feature report `04` e `A0`. Trasferimenti, con report ID incluso:

| Report | Byte | Uso |
|---|---:|---|
| `04` | 52 | DPI (stadi, colori, switch sensore) |
| `05` | 13 | LED, timer sonno, debounce |
| `06` | 9 | polling |
| `08` | 59 | mappa tasti (18 slot) |
| `0C` | 6 | profilo attivo / quanti profili |
| `0A`, `0B`, `07`, `09` | 128, 8, 8, 64 | opachi: **non leggere né scrivere** |
| `A0` | 8 | grant di lettura |
| Input `03` | 5 | eventi spontanei (batteria, cambio DPI, cambio profilo) |

**Sequenza di lettura (ogni grant vale per UNA sola lettura):**
1. `SET_FEATURE` `A0 <rid> <len_dichiarata> 00 <param> 00 00 00`
2. attendi **250 ms**
3. `GET_FEATURE A0`: se il byte 1 è `01` la lettura è concessa (altrimenti rifiutata)
4. `GET_FEATURE <rid>` **una volta**. Il byte 1 della risposta deve valere `len_dichiarata + 2`: se no è un buffer vecchio, ripeti il grant (non rileggere a vuoto)

Frame di grant (ricalcolati dal codice):
- DPI: `A0 04 38 00 01 00 00 00`
- polling: `A0 06 09 00 01 00 00 00`
- LED: `A0 05 0F 00 01 00 00 00`
- tasti: `A0 08 3B 00 01 00 00 00`
- profilo: `A0 0C 0A 00 00 00 00 00` (qui il parametro DEVE essere 0, con 1 i campi slittano di un byte)

**Risposta DPI (52 byte, ID incluso), verificata con un frame catturato dall'autore:**
- byte 2 profilo; byte 3 alto nibble = lift-off; byte 5 = maschera stadi attivi
- byte 6 = maschera "doppio", byte 7 = maschera Y
- byte 8..15 = X per stadio 1..8; byte 16..23 = Y per stadio 1..8; byte 24 = stadio attuale
- byte 25..48 = colori RGB degli 8 stadi; byte 49 = animazione di conferma (non è uno stadio)
- byte 50..51 = checksum u16 **big endian** = somma dei byte 3..49. Ricalcolato: frame catturato `0x0E88` ✓, frame di default del produttore `0x0F68` ✓
- NON mandare mai 56 byte (la dimensione del buffer del produttore): il descrittore ne vuole 52 (su Linux dà EPIPE)

**Decodifica DPI (sensore PAW3311):** per stadio ci sono X, Y e il bit "doppio".
- Y=0: DPI = indice(X nella tabella) × 50 + 50
- Y=1: DPI = indice × 100 + 100
- Y≥2: DPI = 10100 + ((Y<<8 | X) − 199) × 100
- se il bit doppio è 1 il risultato si raddoppia (il valore speciale 20100 usa X=`0xEB`, Y=1, doppio)
- X=0 e Y=0 = stadio inutilizzato (0 DPI)
- sopra 20000 DPI la tabella ripete byte e la codifica non è reversibile: **mostrare, non riscrivere**

Tabella (`DPI_3311`, 220 valori, copiata dal sorgente):
```
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0e, 0x0f, 0x10, 0x11, 0x12,
    0x13, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x22, 0x23, 0x24, 0x25,
    0x26, 0x27, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x36, 0x37, 0x38,
    0x39, 0x3a, 0x3b, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x4a, 0x4b,
    0x4c, 0x4d, 0x4e, 0x4f, 0x51, 0x52, 0x53, 0x54, 0x55, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5e,
    0x5f, 0x60, 0x61, 0x62, 0x63, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70,
    0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7f, 0x80, 0x81, 0x82, 0x83,
    0x84, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x93, 0x94, 0x95, 0x96,
    0x97, 0x98, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa7, 0xa8, 0xa9,
    0xaa, 0xab, 0xac, 0xae, 0xaf, 0xb0, 0xb1, 0xb2, 0xb3, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xbb, 0xbc,
    0xbd, 0xbe, 0xbf, 0xc0, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xcf,
    0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdd, 0xde, 0xdf, 0xe0, 0xe1,
    0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xea, 0xeb, 0x76, 0x77, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8d,
```

**Risposta polling (9 byte):** byte 3: `08`=125, `04`=250, `02`=500, `01`=1000 Hz; byte 4 = complemento a uno del byte 3 (non è un checksum). Il firmware ne costruisce 11 ma ne arrivano 9: non aggiungere byte.

**Batteria:** non si interroga. Arriva da sola come report Input `03`, 5 byte: `03 <id modello> 40|41 <stato> <percentuale>`. Stato: `01` scarica (a batteria), `02` piena, `03` in carica (= cavo collegato, significa che l'evento serve anche per il cambio cavo). Il primo evento dopo la connessione spesso dice 100% senza aver misurato: tieni l'ultimo di una finestra di ~2,5 s. Id modello: `55` X11, `85` X6, `4D` X3, `4E` X3 Max, `BC` M3.

**Conflitto con quanto già scritto sull'X3:** il PID `FA60` è condiviso da X3, X6 e X11, ma l'X3 documentato più su usa un'altra interfaccia (MI_02 COL04, feature da 262 byte, nessuna lettura). Quindi **non decidere il modello dal solo PID**: leggi il descrittore (inventario feature `04:52 05:13 06:9 08:59 0C:6 A0:8` = famiglia X6/X11) e poi l'id modello dagli eventi.

**Rischio Windows non verificato (ipotesi):** `HidD_GetFeature/SetFeature` di solito vogliono un buffer lungo quanto il report feature più grande della collezione (qui il `0A` da 128 byte), non quello del singolo report. ATKsoft non può dirlo (gira su Linux). Da provare con un X6 vero: se `GET 04` con 52 byte fallisce, riprovare con 128 byte e ritagliare.

**Da non fare:** report `07`, `09`, `0A` (layout ignoto o macro con checksum incoerente); disaccoppiamento/pairing; scritture macro. Per DeviceDeck: driver in sola lettura con `SupportLevel::Documented` finché un utente non conferma su Windows.

## Lamzu Maya X 8K: frame batteria completo (HaloBattery `providers/lamzu.py`, MIT)
La licenza di HaloBattery è **MIT** (badge e LICENSE nel README) e `lamzu.py` dichiara come fonti progetti MIT (`Sheroune/lamzu-battery-monitory`, `Quitsoon/gnome-shell-extension-lamzu-battery`, `anany1702/lamzu-ctl`). Va a sostituire "licenza non trovata" scritto sopra. Se il provider fosse ancora "da confermare", il frame resta comunque identico a quello di tre fonti MIT indipendenti.
- ID: dongle 8K `373E:001E`, cavo `373E:001C`. Si usa **solo** la collezione con `interface_number == 2` **e** `usage_page == 0xFFFF`. Nient'altro va aperto.
- Richiesta: `send_feature_report`, buffer di **65 byte** = `00` (report ID) + `00 02 02 00 83` + 59 zeri. Cioè in testa `00 00 02 02 00 83 00 00 …`.
- Risposta: `get_feature_report(0, 65)`, ripetuta fino a 10 volte ogni 50 ms. Il pacchetto utile è `A1 00 02 02 00 83 <carica> <batteria %>`. `<carica>` ≠ 0 vuol dire in carica.
- Verificato su Windows 10/11 da HaloBattery con Python `hid` (solo batteria; **DPI e polling non esistono in nessuna fonte**).
- Mouse addormentato = nessuna risposta: tieni l'ultimo valore per ~5 minuti, grigio.
- Se lo ricostruisco bene: `QUERY = 00 02 02 00 83` ricavato dal docstring e dal fatto che la risposta ricopia `00 02 02 00 83` dopo `A1`. Il file `wlmouse.py` che lo definisce non l'ho visto.

## Pulsar: ID confermati e report ID Sonix risolto
Dalla tabella del README di `packerlschupfer/pulsar-mouse-linux` (MIT):

| Modello | VID:PID | Protocollo |
|---|---|---|
| Xlite Wired / X2 Wired / X2H / X2A Wired | `3710:1401` `1402` `1403` `1404` | Sonix |
| Xlite V4 (cavo) | `3710:3401` | Sonix |
| X3 (cavo) / X3 LHD (cavo) | `3710:3409` / `3710:3410` | Sonix |
| Dongle 1K condiviso / 8K condiviso | `3710:5402` / `3710:5403` | Sonix |
| Feinmann 8K / FO1 (dongle) | `3710:5404` | Sonix |
| Wireless Nordic (21 modelli) | `3554:F507` `F508` `F509` | Nordic 17 B |
| X2 CrazyLight dongle / cavo | `3710:5406` / `3710:3414` | Nordic 17 B sotto VID Sonix |
| Feinmann F01 Noctua dongle / cavo | `3710:5504` / `3710:7507` | Nordic 17 B |
| X2 Wireless (Areson) dongle / cavo | `25A7:FA7C` / `25A7:FA7B` | layout Nordic |

**Il VID non distingue Sonix e Nordic**: decide il PID.

**Report ID dei Sonix = 0.** Il README scrive "Packet: 64 bytes, Interface 3, HID Feature report (wValue=0x0300)". In una richiesta USB di controllo `wValue` = (tipo report << 8) | report ID: `0x03` = Feature, `0x00` = report senza numero. Quindi su Windows con hidapi il buffer è **65 byte: `00` + 64 byte** (così hidapi toglie lo zero e manda 64). Interfaccia **3**. Checksum: u16 little endian della somma dei byte 0..61, nei byte 62..63. Questo chiude la domanda che restava aperta sul "report ID Sonix". È una deduzione dal `wValue`, non un dump del descrittore: resta da confermare con un mouse vero.

I registri X2A: polling `cat 01 reg 09/89 sub 02`; debounce `cat 04 reg 03/83 sub 03`; angle snap `cat 07 reg 04/84`; ripple `07 03/83`; motion sync `07 05/85`; lift-off `07 02/82 sub 03`; stadi DPI `cat 05 reg 04/84 sub 21/15`; stadio attivo `05 01/81 sub 02`; colore stadio `05 05/85 sub 05`; luminosità `03 03/83`; effetto LED `03 04/84 sub 0F` (scrittura/lettura = reg / reg|0x80). Il 0x80 sul registro vale come bit "lettura".

## Glorious / SinoWealth (MouseFlash, MIT): cosa si aggiunge
Provato dall'autore **solo su Model O classico, con WebHID nel browser**: né Eternal né hidapi Windows.
- VID `258A` (SinoWealth, usato anche da altri marchi) e `3794` (Glorious): `258A:0036` Model O/O- (fw `V103`), `258A:0033` Model D, `258A:0027` e `0029`, `0051`, `1007` (marchi diversi, riconoscibili dalla stringa firmware), `3794:A000` Model O Eternal.
- Comandi sul report `05` (byte 1): `01` versione firmware (risposta: 4 caratteri ASCII dal byte 2 del report `05`), `02` selezione profilo, `11` configurazione, `12` mappa tasti, `1A` debounce (solo scrittura, si legge mai), `30` banco macro. `21/22` e `31/32` = profili 2 e 3.
- Se esiste il feature report `06` nel descrittore, quello è il report di configurazione al posto di `04` ("mouse long"). La lunghezza reale della configurazione va da 123 a 167 byte; il mouse risponde con quella, non con 519.
- Polling: nibble basso del byte 10 (`1`=125 … `4`=1000). DPI: byte 13..28; stadio attivo = nibble alto del byte 11 (conta solo gli stadi abilitati).
- Codifica DPI per sensore (byte 9): `06` PMW3360 `dpi/100 − 1` (max 12000); `08` PMW3212 `dpi/100` (7200); `0E` PMW3327 `dpi/100 − 1` (10200); `0F` PMW3389 `dpi/100` (16000). Solo multipli di 100.
- Sul Mac Chromium rifiuta la lettura del report `05` anche dove il `04` funziona: la lettura del firmware non deve mai bloccare la sessione.

## ASUS ROG: cosa dicono i log reali di libratbag
- **Keris Wireless AimPoint `0B05:1A66` (cavo)**: ratbagd ha inviato 64 byte `12 00 00…` e ricevuto `12 00 00 00 09 00 03 00 05 07 00 03 FF FF …` (versione: primaria `00.00.FF`, secondaria `03.00.09`). Mappa tasti (`12 05 00 00 …`) → `12 05 00 00 F0 01 F1 01 F2 01 E4 01 E5 01 E6 01 E8 01 E9 01`: 8 tasti da 2 byte. Risoluzioni (`12 04 00 00 …`) → `12 04 00 00 FF×8 61 15 02 00 …`: **dati non interpretabili** (tutto `FF`), libratbag è crashato. Quindi la lettura DPI del Keris AimPoint non funziona con questo layout; non fidarti.
- **Omni Receiver `0B05:1ACE`**: i modelli dietro il ricevitore condividono il PID e si distinguono con una query di firma; l'issue (`libratbag#1793`) cita MI_02/COL03 e report ID `03`, ma lo prende da G-Helper (GPL): solo descrizione, non inviare niente.
- **Riconciliazione del byte `12`:** nel log il comando è `12 <cmd> 00 00 …` e libratbag lo scrive sul nodo hidraw con il primo byte come numero di report. Per G-Helper (GPL, solo descrizione) invece "report ID 0x00, 64 byte" sul `mi_00`. Le due cose combaciano se sul filo i 64 byte sono `12 <cmd> …` e l'ID è 0: con hidapi Windows sarebbe un buffer da **65 byte `00 12 <cmd> …`**. È una ipotesi coerente con le due fonti, **non verificata**. Il descrittore resta da trovare. Finché non c'è, ASUS resta fuori dal codice.

## SteelSeries: nessuna lettura DPI/polling wireless
Il thread `rivalcfg#215` ha un esperimento su `1038:182E` che invia sequenze con feature report da 642 byte (id `83`, lettura `A1`). Le risposte sembrano scrambled/non deterministiche (due esecuzioni ravvicinate danno blocchi diversi), nessuno le ha decodificate, il codice non ha licenza. Conferma anche che `rivalcfg` non legge la configurazione corrente dal mouse. **Aerox 3/5/9 Wireless, Rival 3 Wireless, Prime Wireless: solo batteria.** Non copiare niente da quel thread.

## Mouse con batteria verificata su Windows (HaloBattery, README e `devices.md`)
HaloBattery è MIT ed è un'app Windows. La colonna "verificato" significa che il livello è stato visto su hardware da loro o da utenti. Non prova DPI né polling. Attenzione: il README dice che i protocolli vengono anche da OpenRazer/Solaar/G-Helper (GPL), quindi **prima di copiare un frame dal loro `protocols.md` controllare la fonte di quella riga**.

| Mouse | Collegamento | Verificato |
|---|---|---|
| Corsair Dark Core RGB Pro SE | dongle 2.4 GHz | sì |
| Logitech G PRO X 2 LIGHTSPEED, G502 LIGHTSPEED / X PLUS | ricevitore | sì |
| MCHOSE A5 Pro Max, G7, M7 Ultra | ricevitore / cavo | sì |
| Pulsar X2 V2 Mini, X2 CrazyLight Mini (e ATK VXE R1 SE+/Pro Max) | dongle + cavo | sì |
| Razer Basilisk V3 Pro / Ultimate, DeathAdder V4 Pro | ricevitore | sì |
| WLmouse Beast X Max | ricevitore 8K + cavo | sì |
| LAMZU Maya X | dongle 8K + cavo | sì |
| ASUS ROG Gladius III Aimpoint e altri ROG/TUF | ricevitore o cavo | no |
| SteelSeries Aerox 3 Wireless, Rival 3 Wireless | dongle | no |
| G-Wolves (vari), Keychron Ultra-Link 8K / M5, Hitscan Hyperlight, AM Infinity 8K, Lofree Hyzen, MCHOSE A7 V2 Ultra | vari | no |
| WLmouse Beast X, Beast X Mini Pro | ricevitore | probabile |

## Da NON inviare / da non copiare (aggiunte)
- ASUS Omni: frame di identificazione da G-Helper (GPL) e qualsiasi RESET/SAVE/SET_PROFILE ASUS.
- SteelSeries: sequenze "secret/no_idea" di `rivalcfg#215` (nessuna licenza, significato ignoto).
- Glorious: scrittura config `04` da 520 byte, banco macro `30`, qualsiasi APPLY/flash. Model I 2 (`093A:821D`): repo `gloriousctl-linux` è EUPL-1.2 e solo scrittura, quindi niente frame.
- WLmouse Beast X: repo senza licenza, nessun frame. G-Wolves Fenrir/Lycan e Keychron: protocolli ricavati dall'analisi del software ufficiale, esclusi.
- Attack Shark: report `07`, `09`, `0A`, pairing/unpairing, scrittura macro.
- Tutti: DFU, reset, pairing, scritture in flash, opcode non documentati.

## Ancora aperto dopo l'ottavo giro
1. Descrittori HID reali di ASUS ROG e dei Pulsar Sonix (per Pulsar ora c'è una deduzione, vedi sopra).
2. Qualunque prova Windows hidapi dei frame esistenti (unica prova: batteria Maya X e Pulsar Nordic in HaloBattery).
3. Zowie, MSI, Gigabyte/AORUS, Microsoft, Cooler Master: nessuna fonte.
4. Roccat Kone/Burst/Kain e Glorious recenti (Model O 2, D 2, I 2): nessun selettore di lettura con licenza ammessa.
5. DPI/polling Maya X 8K e SteelSeries wireless: non esistono in nessuna fonte aperta.
6. Il confronto con la lista dei ~770 modelli non è mai stato fatto (il segnaposto è rimasto vuoto in tutti i giri).

---

# NONO GIRO (01/11/2026): descrittori, prove Windows, audit HaloBattery

Fonte: ZIP `ricerca_hid_nono_giro` (ChatGPT). Ho letto tutti i CSV e ricalcolato l'hash del descrittore. Dove scrivo "provato" intendo **dichiarato dalla fonte**: nessun mouse è stato interrogato né da ChatGPT né da me.

## Unico descrittore HID completo trovato: Pulsar X3, dongle 1K `3710:5402`
Fonte: `santoshpanna/Pulsar-X3-Web-Config`, `PROTOCOL.md` righe 3-35 (MIT). Riferita a X3 FW 1.0.16, Windows 11, hidapi, sessione del 13/10/2026. Non indica con che strumento ha estratto il dump.
```
06 ff ff 09 01 a1 01
09 02 15 81 25 7f 75 08 95 40 81 02     (Input)
09 03 15 81 25 7f 75 08 95 40 91 02     (Output)
09 04 15 81 25 7f 75 08 95 40 b1 82 c0  (Feature)
```
44 byte, SHA-256 `a25f052416556488974a7ffaa47ae51435373dad04128b0e6a8357be01ce97d0` (ricalcolato ✓).
- Usage Page `0xFFFF`, Report Size 8, Report Count 64 per Input, Output e Feature, **nessun item Report ID** → ID 0 (non numerato).
- Windows: buffer da **65 byte** (`00` + 64). Conferma la deduzione fatta nell'ottavo giro dal `wValue=0x0300`.
- **Non estendere** a `5403`, `5404`, ai cavi o ad altri firmware. Le interfacce 2 e 3 hanno lo stesso descrittore in questo dump: 6 righe CSV, non 6 dump indipendenti.

## Prove su Windows (dichiarate dalle fonti)
| Dispositivo | Libreria | Buffer | Esito |
|---|---|---|---|
| Attack Shark X6 1D57:FA60 | non trovato per questo scambio su Windows | non trovato | Nessuna conferma Windows trovata per i buffer 52 o 128. ATKsoft dichiara solo Linux/hidraw e test sul dongle X6. Non usare le implementazioni cross-platform come prova Windows. |
| Attack Shark X11 1D57:FA60 / FA55 | non trovato per questo scambio su Windows | non trovato | Nessun confronto verificato 52 contro 128 trovato. Esiste trasporto node-hid, ma il README dice primarily tested on Linux e dichiara anche analisi statica ufficiale. Frame esclusi. |
| Pulsar X3 FW 1.0.16, dongle 1K 3710:5402 | Python hid (hidapi), Windows 11 | 65 | SI, dichiarato e con risultato pubblicato: 2026-10-13, batteria 51%, DPI corrente 800x800, polling letto nel log. Il validatore usa prefisso zero e get_feature_report(0,65). Il resoconto complessivo … |
| Pulsar X2A v3 eS Mini: cavo 3710:3408 e dongle 8K 3710:5403 | Rust windows-sys; HidD_SetFeature/HidD_GetFeature, Win… | 65 nella costruzione della richiesta; ricezi… | SI, test hardware dichiarato su entrambi i collegamenti in protocol.rs e README. Conferma specifica per X2A v3 eS Mini, non per tutti i PID Pulsar. Nessun log grezzo completo delle due sessioni trova… |
| LAMZU Maya X 373E:001E (dongle); 001C elencato nel codice | Rust hidapi 2.6, applicazione per Windows | 65 | IMPLEMENTAZIONE, non prova hardware indipendente esplicita: README Windows, filtro interfaccia 2/usage page FFFF e chiamate send/get leggibili. Test datato o log su hardware: non trovato. Metodo orig… |
| LAMZU Maya X 373E:001E, dongle 8K (HaloBattery, riferimento gia noto) | Python hid/hidapi; app Windows. OS/versione del singol… | 65 in invio e lettura nel codice | SI, protocols.md dice confermato su Maya X con dongle 8K. E la conferma HaloBattery gia segnalata dall’utente, non una nuova prova indipendente. Test sul cavo 001C non trovato in questa dichiarazione. |
| LAMZU Maya X 373E:001E (conferma separata da HaloBattery) | C# P/Invoke hid.dll: HidD_SetFeature/HidD_GetFeature; … | 65 | SI, dichiarazione hardware precisa: 2026-09-03, lettura 50% come Aurora. Codice con buffer 65 e report ID zero. Conferma di lettura separata; non dimostra una scoperta indipendente del protocollo. I … |
| Attack Shark X11 1D57:FA60 (radio) / FA55 (cavo), conferma supplement… | Python ctypes / Win32 ReadFile; Windows 11 Pro 26200 | 5 per la collezione batteria nei caps; il re… | SI, cattura guidata su hardware reale, 2026-10-14, radio e cavo. La fonte dichiara osservazione passiva, senza invii al mouse. Non e una conferma delle Feature richieste, ne dei buffer 52/128. |

Lettura pratica:
- **Pulsar Sonix 64 B**: confermato su Windows per `5402` (X3) e per X2A v3 eS Mini `5403` + cavo `3408`. Per la ricezione il secondo usa `FeatureReportByteLength` dai caps (`HidP_GetCaps`): è il metodo da copiare in DeviceDeck, non un numero fisso.
- **Maya X**: tre conferme Windows del frame batteria (HaloBattery, Sheroune, 0xLaileb C#). Non provano scoperta indipendente. Cavo `001C` senza conferma.
- **Attack Shark X6/X11**: **nessuna prova Windows** per le Feature DPI/polling. La domanda 52 contro 128 byte resta aperta. L'unica prova Windows (cattura passiva Input batteria, 14/10/2026, Python ctypes, caps = 5 byte, radio e cavo) vale per la batteria. Consiglio per DeviceDeck: leggere `FeatureReportByteLength` dai caps e usare quella lunghezza per ogni feature (come fa il driver Pulsar), senza fissare 52.

## Marchi: esito finale
| Marchio | Modelli | Esito | Dati di lettura |
|---|---|---|---|
| Zowie (BenQ) | EC-CW, EC/DW, U2-DW e famiglia | nessuna fonte con licenza ammessa | non trovato nel perimetro ammesso |
| MSI | Clutch GM31/GM41/GM51 e famiglia | nessuna fonte con licenza ammessa | non trovato un protocollo ammissibile DPI/polling/batteria |
| Gigabyte / AORUS | Mouse AORUS M5/M6 e famiglia | nessuna fonte con licenza ammessa | non trovato |
| Microsoft | Pro IntelliMouse 2019; Classic IntelliMouse … | fonte MIT trovata | Pro: letture DPI e polling; Classic: lettura DPI |
| Cooler Master | Mouse MM710/MM711/MM720/MM731 e famiglia | nessuna fonte con licenza ammessa | non trovato |
| Roccat | Kone EMP, Kone Pure, Kone XTD | fonte MIT trovata | Lettura impostazioni DPI e polling nei driver libratbag |
| Roccat | Burst Core / Burst Pro | nessuna fonte con licenza ammessa | non trovato nel perimetro delle regole |
| Roccat | Kain 100/2xx | nessuna fonte con licenza ammessa | non trovato |
| Glorious | Model O 2 / D 2 / I 2, VID 093A o 258A | nessuna fonte con licenza ammessa | non trovato un protocollo che superi entrambi i filtri licenza e provenienza |

Nota: **Microsoft Pro IntelliMouse 2019** (DPI + polling) e **Classic IntelliMouse 2017** (DPI) hanno codice MIT di lettura (`madsl/Pro-IntelliMouse-Control-Panel`, `intellimouse.py`). **Roccat Kone EMP / Pure / XTD**: MIT esplicita nell'intestazione dei file libratbag `driver-roccat-kone-*.c` (DPI + polling). Nessuno dei due testato su Windows; per entrambi leggere il sorgente prima di scrivere il driver, non è stato ricopiato nulla qui.

## HaloBattery `docs/protocols.md`: audit delle 53 sezioni
Snapshot `2251f0534a8e886c9692ead42056059b4f69d8c0`. La licenza MIT di HaloBattery **non** rende ammesse le filiere GPL (OpenRazer, Solaar, G-Helper, HeadsetControl): quei frame restano esclusi. Sono esclusi anche MCHOSE, G-Wolves e Lofree (letti da codice ufficiale offuscato o decompilato), AJAZZ Control Center e keychron-battery-dkms (GPL). Rivalcfg è WTFPL (ammesso). Sezioni risultate riutilizzabili (SI o PARZIALE):

| Protocollo | Riutilizzabile | Licenza della fonte |
|---|---|---|
| HyperX Cloud Alpha 2 | SI | MIT nel resoconto e provider HaloBattery |
| HyperX Cloud II Core Wireless | SI | MIT HaloBattery + MIT HyperHeadset |
| HyperX Cloud III S Wireless | SI | MIT |
| HyperX Cloud III Wireless | SI | MIT |
| Razer Barracuda Pro (2.4 GHz) | SI | MIT HaloBattery |
| Corsair Dark Core RGB Pro SE | PARZIALE, non frame | Fonti di schema GPL; resoconto della cattura MIT |
| LAMZU Maya X | SI per il percorso MIT dichiarato | MIT |
| MCHOSE G7 | SI | MIT nel resoconto e provider HaloBattery |
| Pulsar X2 V2 Mini, Pulsar X2 CrazyLight Mini, ATK VXE … | PARZIALE | MIT andrewrabert; AGPL-3.0 OpenMouse; software ufficiale pe… |
| SteelSeries Aerox 3 Wireless | PARZIALE | MIT steel-mouse; GPL-2.0 alloyctl; nessuna licenza gort818;… |
| SteelSeries Rival 3 Wireless | SI, ma non provato in HaloBattery | MIT + WTFPL |
| WLmouse Beast X and Beast X Mini Pro | SI per la fonte; hardware non confermato per… | MIT |
| WLmouse Beast X Max | SI | MIT |
| NZXT Lift Elite Wireless (1e71:2101) | SI | MIT HaloBattery |

"PARZIALE" vuol dire: usare solo il percorso MIT/WTFPL identificato, escludendo gli apporti GPL. Il CSV completo con tutte le 53 sezioni è `haloBattery_fonti.csv` nello ZIP originale.

## Cosa resta davvero aperto (e come chiuderlo)
1. **Descrittori ASUS, Maya X (`001C`/`001E`), Pulsar oltre `5402`**: non esistono pubblicati con licenza ammessa. Si chiudono solo con un dump fatto da un utente con il mouse in mano (USB Descriptor Tool, `usbhid-dump`, o `HidD_GetPreparsedData`/`HidP_GetCaps` dal programma stesso).
2. **Attack Shark X6/X11 su Windows**: serve un test reale delle Feature `04` e `06` con il buffer preso dai caps.
3. **Zowie, MSI, Gigabyte, Cooler Master, Roccat Burst/Kain, Glorious Model O 2 / D 2 / I 2**: nessuna fonte ammessa.
4. **SteelSeries wireless DPI/polling**: non esiste decodifica.
5. **Maya X cavo `001C`**: nessuna conferma indipendente.
6. **Confronto con la lista dei ~770 modelli**: mai fatto, il segnaposto non è stato riempito.

## Idea per colmare le lacune senza altre ricerche
Aggiungere a DeviceDeck una funzione "Esporta info HID" che salvi, per il mouse collegato, VID/PID, interfacce, usage page, `InputReportByteLength`/`OutputReportByteLength`/`FeatureReportByteLength` e, se disponibile, il report descriptor. Gli utenti possono incollarlo in una issue e i dump mancanti arrivano dalla community.
