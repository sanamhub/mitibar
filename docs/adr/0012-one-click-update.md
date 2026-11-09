# ADR-0012: One-click update

- **Status:** accepted
- **Date:** 2026-11-09
- **Approved by:** Sanam (owner chose one click on every system, 2026-11-09)
- **Amends:** ADR-0011 point 3 (the app no longer only opens the release page) and ADR-0003
  ("No auto-update in v0.1")

## Context

ADR-0011 added a button that checks GitHub for the latest version and then opens the release
page. The user still downloads a file, finds it in the download folder and runs the installer.
People keep MitiBar running for months, so most of them never do it.

TLS and a `SHA256SUMS` file from the same release do not prove that a file is the one this
project published. Whoever can replace the release assets can replace the checksum file too.
The same is true of the release API answer that names the version. A signature made with a key
that only the release workflow holds does prove it: the key is not in the repository, not in the
app, and not next to the files an attacker can swap.

## Decision

1. About offers **Download and install** when a newer version exists, on Windows, macOS and
   Linux. Only that press downloads anything. There is still no automatic or background update:
   no check at start, no timer, no polling.
2. Every installable asset is signed with **minisign** (Ed25519). The public key is committed as
   `packaging/update-key.pub`; `build.rs` embeds it. When the file is missing the build still
   succeeds, one click is off everywhere, and About keeps today's release-page **Download**.
3. The signature's **trusted comment** is exactly `mitibar <version> <asset name>`, for example
   `mitibar 1.1.0 mitibar_1.1.0_x64-setup.exe`. The app accepts a signature only when the comment
   matches the parsed version and the asset it downloaded. A valid signature made for an older
   version therefore cannot be served as the newer one.
4. The asset follows how the running copy was installed. Only the final step differs, behind one
   `Installer` trait; the check, download, verification and UI are shared code.

   | Running copy | How it is recognised | Asset | Install step |
   |---|---|---|---|
   | Windows, installed | The exe folder is the `InstallLocation` in the per-user uninstall key the NSIS template writes | `mitibar_<v>_<arch>-setup.exe` | Start with `/P /R`, then end MitiBar normally |
   | Windows, portable | Any other Windows exe | `MitiBar-<v>-windows-<arch>-portable.zip` | Unpack `MitiBar.exe`, rename the running exe to `.old`, move the new one in and start it |
   | macOS app | The exe is in `<name>.app/Contents/MacOS` and the bundle's folder is writable | `MitiBar-<v>.dmg` | `hdiutil attach`, `ditto` a `.app.new`, swap, detach, `open -n` |
   | Linux AppImage | `APPIMAGE` names a writable file and the architecture is x86_64 | `mitibar_<v>_x86_64.AppImage` | Stage `<file>.new`, mode 755, rename over the old file and start it |
   | Anything else | A `.deb`, another architecture, or a read-only folder | none | The release page, or the package manager |

   The new copy takes over through the single-instance version hand-over the app already uses.
   No step asks for elevation or an administrator password.
5. The order is always **download, verify, install**. A failure before the swap leaves the
   running copy untouched. A failure during the swap renames the `.old` copy back. A `.old` copy
   and leftover temp folders are removed at the next start.
6. Addresses are built from the parsed version and the asset name, never from the API answer:
   `https://github.com/sanamhub/mitibar/releases/download/v<version>/<asset>` and its
   `.minisig`. Redirects are followed, only HTTPS is allowed, the signature is limited to 4 KB
   and the asset to 64 MB, and the TLS setup is the one ADR-0011 chose.
7. In the release job that already holds every package (the one that writes `SHA256SUMS`), the
   workflow signs every `*-setup.exe`, `*-portable.zip`, `*.dmg` and `*.AppImage` with the secret
   `MITIBAR_UPDATE_SIGNING_KEY`. The `.minisig` files are published with the release and listed
   in `SHA256SUMS`. When the secret is not set the job prints a warning and continues, so a
   release still ships; the app then falls back to the release page.
8. Signing keys stay with the owner. The private key is never in this repository. Losing it means
   publishing one release with a new public key, which users install by hand.

## Consequences

- The shared code grows by the minisign verifier (`minisign-verify` 0.3.0, MIT, no dependencies)
  and the install step; the binary grows by that much plus the install code. The pull request
  records the measured size.
- A release with a missing or wrong signature installs nothing, and the user sees why.
- Updating is still a choice: no request happens until the button is pressed.
- Out of scope: automatic or background updates, `.deb` installs (they need root), delta
  updates, and Authenticode or Apple code-signature checks. The last two are the next step once
  Windows signing and macOS notarization are on (ADR-0003).
