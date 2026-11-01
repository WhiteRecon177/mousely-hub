# Third-party notices and credits

Mousely Hub is released under the MIT licence (see `LICENSE`).

The mouse drivers in `src-tauri/src/mouse/drivers/` were written from scratch. They use **only facts**
(USB ids, interface numbers, byte layouts, conversion tables) published by the projects below, all of
which are under permissive licences. No source code of these projects is copied. We thank their authors.
The licence of each project was read in its `LICENSE` file, unless stated otherwise.

| Project | Author / copyright | Licence | Used for |
|---|---|---|---|
| [rivalcfg](https://github.com/flozz/rivalcfg) 4.17.0 | Fabien Loison (flozz) | WTFPL v2 | SteelSeries Aerox 3/5/9 Wireless and Sensei Ten: DPI and polling commands, DPI code table, ids |
| [OpenSeries](https://crates.io/crates/openseries-gg) (`openseries-gg` 1.0.0) | Copyright (c) 2026 Pico and OpenSeries contributors | MIT | SteelSeries Aerox 3/5 battery, Sensei Ten ids and commands |
| [OpenSharkX11](https://github.com/HarukaYamamoto0) | HarukaYamamoto0 | MIT | Attack Shark X11 battery report |
| [hyperx_pulsefire_dart_reverse_engineering](https://github.com/santeri3700/hyperx_pulsefire_dart_reverse_engineering) | santeri3700 | CC0 1.0 (stated in the README of the repository) | HyperX Pulsefire Dart protocol |
| [hyperx-pulsefire-battery](https://github.com/radoslavchobanov/hyperx-pulsefire-battery) 2.0.0 | Copyright (c) 2025 Radoslav Chobanov | MIT | HyperX Pulsefire Dart battery and DPI query |
| [mxw](https://github.com/korkje/mxw) | Copyright (c) 2023 Ola Naess Kaldestad | MIT | Glorious Model O / O- / D / D- Wireless battery and polling rate |
| [pulsar-mouse-linux](https://github.com/Packerlschupfer/pulsar-mouse-linux) | Copyright (c) 2026 Packerlschupfer | MIT | Pulsar X3 / X3 LHD and the Nordic Pulsar models |

Other protocol notes (Lamzu, VAXEE, Pulsar Nordic read-only drivers and `docs/PROTOCOLLI.md`) come from
public community documentation; each driver names its source in its first lines and in `docs/DRIVERS.md`.

## Projects used for description only

Projects under a copyleft licence (GPL, LGPL, AGPL, EUPL) are never used as a source of bytes or code.

## Licence texts

### MIT

```
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### WTFPL v2

```
            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
                    Version 2, December 2004

 Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>

 Everyone is permitted to copy and distribute verbatim or modified
 copies of this license document, and changing it is allowed as long
 as the name is changed.

            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

  0. You just DO WHAT THE FUCK YOU WANT TO.
```

### CC0 1.0

Public domain dedication: <https://creativecommons.org/publicdomain/zero/1.0/>

## Libraries

The application is built with Tauri, React, Vite, hidapi and other open-source libraries. Their licences
are listed by the package managers:

- Rust: `cd src-tauri && cargo install cargo-license && cargo license`
- JavaScript: `npx license-checker --summary`

## Trademarks

All product names, logos and brands are property of their owners and are used only to say which devices are
compatible. Mousely Hub is not affiliated with, endorsed by or sponsored by any of them.
