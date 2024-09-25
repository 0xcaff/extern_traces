mod proto;
mod timeline_position;
mod view_state;

use crate::proto::{InitialMessage, SymbolInfo, TraceEvent};
use crate::view_state::{fold_spans, SelectedSpanMetadata, ViewState, ViewStateContainer};
use eframe::egui::{
    vec2, Align, CentralPanel, Color32, Frame, Id, Layout, Rounding, ScrollArea, SidePanel, Stroke,
    TopBottomPanel, Ui, Widget,
};
use eframe::emath::Rect;
use eframe::{egui, emath};
use egui_tiles::{Tabs, Tile, Tree};
use ps4libdoc::LoadedDocumentation;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use std::{io, thread};

struct TreeBehaviorArgs<'a> {
    view_state: &'a mut ViewState,
    docs: &'a LoadedDocumentation,
    last_width: Option<f32>,
}

struct TreeBehavior<'a> {
    args: TreeBehaviorArgs<'a>,
    pane_response: Option<PaneResponse>,
}

#[derive(Eq, PartialEq)]
enum Pane {
    CurrentlySelectedSpanDetail(SpanDetailPane),
    SearchPane(SearchPane),
    CurrentSymbolDetailsPane(SymbolDetailPane),
}

enum PaneResponse {
    OpenPane(Pane),
    FocusPane(Pane),
}

#[derive(Eq, PartialEq)]
struct SpanDetailPane;

impl SpanDetailPane {
    pub fn title(&self) -> egui::WidgetText {
        "span detail".to_string().into()
    }

    pub fn pane_ui(&mut self, args: &mut TreeBehaviorArgs, ui: &mut Ui) -> Option<PaneResponse> {
        ui.with_layout(Layout::top_down(Align::Min), |ui| -> Option<()> {
            let view_state = &mut args.view_state;

            let (_thread, span) = &view_state.selected_span_ref()?;
            let span = (*span).clone();

            ui.allocate_space(vec2(ui.available_width(), 0.));

            let Some(symbol) = view_state
                .initial_message
                .symbols
                .get(span.label_id as usize)
            else {
                ui.label("unable to resolve symbol");

                return None;
            };

            render_symbol_info(&view_state.initial_message, args.docs, symbol, ui);

            ui.horizontal(|ui| {
                ui.label("duration");
                let duration_cycles = span.end_time - span.start_time;
                let duration_seconds =
                    duration_cycles as f64 / view_state.initial_message.tsc_frequency as f64;
                ui.label(format_time(duration_seconds));
            });

            ui.horizontal(|ui| {
                ui.label("start");
                let duration_cycles = span.start_time - view_state.initial_message.anchor_timestamp;
                let duration_seconds =
                    duration_cycles as f64 / view_state.initial_message.tsc_frequency as f64;
                ui.label(format_time(duration_seconds));
            });

            let button_response = ui.button("zoom");
            if button_response.clicked() {
                if let Some(last_width) = args.last_width {
                    view_state.timeline_position_state.pan_to(&span, last_width as _);
                }
            }

            Some(())
        });

        None
    }
}

