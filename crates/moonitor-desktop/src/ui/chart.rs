//! Shared resource chart. Receipt time, bounded window, explicit gaps and inspection.
use super::*;
use iced::{Point, Rectangle, mouse};
const WINDOW: u64 = 300_000;
const GAP: u64 = 7_500;
pub(super) const SERIES: [Color; 3] = [CPU, MEMORY, GPU];
pub(super) struct ResourceChart {
    points: Vec<HistoryPoint>,
    end: u64,
}
impl ResourceChart {
    pub(super) fn new(points: &std::collections::VecDeque<HistoryPoint>, end: u64) -> Self {
        Self {
            points: points
                .iter()
                .filter(|p| {
                    p.received_at_ms >= end.saturating_sub(WINDOW) && p.received_at_ms <= end
                })
                .cloned()
                .collect(),
            end,
        }
    }
    fn nearest(&self, at: u64) -> Option<&HistoryPoint> {
        self.points
            .iter()
            .min_by_key(|p| p.received_at_ms.abs_diff(at))
            .filter(|p| p.received_at_ms.abs_diff(at) <= GAP / 2)
    }
}
fn value(p: &HistoryPoint, field: usize) -> Option<f64> {
    [p.cpu_percent, p.memory_percent, p.gpu_percent][field]
        .filter(|v| v.is_finite() && (0.0..=100.0).contains(v))
}
fn connected(a: &HistoryPoint, b: &HistoryPoint, field: usize) -> bool {
    b.received_at_ms > a.received_at_ms
        && b.received_at_ms - a.received_at_ms <= GAP
        && value(a, field).is_some()
        && value(b, field).is_some()
}
impl canvas::Program<Message> for ResourceChart {
    type State = Option<Point>;
    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<widget::Action<Message>> {
        if matches!(
            event,
            iced::Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft)
        ) {
            let next = cursor.position_in(bounds);
            if *state != next {
                *state = next;
                return Some(widget::Action::request_redraw());
            }
        }
        None
    }
    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let left = 33.0;
        let top = 9.0;
        let width = (bounds.width - left - 8.0).max(1.0);
        let height = (bounds.height - top - 24.0).max(1.0);
        let start = self.end.saturating_sub(WINDOW);
        for v in [0.0, 50.0, 100.0] {
            let y = top + height * (1.0 - v / 100.0);
            frame.stroke(
                &canvas::Path::line(Point::new(left, y), Point::new(left + width, y)),
                canvas::Stroke::default()
                    .with_width(0.5)
                    .with_color(tone(LINE, theme)),
            );
            frame.fill_text(canvas::Text {
                content: format!("{v:.0}"),
                position: Point::new(left - 8.0, y),
                color: tone(MUTED, theme),
                size: 10.0.into(),
                font: FONT,
                align_x: alignment::Horizontal::Right.into(),
                align_y: alignment::Vertical::Center,
                ..Default::default()
            });
        }
        for (fraction, label, align) in [
            (0.0, "−5 min", alignment::Horizontal::Left),
            (0.5, "−2 min 30 s", alignment::Horizontal::Center),
            (1.0, "Ora", alignment::Horizontal::Right),
        ] {
            frame.fill_text(canvas::Text {
                content: label.into(),
                position: Point::new(left + width * fraction, top + height + 9.0),
                color: tone(MUTED, theme),
                size: 10.0.into(),
                font: FONT,
                align_x: align.into(),
                ..Default::default()
            });
        }
        for (field, color) in SERIES.into_iter().enumerate() {
            let color = tone(color, theme);
            let point = |p: &HistoryPoint| {
                Point::new(
                    left + width * p.received_at_ms.saturating_sub(start) as f32 / WINDOW as f32,
                    top + height * (1.0 - value(p, field).unwrap_or(0.0) as f32 / 100.0),
                )
            };
            for pair in self.points.windows(2) {
                if connected(&pair[0], &pair[1], field) {
                    frame.stroke(
                        &canvas::Path::line(point(&pair[0]), point(&pair[1])),
                        canvas::Stroke::default().with_width(1.8).with_color(color),
                    );
                }
            }
            if let Some(last) = self.points.last().filter(|p| value(p, field).is_some()) {
                frame.fill(&canvas::Path::circle(point(last), 2.0), color);
            }
        }
        if self.points.is_empty() {
            frame.fill_text(canvas::Text {
                content: "In attesa di campioni nella finestra".into(),
                position: Point::new(left + width / 2.0, top + height / 2.0),
                color: tone(MUTED, theme),
                size: 11.0.into(),
                font: FONT,
                align_x: alignment::Horizontal::Center.into(),
                ..Default::default()
            });
        }
        if let Some(cursor) = state
            .filter(|p| p.x >= left && p.x <= left + width && p.y >= top && p.y <= top + height)
        {
            let at = start + (((cursor.x - left) / width) * WINDOW as f32) as u64;
            frame.stroke(
                &canvas::Path::line(
                    Point::new(cursor.x, top),
                    Point::new(cursor.x, top + height),
                ),
                canvas::Stroke::default()
                    .with_width(1.0)
                    .with_color(tone(MUTED, theme)),
            );
            let label = if let Some(p) = self.nearest(at) {
                let secs = p.received_at_ms / 1000;
                format!(
                    "{:02}:{:02}:{:02} UTC  ·  CPU {}  RAM {}  GPU {}",
                    secs / 3600 % 24,
                    secs / 60 % 60,
                    secs % 60,
                    pct(p.cpu_percent),
                    pct(p.memory_percent),
                    pct(p.gpu_percent)
                )
            } else {
                "Nessun campione in questo intervallo".into()
            };
            let box_width = (label.chars().count() as f32 * 5.3 + 16.0).min(bounds.width);
            let x = (cursor.x - box_width / 2.0).clamp(0.0, (bounds.width - box_width).max(0.0));
            let y = top + 3.0;
            frame.fill_rectangle(
                Point::new(x, y),
                iced::Size::new(box_width, 24.0),
                tone(PAPER, theme),
            );
            frame.fill_text(canvas::Text {
                content: label,
                position: Point::new(x + 8.0, y + 6.0),
                color: tone(INK, theme),
                size: 10.0.into(),
                font: FONT,
                ..Default::default()
            });
        }
        vec![frame.into_geometry()]
    }
    fn mouse_interaction(
        &self,
        _: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn point(t: u64, v: Option<f64>) -> HistoryPoint {
        HistoryPoint {
            received_at_ms: t,
            cpu_percent: v,
            memory_percent: v,
            gpu_percent: v,
        }
    }
    #[test]
    fn gaps_missing_values_and_reversed_time_are_never_connected() {
        let zero = point(10_000, Some(0.0));
        assert!(connected(&zero, &point(15_000, Some(10.0)), 0));
        for other in [
            point(15_000, None),
            point(20_000, Some(0.0)),
            point(5_000, Some(1.0)),
            point(10_000, Some(1.0)),
            point(15_000, Some(f64::NAN)),
        ] {
            assert!(!connected(&zero, &other, 0));
        }
    }
    #[test]
    fn inspection_does_not_fill_gaps_or_include_future_values() {
        let chart = ResourceChart::new(
            &[point(10_000, Some(0.0)), point(30_000, Some(100.0))].into(),
            20_000,
        );
        assert_eq!(chart.nearest(10_000).unwrap().cpu_percent, Some(0.0));
        assert!(chart.nearest(20_000).is_none());
        assert_eq!(chart.points.len(), 1);
    }
}
