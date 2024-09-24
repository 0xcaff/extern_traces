mod proto;
mod view_state;

use crate::proto::{InitialMessage, TraceEvent};
use crate::view_state::{fold_spans, SelectedSpanMetadata, ViewStateContainer};
use eframe::egui::{
    vec2, Align, CentralPanel, Color32, Frame, Layout, Rounding, SidePanel, Stroke, TopBottomPanel,
};
use eframe::emath::Rect;
use eframe::{egui, emath};
use ps4libdoc::LoadedDocumentation;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use std::{io, thread};

struct SpanViewer {
    state: ViewStateContainer,
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
            state: ViewStateContainer::new(),
            docs,
            receiver,
            start_time: Instant::now(),
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            match &mut self.state {
                state @ ViewStateContainer::Empty => {
                    let TraceEvent::Start(init) = event else {
                        unimplemented!();
                    };

                    state.initialize(init);
                }
                ViewStateContainer::Initialized(state) => {
                    let TraceEvent::Span(init) = event else {
                        continue;
                    };
                    state.update_span(init);
                }
            };
        }
    }
}

impl eframe::App for SpanViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_events();

        let ViewStateContainer::Initialized(view_state) = &mut self.state else {
            return;
        };

        TopBottomPanel::top("test").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.label(format!("spans: {}", view_state.total_spans()));
                ui.label(format!("threads: {}", view_state.threads.len()));
                // todo: calculate range here
            });
        });

        CentralPanel::default()
            .frame(Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                if let Some((_thread, span)) = &view_state.selected_span_ref() {
                    SidePanel::right("side_panel").show(ctx, |ui| {
                        ui.with_layout(Layout::top_down(Align::Min), |ui| {
                            ui.allocate_space(vec2(ui.available_width(), 0.));

                            if let Some(symbol) = view_state
                                .initial_message
                                .symbols
                                .get(span.label_id as usize)
                            {
                                let library_name = &view_state
                                    .initial_message
                                    .libraries
                                    .get(&(symbol.library_id as u16))
                                    .map(|it| it.name.as_str());

                                let module_name = &view_state
                                    .initial_message
                                    .modules
                                    .get(&(symbol.module_id as u16))
                                    .map(|it| it.name.as_str());

                                ui.horizontal(|ui| {
                                    ui.label("name");
                                    ui.label(
                                        (|| Some(((*library_name)?, (*module_name)?)))()
                                            .and_then(|(library_name, module_name)| {
                                                self.docs
                                                    .lookup(
                                                        module_name,
                                                        library_name,
                                                        symbol.name.as_ref(),
                                                    )
                                                    .and_then(|it| it.name.clone())
                                            })
                                            .unwrap_or_else(|| {
                                                format!("nid: {}", symbol.name.as_str())
                                            }),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label("library");
                                    ui.label(library_name.unwrap_or("unknown"));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("module");
                                    ui.label(module_name.unwrap_or("unknown"));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("duration");
                                    let duration_cycles = span.end_time - span.start_time;
                                    let duration_seconds = duration_cycles as f64
                                        / view_state.initial_message.tsc_frequency as f64;
                                    ui.label(format_time(duration_seconds));
                                });
                            } else {
                                ui.label("unable to resolve symbol");
                            }
                        });
                    });
                }

                CentralPanel::default()
                    .frame(Frame {
                        fill: ctx.style().visuals.panel_fill,
                        ..Default::default()
                    })
                    .show(ctx, |ui| {
                        let (response, painter) =
                            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

                        let display_position = view_state.position(response.rect.width() as _);
                        let (low, hi) = display_position.range(response.rect.width() as _);
                        let range = hi - low;

                        let cycles_per_pixel = (hi - low) / (response.rect.width() as f64);

                        let is_clicked = response.clicked();

                        let hover_position = response.hover_pos();

                        for (thread_idx, (thread_id, thread_state)) in
                            view_state.threads.iter().enumerate()
                        {
                            let same_thread_as_selected = view_state
                                .selected_span
                                .as_ref()
                                .map_or(false, |it| it.thread_id == *thread_id);

                            let (visible_span_idx, visible_spans) = thread_state
                                .spans
                                .iter()
                                .enumerate()
                                .filter(|(_idx, it)| {
                                    it.end_time as f64 >= low && (it.start_time as f64) < hi
                                })
                                .unzip::<_, _, Vec<_>, Vec<_>>();

                            let view_spans = fold_spans(&visible_spans, cycles_per_pixel as u64);
                            for (start_idx, end_idx) in view_spans {
                                let start_span = visible_spans[start_idx];
                                let end_span = visible_spans[end_idx - 1];
                                let folded = start_idx != end_idx - 1;

                                let x_range = {
                                    let range = response.rect.x_range();
                                    (range.min as f64)..=(range.max as f64)
                                };

                                let span_min = emath::remap(
                                    start_span.start_time as f64,
                                    (low)..=(hi),
                                    x_range.clone(),
                                );
                                let span_max = emath::remap(
                                    end_span.end_time as f64,
                                    (low)..=(hi),
                                    x_range.clone(),
                                );

                                let rect = Rect {
                                    min: egui::pos2(
                                        span_min as f32,
                                        response.rect.min.y + thread_idx as f32 * 10.0f32,
                                    ),
                                    max: egui::pos2(
                                        span_max as f32,
                                        response.rect.min.y + thread_idx as f32 * 10.0f32 + 8.0f32,
                                    ),
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
                                    let is_selected = if same_thread_as_selected {
                                        view_state.selected_span.as_ref().map_or(false, |it| {
                                            it.span_idx == visible_span_idx[start_idx]
                                        })
                                    } else {
                                        false
                                    };

                                    let is_same_type_as_selected = view_state
                                        .selected_span_ref()
                                        .map_or(false, |(_, span)| {
                                            visible_spans[start_idx].label_id == span.label_id
                                        });

                                    if is_hovered && is_clicked {
                                        let selected_span_metadata = SelectedSpanMetadata {
                                            thread_id: *thread_id,
                                            span_idx: visible_span_idx[start_idx],
                                        };

                                        view_state.selected_span.replace(selected_span_metadata);
                                    };

                                    painter.rect_filled(
                                        rect,
                                        Rounding::default(),
                                        if is_selected {
                                            Color32::BLUE
                                        } else if is_same_type_as_selected {
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
                            let percentage = scroll_delta.x / response.rect.width();
                            let diff = -(percentage as f64 * range);

                            view_state.translate_x(diff, display_position);
                        }

                        // Zooming
                        {
                            let Some(hover_position) = hover_position else {
                                return;
                            };

                            let zoom_delta = ctx.input(|it| it.zoom_delta()) as f64;
                            let anchor_position = (hover_position.x / response.rect.width()) as f64;

                            view_state.zoom_anchored(
                                1. / zoom_delta,
                                anchor_position,
                                response.rect.width() as f64,
                                display_position,
                            );
                        }
                    });
            });
    }
}

fn format_time(seconds: f64) -> String {
    if seconds < 1e-6 {
        format!("{:.2} ns", seconds * 1e9)
    } else if seconds < 1e-3 {
        format!("{:.2} µs", seconds * 1e6)
    } else if seconds < 1.0 {
        format!("{:.2} ms", seconds * 1e3)
    } else if seconds < 60.0 {
        format!("{:.2} s", seconds)
    } else {
        let minutes = seconds / 60.0;
        format!("{:.2} m", minutes)
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
