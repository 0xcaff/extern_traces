mod proto;
mod view_state;

use crate::proto::{InitialMessage, SpanEvent, TraceEvent};
use crate::view_state::ViewState;
use eframe::egui::{Color32, Rounding, Stroke};
use eframe::emath::Rect;
use eframe::{egui, emath};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use std::{io, thread};

struct SpanViewer {
    state: ViewState,
    receiver: Receiver<TraceEvent>,

    start_time: Instant,
}

impl SpanViewer {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = channel();

        thread::spawn(move || {
            if let Err(e) = run_server(sender.clone()) {
                eprintln!("Server error: {}", e);
            }
        });

        Self {
            state: ViewState::new(),
            receiver,
            start_time: Instant::now(),
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            self.state.update_trace(event);
        }
    }
}

impl eframe::App for SpanViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("spans: {}", self.state.total_spans()));
            ui.label(format!("threads: {}", self.state.threads.len()));
            ui.label(format!("range: {:#?}", self.state.range()));

            let available_size = ui.available_size();
            let (low, hi) = self.state.range();
            let range = hi - low;

            let low_f = low as f32;
            let hi_f = hi as f32;

            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::focusable_noninteractive());

            let hover_position = ctx.input(|it| it.pointer.hover_pos());

            for (thread_idx, (_thread_id, thread_state)) in self.state.threads.iter().enumerate() {
                let visible_spans = thread_state
                    .spans
                    .iter()
                    .filter(|it| it.end_time >= low && it.start_time < hi)
                    .collect::<Vec<_>>();

                for span in visible_spans {
                    let span_min = emath::remap(
                        span.start_time as f32,
                        low_f..=hi_f,
                        0f32..=available_size.x,
                    );
                    let span_max =
                        emath::remap(span.end_time as f32, low_f..=hi_f, 0f32..=available_size.x);

                    let rect = Rect {
                        min: egui::pos2(span_min, thread_idx as f32 * 10.0f32),
                        max: egui::pos2(span_max, thread_idx as f32 * 10.0f32 + 8.0f32),
                    };

                    let should_highlight = if let Some(hover_position) = hover_position {
                        if rect.contains(hover_position) {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !should_highlight {
                        painter.rect_filled(
                            rect,
                            Rounding::default(),
                            Color32::GREEN,
                        );
                    } else {
                        painter.rect_stroke(
                            rect,
                            Rounding::default(),
                            Stroke::new(1.0f32, Color32::BLACK),
                        );
                    }

                }
            }

            // Panning
            {
                let scroll_delta = ctx.input(|it| it.smooth_scroll_delta);
                let percentage = scroll_delta.x / available_size.x;
                let diff = -((percentage * (range as f32)) as i64);

                self.state.translate_x(diff);
            }

            // Zooming
            {
                let Some(hover_position) = hover_position else {
                    return;
                };

                let zoom_delta = ctx.input(|it| it.zoom_delta());
                let anchor_position = hover_position.x / available_size.x;

                self.state
                    .zoom_anchored(1.0f32 / zoom_delta, anchor_position);
            }
        });

        ctx.request_repaint();
    }
}

fn run_server(sender: Sender<TraceEvent>) -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9090")?;
    println!("server listening on 0.0.0.0:9090");

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

fn handle_client(mut stream: TcpStream, sender: Sender<TraceEvent>) -> io::Result<()> {
    let initial_message = InitialMessage::read(&mut stream)?;
    sender.send(TraceEvent::Start(initial_message)).unwrap();

    loop {
        sender
            .send(TraceEvent::Span(SpanEvent::read(&mut stream)?))
            .unwrap();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "extern_traces",
        options,
        Box::new(|cc| Box::new(SpanViewer::new(cc))),
    )
}