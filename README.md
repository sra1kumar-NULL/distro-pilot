# distropilot

**Save and restore your entire Linux system state across different distributions.**

Ever wiped your old distro only to realize your new one has noisy fans, broken GPU acceleration, missing drivers, and none of your apps? `distropilot` solves that.

Before wiping your working system, save its state (packages, driver configs, power management, firmware, dotfiles, systemd services) to a portable bundle. Apply that bundle on any fresh distro — it maps everything to native equivalents and replays your working configuration.

## The Problem

Traditional migration tools are fractured:

- **Aptik** — same-distro-family only, no hardware config, stale
- **Distro-Plopper** — bash/TUI, captures packages + configs but no driver/power state
- **pdrx** — declarative packages only, no config migration
- **Chezmoi** — dotfiles only, no packages or hardware
- **hw-probe** — hardware audit only, no migration

None of them capture the *working runtime state* — the modprobe overrides, udev rules, ACPI platform profiles, GRUB kernel params, and firmware versions that make your hardware actually work, then replay them across distro families.

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                    distropilot CLI                           │
├──────────────────────────────────────┬──────────────────────┤
│           save (source)              │   apply (target)     │
│                                      │                      │
│  Packages  →  scan + manifest       │  Packages → map +    │
│  Drivers   →  modprobe.d, udev,     │            install    │
│               sysctl, grub          │  Drivers  → copy +   │
│  Power     →  ACPI profile, TLP,    │            rebuild    │
│               sysfs, thermald       │  Power    → restore   │
│  Firmware  →  version manifest      │  Firmware → check     │
│  Dotfiles  →  tar.zst archive       │  Dotfiles → extract   │
│  Systemd   →  enabled services      │  Systemd  → enable    │
│  Hardware  →  lspci, sensors,       │  Validate → verify    │
│               dmidecode             │            sensors    │
└──────────────────────────────────────┴──────────────────────┘
```

## Quick Start

### Build

```bash
# Requires Rust (install via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh)
. "$HOME/.cargo/env"
cargo build --release
```

Binary at `./target/release/distropilot` (~3.2MB, statically linked).

### Step 1: Save your working system

Make sure you're in the project directory and use the full path to the binary (or install it to your PATH first):

```bash
# Run from the project directory:
./target/release/distropilot save /mnt/usb/my-bundle

# Or install to PATH once so you can run 'distropilot' from anywhere:
sudo cp ./target/release/distropilot /usr/local/bin/
distropilot save /mnt/usb/my-bundle
```

This creates a portable bundle:

```
my-bundle/
├── manifest.toml                 # source distro, kernel, timestamp
├── packages/manifest.json        # 200+ user-installed packages
├── drivers/                      # modprobe.d, modules-load.d,
│                                 # udev/rules.d, sysctl.d, grub
├── power/                        # ACPI platform profile, TLP,
│                                 # thermald, sysfs settings
├── firmware/manifest             # linux-firmware version info
├── dotfiles/                     # ~/.config/ + home dotfiles
│   ├── config.tar.zst            #   (zstd-compressed tar)
│   ├── home.tar.zst
│   ├── mozilla.tar.zst           # Firefox profiles (if present)
│   └── ssh.tar.zst               # SSH keys (--include-ssh only)
├── systemd/                      # enabled system + user services
└── hardware/                     # lspci, lsusb, sensors, dmidecode
```

Save as a single compressed file (for USB/external drive):
```bash
distropilot save --bundle /mnt/usb/my-bundle.tar.zst
```

Preview without writing:
```bash
distropilot save --dry-run /mnt/usb/my-bundle
```

### Step 2: Apply on a fresh distro

```bash
# Boot fresh Debian/Fedora install, plug in the USB:
distropilot apply /mnt/usb/my-bundle
```

This does everything in order:

1. **Packages** → maps each app to target-native names (e.g. `firefox` on Arch → `firefox-esr` on Debian), installs via apt/pacman/dnf with Flatpak fallback
2. **Drivers** → copies modprobe.d, modules-load.d, udev rules, sysctl configs into `/etc/`, merges GRUB kernel params, rebuilds initramfs, updates bootloader
3. **Firmware** → checks that linux-firmware on target matches source version
4. **Power** → applies ACPI platform profile, restores TLP/thermald configs
5. **Dotfiles** → extracts `~/.config/` and home dotfiles to `~/`
6. **Systemd** → enables all saved system + user services
7. **Validate** → reads sensors, checks dmesg for firmware errors, verifies GPU driver

Preview without changes:
```bash
distropilot apply --dry-run /mnt/usb/my-bundle
```

Apply only specific sections:
```bash
distropilot apply --step packages /mnt/usb/my-bundle
distropilot apply --step drivers /mnt/usb/my-bundle
distropilot apply --step dotfiles /mnt/usb/my-bundle
```

## Commands

| Command | Description |
|---------|-------------|
| `save <path>` | Snapshot current system to a bundle directory |
| `save --dry-run <path>` | Preview what would be saved |
| `save --include-ssh <path>` | Include SSH keys (⚠️ SECURITY: copies private keys to the bundle) |
| `save --bundle <path>` | Pack output as single `.tar.zst` file (removes directory) |
| `apply <path>` | Restore bundle onto current system (accepts directory or `.tar.zst` file) |
| `apply --dry-run <path>` | Preview changes without making any |
| `apply --step <name> <path>` | Run only one step (packages, drivers, dotfiles, firmware, power, systemd) |
| `apply --no-validate <path>` | Skip post-apply validation |
| `map <name>` | Look up a package's name across distros |
| `map <name> --from arch --to debian` | Specify source and target |
| `inspect <path>` | Show bundle metadata without applying |

### Examples

```bash
# See how Firefox maps from Arch to Debian
distropilot map firefox --from arch --to debian
# → "firefox-esr" via apt, or "org.mozilla.firefox" via flatpak

