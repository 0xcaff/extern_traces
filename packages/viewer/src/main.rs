mod proto;
mod state;

use crate::proto::{InitialMessage, SpanEvent, TraceEvent};
use crate::state::State;
use eframe::egui;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use std::{io, thread};

struct SpanViewer {
    state: State,
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
            state: State::new(),
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

            if let Some(initial_message) = &self.state.initial_message {
                let elapsed = initial_message.anchor().elapsed().unwrap();
                ui.label(format!("time: {:?}", elapsed));
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
