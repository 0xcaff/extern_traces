use crate::app::start::StartScene;
use crate::app::tracing::TracingScene;
use eframe::egui::Context;
use ps4libdoc::LoadedDocumentation;

mod start;
mod tracing;

enum Scene {
    Start(StartScene),
    Tracing(TracingScene),
}

impl Scene {
    pub fn initial() -> Scene {
        Scene::Start(StartScene::initial())
    }
}

pub struct App {
    docs: LoadedDocumentation,
    scene: Scene,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let docs = LoadedDocumentation::bundled().unwrap();

        App {
            docs,
            scene: Scene::initial(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let next_scene = match self.scene {
            Scene::Start(ref mut landing_page) => landing_page.update(ctx),
            Scene::Tracing(ref mut tracing_scene) => tracing_scene.update(ctx, &self.docs),
        };

        if let Some(next_scene) = next_scene {
            self.scene = next_scene;
        }
    }
}