# See what's in a bundle
distropilot inspect /mnt/usb/my-bundle

# Install only packages and dotfiles on a new machine
distropilot apply --step packages /mnt/usb/my-bundle
distropilot apply --step dotfiles /mnt/usb/my-bundle
```

## Supported Distros

| Family | Capture | Apply | Examples |
|--------|---------|-------|----------|
| **Arch** | `pacman -Qq --explicit` | `pacman -S` | Arch, CachyOS, EndeavourOS, Manjaro |
| **Debian** | `apt-mark showmanual` | `apt install` | Debian, Ubuntu, Mint, Pop!_OS, Kali |
| **Fedora / RHEL** | `rpm -qa` | `dnf install` | Fedora, RHEL, CentOS, Rocky, Alma |
| **openSUSE** | `rpm -qa` | `zypper install` | openSUSE Tumbleweed, Leap, SUSE |
| **Cross-platform** | `flatpak list --app` | `flatpak install` | All of the above |

Arch capture lists only explicitly installed packages. Debian lists manually installed packages. Fedora/RHEL/openSUSE capture all RPM packages (includes dependencies — more thorough but less curated). Flatpak apps are captured alongside native packages on any distro.

Everything else (Void, Gentoo, NixOS, Alpine, Solus) uses package managers outside these families — those are niche and their users are unlikely to need distro-hopping between them and mainstream distros.

## Cross-Distro Package Mapping

The `mappings/packages.json` database contains **76 common desktop applications** mapped across Arch, Debian, Ubuntu, Fedora, and Flatpak. When applying, the engine:

1. Looks up the exact target distro name first
2. Falls back to id_like family (e.g. Ubuntu → Debian)
3. Falls back to Flatpak if no native package exists
4. Warns and skips if nothing is found

To add a new mapping, edit `mappings/packages.json`:

```json
{
  "my-app": {
    "arch": "my-app",
    "debian": "my-app",
    "ubuntu": "my-app",
    "fedora": "my-app",
    "flatpak": "org.example.MyApp"
  }
}
```

## Tech Stack

| Component | Technology | Why |
|-----------|------------|-----|
| Language | Rust | Static binary, no runtime deps, memory-safe |
| CLI | clap | Auto-generated help, subcommands, validation |
| Serialization | serde + serde_json + toml | Bundle manifest + package DB |
| Compression | zstd + tar | Fast compression for dotfiles |
| System calls | std::process::Command | Runs lspci, pacman, apt, systemctl, etc. |
| Sudo | sudo -v caching | Prompt once, session stays fresh |

## Prerequisites

- **Linux** with `/etc/os-release` (all major distros)
- `sudo` access (for applying configs, installing packages)
- For `save`: the tools you're capturing (pacman, apt, lspci, sensors, dmidecode — optional, missing ones are skipped)
- For `apply`: the target's package manager (apt, pacman, dnf, or zypper)

## Local Testing

You can test most functionality on your current system without modifying anything.

### Read-only commands (safe to run anytime)

```bash
# Help and version
./target/release/distropilot --help
./target/release/distropilot --version

# Package mapping lookups
./target/release/distropilot map neovim --from arch --to debian
./target/release/distropilot map firefox --from arch --to ubuntu
./target/release/distropilot map nvidia-dkms --from arch --to fedora

# Inspect a saved bundle
./target/release/distropilot inspect /tmp/test-bundle-2
```

### Dry-run save (safe — scans without writing)

```bash
./target/release/distropilot save --dry-run /tmp/test-dry
```

This scans your system and reports what would be captured:
- Distro detection
- Package count
- Driver config directories found
- Power state
- Firmware info
- Dotfiles
- Systemd services
- Hardware profile

### Dry-run apply (safe — shows what would change)

If you have a saved bundle from a previous `save`:

```bash
./target/release/distropilot apply --dry-run /tmp/test-bundle-2
```

This reads the bundle and shows what packages would be installed, what configs would be copied, etc. — without making any actual changes.

### End-to-end test (on your current machine)

```bash
# 1. Save your current system state
rm -rf /tmp/e2e-test
./target/release/distropilot save /tmp/e2e-test

