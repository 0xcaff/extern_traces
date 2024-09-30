use crate::app::tracing::TracingScene;
use crate::app::Scene;
use eframe::egui::{vec2, CentralPanel, Context, Frame, Key, Margin};
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

        if let Some(file_path) = ctx.input(|it| {
            it.raw
                .dropped_files
                .iter()
                .flat_map(|it| it.path.clone())
                .next()
        }) {
            return Some(Scene::Tracing(TracingScene::from_file_path(
                ctx.clone(),
                file_path,
            )));
        }

        CentralPanel::default()
            .frame(Frame {
                fill: ctx.style().visuals.panel_fill,
                inner_margin: Margin::symmetric(8., 4.),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.allocate_space(vec2(ui.available_width(), 0.));

                ui.heading("extern_traces");

                let version = env!("CARGO_PKG_VERSION");
                let git_short_sha = env!("GIT_SHA_SHORT");

                ui.label(format!("v{version} @ {git_short_sha}"));

                ui.allocate_space(vec2(0., 4.));

                ui.vertical(|ui| {
                    ui.label("load traces");
                    let button = ui.button("open");
                    if button.clicked() {
                        if let Some(file_path) = FileDialog::new().pick_file() {
                            next_scene.replace(Scene::Tracing(TracingScene::from_file_path(
                                ctx.clone(),
                                file_path,
                            )));
                        }
                    }
                });

                ui.allocate_space(vec2(0., 16.));

                ui.vertical(|ui| {
                    ui.label("listen for traces");

                    let text_edit = ui.text_edit_singleline(&mut self.address);

                    let button = ui.button("listen");
                    if (text_edit.lost_focus()
                        && text_edit.ctx.input(|r| r.key_pressed(Key::Enter)))
                        || button.clicked()
                    {
                        match SocketAddr::from_str(self.address.as_str()) {
                            Ok(addr) => {
                                next_scene.replace(Scene::Tracing(TracingScene::from_network(
                                    ctx.clone(),
                                    addr,
                                )));
                            }
                            Err(err) => {
                                self.last_error = Some(err);
                            }
                        };
                    }
                });

                if let Some(it) = &self.last_error {
                    ui.colored_label(ui.style().visuals.error_fg_color, format!("{:?}", it));
                }
            });

        next_scene
    }
}
