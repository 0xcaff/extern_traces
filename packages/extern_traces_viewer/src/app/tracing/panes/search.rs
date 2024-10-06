use crate::app::tracing::panes::{PaneKey, PaneResponse, TreeBehaviorArgs};
use eframe::egui;
use eframe::egui::{vec2, ScrollArea, Ui};

pub struct SearchPane {
    current_text: String,
}

impl SearchPane {
    pub fn init() -> SearchPane {
        SearchPane {
            current_text: "".to_string(),
        }
    }

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

                    pane_response
                        .replace(PaneResponse::FocusPane(PaneKey::CurrentSymbolDetailsPane));
                }
            }
        });

        pane_response
    }
}
