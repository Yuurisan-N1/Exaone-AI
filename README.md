<div align="center">

<img width="100%" alt="header" src="https://capsule-render.vercel.app/api?type=waving&height=210&text=Exaone%20AI%20Bot&fontAlign=50&fontAlignY=36&fontSize=56&desc=Auto%20Signup%20%7C%20OTP%20Verify%20%7C%20API%20Key%20%7C%20Enroll%20Mode&descAlign=50&descAlignY=58"/>

<img alt="typing" src="https://readme-typing-svg.demolab.com?font=Inter&size=18&duration=3000&pause=650&center=true&vCenter=true&width=900&lines=Auto+Signup+via+Disposable+Inbox;OTP+Verification+%7C+Consent+Gate;Portal+Login+%7C+API+Key+Creation;Enroll+Mode+%7C+Probe+%7C+Proxy+Support"/>

<p>
  <img alt="rust" src="https://img.shields.io/badge/Rust-2021-f74c00?logo=rust&logoColor=white"/>
  <img alt="platform" src="https://img.shields.io/badge/Platform-EXAONE%20AI-111111"/>
  <img alt="build" src="https://github.com/Yuurisan-N1/Exaone-AI/actions/workflows/build.yml/badge.svg"/>
  <img alt="proxy" src="https://img.shields.io/badge/Proxy-Supported-111111"/>
  <img alt="author" src="https://img.shields.io/badge/by-Yuurisandesu-111111"/>
</p>

<p>
  <b>Exaone AI Bot</b> is an account automation client for EXAONE AI.<br/>
  It opens a disposable mailbox, signs up a fresh account, confirms the verification email, walks the consent gate, connects the portal session, and creates an API key, all automatically. Use enroll mode to register any number of accounts in one run. Each key is appended to <code>data.txt</code> and used in every subsequent cycle.<br/>
  Built and distributed by <b>Yuurisandesu</b>.
</p>

</div>

---

## Table of Contents

