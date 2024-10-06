use crate::app::tracing::panes::{PaneKey, PaneResponse, TreeBehaviorArgs};
use crate::app::tracing::utils::render_symbol_info;
use crate::app::tracing::view_state::SpanRef;
use eframe::egui;
use eframe::egui::{vec2, ScrollArea, TextStyle, Ui};

pub struct SymbolDetailPane {
    pub last_matching: Option<(usize, Vec<SpanRef>)>,
}

impl SymbolDetailPane {
    pub fn title(&self) -> egui::WidgetText {
        // todo: use symbol
        "symbol detail".to_string().into()
    }

    pub fn pane_ui(&mut self, args: &mut TreeBehaviorArgs, ui: &mut Ui) -> Option<PaneResponse> {
        let symbol_idx = args.view_state.current_symbol_detail?;

        let should_replace = if self.last_matching.is_none() {
            true
        } else {
            let last = self.last_matching.as_ref().unwrap().0;
            last != symbol_idx
        };

        if should_replace {
            self.last_matching.replace((
                symbol_idx,
                args.view_state
                    .threads
                    .iter()
                    .flat_map(|(thread_id, spans)| {
                        spans
                            .spans
                            .iter()
                            .enumerate()
                            .map(|(span_idx, span)| (*thread_id, span_idx, span))
                    })
                    .filter(|(_thread_id, _span_idx, span)| (span.label_id as usize) == symbol_idx)
                    .map(|(thread_id, span_idx, _)| SpanRef {
                        span_idx,
                        thread_id,
                    })
                    .collect(),
            ));
        };

        let matching = &self.last_matching.as_ref().unwrap().1;

        render_symbol_info(
            &args.view_state.initial_message,
            args.docs,
            &args.view_state.initial_message.symbols[symbol_idx],
            ui,
        );

        let mut pane_response = None;

        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        ScrollArea::vertical().show_rows(ui, row_height, matching.len(), |ui, row_range| {
            ui.allocate_space(vec2(ui.available_width(), 0.));

            for idx in row_range {
                let link_response = ui.link(format!("{:?}", idx));

                if link_response.clicked() {
                    args.view_state.selected_span.replace(matching[idx].clone());

                    pane_response.replace(PaneResponse::FocusPane(
                        PaneKey::CurrentlySelectedSpanDetail,
                    ));
                }
            }
        });

        pane_response
    }
}