fn render_symbol_info(args: &InitialMessage, docs: &LoadedDocumentation, symbol: &SymbolInfo, ui: &mut Ui) {
    let library_name = &args
        .libraries
        .get(&(symbol.library_id as u16))
        .map(|it| it.name.as_str());

    let module_name = &args
        .modules
        .get(&(symbol.module_id as u16))
        .map(|it| it.name.as_str());

    ui.horizontal(|ui| {
        ui.label("name");
        ui.label(
            (|| Some(((*library_name)?, (*module_name)?)))()
                .and_then(|(library_name, module_name)| {
                    docs
                        .lookup(module_name, library_name, symbol.name.as_ref())
                        .and_then(|it| it.name.clone())
                })
                .unwrap_or_else(|| format!("nid: {}", symbol.name.as_str())),
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
}

#[derive(Eq, PartialEq)]
struct SearchPane {
    current_text: String,
}

impl SearchPane {
    pub fn title(&self) -> egui::WidgetText {
        "search".to_string().into()
    }

    pub fn pane_ui(&mut self, args: &mut TreeBehaviorArgs, ui: &mut Ui) -> Option<PaneResponse> {
        let mut pane_response = None;

        ScrollArea::vertical().show(ui, |ui| {
            ui.allocate_space(vec2(ui.available_width(), 0.));

            ui.add(
                egui::TextEdit::singleline(&mut self.current_text)
                    .desired_width(ui.available_width()),
            );

            for (symbol_idx, symbol) in args.view_state.initial_message.symbols.iter().enumerate() {
                let library_name = &args
                    .view_state
                    .initial_message
                    .libraries
                    .get(&(symbol.library_id as u16))
                    .map(|it| it.name.as_str());

                let module_name = &args
                    .view_state
                    .initial_message
                    .modules
                    .get(&(symbol.module_id as u16))
                    .map(|it| it.name.as_str());

                let symbol_name = (|| Some(((*library_name)?, (*module_name)?)))()
                    .and_then(|(library_name, module_name)| {
                        args.docs
                            .lookup(module_name, library_name, symbol.name.as_ref())
                            .and_then(|it| it.name.clone())
                    })
                    .unwrap_or_else(|| format!("{}", symbol.name.as_str()));

                let text = format!(
                    "{}::{}::{}",
                    library_name.unwrap_or("unknown"),
                    module_name.unwrap_or("unknown"),
                    symbol_name
                );

                if !text.contains(self.current_text.as_str()) {
                    continue;
                }

                let link_response = ui.link(text);

                if link_response.clicked() {
                    args.view_state.current_symbol_detail = Some(symbol_idx);

                    pane_response.replace(PaneResponse::FocusPane(Pane::CurrentSymbolDetailsPane(
                        SymbolDetailPane,
                    )));
                }
            }
        });

        pane_response
    }
}

#[derive(Eq, PartialEq)]
struct SymbolDetailPane;

impl SymbolDetailPane {
    pub fn title(&self) -> egui::WidgetText {
        // todo: use symbol
        "symbol detail".to_string().into()
    }

    pub fn pane_ui(&mut self, args: &mut TreeBehaviorArgs, ui: &mut Ui) -> Option<PaneResponse> {
        let symbol_idx = args.view_state.current_symbol_detail?;

        ScrollArea::vertical().show(ui, |ui| {
            ui.allocate_space(vec2(ui.available_width(), 0.));

            ScrollArea::vertical().show(ui, |ui| {
                render_symbol_info(
                    &args.view_state.initial_message,
                    args.docs,
                    &args.view_state.initial_message.symbols[symbol_idx],
                    ui,
                );

                let it = args
                    .view_state
                    .threads
                    .iter()
                    .flat_map(|(thread_id, spans)| {
                        spans.spans.iter().map(|span| (*thread_id, span))
                    })
                    .filter(|(thread_id, span)| (span.label_id as usize) == symbol_idx);
                for (idx, (thread_id, span)) in it.enumerate() {
                    ui.label(format!("{:?}", idx));
                }
            });
        });

        None
    }
}

impl<'a> egui_tiles::Behavior<Pane> for TreeBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        self.pane_response = match pane {
            Pane::CurrentlySelectedSpanDetail(pane) => pane.pane_ui(&mut self.args, ui),
            Pane::SearchPane(pane) => pane.pane_ui(&mut self.args, ui),
            Pane::CurrentSymbolDetailsPane(pane) => pane.pane_ui(&mut self.args, ui),
        };

        egui_tiles::UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::CurrentlySelectedSpanDetail(pane) => pane.title(),
            Pane::SearchPane(pane) => pane.title(),
            Pane::CurrentSymbolDetailsPane(pane) => pane.title(),
        }
    }
}

struct SpanViewer {
    last_width: Option<f32>,
    state: ViewStateContainer,
    receiver: Receiver<TraceEvent>,
    docs: LoadedDocumentation,
    tree: Tree<Pane>,
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
            last_width: None,
            state: ViewStateContainer::new(),
            docs,
            receiver,
            start_time: Instant::now(),
            tree: {
                let mut tiles = egui_tiles::Tiles::default();

                let currently_selected_span_detail_pane =
                    tiles.insert_pane(Pane::CurrentlySelectedSpanDetail(SpanDetailPane));

                let search_pane = tiles.insert_pane(Pane::SearchPane(SearchPane {
                    current_text: "".to_string(),
                }));

                let symbol_pane =
                    tiles.insert_pane(Pane::CurrentSymbolDetailsPane(SymbolDetailPane));

                let root = tiles.insert_container(Tabs::new(vec![
                    currently_selected_span_detail_pane,
                    symbol_pane,
                    search_pane,
                ]));

                Tree::new("detail_tree", root, tiles)
            },
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