- [Requirements](#requirements)
- [Installation](#installation)
- [Configuration](#configuration)
- [Running the Bot](#running-the-bot)
- [Run Modes](#run-modes)
- [Features](#features)
- [File Structure](#file-structure)
- [Disclaimer](#disclaimer)

---

## Requirements

- Git
- A supported platform: Linux x86_64, Windows x86_64, or macOS Apple Silicon

No compiler or runtime is needed. The bot ships as a prebuilt binary.

---

## Installation

**Clone the repository:**

```bash
git clone https://github.com/Yuurisan-N1/Exaone-AI.git
cd Exaone-AI
```

**Download the binary for your platform from the latest release:**

https://github.com/Yuurisan-N1/Exaone-AI/releases/latest

| File | Platform |
|---|---|
| `exaone-bot-linux-x86_64` | Linux x86_64 |
| `exaone-bot-windows-x86_64.exe` | Windows x86_64 |
| `exaone-bot-macos-aarch64` | macOS Apple Silicon |

Place the downloaded binary in the same folder as `data.txt`, `proxy.txt`, and `config.json`.

**Linux / macOS — mark executable:**

```bash
chmod +x exaone-bot-linux-x86_64
```

---

## Configuration

### 1. API Keys (data.txt)

`data.txt` holds the API keys of enrolled accounts, one per line. On the first run the file can be empty; the bot signs up a fresh account and writes its key automatically. Use enroll mode to fill the file with multiple keys before the first run cycle.

### 2. Proxy (proxy.txt)

Fill `proxy.txt` with proxies, one per line (optional, leave empty to connect directly):

```
http://user:pass@host:port
socks5://user:pass@host:port
```

Proxies are paired to accounts by index. Proxy credentials are masked in the log.

### 3. Bot Settings (config.json)

`sleep_seconds` controls how many seconds the bot waits between cycles. The value is clamped between 60 and 86400 seconds. The default is 3600 (one hour).

---

## Running the Bot

Make `run.sh` executable first (Linux / macOS / Termux):

```bash
chmod +x run.sh
```

Then choose a mode:

```bash
./run.sh run              # run one cycle, sign up if data.txt holds no key
./run.sh enroll <n>       # sign up n accounts and append every key to data.txt
./run.sh bg               # run in background, logs to exaone-bot.log
./run.sh bgprobe          # run probe in background
./run.sh screen           # run in a detached screen session
./run.sh stop             # stop the background or screen session
./run.sh probe            # check the hosts and every proxy
./run.sh selftest         # offline checks, no network
./run.sh build            # build from source (requires full Rust toolchain)
```

For background mode, tail the log anytime:

```bash
tail -f exaone-bot.log
```

For screen mode, attach anytime:

```bash
screen -r exaone-bot
```

Press `Ctrl+C` once to stop the bot cleanly when running in foreground.

### Manual run

```bash
# Linux / macOS / Termux
./exaone-bot-linux-x86_64

# Windows
.\exaone-bot-windows-x86_64.exe
```

---

## Run Modes

| Mode | Command | Description |
|---|---|---|
| run | `./run.sh run` | One cycle: sign up if no key exists, then drive with every key |
| enroll | `./run.sh enroll <n>` | Sign up n accounts and append every key to data.txt |
| bg | `./run.sh bg` | Same as run but in background with log file |
| bgprobe | `./run.sh bgprobe` | Probe in background |
| screen | `./run.sh screen` | Same as run in a detached screen session |
| probe | `./run.sh probe` | Check the hosts and every proxy |
| selftest | `./run.sh selftest` | Offline checks, no network |
| stop | `./run.sh stop` | Stop background or screen session |

---

## Features

### Auto Signup via Disposable Mailbox
When no key exists in `data.txt` or enroll mode is used, the bot opens a disposable mailbox automatically through one of four providers tried in order: mail.tm, mail.io, mail.lol, and Guerrilla Mail. It generates a random email address with the `ex-` prefix and submits the signup form with a fresh password.

### OTP Verification
After signup the bot requests a verification code and polls the inbox until the email arrives. The code is extracted and submitted automatically without any manual step.

### Consent Gate
After verification the bot walks the consent gate and accepts all required terms on the account's behalf so the portal session can proceed.

### Portal Login
The bot navigates the portal login flow and obtains an access token and refresh token for the account. These are stored in `session.json` alongside the email, password, and API key.

### API Key Creation
A named API key is created under the portal session and appended to `data.txt`. The key is logged in masked form. All account credentials are saved to `session.json`.

### Enroll Mode
Running `enroll <n>` signs up exactly n accounts one by one, writing each key to `data.txt` as it completes. Each account uses its own proxy slot from `proxy.txt`.

### Probe Mode
Probe checks the identity host, the portal host, the showroom host, and the exit country of every proxy in `proxy.txt` without signing up or modifying any files.

### Proxy Support
Proxies are loaded from `proxy.txt` and assigned to accounts by position in round-robin order. Running without proxies is fully supported.

### Auto Countdown
After each cycle completes the bot shows a live `HH:MM:SS` countdown until the next cycle starts.

---

## File Structure

```text
Exaone-AI/
├── .github/
│   └── workflows/
│       └── build.yml                    # CI: 3-platform matrix build
├── assets/
│   ├── icon.ico                         # Windows binary icon
│   └── icon.png                         # App icon
├── src/
│   ├── cli/
│   │   ├── dispatch.rs                  # CLI mode selector and usage text
│   │   ├── enroll.rs                    # Enroll mode: batch account signup
│   │   ├── mod.rs
│   │   ├── probe.rs                     # Probe mode: host and proxy check
│   │   └── selftest.rs                  # Offline self-test checks
│   ├── job/
│   │   ├── loop.rs                      # Cycle runner: signup when empty, drive every key
│   │   ├── mod.rs
│   │   └── onboard.rs                   # Signup flow and API key creation (distributed separately)
│   ├── net/
│   │   ├── client.rs                    # HTTP client and cookie jar
│   │   ├── egress.rs                    # Proxy parser and exit country check
│   │   └── mod.rs
│   ├── step/
│   │   ├── apikey.rs                    # API key creation request
│   │   ├── consent.rs                   # Consent gate: terms fetch and accept
│   │   ├── mailbox.rs                   # Disposable mailbox: mail.tm, mail.io, mail.lol, Guerrilla (distributed separately)
│   │   ├── mod.rs
│   │   ├── otp.rs                       # OTP send and verify
│   │   ├── portal.rs                    # Portal login flow and token extraction (distributed separately)
│   │   └── signup.rs                    # Account registration form and identity generation
│   ├── support/
│   │   ├── banner.rs                    # ASCII banner and terminal title
│   │   ├── cfg/
│   │   │   ├── loader.rs                # Config, key list, and proxy loader
│   │   │   ├── mod.rs
│   │   │   └── schema.rs                # Config schema and sleep setting
│   │   ├── consts/
│   │   │   ├── app.rs                   # File name constants and account generation params
│   │   │   ├── mod.rs
│   │   │   └── net.rs                   # Base URLs and API path constants (distributed separately)
│   │   ├── error.rs                     # Error types and result alias
│   │   ├── log/
│   │   │   ├── format.rs                # Colored log helpers and countdown
│   │   │   ├── init.rs                  # Signal handler setup
│   │   │   └── mod.rs
│   │   └── mod.rs
│   ├── wire/
│   │   ├── endpoint.rs                  # API endpoint path constants (distributed separately)
│   │   ├── envelope.rs                  # JSON request body builders
│   │   ├── mod.rs
│   │   └── shape.rs                     # JSON response parsers
│   └── main.rs                          # Entry point, Ctrl+C handler, mode dispatch
├── .gitignore
├── build.rs                             # Build script: embeds icon into Windows binary
├── Cargo.lock
├── Cargo.toml                           # Project manifest and dependencies
├── config.json                          # Sleep duration setting
├── data.txt                             # API keys, one per line (written by signup and enroll)
├── HOWTO.txt                            # Quick start guide
├── LICENSE                              # GNU AGPL v3.0
├── Makefile                             # make targets: build, release, run, start, clean, size
├── proxy.txt                            # Proxy list (optional)
├── README.md                            # This file
├── run.sh                               # Run helper: run, enroll, bg, bgprobe, screen, probe, selftest, stop
└── session.json                         # Account email, password, and key cache (auto-generated)
```

---

## Disclaimer

This tool is built for educational and technical exploration purposes. Use it wisely and at your own responsibility.

---

<div align="center">
<img width="100%" alt="footer" src="https://capsule-render.vercel.app/api?type=waving&height=120&section=footer"/>
</div>