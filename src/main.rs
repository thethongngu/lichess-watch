mod state;
mod ui;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use state::FeedResponse;
use std::time::Duration;
use ui::Tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const ENDPOINT: &str = "https://lichess.org/api/tv/feed";

    let mut res = reqwest::get(ENDPOINT).await?;
    let mut my_tui = Tui::new();
    let stop_event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

    while let Some(chunk) = res.chunk().await? {
        let opt_res: Result<FeedResponse, serde_json::Error> = serde_json::from_slice(&chunk);
        if opt_res.is_err() {
            continue;
        }

        let res = opt_res.unwrap();

        match res {
            FeedResponse::Feature { id: _, orientation: _, players, fen } => {
                let (white, black) = (&players.white_player, &players.black_player);
                my_tui.update_white_initial(&white.user_info.name, white.rating, (&white.user_info.title).as_deref());
                my_tui.update_black_initial(&black.user_info.name, black.rating, (&black.user_info.title).as_deref());

                my_tui.update_white_time(white.seconds);
                my_tui.update_black_time(black.seconds);

                my_tui.update_board_fen(fen);
            }
            FeedResponse::Fen { fen, lm: _, wc, bc } => {
                my_tui.update_white_time(wc);
                my_tui.update_black_time(bc);
                my_tui.update_board_fen(fen);
            }
        }

        my_tui.render()?;

        // Ctrl + C to stop
        if poll(Duration::from_millis(100))? {
            if let Ok(input) = read() {
                if input == stop_event {
                    my_tui.stop();
                    break;
                }
            }
        }
    }

    Ok(())
}