        let panel = TopBottomPanel::top("summary_toolbar").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.label(" ");
            });
        });

        SidePanel::right("side_panel").show(ctx, |ui| {
            let mut behavior = TreeBehavior {
                args: TreeBehaviorArgs {
                    last_width: self.last_width,
                    view_state,
                    docs: &self.docs,
                },
                pane_response: None,
            };

            self.tree.ui(&mut behavior, ui);

            if let Some(pane_response) = behavior.pane_response {
                match pane_response {
                    PaneResponse::OpenPane(pane) => {
                        let pane_id = self.tree.tiles.insert_pane(pane);
                        let root_id = self.tree.root.unwrap();
                        let container = self.tree.tiles.get_container(root_id).unwrap();

                        self.tree.move_tile_to_container(
                            pane_id,
                            root_id,
                            container.num_children(),
                            false,
                        );
                    }
                    PaneResponse::FocusPane(pane) => {
                        self.tree.make_active(|it, tile| {
                            let Tile::Pane(needle_pane) = tile else {
                                return false;
                            };

                            needle_pane == &pane
                        });
                    }
                }
            }
        });

        let central_panel_response = CentralPanel::default()
            .frame(Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show(ctx, |ui| -> (f64, f64) {
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

                self.last_width.replace(ui.available_width());

                let display_position = view_state.timeline_position_state.position(response.rect.width() as _);
                let (low, hi) = display_position.range(response.rect.width() as _);
                let range = hi - low;

                let cycles_per_pixel = (hi - low) / (response.rect.width() as f64);

                let is_clicked = response.clicked();

                let hover_position = response.hover_pos();

                for (thread_idx, (thread_id, thread_state)) in view_state.threads.iter().enumerate()
                {
                    let same_thread_as_selected = view_state
                        .selected_span
                        .as_ref()
                        .map_or(false, |it| it.thread_id == *thread_id);

                    let (visible_span_start_idx, visible_spans) = {
                        let spans = &thread_state.spans;

                        let i = spans.partition_point(|span| (span.end_time as f64) < low);
                        let j = spans.partition_point(|span| (span.start_time as f64) < hi);

                        let visible_spans = &spans[i..j];

                        (i, visible_spans)
                    };

                    let view_spans = fold_spans(visible_spans, cycles_per_pixel as u64);
                    for (start_idx, end_idx) in view_spans {
                        let start_span = &visible_spans[start_idx];
                        let end_span = &visible_spans[end_idx - 1];
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
                        let span_max =
                            emath::remap(end_span.end_time as f64, (low)..=(hi), x_range.clone());

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
                                    it.span_idx == visible_span_start_idx + start_idx
                                })
                            } else {
                                false
                            };

                            let is_same_type_as_selected =
                                view_state.selected_span_ref().map_or(false, |(_, span)| {
                                    visible_spans[start_idx].label_id == span.label_id
                                });

                            if is_hovered && is_clicked {
                                let selected_span_metadata = SelectedSpanMetadata {
                                    thread_id: *thread_id,
                                    span_idx: visible_span_start_idx + start_idx,
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
                if hover_position.is_some() {
                    let scroll_delta = ctx.input(|it| it.smooth_scroll_delta);
                    let percentage = scroll_delta.x / response.rect.width();
                    let diff = -(percentage as f64 * range);

                    view_state.timeline_position_state.translate_x(diff, display_position);
                }

                // Zooming
                (|| -> Option<()> {
                    let hover_position = hover_position?;
                    let zoom_delta = ctx.input(|it| it.zoom_delta()) as f64;
                    let anchor_position = (hover_position.x / response.rect.width()) as f64;

                    view_state.timeline_position_state.zoom_anchored(
                        1. / zoom_delta,
                        anchor_position,
                        response.rect.width() as f64,
                        display_position,
                    );

                    Some(())
                })();

                (low, hi)
            });

        egui::Area::new(Id::new("summary_toolbar_contents"))
            .fixed_pos(panel.response.rect.min)
            .show(ctx, |ui| {
                ui.set_max_size(panel.response.rect.size());

                ui.horizontal_centered(|ui| {
                    ui.add_space(ui.spacing().menu_spacing);

                    ui.label(format!("spans: {}", view_state.total_spans()));
                    ui.label(format!("threads: {}", view_state.threads.len()));

                    {
                        let range = central_panel_response.inner;
                        let (min, max) = range;
                        let duration =
                            (max - min) / view_state.initial_message.tsc_frequency as f64;
                        ui.label(format!("visible: {}", format_time(duration)));
                    }

                    {
                        let range = central_panel_response.inner;
                        let (min, _max) = range;
                        let duration = (min - view_state.initial_message.anchor_timestamp as f64)
                            / view_state.initial_message.tsc_frequency as f64;
                        ui.label(format!("start: {}", format_time(duration)));
                    }

                    {
                        let range = central_panel_response.inner;
                        let (_min, max) = range;
                        let duration = (max - view_state.initial_message.anchor_timestamp as f64)
                            / view_state.initial_message.tsc_frequency as f64;
                        ui.label(format!("end: {}", format_time(duration)));
                    }
                });

                ui.add_space(ui.spacing().menu_spacing);
            });
    }
}

fn format_time(seconds: f64) -> String {
    let abs_seconds = seconds.abs();
    let sign = if seconds < 0.0 { "-" } else { "" };

    let (value, unit) = if abs_seconds < 1e-6 {
        (abs_seconds * 1e9, "ns")
    } else if abs_seconds < 1e-3 {
        (abs_seconds * 1e6, "µs")
    } else if abs_seconds < 1.0 {
        (abs_seconds * 1e3, "ms")
    } else if abs_seconds < 60.0 {
        (abs_seconds, "s")
    } else {
        (abs_seconds / 60.0, "m")
    };

    format!("{}{:.2} {}", sign, value, unit)
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
        Box::new(|cc| Ok(Box::new(SpanViewer::new(cc)))),
    )
}
