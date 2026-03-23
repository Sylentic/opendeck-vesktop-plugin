use crate::bridge;

use std::collections::HashMap;

use openaction::{Action, ActionUuid, Instance, OpenActionResult, async_trait, visible_instances};

/// Spawn a background task that listens for voice state updates from Vencord
/// and keeps the Stream Deck button states in sync.
pub fn spawn_state_listener() {
	tokio::spawn(async {
		let mut rx = bridge::subscribe_state();
		while let Ok(state) = rx.recv().await {
			update_all_buttons(
				ToggleMuteAction::UUID,
				if state.self_mute { 1 } else { 0 },
			)
			.await;
			update_all_buttons(
				ToggleDeafenAction::UUID,
				if state.self_deaf { 1 } else { 0 },
			)
			.await;
		}
	});
}

async fn update_all_buttons(action_uuid: ActionUuid, state: u16) {
	for instance in visible_instances(action_uuid).await {
		if let Err(e) = instance.set_state(state).await {
			log::error!("Failed to update state for {}: {}", action_uuid, e);
		}
	}
}

// --- Toggle Mute ---

pub struct ToggleMuteAction;

#[async_trait]
impl Action for ToggleMuteAction {
	const UUID: ActionUuid = "com.sylentic.opendeck-vesktop.togglemute";
	type Settings = HashMap<String, String>;

	async fn key_up(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		bridge::send_command("toggleMute");
		Ok(())
	}

	async fn will_appear(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		bridge::send_command("getState");
		Ok(())
	}
}

// --- Toggle Deafen ---

pub struct ToggleDeafenAction;

#[async_trait]
impl Action for ToggleDeafenAction {
	const UUID: ActionUuid = "com.sylentic.opendeck-vesktop.toggledeafen";
	type Settings = HashMap<String, String>;

	async fn key_up(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		bridge::send_command("toggleDeafen");
		Ok(())
	}

	async fn will_appear(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		bridge::send_command("getState");
		Ok(())
	}
}

// --- Push to Mute ---

pub struct PushToMuteAction;

#[async_trait]
impl Action for PushToMuteAction {
	const UUID: ActionUuid = "com.sylentic.opendeck-vesktop.pushtomute";
	type Settings = HashMap<String, String>;

	async fn key_down(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		bridge::send_command("mute");
		Ok(())
	}

	async fn key_up(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		bridge::send_command("unmute");
		Ok(())
	}
}

// --- Push to Talk ---

pub struct PushToTalkAction;

#[async_trait]
impl Action for PushToTalkAction {
	const UUID: ActionUuid = "com.sylentic.opendeck-vesktop.pushtotalk";
	type Settings = HashMap<String, String>;

	async fn key_down(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		bridge::send_command("unmute");
		Ok(())
	}

	async fn key_up(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		bridge::send_command("mute");
		Ok(())
	}
}
