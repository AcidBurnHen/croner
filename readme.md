![croner](images/ico.png)
# Croner

Croner is a high-performance, cross-platform cron-style job runner implemented in Rust.  
It’s a complete rewrite of [PyCroner](https://github.com/AcidBurnHen/pycroner) — the lightweight Python-based job runner that inspired it — designed for **speed**, **stability**, and **local-first execution**.

## Why Croner?

PyCroner became an essential tool at my work.  
We needed something dead simple, predictable, and able to run scheduled jobs on **both Windows and Linux** without relying on platform-specific schedulers.

PyCroner solved that problem well — but as adoption grew, so did the need for even better performance and robustness.  
That’s why I rewrote it in Rust: to keep the simplicity, but make it **blazing fast** and capable of handling heavy workloads without breaking a sweat.

Croner keeps the same goal:
- **One config file** — same jobs work everywhere.
- **No OS-specific setup** — no Task Scheduler, no crontab syncing.
- **Hot reload** — update your jobs without restarting the runner.
- **Fanout** — run the same job multiple times with different args or in parallel.

And it’s **much faster** thanks to Rust’s zero-cost abstractions and optimized parsing.

## Features

- **Cross-platform** — works the same on Windows, macOS, and Linux.
- **Fast startup & parsing** — minimal overhead before your jobs run.
- **Five-field cron expression support** (minute, hour, day, month, weekday).
- **Fanout support** — integer (repeat job N times) or list of arguments.
- **Hot reload** — automatically reloads config on change.
- **Minimal config format** — no YAML complexity, optimized for quick parsing.

---

## Installation

### 1. Install via prebuilt binaries (recommended)

Precompiled executables are available on the [Releases page](https://github.com/AcidBurnHen/croner/releases/latest).

#### macOS / Linux (curl | sh)
```bash
curl -fsSL https://raw.githubusercontent.com/AcidBurnHen/croner/master/installers/install.sh | sh
``` 
Or pin a version:
```bash
VERSION=v0.1.3 curl -fsSL https://raw.githubusercontent.com/AcidBurnHen/croner/master/installers/install.sh | sh
```

#### Windows (PowerShell)
```powershell
iwr -useb https://raw.githubusercontent.com/AcidBurnHen/croner/master/installers/install.ps1 | iex
```

Or pin a version:
```powershell
iwr -useb https://raw.githubusercontent.com/AcidBurnHen/croner/master/installers/install.ps1 | iex -Version v0.1.3
```

---

### 2. Download manually
You can manually grab the latest binaries from:
- **Linux** (musl): [croner-latest-x86_64-unknown-linux-musl.tar.gz](https://github.com/AcidBurnHen/croner/releases/download/v0.1.3/croner-v0.1.3-x86_64-unknown-linux-musl.tar.gz)  
- **Windows**: [croner-latest-x86_64-pc-windows-msvc.zip](https://github.com/AcidBurnHen/croner/releases/download/v0.1.3/croner-v0.1.3-x86_64-pc-windows-msvc.zip)  
- **macOS ARM**: [croner-latest-aarch64-apple-darwin.tar.gz](https://github.com/AcidBurnHen/croner/releases/download/v0.1.3/croner-v0.1.3-aarch64-apple-darwin.tar.gz)  
- **macOS Intel**: [croner-latest-x86_64-apple-darwin.tar.gz](https://github.com/AcidBurnHen/croner/releases/download/v0.1.3/croner-v0.1.3-x86_64-apple-darwin.tar.gz)  
---

### 3. From source
```bash
git clone https://github.com/AcidBurnHen/croner.git
cd croner
cargo build --release
```
The binary will be in `target/release/croner`.

---

## Usage

### Create a `config.croner`
Example:
```
[job:job1]
schedule = * * * * *         # Every minute
command = "echo Hello from job1 , with changes"
fanout = ["one", "two"]

[job:daily_etl]
schedule = 0 2 * * *
command = python etl.py
fanout = ["--source=internal --mode=full", "--source=external --mode=delta"]

[job:ping]
schedule = * * * * *
command = python ping.py
```

### Run from CLI
```bash
croner
```
By default it looks for `config.croner` in the current directory.

You can get the full list of options with 
```bash 
croner --help 
# or 
croner -h
```

Check version 
```bash 
croner --version 
# or 
croner -v 
```

You can specify a different working directory or config file:
```bash
croner --at /path/to/project
croner --config /path/to/custom_config.croner
```

---

## Uninstallation

To remove the tool simply run the command 
```bash 
croner --uninstall 
```

## Configuration format
See [spec.md](spec.md) for the complete format reference.

---

## Example: Fanout in action
If a job is fanned out, each parallel run gets its own indexed color-coded prefix in the output:
```
[job1#1] ...
[job1#2] ...
```

---

## Credits

Croner is based on the ideas and design of [PyCroner](https://github.com/AcidBurnHen/pycroner), which started as a personal tool and became widely adopted for its simplicity and cross-platform nature.

The Rust rewrite aims to carry that legacy forward — faster, safer, and ready for heavier workloads.

