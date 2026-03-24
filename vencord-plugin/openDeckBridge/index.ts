/*
 * OpenDeck Vesktop Bridge — Vencord User Plugin
 *
 * This plugin runs inside Vesktop and bridges voice controls
 * to/from the OpenDeck Stream Deck plugin over a local WebSocket.
 *
 * Install: copy this folder into Vencord's src/userplugins/ and rebuild,
 * or place it in your Vesktop user plugins directory.
 *
 * License: GPL-3.0-or-later
 */

import definePlugin from "@utils/types";
import { findByPropsLazy, findStoreLazy } from "@webpack";
import { FluxDispatcher } from "@webpack/common";

const BRIDGE_PORT = 28196;
const RECONNECT_DELAY_MS = 5000;

// Discord's internal modules.
const MediaEngineActions = findByPropsLazy("toggleSelfMute", "toggleSelfDeaf");
const MediaEngineStore = findStoreLazy("MediaEngineStore");
const ApplicationStreamingStore = findStoreLazy("ApplicationStreamingStore");
const ApplicationStreamingActions = findByPropsLazy("startStreaming", "stopStreaming");

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let running = false;

function sendState() {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    const selfMute = MediaEngineStore.isSelfMute();
    const selfDeaf = MediaEngineStore.isSelfDeaf();
    const localVideo = !MediaEngineStore.getLocalVideoDisabled();
    const streaming = !!ApplicationStreamingStore.getCurrentUserActiveStream();

    ws.send(JSON.stringify({ selfMute, selfDeaf, localVideo, streaming }));
}

function handleCommand(data: string) {
    let msg: { cmd: string; };
    try {
        msg = JSON.parse(data);
    } catch {
        return;
    }

    switch (msg.cmd) {
        case "toggleMute":
            MediaEngineActions.toggleSelfMute();
            break;
        case "toggleDeafen":
            MediaEngineActions.toggleSelfDeaf();
            break;
        case "mute":
            if (!MediaEngineStore.isSelfMute()) {
                MediaEngineActions.toggleSelfMute();
            }
            break;
        case "unmute":
            if (MediaEngineStore.isSelfMute()) {
                MediaEngineActions.toggleSelfMute();
            }
            break;
        case "deafen":
            if (!MediaEngineStore.isSelfDeaf()) {
                MediaEngineActions.toggleSelfDeaf();
            }
            break;
        case "undeafen":
            if (MediaEngineStore.isSelfDeaf()) {
                MediaEngineActions.toggleSelfDeaf();
            }
            break;
        case "toggleVideo":
            // Toggle video by dispatching the update event with inverted state
            const currentVideoDisabled = MediaEngineStore.getLocalVideoDisabled();
            FluxDispatcher.dispatch({
                type: "CALL_LOCAL_VIDEO_UPDATED",
                videoDisabled: !currentVideoDisabled
            });
            break;
        case "toggleStream": {
            const activeStream = ApplicationStreamingStore.getCurrentUserActiveStream();
            if (activeStream) {
                ApplicationStreamingActions.stopStreaming(activeStream);
            } else {
                ApplicationStreamingActions.startStreamingUI();
            }
            break;
        }
        case "getState":
            sendState();
            return; // don't send state twice
    }

    // After any toggle, send the new state back to OpenDeck
    // (small delay to let Discord process the change first).
    setTimeout(sendState, 50);
}

function connect() {
    if (!running) return;
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return;

    try {
        ws = new WebSocket(`ws://127.0.0.1:${BRIDGE_PORT}`);
    } catch {
        scheduleReconnect();
        return;
    }

    ws.onopen = () => {
        console.log("[OpenDeck Bridge] Connected to OpenDeck plugin");
        sendState();
    };

    ws.onmessage = (event) => {
        if (typeof event.data === "string") {
            handleCommand(event.data);
        }
    };

    ws.onclose = () => {
        console.log("[OpenDeck Bridge] Connection closed");
        ws = null;
        scheduleReconnect();
    };

    ws.onerror = () => {
        ws?.close();
    };
}

function scheduleReconnect() {
    if (!running) return;
    if (reconnectTimer) return;
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        connect();
    }, RECONNECT_DELAY_MS);
}

function onVoiceStateChange() {
    sendState();
}

export default definePlugin({
    name: "OpenDeckBridge",
    description: "Bridges Vesktop voice controls to the OpenDeck Stream Deck plugin via a local WebSocket.",
    authors: [{ name: "Sylentic", id: 0n }],

    start() {
        running = true;
        connect();

        // Listen for voice/video/stream changes so we can push state to OpenDeck.
        FluxDispatcher.subscribe("AUDIO_TOGGLE_SELF_MUTE", onVoiceStateChange);
        FluxDispatcher.subscribe("AUDIO_TOGGLE_SELF_DEAF", onVoiceStateChange);
        FluxDispatcher.subscribe("CALL_LOCAL_VIDEO_UPDATED", onVoiceStateChange);
        FluxDispatcher.subscribe("STREAM_CREATE", onVoiceStateChange);
        FluxDispatcher.subscribe("STREAM_DELETE", onVoiceStateChange);
    },

    stop() {
        running = false;

        FluxDispatcher.unsubscribe("AUDIO_TOGGLE_SELF_MUTE", onVoiceStateChange);
        FluxDispatcher.unsubscribe("AUDIO_TOGGLE_SELF_DEAF", onVoiceStateChange);
        FluxDispatcher.unsubscribe("CALL_LOCAL_VIDEO_UPDATED", onVoiceStateChange);
        FluxDispatcher.unsubscribe("STREAM_CREATE", onVoiceStateChange);
        FluxDispatcher.unsubscribe("STREAM_DELETE", onVoiceStateChange);

        if (reconnectTimer) {
            clearTimeout(reconnectTimer);
            reconnectTimer = null;
        }
        if (ws) {
            ws.close();
            ws = null;
        }
    },
});
