mod proto;
mod state;

use crate::proto::SpanEvent;
use crate::state::State;
use eframe::egui;
use std::net::{TcpListener, TcpStream};
use std::ops::{Add, Sub};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

struct TraceViewer {
    events: Receiver<SpanEvent>,
    state: State,
    view_start: u64,
    view_end: u64,
}

impl TraceViewer {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = channel();

        thread::spawn(move || {
            if let Err(e) = run_server(sender.clone()) {
                eprintln!("server error: {}", e);
            }
        });

        Self {
            events: receiver,
            state: State::new(),
            view_start: 0,
            view_end: 1000000000,
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            self.state.update(event);
        }
    }

    fn draw_spans(&mut self, ui: &mut egui::Ui) {
        ui.label(format!(
            "view start: {}, view end: {}",
            self.view_start, self.view_end
        ));

        ui.label(format!("total spans: {}", self.state.total_spans()));

        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::drag());
        let rect = response.rect;

        let time_to_x = |time: u64| -> f32 {
            let ratio = (time - self.view_start) as f32 / (self.view_end - self.view_start) as f32;
            rect.left() + ratio * rect.width()
        };

        for (idx, (thread_id, thread)) in self.state.threads.iter().enumerate() {
            let y = rect.top() + idx as f32 * 30.0;

            painter.text(
                egui::pos2(rect.left(), y),
                egui::Align2::LEFT_CENTER,
                format!("thread {}", thread_id),
                egui::FontId::default(),
                egui::Color32::WHITE,
            );

            for span in &thread.spans {
                if span.end_time < self.view_start || span.start_time > self.view_end {
                    continue;
                }

                let start_x = time_to_x(span.start_time).max(rect.left());
                let end_x = time_to_x(span.end_time).min(rect.right());

                painter.rect_filled(
                    egui::Rect::from_two_pos(
                        egui::pos2(start_x, y + 2.0),
                        egui::pos2(end_x, y + 18.0),
                    ),
                    0.0,
                    egui::Color32::from_rgb(100, 150, 200),
                );

                let span_center_x = (start_x + end_x) / 2.0;
                painter.text(
                    egui::pos2(span_center_x, y + 10.0),
                    egui::Align2::CENTER_CENTER,
                    format!("Label {}", span.label_id),
                    egui::FontId::proportional(10.0),
                    egui::Color32::BLACK,
                );
            }

            ui.label(format!(
                "thread {} spans: {}",
                thread_id,
                thread.spans.len(),
            ));
        }

        if response.dragged() {
            let delta = response.drag_delta().x / rect.width();
            let range = (self.view_end - self.view_start) as f32;
            let time_delta = (delta * range) as i32;

            self.view_start = add_signed_saturating(self.view_start, time_delta);
            self.view_end = add_signed_saturating(self.view_end, time_delta);
        }
    }
}

fn add_signed_saturating(base: u64, value: i32) -> u64 {
    if value.is_positive() {
        base.saturating_add(u64::from(value as u16))
    } else {
        base.saturating_sub(u64::from((value * -1) as u16))
    }
}

impl eframe::App for TraceViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("spans: {}", self.state.total_spans()));
            ui.label(format!("threads: {}", self.state.threads.len()));

            self.draw_spans(ui);
        });

        ctx.request_repaint();
    }
}

fn handle_client(mut stream: TcpStream, sender: Sender<SpanEvent>) -> anyhow::Result<()> {
    loop {
        let message = SpanEvent::read(&mut stream)?;
        sender.send(message)?;
    }
}

fn run_server(sender: Sender<SpanEvent>) -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9090")?;
    println!("Server listening on 0.0.0.0:9090");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let sender_clone = sender.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, sender_clone) {
                        eprintln!("error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("error accepting client: {}", e);
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "extern_trace_viewer",
        options,
        Box::new(|cc| Box::new(TraceViewer::new(cc))),
    )
}
