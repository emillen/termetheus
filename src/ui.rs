use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::prometheus::Prometheus;
use crate::prometheus::QueryData;
use crate::time_utils::{get_now, get_one_hour_before_date, timestamp_to_time_string};
use tui::Terminal;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use std::{
    io,
    time::{Duration, Instant},
};

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    tick_rate: Duration,
    prometheus: Prometheus,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let now = get_now();
    let one_hour_before = get_one_hour_before_date(&now);

    let metrics = prometheus
        .get_metrics(one_hour_before, now)
        .await
        .expect("did not get the metrics");

    loop {
        let data_copy = metrics.data.clone();
        terminal.draw(|f| ui(f, data_copy))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

#[derive(Debug)]
struct Bounds {
    min: f64,
    max: f64,
    inbetween: f64,
}

impl Bounds {
    fn new(min: f64, max: f64) -> Self {
        if min == max {
            return Self {
                min: min - 1.0,
                max: max + 1.0,
                inbetween: min,
            }
        }
        let space_in_between = max - min;
        let inbetween = min + space_in_between / 2.0;
        Self {
            min,
            max,
            inbetween,
        }
    }
}

fn get_bounds(data: &[Vec<(f64, f64)>]) -> (Bounds, Bounds) {
    let mut flat = data.iter().flatten().collect::<Vec<_>>();
    flat.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let x_smallest = flat.first().unwrap().0;
    let x_largest = flat.last().unwrap().0;
    flat.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let y_smallest = flat.first().unwrap().1;
    let y_largest = flat.last().unwrap().1;

    (
        Bounds::new(x_smallest, x_largest),
        Bounds::new(y_smallest, y_largest),
    )
}

fn get_points(data: &QueryData) -> Vec<Vec<(f64, f64)>> {
    data
        .result
        .iter()
        .map(|metric| {
            metric
                .values
                .iter()
                .map(|value| {
                    let x = value.0;
                    let y = value.1.parse::<f64>().unwrap();
                    (x, y)
                })
                .collect()
        })
        .collect::<Vec<Vec<(f64, f64)>>>()
}

fn generate_legend_entry(hashmap: &std::collections::HashMap<String, String>) -> String {
    let mut legend_entry = String::new();
    legend_entry.push_str("{");
    for (key, value) in hashmap {
        legend_entry.push_str(&format!("{}=\"{}\", ", key, value));
    }

    let mut chars = legend_entry.chars();
    chars.next_back();
    chars.next_back();

    let mut legend_entry = chars.as_str().to_string();

    legend_entry.push_str("}");
    return legend_entry;
}

fn get_datasets <'a>(data: &'a QueryData , points: &'a [Vec<(f64, f64)>] ) -> Vec<Dataset<'a>> {
    let colors = vec![
        Color::Red,
        Color::Green,
        Color::Blue,
        Color::Yellow,
        Color::Magenta,
        Color::Cyan,
        Color::LightRed,
        Color::LightGreen,
        Color::LightBlue,
        Color::LightYellow,
        Color::LightMagenta,
        Color::LightCyan,
    ];
    let mut datasets = vec![];
    for (i, point) in points.iter().enumerate() {
        let dataset = Dataset::default()
            .name(generate_legend_entry(&data.result[i].metric).chars().into_iter().take(50).collect::<String>())
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(colors[i % colors.len()]))
            .data(point);
        datasets.push(dataset);
    }

    datasets.clone()
}

fn ui<B: Backend>(f: &mut Frame<B>, data: QueryData) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let points = get_points(&data);
    let (x_bounds, y_bounds) = get_bounds(&points);
    let datasets = get_datasets(&data, &points);

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "Graphingoooo",
                    Style::default(),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("X Axis")
                .style(Style::default().fg(Color::Gray))
                .bounds([x_bounds.min, x_bounds.max])
                .labels(vec![Span::raw( timestamp_to_time_string(x_bounds.min)), Span::raw( timestamp_to_time_string(x_bounds.inbetween)), Span::raw( timestamp_to_time_string(x_bounds.max)) ]),
        )
        .y_axis(
            Axis::default()
                .title("Y Axis")
                .style(Style::default().fg(Color::Gray))
                .bounds([y_bounds.min, y_bounds.max]) 
                .labels(vec![Span::raw(format!("{:.2}", y_bounds.min)), Span::raw(format!("{:.2}", y_bounds.inbetween)), Span::raw(format!("{:.2}", y_bounds.max)) ])
        );
    f.render_widget(chart, chunks[0]);
}