# 2. Inspect the bundle
./target/release/distropilot inspect /tmp/e2e-test

# 3. Look at what's inside
ls -la /tmp/e2e-test/
cat /tmp/e2e-test/manifest.toml

# 4. Check how your packages would map
# (Package manifest is in JSON):
python3 -c "
import json
with open('/tmp/e2e-test/packages/manifest.json') as f:
    data = json.load(f)
print(f'Packages saved: {len(data[\"packages\"])}')
for pkg in data['packages'][:5]:
    print(f'  {pkg[\"app_name\"]} ({pkg[\"pm\"]})')
print('  ...')
"

# 5. Verify the dotfiles archive is valid
tar --zstd -tf /tmp/e2e-test/dotfiles/config.tar.zst 2>/dev/null | head -5
```

## Building from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"

# Clone and build
git clone <repo-url> distropilot
cd distropilot
cargo build --release

# The binary is at:
./target/release/distropilot

# Optional: install to PATH
sudo cp ./target/release/distropilot /usr/local/bin/
```

## Known Limitations

### Desktop Environment (DE) Lock-In

`distropilot` saves **raw `~/.config/` files**. These are DE-specific:
- KDE → KDE restore works perfectly (Plasma configs, panel layouts, krunner, dolphin)
- KDE → GNOME: KDE configs are ignored by GNOME; GNOME-specific settings (stored in dconf, not flat files) are NOT captured or restored
- GNOME → KDE: Same problem in reverse — GNOME uses dconf binary DB, KDE won't read it

**Recommendation:** Use `distropilot` for system-level state (packages, drivers, power, firmware, services). For DE-specific settings, stay within the same desktop environment family, or use each DE's native migration tools alongside (e.g. `dconf dump /` for GNOME, or copy `~/.config/plasma-*` manually).

### SSH Keys — Opt-In

SSH private keys in `~/.ssh/` are **not captured by default**. Use `--include-ssh` to include them:

```bash
distropilot save --include-ssh /mnt/usb/my-bundle
```

⚠️ **WARNING:** This copies your private keys into the bundle. Anyone with access to the bundle can use these keys. Store the bundle securely and delete it after use.

### Game Data (Steam, Lutris — NOT Captured)

- **Lutris configs** are in `~/.config/lutris/` — these ARE captured
- **Lutris game installs** in `~/.local/share/lutris/` are NOT captured (can be 50GB+)
- **Steam** stores everything in `~/.local/share/Steam/` — NOT captured. Steam games are 50-200GB and impractical to bundle
- **Steam login** session tokens are in the same dir — not captured

**Recommendation:** Keep your Steam library on a separate drive/partition and symlink it on the new distro. Lutris game libraries need manual migration.

### Browser Sign-Ins & Profiles

- **Firefox** (`~/.mozilla/firefox/`) — captured in `mozilla.tar.zst` (bookmarks, passwords, extensions, login sessions)
- **Chromium / Chrome / Brave / Vivaldi / Helium** — config in `~/.config/`, captured in `config.tar.zst`
- **Flatpak-browsers** (`~/.var/app/org.mozilla.firefox/`, etc.) — NOT captured (app data lives outside `~/.config/` and `~/.mozilla/`)

**Note:** Large browser caches will increase bundle size. The tool captures the whole profile directory.

### Spotify & App Logins

- **Spotify** stores login sessions in `~/.config/spotify/` — captured in `config.tar.zst`
- **Discord, Slack, Telegram** — configs in `~/.config/` — captured
- **Flatpak apps** — data in `~/.var/app/` — NOT captured

### Flatpak App Data

Flatpak **packages** are now captured (via `flatpak list --app`) and can be reinstalled on the target system. However, app data inside `~/.var/app/` is still not bundled:
- Flatpak-installed browsers → profiles not captured (use native packages instead)
- Flatpak Discord, Spotify, etc. → login sessions not captured

For complete Flatpak migration, use `flatpak create-usb` alongside `distropilot`.

## Architecture

```
src/
├── main.rs              → Entry point, CLI dispatch
├── cli.rs               → save, apply, map, inspect definitions
├── save.rs              → Capture orchestrator
├── apply.rs             → Restore orchestrator
├── bundle.rs            → Bundle manifest (TOML) read/write
├── distro.rs            → /etc/os-release parser, distro detection
├── configs.rs           → Driver config capture/apply
├── dotfiles.rs          → zstd+tar dotfile backup
├── firmware.rs          → linux-firmware manifest
├── power.rs             → ACPI, TLP, sysfs power state
├── systemd.rs           → Enabled services (system + user)
├── hardware.rs          → lspci, sensors, dmidecode + validation
├── packages/
│   ├── mod.rs           → Package manifest + install engine
│   ├── pm_detect.rs     → Package manager auto-detection
│   ├── pacman.rs        → Arch package list
│   ├── apt.rs           → Debian package list
│   └── mapping.rs       → Cross-distro name resolution
└── util.rs              → sudo caching, command execution, initramfs/bootloader
```
## License

MIT
