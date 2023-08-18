use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Stdout},
    path::Path,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use powershell_script::PsScriptBuilder;
use ratatui::{prelude::*, widgets::*};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    run(&mut terminal)?;
    restore_terminal(&mut terminal)?;
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

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    let bookmarks_vec = read_lines("bookmarks.txt")?;
    let mut selected_folder = 0;

    Ok(loop {
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
                        if i == selected_folder {
                            x.style(Style::default().fg(Color::Black).bg(Color::White))
                        } else {
                            x
                        }
                    })
                    .collect::<Vec<ListItem>>(),
            )
            .block(Block::default().title("Bookmarks").borders(Borders::ALL));
            frame.render_widget(folder_names, horiz_chunks[0]);

            let selected_folder = bookmarks_vec.get(selected_folder).unwrap();

            let folder_previews = Paragraph::new(get_folder_preview(selected_folder)).block(
                Block::default()
                    .title("Folder preview")
                    .borders(Borders::ALL),
            );
            frame.render_widget(folder_previews, horiz_chunks[1]);

            let search_bar = Paragraph::new(" -> ").block(Block::default().borders(Borders::ALL));
            frame.render_widget(search_bar, vert_chunks[1]);
        })?;
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    // BUG: This stuff don't work
                    KeyCode::Up | KeyCode::Right => {selected_folder = (selected_folder - 1).wrapping_rem_euclid(bookmarks_vec.len())},
                    KeyCode::Down | KeyCode::Left => {selected_folder = (selected_folder + 1).rem_euclid(bookmarks_vec.len())},
                    _ => (),
                }
                //if KeyCode::Char('q') == key.code {
                //    break;
                //}
            }
        }
    })
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
