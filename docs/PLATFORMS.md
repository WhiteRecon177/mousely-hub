# Platforms

Mousely Hub is built for **Windows** (main target), **macOS** and **Linux**. Windows is the only
system the author uses every day, so it is the best tested. Reports for the others are welcome.

| Feature                          | Windows | macOS | Linux |
| -------------------------------- | :-----: | :---: | :---: |
| Mouse: DPI, polling rate, battery | yes | untested | untested |
| Webcam controls                  | yes | untested | untested |
| Macros (record and play)         | yes | untested | untested |
| Button remapping                 | yes | not yet | not yet |
| Start with the system            | yes | untested | untested |
| Automatic updates                | yes | untested | AppImage only |

## macOS

- The app is not signed with an Apple developer certificate or notarised. The first time,
  right-click the app, choose **Open**, then **Open** again (or allow it in
  System Settings -> Privacy & Security).
- Macros need two permissions in System Settings -> Privacy & Security:
  **Accessibility** and **Input Monitoring**.
- Webcam: macOS asks for camera access the first time.

## Linux

- Install the `.deb` (Debian, Ubuntu) or run the `.AppImage`.
- Reading the mouse needs access to the `hidraw` device. If the mouse is not found, add this rule
  and replug the mouse:

  ```
  # /etc/udev/rules.d/70-mousely-hub.rules
  KERNEL=="hidraw*", ATTRS{idVendor}=="046d", MODE="0660", TAG+="uaccess"
  ```

  then run `sudo udevadm control --reload-rules && sudo udevadm trigger`.
- Macros work on X11. Wayland does not let apps read global keys, so on Wayland pick an X11
  session to record macros.
- The tray icon needs `libayatana-appindicator` (GNOME also needs the AppIndicator extension).

## Why remapping is Windows-only for now

Remapping has to intercept a mouse button before the system sees it. Windows has a safe,
documented way to do it. macOS and Linux need extra permissions (or root) and different code.
A mistake here could swallow clicks, so it stays off there until it can be tested on real
machines. Help is welcome: see CONTRIBUTING.md.
