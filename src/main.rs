mod actions;
mod bridge;

use actions::*;

use openaction::{OpenActionResult, register_action, run};

#[tokio::main]
async fn main() -> OpenActionResult<()> {
	{
		use simplelog::*;
		if let Err(error) = TermLogger::init(
			LevelFilter::Debug,
			Config::default(),
			TerminalMode::Stdout,
			ColorChoice::Never,
		) {
			eprintln!("Logger initialization failed: {}", error);
		}
	}

	// Start the WebSocket bridge server that the Vencord plugin connects to.
	tokio::spawn(bridge::run_server());

	// Listen for voice state updates from Vencord and sync button states.
	actions::spawn_state_listener();

	register_action(ToggleMuteAction).await;
	register_action(ToggleDeafenAction).await;
	register_action(PushToMuteAction).await;
	register_action(PushToTalkAction).await;

	run(std::env::args().collect()).await
}