pub async fn run(prometheus: Prometheus) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    run_app(&mut terminal, tick_rate, prometheus).await?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use regex::Regex;
    use crate::prometheus::{QueryValue, QueryResult};

    use super::*;
    #[test]
    fn test_get_points() {
        let mut metric = HashMap::new();
        metric.insert("__name__".to_string(), "test".to_string());
        let data = QueryData {
            result_type: "matrix".to_string(),
                result: vec![QueryResult {
                    metric,
                    values: vec![QueryValue(1.0, "1.0".to_string()), QueryValue(2.0, "2.0".to_string())],
                }],
        };
        let points = get_points(&data);
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].len(), 2);
        assert_eq!(points[0][0], (1.0, 1.0));
        assert_eq!(points[0][1], (2.0, 2.0));
    }

    #[test]
    fn test_get_datasets() {
        let mut metric = HashMap::new();
        metric.insert("__name__".to_string(), "test".to_string());
        let data = QueryData {
            result_type: "matrix".to_string(),
                result: vec![QueryResult {
                    metric: metric.clone(),
                    values: vec![QueryValue(1.0, "1.0".to_string()), QueryValue(2.0, "2.0".to_string())],
                },QueryResult {
                    metric: metric.clone(),
                    values: vec![QueryValue(1.0, "1.0".to_string()), QueryValue(2.0, "2.0".to_string())],
                }],
        };
        let points = get_points(&data);
        let datasets = get_datasets(&data, &points);

        assert_eq!(datasets.len(), 2);
    }

    #[test]
    fn test_get_bounds() {
        let data = vec![
            vec![(1.0, 1.0), (2.0, 2.0), (3.0, 3.0)],
            vec![(1.0, 1.0), (2.0, 2.0), (3.0, 3.0)],
        ];
        let (x_bounds, y_bounds) = get_bounds(&data);
        assert_eq!(x_bounds.min, 1.0);
        assert_eq!(x_bounds.max, 3.0);
        assert_eq!(x_bounds.inbetween, 2.0);
        assert_eq!(y_bounds.min, 1.0);
        assert_eq!(y_bounds.max, 3.0);
        assert_eq!(y_bounds.inbetween, 2.0);
    }

    #[test]
    fn test_get_bounds_same_min_max() {
        let data = vec![
            vec![(1.0, 1.0), (1.0, 1.0), (1.0, 1.0)],
            vec![(1.0, 1.0), (1.0, 1.0), (1.0, 1.0)],
        ];
        let (x_bounds, y_bounds) = get_bounds(&data);
        assert_eq!(x_bounds.min, 0.0);
        assert_eq!(x_bounds.max, 2.0);
        assert_eq!(x_bounds.inbetween, 1.0);
        assert_eq!(y_bounds.min, 0.0);
        assert_eq!(y_bounds.max, 2.0);
        assert_eq!(y_bounds.inbetween, 1.0);
    }

    #[test]
    fn test_generate_legend_entry () {
        let mut metric = HashMap::new();
        metric.insert("instance".to_string(), "testinstance".to_string());
        metric.insert("__name__".to_string(), "test".to_string());
        metric.insert("thing".to_string(), "testthing".to_string());
        let result = generate_legend_entry(&metric);
        let re = Regex::new(r#"\{(.*?=".*?",?)*\}"#).unwrap();

        assert!(re.is_match(&result));
        assert!(result.contains("instance=\"testinstance\""));
        assert!(result.contains("__name__=\"test\""));
        assert!(result.contains("thing=\"testthing\""));
    }
}
