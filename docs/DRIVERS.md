# Experimental drivers

Mousely Hub can already **show** the state of some non-Logitech mice. These drivers only send
reading commands, so they cannot change anything on the mouse. They are not tested on real
hardware yet, so the app marks them "not tested on this model".

| Brand | Models | What is read | Needs |
|---|---|---|---|
| Pulsar | X2 CrazyLight, Feinmann F01 Noctua Edition (cable and dongle) | polling rate, battery (estimate from the voltage), DPI of the first preset | - |
| VAXEE | models with the VAXEE dongle or cable (checked on one model by others) | battery, charging, polling rate | - |
| Lamzu | Maya X 8K (cable and dongle) | battery, charging | - |
| SteelSeries | Aerox 3 (cable) and Aerox 3 / 5 / 9 Wireless (receiver, cable, special editions) | battery and charging (wireless models); **writes** DPI and polling rate | - |
| SteelSeries | Sensei Ten (cable) | writes DPI and polling rate | - |
| Pulsar | X3, X3 LHD (cable only) | battery, charging, DPI, polling; **writes** DPI and polling rate | - |
| HyperX | Pulsefire Dart (dongle and cable) | battery, charging, DPI; **writes** DPI and polling rate | - |
| Glorious | Model O, O-, D, D- Wireless (dongle and cable) | battery; **writes** polling rate | - |
| Attack Shark | X11 (dongle or cable) | battery (only when the mouse reports it by itself) | - |

## What is NOT done on purpose
- No writing, except SteelSeries Aerox 3 (cable), Aerox 3/5/9 Wireless and Sensei Ten, HyperX Pulsefire Dart (DPI and polling rate) and Glorious (polling rate only), see below.
- The battery of Pulsar is an **estimate** computed from the voltage.
- The DPI of Pulsar is shown only when the first DPI preset is the active one, because the notes
  only explain how to read that preset.
- No firmware, pairing, reset, DFU or memory-write command is ever sent.

## If you own one of these mice
Open the app, go to the Mouse tab and press **Diagnostics**: it lists what the mouse answered.
Copy that text into an issue (it contains no serial number or personal data), and say whether the
numbers on screen match the mouse. That is what turns an experimental driver into a tested one.

## Where the protocol notes come from
`docs/PROTOCOLLI.md`: only facts (ids, byte frames) taken from projects with permissive licences;
no code was copied. Each driver says in its first lines where its facts come from.

## SteelSeries Aerox 3 (cable) and Aerox 3 / 5 / 9 Wireless

Facts taken from two permissively licensed projects; the code is written from scratch:
- OpenSeries (crate `openseries-gg` 1.0.0, MIT, copyright 2026 Pico and OpenSeries contributors):
  ids and the battery answer.
- rivalcfg 4.17.0 by Fabien Loison (flozz), WTFPL: the DPI and polling commands, the TrueMove Air
  and TrueMove Core DPI code tables, and the Aerox 3 (cable) / Aerox 9 ids.

For the wireless models (TrueMove Air), the driver sends the battery question (`92`, `D2` through
the receiver), one DPI preset (`2D 01 00 <code>`, 100-18000 DPI in steps of 100) and the polling
rate (`2B <code>` with 125=3, 250=2, 500=1, 1000=0). Through the receiver the command byte gets
bit `0x40` and the 64-byte answer is read and dropped.

For the wired Aerox 3 (`1038:1836`, TrueMove Core, no battery), it sends one DPI preset
(`2D 01 00 <code>`, 200-8500 DPI in steps of 100) and the polling rate (`2B <code>` with
125=4, 250=3, 500=2, 1000=1).

The "save to the mouse memory" command is **never** sent, so the settings last until the mouse is
switched off; lighting, buttons, sleep timer, pairing, firmware and reset are never touched.
The mouse cannot be asked for its DPI or polling rate, so the app shows the last value it wrote.
Aerox 3 wired (`1038:1836`) has been tested on real hardware (DPI and polling rate write). Wireless models are not tested on real hardware by us ("expected"). Wired Aerox 5 has no battery and is not handled.

Still not done (no verified permissive source): Aerox 3 Wireless Gen 2, other Attack Shark
models, Microsoft, Roccat, Razer.

## Attack Shark X11

Facts taken from OpenSharkX11 (`docs/protocol/PROTOCOL_EN.md`, MIT licence; its driver is
copyright 2026 HarukaYamamoto0). The documented protocol is write-only, so this driver never sends
anything: it listens on interface 2 for the packet the mouse sends by itself (`03 55 40 01 <percent>`)
and remembers the last value. The battery can therefore appear late, or not at all until the mouse
reports it. DPI and polling cannot be read (the DPI button packet gives only the stage 1..6). The
project documents that report id `0x0B` made the dongle lose its pairing; it is never used.
Not tested on real hardware by us ("expected").

## SteelSeries Sensei Ten
Facts from rivalcfg 4.17.0 (WTFPL) and OpenSeries 1.0.0 (MIT). Writes one DPI preset (`55 00 01 01 <lo> <hi>`,
50-18000 DPI) and the polling rate (`54 00 <code>`). The save command is never sent. No battery (cable).

## HyperX Pulsefire Dart
Facts from the CC0 protocol document by santeri3700 and from hyperx-pulsefire-battery 2.0.0 (MIT, Radoslav
Chobanov). Reads battery, charging and the DPI of the active profile; writes the DPI of the active profile
(`D3 02 ...`) and the polling rate (`D0 ...`). The save command (`DE`), lighting, buttons and macros are never sent.
The polling command comes from the CC0 document (`D0`); the README of the MIT project lists `DA`, which does not
match, so the first test on real hardware should check it.

## Glorious Model O / O- / D / D- Wireless
Facts from mxw (MIT, Ola Naess Kaldestad). Reads the battery and writes the polling rate through feature reports
on interface 2. DPI is not written on purpose: it belongs to numbered profiles and the active one cannot be read.

## Pulsar X3 / X3 LHD (cable)
Facts from pulsar-mouse-linux (MIT, Packerlschupfer), files `x2a.py`, `x3.py`, `x3_lhd.py`, `xlite_v4_wireless.py`.
Feature reports on interface 3. Reads battery, charging, active DPI and polling rate. Setting the DPI reads the
active profile and its stage table, changes only the active stage and writes the table back (so the other stages
are kept). The source documents no RAM-only mode: the values stay in the mouse profile, like with the official
software. Never sent: reset, profile change, buttons, macros, lighting, firmware. The 1K/8K dongles and the other
Pulsar models (X2, X2A, Xlite V3, Feinmann) are not handled yet.
