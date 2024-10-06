use crate::app::tracing::panes::{PaneKey, PaneResponse, TreeBehaviorArgs};
use crate::proto::TraceCommand;
use eframe::egui;
use eframe::egui::Ui;

pub struct GraphicsCapturePane {}

impl GraphicsCapturePane {
    pub fn title(&self) -> egui::WidgetText {
        "graphics capture".to_string().into()
    }

    pub fn pane_ui(&mut self, args: &mut TreeBehaviorArgs, ui: &mut Ui) -> Option<PaneResponse> {
        let mut pane_response = None;

        if let Some(ref mut commands) = args.commands {
            let button = ui.button("capture frame");
            if button.clicked() {
                commands.send(TraceCommand::CaptureFrame).unwrap();
            }
        }

        for (idx, span_ref) in args.view_state.extra_data_messages.iter().enumerate() {
            let link_response = ui.link(format!("{:?}", idx));

            if link_response.clicked() {
                args.view_state.selected_span.replace(span_ref.clone());

                pane_response.replace(PaneResponse::FocusPane(
                    PaneKey::CurrentlySelectedSpanDetail,
                ));
            }
        }

        pane_response
    }
}
