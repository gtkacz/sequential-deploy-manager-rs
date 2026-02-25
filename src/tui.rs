use std::collections::HashSet;
use std::io::{self, Stdout};
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Clear, Wrap};
use ratatui::Terminal;

use crate::config::Config;
use crate::git::Git;

pub struct App {
    pub config: Config,
    pub selected_repos: HashSet<(usize, usize)>, // (group_idx, repo_idx)
    pub cursor_group: usize,
    pub cursor_repo: usize,
    pub view: View,
    pub root_path: String,
    pub confirmed: bool,
    pub error_message: Option<String>,
}

#[derive(PartialEq)]
pub enum View {
    Selection,
    RootPrompt,
    Confirmation,
    Error,
}

impl App {
    pub fn new(config: Config) -> Self {
        let root = config.root_path.clone().unwrap_or_else(|| "..".to_string());
        Self {
            config,
            selected_repos: HashSet::new(),
            cursor_group: 0,
            cursor_repo: 0,
            view: View::Selection,
            root_path: root,
            confirmed: false,
            error_message: None,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_app(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err);
        }

        Ok(())
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        return Ok(());
                    }
                    match self.view {
                        View::Selection => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Down => self.move_cursor_down(),
                            KeyCode::Up => self.move_cursor_up(),
                            KeyCode::Char(' ') => self.toggle_selection(),
                            KeyCode::Enter => {
                                if self.selected_repos.is_empty() {
                                    self.error_message = Some("No repositories selected. Please select at least one.".to_string());
                                    self.view = View::Error;
                                } else {
                                    self.view = View::RootPrompt;
                                }
                            },
                            _ => {}
                        },
                        View::RootPrompt => match key.code {
                            KeyCode::Esc => self.view = View::Selection,
                            KeyCode::Enter => {
                                // Validate repositories
                                if let Err(msg) = self.validate_repos() {
                                    self.error_message = Some(msg);
                                    self.view = View::Error;
                                } else {
                                    self.view = View::Confirmation;
                                }
                            },
                            KeyCode::Backspace => { self.root_path.pop(); },
                            KeyCode::Char(c) => self.root_path.push(c),
                            _ => {}
                        },
                        View::Error => match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                self.error_message = None;
                                self.view = View::RootPrompt;
                            },
                            _ => {}
                        },
                        View::Confirmation => match key.code {
                            KeyCode::Esc => self.view = View::RootPrompt,
                            KeyCode::Char('y') | KeyCode::Enter => {
                                self.confirmed = true;
                                return Ok(());
                            },
                            KeyCode::Char('n') => self.view = View::Selection,
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    fn validate_repos(&self) -> Result<(), String> {
        let root = PathBuf::from(&self.root_path);
        let mut missing = Vec::new();

        for &(g_idx, r_idx) in &self.selected_repos {
            if let Some(group) = self.config.groups.get(g_idx) {
                if let Some(repo) = group.repositories.get(r_idx) {
                    let path = root.join(&repo.path);
                    if !Git::check_exists(&path) {
                        missing.push(repo.path.clone());
                    }
                }
            }
        }

        if !missing.is_empty() {
            return Err(format!("The following repositories were not found under '{}':\n{}", 
                self.root_path, missing.join(", ")));
        }

        Ok(())
    }

    fn move_cursor_down(&mut self) {
        if self.config.groups.is_empty() { return; }
        
        let current_group_len = self.config.groups[self.cursor_group].repositories.len();
        
        if self.cursor_repo + 1 < current_group_len {
            self.cursor_repo += 1;
        } else {
            // Move to next group
            if self.cursor_group + 1 < self.config.groups.len() {
                self.cursor_group += 1;
                self.cursor_repo = 0;
            }
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_repo > 0 {
            self.cursor_repo -= 1;
        } else {
            // Move to previous group
            if self.cursor_group > 0 {
                self.cursor_group -= 1;
                self.cursor_repo = self.config.groups[self.cursor_group].repositories.len() - 1;
            }
        }
    }

    fn toggle_selection(&mut self) {
        let key = (self.cursor_group, self.cursor_repo);
        if self.selected_repos.contains(&key) {
            self.selected_repos.remove(&key);
        } else {
            self.selected_repos.insert(key);
        }
    }

    fn ui(&self, f: &mut ratatui::Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.area());

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_chunks[0]);

        self.render_list(f, content_chunks[0]);
        self.render_graph(f, content_chunks[1]);
        self.render_footer(f, main_chunks[1]);

        if self.view == View::RootPrompt {
            self.render_prompt(f);
        } else if self.view == View::Confirmation {
            self.render_confirmation(f);
        } else if self.view == View::Error {
            self.render_error(f);
        }
    }

    fn render_list(&self, f: &mut ratatui::Frame, area: Rect) {
        let mut items = Vec::new();
        
        for (g_idx, group) in self.config.groups.iter().enumerate() {
            items.push(ListItem::new(Span::styled(
                format!("Group {}", g_idx + 1),
                Style::default().add_modifier(Modifier::BOLD),
            )));
            
            for (r_idx, repo) in group.repositories.iter().enumerate() {
                let is_selected = self.selected_repos.contains(&(g_idx, r_idx));
                let is_cursor = self.cursor_group == g_idx && self.cursor_repo == r_idx;
                
                let symbol = if is_selected { "●" } else { "○" };
                let style = if is_cursor {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} {} (delta: {})", symbol, repo.path, repo.delta), style)
                ])));
            }
        }

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Repositories"));
        f.render_widget(list, area);
    }

    fn render_graph(&self, f: &mut ratatui::Frame, area: Rect) {
        let mut current_time = 0.0;
        let mut graph_lines = Vec::new();
        
        let mut has_previous_group = false;

        for (g_idx, group) in self.config.groups.iter().enumerate() {
            let mut max_delta = 0.0;
            let mut group_selected_repos = Vec::new();
            
            for (r_idx, repo) in group.repositories.iter().enumerate() {
                if self.selected_repos.contains(&(g_idx, r_idx)) {
                    group_selected_repos.push(repo);
                    if repo.delta > max_delta {
                        max_delta = repo.delta;
                    }
                }
            }
            
            if !group_selected_repos.is_empty() {
                if has_previous_group {
                     graph_lines.push(ListItem::new(Span::styled(
                        "  ↓",
                        Style::default().fg(Color::DarkGray)
                    )));
                }

                let group_header = format!("Group {} (Start: {:.1}m)", g_idx + 1, current_time);
                graph_lines.push(ListItem::new(Span::styled(
                    group_header,
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
                )));

                for (i, repo) in group_selected_repos.iter().enumerate() {
                    let connector = if i == group_selected_repos.len() - 1 {
                        "└──"
                    } else {
                        "├──"
                    };
                    
                    graph_lines.push(ListItem::new(Line::from(vec![
                        Span::raw(format!("  {} ", connector)),
                        Span::styled(&repo.path, Style::default().fg(Color::White)),
                        Span::styled(format!(" (delta: {:.1}m)", repo.delta), Style::default().fg(Color::Gray)),
                    ])));
                }
                
                current_time += max_delta;
                has_previous_group = true;
            }
        }
        
        if graph_lines.is_empty() {
             graph_lines.push(ListItem::new(Span::styled(
                "Select repositories to see the execution graph",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
            )));
        } else {
            graph_lines.push(ListItem::new(""));
            graph_lines.push(ListItem::new(Span::styled(
                format!("Total Estimated Time: {:.1} minutes", current_time),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            )));
        }

        let list = List::new(graph_lines)
            .block(Block::default().borders(Borders::ALL).title("Execution Graph"));
        f.render_widget(list, area);
    }

    fn render_prompt(&self, f: &mut ratatui::Frame) {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area);
        
        let block = Block::default().title("Root Repository Path").borders(Borders::ALL);
        let text = Paragraph::new(self.root_path.clone()).block(block);
        f.render_widget(text, area);
    }

    fn render_confirmation(&self, f: &mut ratatui::Frame) {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area);
        
        let block = Block::default().title("Confirm Execution").borders(Borders::ALL);
        let text = Paragraph::new("Start execution? (y/n)").block(block).style(Style::default().fg(Color::Red));
        f.render_widget(text, area);
    }

    fn render_error(&self, f: &mut ratatui::Frame) {
        let area = centered_rect(60, 40, f.area());
        f.render_widget(Clear, area);
        
        let block = Block::default().title("Error").borders(Borders::ALL).style(Style::default().fg(Color::Red));
        let msg = self.error_message.clone().unwrap_or_default();
        let text = Paragraph::new(msg).block(block).wrap(Wrap { trim: true });
        f.render_widget(text, area);
    }

    fn render_footer(&self, f: &mut ratatui::Frame, area: Rect) {
        let help_text = match self.view {
            View::Selection => "↑/↓: Navigate | Space: Toggle | Enter: Next | q: Quit | Ctrl+c: Exit",
            View::RootPrompt => "Enter: Confirm | Esc: Back | Ctrl+c: Exit",
            View::Confirmation => "y/Enter: Yes | n: No | Esc: Back | Ctrl+c: Exit",
            View::Error => "Enter/Esc: Dismiss | Ctrl+c: Exit",
        };

        let footer = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Commands"));
        f.render_widget(footer, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}
