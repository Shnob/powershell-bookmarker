use std::{
    error::Error,
    fs::{File, self},
    io::{self, BufRead, BufReader, Stdout},
    path::Path,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use powershell_script::PsScriptBuilder;
use ratatui::{prelude::*, widgets::*};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    let choice = run(&mut terminal)?;
    restore_terminal(&mut terminal)?;

    let choice = choice.unwrap_or_else(|| ".".to_string());

    fs::write("C:/Users/Jake/AppData/Local/Temp/powershell_script_temp_file.txt", choice).unwrap();

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(terminal.show_cursor()?)
}

enum UiMode {
    Normal,
    Search,
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<Option<String>, Box<dyn Error>> {
    let bookmarks_vec = read_lines("bookmarks.txt")?;
    let mut selected_index: usize = 0;
    let mut user_search = String::new();
    let mut ui_mode = UiMode::Search;

    loop {
        terminal.draw(|frame| {
            let vert_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
                .split(frame.size());

            let horiz_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
                .split(vert_chunks[0]);

            let folder_names = List::new(
                bookmarks_vec
                    .iter()
                    .enumerate()
                    .map(|(i, x)| (i, ListItem::new(x.as_str())))
                    .map(|(i, x)| {
                        if i == selected_index {
                            x.style(Style::default().fg(Color::Black).bg(Color::White))
                        } else {
                            x
                        }
                    })
                    .collect::<Vec<ListItem>>(),
            )
            .block(Block::default().title("Bookmarks").borders(Borders::ALL));
            frame.render_widget(folder_names, horiz_chunks[0]);

            let selected_folder = bookmarks_vec.get(selected_index).unwrap();

            let folder_previews = Paragraph::new(get_folder_preview(selected_folder)).block(
                Block::default()
                    .title("Folder preview")
                    .borders(Borders::ALL),
            );
            frame.render_widget(folder_previews, horiz_chunks[1]);

            let search_bar = Paragraph::new(format!(" -> {}", user_search))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(search_bar, vert_chunks[1]);
        })?;

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if let KeyEventKind::Release = key.kind {
                    continue;
                }
                match ui_mode {
                    UiMode::Normal => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up | KeyCode::Right | KeyCode::Char('k') => {
                            selected_index = (selected_index as isize - 1)
                                .rem_euclid(bookmarks_vec.len() as isize)
                                as usize;
                        }
                        KeyCode::Down | KeyCode::Left | KeyCode::Char('j') => {
                            selected_index = (selected_index as isize + 1)
                                .rem_euclid(bookmarks_vec.len() as isize)
                                as usize;
                        }
                        KeyCode::Enter => {
                            return Ok(Some(bookmarks_vec[selected_index].clone()));
                        }
                        _ => (),
                    },
                    UiMode::Search => match key.code {
                        KeyCode::Esc => {
                            ui_mode = UiMode::Normal;
                        }
                        KeyCode::Backspace => {
                            user_search.pop();
                        }
                        KeyCode::Char(c) => {
                            user_search = format!("{}{}", user_search, c);
                        }
                        KeyCode::Enter => {
                            return Ok(Some(bookmarks_vec[selected_index].clone()));
                        }
                        _ => (),
                    },
                }
            }
        }
    }
    Ok(None)
}

fn get_folder_preview(filename: &str) -> String {
    let ps = PsScriptBuilder::new()
        .no_profile(true)
        .non_interactive(true)
        .hidden(false)
        .print_commands(false)
        .build();

    let output = ps.run(&format!(r#"ls -Name -Force "{}""#, filename));

    match output {
        Ok(output) => output.stdout().unwrap(),
        Err(e) => panic!("{}", e),
    }
}

fn read_lines<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let buf = BufReader::new(file);
    Ok(buf.lines().filter_map(|l| l.ok()).collect())
}
