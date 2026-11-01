# Contributing

Thanks for helping. The biggest gap is **mouse support**, and the most useful thing you can do is give us data from your own mouse.

## Test an experimental mouse

The drivers marked *Experimental* in the README have not been tried on real hardware. Open an issue with the model, the connection, what worked, and the text of the **Diagnostics** button (Mouse tab). That is the most useful contribution.

## Add your mouse without coding

Open an issue with:
1. Brand, model, and whether you use the cable, a dongle or Bluetooth.
2. USB VID:PID (Device Manager → Properties → Details → Hardware Ids).
3. The HID report descriptor if you can get it (USB Descriptor Tool, `usbhid-dump`, or `lsusb -v`).
4. What the vendor software shows for DPI, polling rate and battery, to compare with what Mousely Hub reads.

## Licence rules for protocol code (important)

We only copy operational frames from sources licensed **MIT, 0BSD, CC0, WTFPL, Apache-2.0 or BSD**.

- **GPL / AGPL / EUPL projects** (OpenRazer, ckb-next, Solaar, G-Helper, OpenMouse...). Always check the LICENSE file of the project itself (not the README or the package page) before using anything from it: description only. Do not copy bytes or frames.
- **Nothing derived from decompiling or statically analysing vendor software.**
- **No source without a licence.** If a repository has none, treat it as all rights reserved.
- Always cite the source file, the commit and the licence in the pull request.

## Safety rules for drivers

- Prefer **read-only** access. Never send firmware, DFU, reset, pairing or flash-save commands.
- Write only DPI and polling rate, and only with a frame documented by a permissively licensed source. Keep the driver *Expected* until someone confirms it on real hardware.
- Mark every driver with its support level (documented, tested, unknown) and say which OS it was tested on.
- On Windows use the `FeatureReportByteLength` from the device caps for buffer sizes instead of hard-coding them.

## Code

- Rust: `cargo fmt` and no warnings from `cargo clippy`.
- TypeScript: strict mode, no `any`.
- Keep it light: no new dependency without a reason, no polling when an event exists.
- See `docs/PROTOCOLLI.md` and `docs/DRIVERS.md` before touching a driver.
