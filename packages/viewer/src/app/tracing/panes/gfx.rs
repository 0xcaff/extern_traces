use crate::app::tracing::panes::{PaneResponse, TreeBehaviorArgs};
use eframe::egui;
use eframe::egui::Ui;
use crate::proto::TraceCommand;

pub struct GraphicsCapturePane {}

impl GraphicsCapturePane {
    pub fn title(&self) -> egui::WidgetText {
        "graphics capture".to_string().into()
    }

    pub fn pane_ui(&mut self, args: &mut TreeBehaviorArgs, ui: &mut Ui) -> Option<PaneResponse> {
        if let Some(ref mut commands) = args.commands {
            let button = ui.button("capture frame");
            if button.clicked() {
                commands.send(TraceCommand::CaptureFrame).unwrap();
            }
        }

        None
    }
}
