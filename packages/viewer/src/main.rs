mod proto;
mod view_state;

use crate::proto::{InitialMessage, TraceEvent};
use crate::view_state::{fold_spans, SelectedSpanMetadata, ViewState};
use eframe::egui::{Align, CentralPanel, Color32, Frame, Layout, Rounding, SidePanel, Stroke};
use eframe::emath::{Rangef, Rect};
use eframe::{egui, emath};
use ps4libdoc::LoadedDocumentation;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use std::{io, thread};

struct SpanViewer {
    state: ViewState,
    receiver: Receiver<TraceEvent>,
    docs: LoadedDocumentation,

    start_time: Instant,
}

impl SpanViewer {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = channel();

        let docs = LoadedDocumentation::bundled().unwrap();

        thread::spawn(move || {
            if let Err(e) = run_server(sender.clone()) {
                eprintln!("Server error: {}", e);
            }
        });

        Self {
            state: ViewState::new(),
            docs,
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

        SidePanel::right("side_panel").show(ctx, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                ui.label(format!("spans: {}", self.state.total_spans()));
                ui.label(format!("threads: {}", self.state.threads.len()));
                ui.label(format!("range: {:#?}", self.state.range()));

                if let Some((_thread, span)) = &self.state.selected_span_ref() {
                    if let Some(initial_message) = &self.state.initial_message {
                        let symbol = &initial_message.symbols[span.label_id as usize];

                        let library = &initial_message
                            .libraries
                            .get(&(symbol.library_id as u16))
                            .unwrap();

                        let module = &initial_message
                            .modules
                            .get(&(symbol.module_id as u16))
                            .unwrap();

                        ui.label(format!("{}", module.name));
                        ui.label(format!("{}", library.name));
                        ui.label(format!("{}", symbol.name));

                        let resolved = self.docs.lookup(&module.name, &library.name, &symbol.name);

                        ui.label(format!("{:#?}", resolved.and_then(|it| it.name.as_ref())));
                    }
                }
            });
        });

        CentralPanel::default()
            .frame(Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                let available_size = ui.available_size();
                let (low, hi) = self.state.range();
                let range = hi - low;

                let (ticks, interval) = {
                    let magnitude = range.ilog10();
                    let base_interval = 10u64.pow(magnitude);

                    let interval = {
                        let segments = range / base_interval;
                        if segments < 5 {
                            base_interval / 10
                        } else {
                            base_interval
                        }
                    };

                    let ticks = (range / interval) + 1;

                    (ticks, interval)
                };

                let low_f = low as f32;
                let hi_f = hi as f32;

                let cycles_per_pixel = (hi_f - low_f) / available_size.x;
                let (response, painter) = ui
                    .allocate_painter(ui.available_size(), egui::Sense::focusable_noninteractive());

                {
                    let base = (low_f / interval as f32).floor() * interval as f32;
                    for idx in 0..ticks {
                        let position = emath::remap(
                            base + idx as f32 * interval as f32,
                            low_f..=hi_f,
                            0f32..=available_size.x,
                        );

                        painter.vline(
                            position,
                            Rangef::new(0.0, available_size.y),
                            Stroke::new(1.0f32, Color32::BLACK),
                        );
                    }
                }

                let is_clicked = ctx.input(|it| it.pointer.any_click());

                let hover_position = ctx.input(|it| it.pointer.hover_pos());

                for (thread_idx, (thread_id, thread_state)) in self.state.threads.iter().enumerate()
                {
                    let (visible_span_idx, visible_spans) = thread_state
                        .spans
                        .iter()
                        .enumerate()
                        .filter(|(_idx, it)| it.end_time >= low && it.start_time < hi)
                        .unzip::<_, _, Vec<_>, Vec<_>>();
                    
                    let view_spans = fold_spans(&visible_spans, cycles_per_pixel as u64);
                    for (start_idx, end_idx) in view_spans {
                        let start_span = visible_spans[start_idx];
                        let end_span = visible_spans[end_idx - 1];
                        let folded = start_idx != end_idx - 1;

                        let span_min = emath::remap(
                            start_span.start_time as f32,
                            low_f..=hi_f,
                            0f32..=available_size.x,
                        );
                        let span_max = emath::remap(
                            end_span.end_time as f32,
                            low_f..=hi_f,
                            0f32..=available_size.x,
                        );

                        let rect = Rect {
                            min: egui::pos2(span_min, thread_idx as f32 * 10.0f32),
                            max: egui::pos2(span_max, thread_idx as f32 * 10.0f32 + 8.0f32),
                        };

                        let is_hovered = if let Some(hover_position) = hover_position {
                            if rect.contains(hover_position) {
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if folded {
                            painter.rect_filled(rect, Rounding::default(), Color32::YELLOW);
                        } else {
                            let is_same_type_as_selected =
                                self.state.selected_span_ref().map_or(false, |(_, span)| {
                                    visible_spans[start_idx].label_id == span.label_id
                                });

                            if is_same_type_as_selected {}

                            if is_hovered && is_clicked {
                                let selected_span_metadata = SelectedSpanMetadata {
                                    thread_id: *thread_id,
                                    span_idx: visible_span_idx[start_idx],
                                };

                                self.state.selected_span.replace(selected_span_metadata);
                            };

                            painter.rect_filled(
                                rect,
                                Rounding::default(),
                                if is_same_type_as_selected {
                                    Color32::LIGHT_BLUE
                                } else {
                                    Color32::GREEN
                                },
                            );
                        }

                        if is_hovered {
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
        sender.send(TraceEvent::read(&mut stream)?).unwrap();
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
