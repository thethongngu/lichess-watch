use std::io::{self, Stdout};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Cell, Paragraph, Row, Table};
use tui::{backend::CrosstermBackend, Terminal};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    terminal::CompletedFrame,
};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    fen: String,
    white_initial: String,
    black_initial: String,
    white_time: String,
    black_time: String,
    board_area: Rect,
    white_initial_area: Rect,
    black_initial_area: Rect,
    white_time_area: Rect,
    black_time_area: Rect,
}

fn get_cell_color(row: usize, col: usize) -> Color {
    if (row + col) % 2 == 0 {
        Color::Rgb(172, 125, 88)
    } else {
        Color::Rgb(238, 211, 172)
    }
}

fn get_player_color(str: &char) -> Color {
    match str {
        'r' | 'n' | 'b' | 'q' | 'k' | 'p' => Color::Rgb(0, 0, 1),
        _ => Color::White,
    }
}

pub fn convert_chr_to_piece(chr: &char) -> String {
    match chr {
        'r' | 'R' => " ♜ ".to_string(),
        'n' | 'N' => " ♞ ".to_string(),
        'b' | 'B' => " ♝ ".to_string(),
        'q' | 'Q' => " ♛ ".to_string(),
        'k' | 'K' => " ♚ ".to_string(),
        'p' | 'P' => " ♟ ".to_string(),
        _ => " ".to_string(),
    }
}

pub fn get_cell(content: String, cell_color: Color, piece_color: Color) -> Cell<'static> {
    Cell::from(content).style(
        Style::default()
            .bg(cell_color)
            .fg(piece_color)
            .add_modifier(Modifier::BOLD),
    )
}

impl Tui {
    pub fn new() -> Tui {
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        let frame_size = terminal.get_frame().size();

        let drawable_content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(40),
                Constraint::Percentage(100),
            ])
            .split(frame_size)[1];

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Min(18),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(drawable_content);

        let player_info_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(8)]);

        let white_info_block = player_info_layout.split(main_layout[0]);
        let black_info_block = player_info_layout.split(main_layout[4]);

        Tui {
            terminal: terminal,
            fen: "".to_string(),
            white_initial: "".to_string(),
            black_initial: "".to_string(),
            white_time: "60".to_string(),
            black_time: "60".to_string(),
            board_area: main_layout[2],
            white_initial_area: white_info_block[0],
            white_time_area: white_info_block[1],
            black_initial_area: black_info_block[0],
            black_time_area: black_info_block[1],
        }
    }

    pub fn render(&mut self) -> Result<CompletedFrame<'_>, std::io::Error> {
        let board_state = Self::generate_board(&self.fen);

        self.terminal.draw(|f| {
            let table = Table::new(board_state)
                .style(Style::default().fg(Color::White))
                .header(Row::new(vec!["", "A", "B", "C", "D", "E", "F", "G", "H"]).bottom_margin(1))
                .column_spacing(0)
                .widths(&[
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(4),
                ]);

            f.render_widget(table, self.board_area);
            f.render_widget(
                Paragraph::new(self.white_initial.clone()).alignment(Alignment::Left),
                self.white_initial_area,
            );
            f.render_widget(
                Paragraph::new(self.black_initial.clone()).alignment(Alignment::Left),
                self.black_initial_area,
            );
            f.render_widget(
                Paragraph::new(self.white_time.clone()).alignment(Alignment::Right),
                self.white_time_area,
            );
            f.render_widget(
                Paragraph::new(self.black_time.clone()).alignment(Alignment::Right),
                self.black_time_area,
            );
        })
    }

    pub fn update_board_fen(&mut self, fen: String) {
        self.fen = fen;
    }

    pub fn generate_board(fen: &str) -> Vec<Row> {
        let mut board_state = vec![];

        // Generate board state row-by-row
        for (row_id, row) in fen.trim().split('/').enumerate() {
            let mut row_state = vec![Cell::from((row_id + 1).to_string())];
            let mut col = 0;
            let mut num_empty = 0;
            let mut char_iter = row.chars();

            while col < 8 {
                if num_empty > 0 {
                    row_state.push(get_cell("".to_string(), get_cell_color(row_id, col), Color::White));
                    num_empty -= 1;
                } else {
                    let chr_option = char_iter.next();
                    if chr_option.is_none() {
                        break;
                    }
                    let chr = chr_option.unwrap();

                    if chr.is_ascii_digit() {
                        num_empty = chr.to_string().parse().unwrap();
                        continue;
                    }

                    let piece = convert_chr_to_piece(&chr);
                    let piece_color = get_player_color(&chr);
                    row_state.push(get_cell(piece, get_cell_color(row_id, col), piece_color));
                }

                col += 1;
            }

            board_state.push(Row::new(row_state).height(2));
        }

        board_state
    }

    fn get_default_text() -> &'static str {
        ""
    }

    pub fn update_white_initial(&mut self, name: &str, rating: i32, opt_title: Option<&str>) {
        let title = opt_title.unwrap_or(Tui::get_default_text());
        self.white_initial = format!("{}:{} ({})", title, name, rating);
    }

    pub fn update_black_initial(&mut self, name: &str, rating: i32, opt_title: Option<&str>) {
        let title = opt_title.unwrap_or(Tui::get_default_text());
        self.black_initial = format!("{}:{} ({})", title, name, rating);
    }

    pub fn update_white_time(&mut self, time: i32) {
        self.white_time = format!("Time:{}s", time);
    }

    pub fn update_black_time(&mut self, time: i32) {
        self.black_time = format!("Time:{}s", time);
    }

    pub fn stop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
