# OpenDeck Vesktop Plugin

An [OpenAction](https://openaction.amankhanna.me/) / [OpenDeck](https://github.com/nekename/OpenDeck) plugin for controlling **Vesktop** (Discord voice: mute, deafen, push-to-talk, push-to-mute) from your Elgato Stream Deck.

## Why a custom plugin?

The existing [OpenAction Discord plugin](https://marketplace.rivul.us/plugin/me.amankhanna.oadiscord) uses Discord's native IPC RPC protocol (`discord-ipc-*` sockets). Vesktop doesn't implement the voice-control subset of that protocol (its built-in arRPC only supports Rich Presence). This plugin works around that limitation with a **two-part bridge architecture**:

1. **OpenDeck plugin** (Rust) — runs as a standard OpenAction plugin and hosts a local WebSocket server on `127.0.0.1:28196`.
2. **Vencord plugin** (TypeScript) — runs inside Vesktop, connects to the WebSocket server, and calls Discord's internal `toggleSelfMute()` / `toggleSelfDeaf()` functions.

## Actions

| Action | Description |
|---|---|
| **Toggle Mute** | Mute / unmute yourself |
| **Toggle Deafen** | Deafen / undeafen yourself |
| **Push to Mute** | Muted while holding key |
| **Push to Talk** | Unmuted while holding key |

Button states sync in real-time — if you mute from Vesktop's UI, the Stream Deck button updates automatically.

## Installation

### Part 1: Vencord Plugin (inside Vesktop)

Vesktop bundles its own copy of Vencord in `~/.config/vesktop/sessionData/vencordFiles/`. You need to build a custom Vencord with this user plugin and replace those files.

1. Clone Vencord:
   ```sh
   git clone https://github.com/Vendicated/Vencord
   cd Vencord
   ```

2. Copy the bridge plugin into the user plugins folder:
   ```sh
   cp -r /path/to/opendeck-vesktop-plugin/vencord-plugin/openDeckBridge src/userplugins/openDeckBridge
   ```

3. (Optional) Edit `src/userplugins/openDeckBridge/index.ts` and replace the placeholder `id: 0n` in the `authors` array with your Discord user ID (a BigInt, e.g. `123456789012345678n`).

4. Build Vencord:
   ```sh
   pnpm install
   pnpm build
   ```

5. Copy the built files into Vesktop's Vencord directory:
   ```sh
   cp dist/vencordDesktop* ~/.config/vesktop/sessionData/vencordFiles/
   ```
   > **Note:** Vesktop may overwrite these files when it auto-updates Vencord. To prevent this, disable **"Check for Vencord updates on startup"** in Vesktop's settings.

6. Restart Vesktop, then go to **Settings → Vencord → Plugins** and enable **OpenDeckBridge**. Restart Vesktop once more.

### Part 2: OpenDeck Plugin (Stream Deck)

1. Make sure you have Rust installed (`rustup`) with the target for your architecture:
   ```sh
   rustup target add x86_64-unknown-linux-gnu
   ```

2. Build and install:
   ```sh
   cd /path/to/opendeck-vesktop-plugin
   ./build.sh ~/.config/opendeck/plugins/com.sylentic.opendeck-vesktop.sdPlugin opendeck-vesktop x86_64-unknown-linux-gnu
   ```
   Adjust the plugins path if your OpenDeck config directory is different (e.g. Flatpak: `~/.var/app/me.amankhanna.opendeck/data/opendeck/plugins/`).
   Use `aarch64-unknown-linux-gnu` instead if you are on ARM.

3. Restart OpenDeck. The "Vesktop" category should now appear in the action list.

## How It Works

```
┌─────────────┐   WebSocket    ┌───────────────────┐   Discord Internal   ┌──────────┐
│  Stream Deck │◄─────────────►│  OpenDeck Plugin   │◄─────────────────────►│  Vesktop  │
│  (hardware)  │   (OpenAction) │  (Rust, ws:28196) │   (Vencord Plugin)   │  (voice)  │
└─────────────┘                └───────────────────┘                       └──────────┘
```

1. You press a button on the Stream Deck
2. OpenDeck sends a key event to the Rust plugin
3. The Rust plugin sends `{"cmd": "toggleMute"}` over WebSocket
4. The Vencord plugin receives it and calls `MediaEngineActions.toggleSelfMute()`
5. Discord processes the mute toggle
6. The Vencord plugin detects the state change via Flux and sends `{"selfMute": true, "selfDeaf": false}` back
7. The Rust plugin updates the Stream Deck button icon

## Protocol

Commands (OpenDeck → Vencord):
- `{"cmd": "toggleMute"}` — toggle self-mute
- `{"cmd": "toggleDeafen"}` — toggle self-deafen
- `{"cmd": "mute"}` / `{"cmd": "unmute"}` — set mute state
- `{"cmd": "deafen"}` / `{"cmd": "undeafen"}` — set deafen state
- `{"cmd": "getState"}` — request current voice state

State (Vencord → OpenDeck):
- `{"selfMute": bool, "selfDeaf": bool}`

## License

GPL-3.0-or-later
