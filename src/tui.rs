use core::future::Future;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::{StreamExt, future::FutureExt, select};
use futures_timer::Delay;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Text;
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState};
use ratatui::{DefaultTerminal, Frame};
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

pub struct LivePanel<D, F, Fut>
where
    D: TableData,
    F: Fn() -> Fut,
    Fut: Future<Output = Vec<D>>,
{
    state: TableState,
    items: Vec<D>,
    updater: Box<F>,
    last_update: Option<Instant>,
}

impl<D, F, Fut> LivePanel<D, F, Fut>
where
    D: TableData,
    F: Fn() -> Fut,
    Fut: Future<Output = Vec<D>>,
{
    pub fn with_updater(updater: F) -> Self {
        Self {
            state: TableState::default(),
            items: vec![],
            updater: Box::new(updater),
            last_update: None,
        }
    }

    async fn update_data(&mut self) {
        self.items = (self.updater)().await;
    }

    pub async fn run_tui(&mut self) -> anyhow::Result<()> {
        let mut reader = EventStream::new();
        let mut term = ratatui::init();

        loop {
            let mut event = reader.next().fuse();
            let mut delay = Delay::new(Duration::from_millis(200)).fuse();

            select! {
                _ = delay => {
                    if self.last_update.is_none() ||
                    self.last_update.map(|t|t.elapsed()).unwrap() > Duration::from_secs(5) {
                        let _ = self.update_data().await;
                        self.last_update = Some(Instant::now());
                    }
                    // let _ = term.clear().unwrap();
                    let _ = term.draw(|frame| {
                        self.render(frame);
                    });
                }
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => match event {
                            Event::Key(KeyEvent {
                                code,
                                ..
                            }) => match code {
                                KeyCode::Esc | KeyCode::Char('q') => break,
                                KeyCode::Up | KeyCode::Char('k') => {
                                    self.scroll_up(&mut term);
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    self.scroll_down(&mut term);
                                }
                                _ => {}
                            },
                            _ => {}
                        },
                        Some(Err(e)) => println!("Error: {:?}\r", e),
                        None => break,
                    }
                }
            };
        }
        ratatui::restore();
        Ok(())
    }

    fn scroll_up(&mut self, term: &mut DefaultTerminal) {
        let previous_offset = self.state.offset();
        *(self.state.offset_mut()) = self.state.offset_mut().saturating_sub(1);
        if self.state.offset() != previous_offset {
            // let _ = term.clear().unwrap();
            let _ = term.draw(|frame| {
                self.render(frame);
            });
        } else {
            // beep
            print!("\x07");
            std::io::stdout().lock().flush().unwrap(); // 确保立即输
        }
    }

    fn scroll_down(&mut self, term: &mut DefaultTerminal) {
        let previous_offset = self.state.offset();
        *(self.state.offset_mut()) = self.state.offset_mut().saturating_add(1);
        if self.state.offset() != previous_offset {
            // let _ = term.clear().unwrap();
            let _ = term.draw(|frame| {
                self.render(frame);
            });
        } else {
            // beep
            print!("\x07");
            std::io::stdout().lock().flush().unwrap(); // 确保立即输
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(2)]);
        let rects = vertical.split(frame.area());
        self.render_table(frame, rects[0]);
        self.render_footer(frame, rects[1]);
    }

    fn render_footer(&mut self, frame: &mut Frame, area: Rect) {
        // let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let info_footer = Paragraph::new(Text::from(format!(
            "updated {} seconds ago",
            self.last_update
                .map(|t| t.elapsed())
                .unwrap_or(Duration::from_secs(0))
                .as_secs()
        )))
        .centered();
        frame.render_widget(info_footer, area);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header = D::header()
            .into_iter()
            .map(|s| Cell::from(s.into()))
            .collect::<Row>()
            .height(1);
        self.items.sort_by(D::comparator);
        let items = self.items.iter().map(|d| d).collect::<Vec<_>>();
        let rows = D::to_rows(&items);
        let t = Table::new(rows, D::column_constraints()).header(header);
        frame.render_stateful_widget(t, area, &mut self.state);
    }
}

pub trait TableData {
    fn header() -> Vec<impl Into<String>>;

    fn column_constraints() -> Vec<Constraint>;

    fn to_rows<'a>(data: &'a Vec<&Self>) -> Vec<Row<'a>>;

    fn comparator(&self, other: &Self) -> std::cmp::Ordering;
}
