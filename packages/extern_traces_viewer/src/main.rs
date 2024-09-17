use eframe::egui;
use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
struct SpanStart {
    thread_id: u64,
    time: u64,
    label_id: u64,
}

#[derive(Debug, Clone)]
struct SpanEnd {
    thread_id: u64,
    time: u64,
}

#[derive(Debug, Clone)]
enum SpanEvent {
    Start(SpanStart),
    End(SpanEnd),
}

#[derive(Debug, Clone)]
struct Span {
    start_time: u64,
    end_time: u64,
    label_id: u64,
}

struct SpanViewer {
    events: Vec<SpanEvent>,
    spans: HashMap<u64, Vec<Span>>,  // Key is thread_id
    receiver: Receiver<SpanEvent>,
    active_spans: HashMap<u64, SpanStart>,
    start_time: Instant,
    view_start: u64,
    view_end: u64,
    thread_order: Vec<u64>,
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
            events: Vec::new(),
            spans: HashMap::new(),
            receiver,
            active_spans: HashMap::new(),
            start_time: Instant::now(),
            view_start: 0,
            view_end: 1000000000, // 1 second initial view
            thread_order: Vec::new(),
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            self.events.push(event.clone());
            match event {
                SpanEvent::Start(start) => {
                    if !self.thread_order.contains(&start.thread_id) {
                        self.thread_order.push(start.thread_id);
                    }
                    self.active_spans.insert(start.thread_id, start);
                },
                SpanEvent::End(end) => {
                    if let Some(start) = self.active_spans.remove(&end.thread_id) {
                        let span = Span {
                            start_time: start.time,
                            end_time: end.time,
                            label_id: start.label_id,
                        };
                        self.spans.entry(start.thread_id).or_insert_with(Vec::new).push(span);
                    }
                },
            }
        }
    }

    fn draw_spans(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::drag());
        let rect = response.rect;

        let time_to_x = |time: u64| -> f32 {
            let ratio = (time - self.view_start) as f32 / (self.view_end - self.view_start) as f32;
            rect.left() + ratio * rect.width()
        };

        // Debug information
        ui.label(format!("View start: {}, View end: {}", self.view_start, self.view_end));
        ui.label(format!("Total spans: {}", self.total_spans()));

        for (i, &thread_id) in self.thread_order.iter().enumerate() {
            let y = rect.top() + i as f32 * 30.0; // Increased vertical spacing
            painter.text(
                egui::pos2(rect.left(), y),
                egui::Align2::LEFT_CENTER,
                format!("Thread {}", thread_id),
                egui::FontId::default(),
                egui::Color32::WHITE,
            );

            if let Some(spans) = self.spans.get(&thread_id) {
                for span in spans {
                    if span.end_time >= self.view_start && span.start_time <= self.view_end {
                        let start_x = time_to_x(span.start_time).max(rect.left());
                        let end_x = time_to_x(span.end_time).min(rect.right());

                        // Draw span rectangle
                        painter.rect_filled(
                            egui::Rect::from_two_pos(
                                egui::pos2(start_x, y + 2.0),
                                egui::pos2(end_x, y + 18.0),
                            ),
                            0.0,
                            egui::Color32::from_rgb(100, 150, 200),
                        );

                        // Draw span label
                        let span_center_x = (start_x + end_x) / 2.0;
                        painter.text(
                            egui::pos2(span_center_x, y + 10.0),
                            egui::Align2::CENTER_CENTER,
                            format!("Label {}", span.label_id),
                            egui::FontId::proportional(10.0),
                            egui::Color32::BLACK,
                        );
                    }
                }
            }

            // Debug information for each thread
            ui.label(format!("Thread {} spans: {}", thread_id, self.spans.get(&thread_id).map_or(0, |spans| spans.len())));
        }

        if response.dragged() {
            let delta = response.drag_delta().x;
            let time_delta = ((self.view_end - self.view_start) as f32 * delta / rect.width()) as u64;
            self.view_start = self.view_start.saturating_sub(time_delta);
            self.view_end = self.view_end.saturating_sub(time_delta);
        }
    }

    fn total_spans(&self) -> usize {
        self.spans.values().map(|v| v.len()).sum()
    }
}

impl eframe::App for SpanViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Span Viewer");
            ui.label(format!("Number of spans: {}", self.total_spans()));
            ui.label(format!("Number of threads: {}", self.thread_order.len()));
            ui.label(format!("Time since start: {:?}", self.start_time.elapsed()));

            self.draw_spans(ui);
        });

        ctx.request_repaint();
    }
}

fn handle_client(mut stream: TcpStream, sender: Sender<SpanEvent>) -> std::io::Result<()> {
    println!("New client connected");
    loop {
        let mut message_tag = [0u8; 8];
        stream.read_exact(&mut message_tag)?;
        let message_tag = u64::from_le_bytes(message_tag);

        match message_tag {
            0 => { // SpanStart
                let mut data = [0u8; 24];
                stream.read_exact(&mut data)?;
                let thread_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
                let time = u64::from_le_bytes(data[8..16].try_into().unwrap());
                let label_id = u64::from_le_bytes(data[16..24].try_into().unwrap());
                sender.send(SpanEvent::Start(SpanStart { thread_id, time, label_id })).unwrap();
            },
            1 => { // SpanEnd
                let mut data = [0u8; 16];
                stream.read_exact(&mut data)?;
                let thread_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
                let time = u64::from_le_bytes(data[8..16].try_into().unwrap());
                sender.send(SpanEvent::End(SpanEnd { thread_id, time })).unwrap();
            },
            _ => eprintln!("Unknown message tag: {}", message_tag),
        }
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
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting client: {}", e);
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
        "Span Viewer",
        options,
        Box::new(|cc| Box::new(SpanViewer::new(cc))),
    )
}
