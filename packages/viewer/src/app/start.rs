use crate::app::tracing::TracingScene;
use crate::app::Scene;
use eframe::egui::{vec2, CentralPanel, Context, Frame, Margin};
use rfd::FileDialog;
use std::net::{AddrParseError, SocketAddr};
use std::str::FromStr;

pub struct StartScene {
    address: String,
    last_error: Option<AddrParseError>,
}

impl StartScene {
    pub fn initial() -> StartScene {
        StartScene {
            address: "0.0.0.0:9090".to_string(),
            last_error: None,
        }
    }

    pub fn update(&mut self, ctx: &Context) -> Option<Scene> {
        let mut next_scene = None;

        CentralPanel::default()
            .frame(Frame {
                fill: ctx.style().visuals.panel_fill,
                inner_margin: Margin::symmetric(8., 4.),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.allocate_space(vec2(ui.available_width(), 0.));

                ui.heading("extern_traces");

                ui.label("version 1.0.0");

                ui.allocate_space(vec2(0., 4.));

                ui.vertical(|ui| {
                    ui.label("load traces");
                    let button = ui.button("open");
                    if button.clicked() {
                        if let Some(file_path) = FileDialog::new().pick_file() {
                            println!("{:#?}", file_path);
                        }
                    }
                });

                ui.allocate_space(vec2(0., 16.));

                ui.vertical(|ui| {
                    ui.label("listen for traces");

                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.address);
                    });

                    let button = ui.button("listen");
                    if button.clicked() {
                        match SocketAddr::from_str(self.address.as_str()) {
                            Ok(addr) => {
                                next_scene.replace(Scene::Tracing(TracingScene::from_network(addr)));
                            }
                            Err(err) => {
                                self.last_error = Some(err);
                            }
                        };
                    }
                });
            });

        next_scene
    }
}
