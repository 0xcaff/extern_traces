mod app;
mod proto;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "extern_traces",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
