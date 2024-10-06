mod panes;
mod timeline_position;
mod utils;
mod view_state;

use crate::app::tracing::panes::{create_tree, Pane, PaneResponse, TreeBehavior, TreeBehaviorArgs};
use crate::app::tracing::utils::{format_time, human_readable_size};
use crate::app::tracing::view_state::{SpanRef, ThreadSpan, ViewState, ViewStateContainer};
use crate::app::Scene;
use crate::proto::{InitialMessage, LibraryInfo, ModuleInfo, TraceCommand, TraceEvent};
use eframe::egui::scroll_area::ScrollBarVisibility;
use eframe::egui::{
    pos2, vec2, Align, Align2, CentralPanel, Color32, Context, FontFamily, FontId, Frame, Id,
    Layout, Margin, Painter, Rect, Rounding, ScrollArea, SidePanel, Stroke, TopBottomPanel, Ui,
    Vec2b,
};
use eframe::{egui, emath};
use egui_tiles::{Tile, Tree};
use ps4libdoc::{LoadedDocumentation, SymbolDocumentation};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use std::{io, thread};

pub struct TracingScene {
    last_width: Option<f32>,
    state: ViewStateContainer,
    receiver: Receiver<TraceEvent>,
    tree: Tree<Pane>,
    loading_thread_handle: JoinHandle<io::Result<()>>,
    sender_commands: Option<Sender<TraceCommand>>,
}

impl TracingScene {
    pub fn from_network(ctx: Context, socket_addr: SocketAddr) -> TracingScene {
        let (sender, receiver) = channel();
        let (sender_commands, receiver_commands) = channel();

        let loading_thread_handle =
            thread::spawn(move || run_server(ctx, socket_addr, sender.clone(), receiver_commands));

        TracingScene {
            last_width: None,
            state: ViewStateContainer::new(),
            receiver,
            sender_commands: Some(sender_commands),
            tree: create_tree(),
            loading_thread_handle,
        }
    }

