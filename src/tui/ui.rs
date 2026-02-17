use crate::tui::app::{App, InputMode, ScanState};
use crate::tui::theme;
use crate::types::ScanStatus;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Row, Table},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header/Input
                Constraint::Length(3), // Progress
                Constraint::Min(0),    // Table
                Constraint::Length(4), // Status/Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    // 1. Header & Input
    let header_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)].as_ref())
        .split(chunks[0]);

    f.render_widget(
        Paragraph::new("🔍 RageScanner").style(
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        header_chunk[0],
    );

    let input_style = match app.input_mode {
        InputMode::Normal => Style::default(),
        InputMode::Editing => Style::default().fg(Color::Yellow),
    };

    let input = Paragraph::new(format!("RANGE: [{}]", app.input))
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Range Input (i:Edit Enter:Scan) "),
        );
    f.render_widget(input, header_chunk[1]);

    // Cursor in editing mode
    if app.input_mode == InputMode::Editing {
        f.set_cursor_position((
            header_chunk[1].x + 9 + app.input.len() as u16,
            header_chunk[1].y + 1,
        ));
    }

    // 2. Progress Gauge
    if app.scan_state == ScanState::Scanning || app.progress > 0 {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Progress "))
            .gauge_style(Style::default().fg(theme::PRIMARY))
            .percent(app.progress as u16);
        f.render_widget(gauge, chunks[1]);
    } else {
        f.render_widget(
            Paragraph::new("Ready to scan.").block(Block::default().borders(Borders::ALL)),
            chunks[1],
        );
    }

    // 3. Results Table
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(theme::PRIMARY);
    let header_cells = ["STAT", "HOSTNAME / MAC", "IP ADDRESS", "VENDOR"]
        .iter()
        .map(|h| {
            Span::styled(
                *h,
                Style::default()
                    .fg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_results()
        .into_iter()
        .map(|item| {
            let (status_icon, status_color) = match item.status {
                ScanStatus::Online => ("●", theme::ONLINE),
                ScanStatus::Offline => ("○", theme::OFFLINE),
                ScanStatus::Scanning => ("◌", theme::PRIMARY),
                ScanStatus::SystemError(_) => ("!", theme::ERROR),
            };

            let hostname = item
                .hostname
                .clone()
                .unwrap_or_else(|| "Unknown Device".to_string());
            let mac = item
                .mac
                .clone()
                .unwrap_or_else(|| "--:--:--:--:--:--".to_string());
            let vendor = item.vendor.clone().unwrap_or_else(|| "---".to_string());

            Row::new(vec![
                Line::from(vec![Span::styled(
                    status_icon.to_string(),
                    Style::default().fg(status_color),
                )]),
                Line::from(vec![
                    Span::styled(hostname, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(mac, Style::default().fg(theme::TEXT_DIM)),
                ]),
                Line::from(vec![Span::styled(
                    item.ip.to_string(),
                    Style::default().fg(theme::PRIMARY),
                )]),
                Line::from(vec![Span::raw(vendor)]),
            ])
        })
        .collect();

    let t = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Min(30),
            Constraint::Length(18),
            Constraint::Length(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Scan Results (↑↓:Nav Enter:Details Tab:Filter) "),
    )
    .row_highlight_style(selected_style)
    .highlight_symbol(">> ");

    f.render_stateful_widget(t, chunks[2], &mut app.table_state);

    // 4. Status Bar
    let online_count = app
        .results
        .iter()
        .filter(|r| r.status == ScanStatus::Online)
        .count();
    let status_text = format!(
        " {} Found | {} Online | Mode: {:?} | q:Quit s:Stop",
        app.results.len(),
        online_count,
        app.scan_state
    );
    let attr = " (c) WSALIGAN ";

    let footer = Paragraph::new(vec![
        Line::from(Span::styled(
            status_text,
            Style::default().fg(theme::TEXT_DIM),
        )),
        Line::from(Span::styled(attr, Style::default().fg(theme::TEXT_DIM))),
    ])
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[3]);

    // 5. Detail Popup
    if app.show_detail
        && let Some(selected_idx) = app.table_state.selected()
        && let Some(res) = app.filtered_results().get(selected_idx)
    {
        render_detail_popup(f, res);
    }
}

fn render_detail_popup(f: &mut Frame, res: &crate::types::ScanResult) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Device Details (Esc:Close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    let mut text = vec![
        Line::from(vec![
            Span::styled(
                "IP ADDRESS: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(res.ip.to_string()),
        ]),
        Line::from(vec![
            Span::styled(
                "HOSTNAME:   ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(res.hostname.as_deref().unwrap_or("Unknown")),
        ]),
        Line::from(vec![
            Span::styled(
                "MAC ADDR:   ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(res.mac.as_deref().unwrap_or("---")),
        ]),
        Line::from(vec![
            Span::styled(
                "VENDOR:     ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(res.vendor.as_deref().unwrap_or("---")),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "ACTIVE PORTS:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
    ];

    if res.open_ports.is_empty() {
        text.push(Line::from(Span::styled(
            "  No open ports found or scan incomplete.",
            Style::default().fg(theme::TEXT_DIM),
        )));
    } else {
        for port in &res.open_ports {
            let service = match port {
                80 => "HTTP",
                443 => "HTTPS",
                22 => "SSH",
                21 => "FTP",
                3389 => "RDP",
                445 => "SMB",
                _ => "Unknown",
            };
            text.push(Line::from(format!("  • Port {}: {}", port, service)));
        }
    }

    let p = Paragraph::new(text).block(block);
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
