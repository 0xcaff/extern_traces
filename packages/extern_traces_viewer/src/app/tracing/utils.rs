use crate::proto::{InitialMessage, SymbolInfo};
use eframe::egui::Ui;
use ps4libdoc::LoadedDocumentation;

pub fn render_symbol_info(
    args: &InitialMessage,
    docs: &LoadedDocumentation,
    symbol: &SymbolInfo,
    ui: &mut Ui,
) {
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
                    docs.lookup(module_name, library_name, symbol.name.as_ref())
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

pub fn human_readable_size(value: usize) -> String {
    let suffixes = ["", "k", "m", "b", "t"];
    let mut divisor = 1;
    let mut suffix_index = 0;

    while value / divisor >= 1000 && suffix_index < suffixes.len() - 1 {
        divisor *= 1000;
        suffix_index += 1;
    }

    let whole_part = value / divisor;
    let remainder = value % divisor;

    if remainder == 0 {
        format!("{}{}", whole_part, suffixes[suffix_index])
    } else {
        let decimal_part = remainder * 100 / divisor;
        format!(
            "{}.{:02}{}",
            whole_part, decimal_part, suffixes[suffix_index]
        )
    }
}

pub fn format_time(seconds: f64) -> String {
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
