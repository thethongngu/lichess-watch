use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers, poll};
use serde_json::{Value, from_slice};
use std::time::Duration;
use ui::Tui;

mod board;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut res = reqwest::get("https://lichess.org/api/tv/feed").await?;
    let mut my_tui = Tui::new();
    let stop_event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

    while let Some(chunk) = res.chunk().await? {

        let json: Value = from_slice(&chunk)?;

        if is_new_game(&json) {
            let (white_json, black_json) = if json["d"]["players"][0]["color"] == "white" {
                (&json["d"]["players"][0], &json["d"]["players"][1])
            } else { 
                (&json["d"]["players"][1], &json["d"]["players"][0])
            };
            
            my_tui.update_white_initial(
                white_json["user"]["name"].as_str().unwrap_or(""), 
                white_json["rating"].as_u64().unwrap_or(0), 
                white_json["user"]["title"].as_str().unwrap_or("")
            );
            my_tui.update_black_initial(
                black_json["user"]["name"].as_str().unwrap_or(""), 
                black_json["rating"].as_u64().unwrap_or(0), 
                black_json["user"]["title"].as_str().unwrap_or("")
            );
            my_tui.update_white_time(white_json["seconds"].as_u64().unwrap_or(0));
            my_tui.update_black_time(black_json["seconds"].as_u64().unwrap_or(0));

        } else {
            my_tui.update_white_time(json["d"]["wc"].as_u64().unwrap_or(0));
            my_tui.update_black_time(json["d"]["bc"].as_u64().unwrap_or(0));
        }

        my_tui.update_board_fen(json["d"]["fen"].to_string());

        my_tui.render();
        
        if poll(Duration::from_millis(100))? {
            if let Ok(input) = read() {
                if input == stop_event {
                    my_tui.stop();
                } 
            }
        }
    }

    Ok(())
}

pub fn is_new_game(json: &Value) -> bool {
    json["t"] == "featured"
}