    pub fn from_file_path(ctx: Context, path: PathBuf) -> TracingScene {
        let (sender, receiver) = channel();

        let loading_thread_handle = thread::spawn(move || -> io::Result<()> {
            let file = File::open(path)?;

            read_stream(ctx, file, sender)?;

            Ok(())
        });

        TracingScene {
            last_width: None,
            state: ViewStateContainer::Empty,
            receiver,
            tree: create_tree(),
            loading_thread_handle,
            sender_commands: None,
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

    pub fn update(&mut self, ctx: &Context, docs: &LoadedDocumentation) -> Option<Scene> {
        self.process_events();
        let mut next_scene = None;

        let ViewStateContainer::Initialized(view_state) = &mut self.state else {
            return None;
        };

        let panel = TopBottomPanel::top("summary_toolbar").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.label(" ");
            });
        });

        SidePanel::right("side_panel")
            .frame(Frame::side_top_panel(&ctx.style()).inner_margin(Margin::ZERO))
            .show(ctx, |ui| {
                let mut behavior = TreeBehavior {
                    args: TreeBehaviorArgs {
                        last_width: self.last_width,
                        view_state,
                        docs: &docs,
                        commands: self.sender_commands.as_mut(),
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
                            self.tree.make_active(|_it, tile| {
                                let Tile::Pane(needle_pane) = tile else {
                                    return false;
                                };

                                needle_pane.key() == pane
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
            .show(ctx, |ui| {
                render_central_panel_contents(ui, ctx, view_state, &mut self.last_width, &docs)
            });

        egui::Area::new(Id::new("summary_toolbar_contents"))
            .fixed_pos(panel.response.rect.min)
            .show(ctx, |ui| {
                ui.set_max_size(panel.response.rect.size());

                ui.horizontal_centered(|ui| {
                    ui.add_space(ui.spacing().menu_spacing);

                    ui.label(format!(
                        "spans: {}",
                        human_readable_size(view_state.total_spans())
                    ));

                    {
                        let range = central_panel_response.inner;
                        let (_, _, spans) = range;
                        ui.label(format!("visible spans: {}", human_readable_size(spans)));
                    }

                    ui.label(format!("threads: {}", view_state.threads.len()));

                    {
                        let range = central_panel_response.inner;
                        let (min, max, _) = range;
                        let duration =
                            (max - min) / view_state.initial_message.tsc_frequency as f64;
                        ui.label(format!("visible: {}", format_time(duration)));
                    }

                    {
                        let range = central_panel_response.inner;
                        let (min, _max, _) = range;
                        let duration = (min - view_state.initial_message.anchor_timestamp as f64)
                            / view_state.initial_message.tsc_frequency as f64;
                        ui.label(format!("start: {}", format_time(duration)));
                    }

                    {
                        let range = central_panel_response.inner;
                        let (_min, max, _) = range;
                        let duration = (max - view_state.initial_message.anchor_timestamp as f64)
                            / view_state.initial_message.tsc_frequency as f64;
                        ui.label(format!("end: {}", format_time(duration)));
                    }
                });

                ui.add_space(ui.spacing().menu_spacing);
            });

        next_scene
    }
}

fn read_stream(ctx: Context, mut stream: impl Read, sender: Sender<TraceEvent>) -> io::Result<()> {
    let initial_message = InitialMessage::read(&mut stream)?;
    sender.send(TraceEvent::Start(initial_message)).unwrap();
    ctx.request_repaint();

    loop {
        let trace_event = TraceEvent::read(&mut stream)?;
        if let TraceEvent::CountersUpdate(counters) = &trace_event {
            println!("{:#?}", counters);
        }

        sender.send(trace_event).unwrap();
        ctx.request_repaint()
    }
}

fn run_server(
    ctx: Context,
    addr: SocketAddr,
    sender: Sender<TraceEvent>,
    receiver: Receiver<TraceCommand>,
) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    let Some(connection) = listener.incoming().next() else {
        return Ok(());
    };

    let stream = connection?;
    let mut stream_clone = stream.try_clone()?;

    let sender_clone = sender.clone();
    let ctx_clone = ctx.clone();
    thread::spawn(move || {
        if let Err(e) = read_stream(ctx_clone, stream, sender_clone) {
            eprintln!("error handling client: {}", e);
        }
    });

    for command in receiver.into_iter() {
        stream_clone.write_all(&[command as u8])?;
    }

    Ok(())
}

fn render_central_panel_contents(
    ui: &mut Ui,
    ctx: &Context,
    view_state: &mut ViewState,
    last_width: &mut Option<f32>,
    docs: &LoadedDocumentation,
) -> (f64, f64, usize) {
    let available_width = ui.available_width();
    last_width.replace(available_width);

    let thread_row_height = 20.;
    let thread_row_padding_vertical = 2.;

    let total_height = view_state.threads.len() as f32 * thread_row_height;

    let response = ScrollArea::vertical()
        .auto_shrink(Vec2b::FALSE)
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
        .show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                vec2(available_width, total_height),
                egui::Sense::click_and_drag(),
            );

            let display_position = view_state
                .timeline_position_state
                .position(response.rect.width() as _);

            let (low, hi) = display_position.range(response.rect.width() as _);
            let range = hi - low;

            let is_clicked = response.clicked();

            let hover_position = response.hover_pos();

            let mut total_visible = 0;

            for (thread_idx, (thread_id, thread_state)) in view_state.threads.iter_mut().enumerate()
            {
                let same_thread_as_selected = view_state
                    .selected_span
                    .as_ref()
                    .map_or(false, |it| it.thread_id == *thread_id);

                let visible_range = {
                    let spans = &thread_state.spans;

                    let i = spans.partition_point(|span| (span.end_time as f64) < low);
                    let j = spans.partition_point(|span| (span.start_time as f64) < hi);

                    i..j
                };

                total_visible += visible_range.len();

                let view_spans = thread_state.folded_spans_state.fold(
                    visible_range,
                    &thread_state.spans,
                    display_position.cycles_per_pixel as u64 * 4,
                );

                for (start_idx, end_idx) in view_spans {
                    let start_span = &thread_state.spans[*start_idx];
                    let end_span = &thread_state.spans[*end_idx - 1];
                    let folded = *start_idx != *end_idx - 1;

                    let x_range = {
                        let range = response.rect.x_range();
                        (range.min as f64)..=(range.max as f64)
                    };

                    let span_min =
                        emath::remap(start_span.start_time as f64, (low)..=(hi), x_range.clone());
                    let span_max =
                        emath::remap(end_span.end_time as f64, (low)..=(hi), x_range.clone());

                    let rect = Rect {
                        min: pos2(
                            span_min as f32,
                            response.rect.min.y + thread_idx as f32 * thread_row_height,
                        ),
                        max: pos2(
                            span_max as f32,
                            response.rect.min.y
                                + thread_idx as f32 * thread_row_height
                                + (thread_row_height - thread_row_padding_vertical),
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
                        let span = &thread_state.spans[*start_idx];

                        let is_selected = if same_thread_as_selected {
                            view_state
                                .selected_span
                                .as_ref()
                                .map_or(false, |it| it.span_idx == *start_idx)
                        } else {
                            false
                        };

                        let is_same_type_as_selected = view_state
                            .current_symbol_detail
                            .map_or(false, |it| it == (span.label_id as usize));

                        if is_hovered && is_clicked {
                            let selected_span_metadata = SpanRef {
                                thread_id: *thread_id,
                                span_idx: *start_idx,
                            };

                            view_state.selected_span.replace(selected_span_metadata);
                            view_state.current_symbol_detail.replace(span.label_id as _);
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

                        let text_color = ui.style().visuals.text_color();
                        render_text(
                            &painter,
                            span,
                            &view_state.initial_message,
                            docs,
                            rect,
                            text_color,
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

                view_state
                    .timeline_position_state
                    .translate_x(diff, display_position);
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

            (low, hi, total_visible)
        });

    response.inner
}

fn render_text(
    painter: &Painter,
    span: &ThreadSpan,
    initial_message: &InitialMessage,
    docs: &LoadedDocumentation,
    rect: Rect,
    color: Color32,
) {
    let Some(symbol) = initial_message.symbols.get(span.label_id as usize) else {
        return;
    };

    let library = initial_message.libraries.get(&(symbol.library_id as _));
    let module = initial_message.modules.get(&(symbol.module_id as _));

    let resolved_symbol_name =
        (|| -> Option<(&ModuleInfo, &LibraryInfo, Option<&SymbolDocumentation>)> {
            let library = library?;
            let module = module?;
            let resolved_symbol = docs.lookup(&module.name, &library.name, &symbol.name);

            Some((module, library, resolved_symbol))
        })();

    let text = match resolved_symbol_name {
        Some((
            _module,
            _library,
            Some(SymbolDocumentation {
                name: Some(resolved_symbol_name),
                ..
            }),
        )) => format!("{}", resolved_symbol_name),
        Some((module, library, Some(SymbolDocumentation { name: None, .. })))
        | Some((module, library, None)) => {
            format!("{}::{}::{}", module.name, library.name, symbol.name)
        }
        None => format!("{}", symbol.name),
    };

    let layout = painter.layout_no_wrap(
        text,
        FontId {
            size: rect.height() * 0.8,
            family: FontFamily::Monospace,
        },
        color,
    );

    let text_layout_rect = Align2::LEFT_CENTER.anchor_size(
        pos2(rect.min.x.max(0.), rect.y_range().center()),
        layout.size(),
    );

    if text_layout_rect.x_range().max >= rect.x_range().max {
        return;
    }

    painter.galley(text_layout_rect.min, layout, color);
}
