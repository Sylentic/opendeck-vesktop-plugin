use std::sync::OnceLock;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

/// Port that the Vencord plugin connects to.
const BRIDGE_PORT: u16 = 28196;

/// Commands sent from the OpenDeck plugin to the Vencord plugin.
#[derive(Serialize, Clone, Debug)]
pub struct BridgeCommand {
	pub cmd: &'static str,
}

/// State updates received from the Vencord plugin.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct VoiceState {
	#[serde(rename = "selfMute")]
	pub self_mute: bool,
	#[serde(rename = "selfDeaf")]
	pub self_deaf: bool,
	#[serde(rename = "localVideo")]
	pub local_video: bool,
	#[serde(rename = "streaming")]
	pub streaming: bool,
}

/// Global channel for sending commands to Vencord.
fn command_sender() -> &'static broadcast::Sender<String> {
	static TX: OnceLock<broadcast::Sender<String>> = OnceLock::new();
	TX.get_or_init(|| broadcast::channel(64).0)
}

/// Global channel for receiving state updates from Vencord.
fn state_sender() -> &'static broadcast::Sender<VoiceState> {
	static TX: OnceLock<broadcast::Sender<VoiceState>> = OnceLock::new();
	TX.get_or_init(|| broadcast::channel(64).0)
}

/// Send a command to all connected Vencord clients.
pub fn send_command(cmd: &'static str) {
	let json = serde_json::to_string(&BridgeCommand { cmd }).unwrap();
	// Ignore error if no receivers are connected.
	let _ = command_sender().send(json);
}

/// Subscribe to voice state updates from Vencord.
pub fn subscribe_state() -> broadcast::Receiver<VoiceState> {
	state_sender().subscribe()
}

/// Run the WebSocket server that the Vencord plugin connects to.
pub async fn run_server() {
	let addr = format!("127.0.0.1:{}", BRIDGE_PORT);
	let listener = match TcpListener::bind(&addr).await {
		Ok(l) => {
			log::info!("Vesktop bridge listening on ws://{}", addr);
			l
		}
		Err(e) => {
			log::error!("Failed to bind bridge server on {}: {}", addr, e);
			return;
		}
	};

	loop {
		let (stream, peer) = match listener.accept().await {
			Ok(v) => v,
			Err(e) => {
				log::error!("Failed to accept connection: {}", e);
				continue;
			}
		};

		log::info!("Vencord bridge connection from {}", peer);
		tokio::spawn(handle_connection(stream));
	}
}

async fn handle_connection(stream: tokio::net::TcpStream) {
	let ws = match accept_async(stream).await {
		Ok(ws) => ws,
		Err(e) => {
			log::error!("WebSocket handshake failed: {}", e);
			return;
		}
	};

	let (mut ws_sink, mut ws_stream) = ws.split();
	let mut cmd_rx = command_sender().subscribe();

	// Send a getState request on connection so buttons sync immediately.
	let init = serde_json::to_string(&BridgeCommand { cmd: "getState" }).unwrap();
	if let Err(e) = ws_sink.send(Message::Text(init.into())).await {
		log::error!("Failed to send initial getState: {}", e);
		return;
	}

	loop {
		tokio::select! {
			// Forward commands from OpenDeck actions to the Vencord plugin.
			Ok(json) = cmd_rx.recv() => {
				if let Err(e) = ws_sink.send(Message::Text(json.into())).await {
					log::error!("Failed to send command to Vencord: {}", e);
					break;
				}
			}
			// Receive state updates from the Vencord plugin.
			msg = ws_stream.next() => {
				match msg {
					Some(Ok(Message::Text(text))) => {
						match serde_json::from_str::<VoiceState>(&text) {
							Ok(state) => {
								let _ = state_sender().send(state);
							}
							Err(e) => {
								log::warn!("Invalid state JSON from Vencord: {}", e);
							}
						}
					}
					Some(Ok(Message::Close(_))) | None => {
						log::info!("Vencord bridge connection closed");
						break;
					}
					Some(Err(e)) => {
						log::error!("WebSocket error: {}", e);
						break;
					}
					_ => {}
				}
			}
		}
	}
}
