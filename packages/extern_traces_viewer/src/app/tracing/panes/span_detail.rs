use crate::app::tracing::panes::render::render_frame;
use crate::app::tracing::panes::{PaneResponse, TreeBehaviorArgs};
use crate::app::tracing::utils::{format_time, render_symbol_info};
use crate::gfx_debug::{DebugHandle, ExtraData, GraphicsContext};
use anyhow::bail;
use eframe::egui;
use eframe::egui::load::SizedTexture;
use eframe::egui::{
    vec2, Align, Image, ImageSource, Layout, TextureHandle, TextureOptions, Ui, Widget,
};

pub struct SpanDetailPane {
    last_image: Option<TextureHandle>,
    debug_handle: DebugHandle,
    ctx: GraphicsContext,
}

impl SpanDetailPane {
    pub fn init() -> SpanDetailPane {
        let (ctx, debug_handle) = GraphicsContext::init();

        SpanDetailPane {
            last_image: None,
            ctx,
            debug_handle,
        }
    }
}

impl SpanDetailPane {
    pub fn title(&self) -> egui::WidgetText {
        "span detail".to_string().into()
    }

    pub fn pane_ui(&mut self, args: &mut TreeBehaviorArgs, ui: &mut Ui) -> Option<PaneResponse> {
        ui.with_layout(Layout::top_down(Align::Min), |ui| -> Option<()> {
            let view_state = &mut args.view_state;

            let selected_span = view_state.selected_span.as_ref()?;
            let thread = view_state.threads.get(&selected_span.thread_id)?;
            let span = &thread.spans[selected_span.span_idx];

            ui.allocate_space(vec2(ui.available_width(), 0.));

            let Some(symbol) = view_state
                .initial_message
                .symbols
                .get(span.label_id as usize)
            else {
                ui.label("unable to resolve symbol");

                return None;
            };

            render_symbol_info(&view_state.initial_message, args.docs, symbol, ui);

            ui.horizontal(|ui| {
                ui.label("duration");
                let duration_cycles = span.end_time - span.start_time;
                let duration_seconds =
                    duration_cycles as f64 / view_state.initial_message.tsc_frequency as f64;
                ui.label(format_time(duration_seconds));
            });

            ui.horizontal(|ui| {
                ui.label("start");
                let duration_cycles = span.start_time - view_state.initial_message.anchor_timestamp;
                let duration_seconds =
                    duration_cycles as f64 / view_state.initial_message.tsc_frequency as f64;
                ui.label(format_time(duration_seconds));
            });

            if let Some(it) = &self.last_image {
                Image::new(ImageSource::Texture(SizedTexture::from_handle(it)))
                    .max_width(ui.available_width())
                    .ui(ui);
            }

            let button_response = ui.button("zoom");
            if button_response.clicked() {
                if let Some(last_width) = args.last_width {
                    view_state
                        .timeline_position_state
                        .pan_to(&span, last_width as _);
                }
            }

            if let Some(extra_data) = &span.extra_data {
                let button_response = ui.button("copy raw data");
                if button_response.clicked() {
                    let encoded = hex::encode(extra_data.as_slice());
                    ui.output_mut(|it| it.copied_text = encoded);
                }

                {
                    let button_response = ui.button("render frame");
                    if button_response.clicked() {
                        let result = (|| -> Result<_, anyhow::Error> {
                            let extra_data = ExtraData::parse(extra_data.as_slice())?;
                            let Some(image) =
                                render_frame(&self.ctx, &mut self.debug_handle, &extra_data)?
                            else {
                                bail!("missing image");
                            };

                            Ok(image)
                        })();

                        match result {
                            Ok(image) => {
                                let texture = ui.ctx().load_texture(
                                    "frame",
                                    image,
                                    TextureOptions::default(),
                                );

                                self.last_image.replace(texture);
                            }
                            Err(err) => {
                                println!("{:?}", err);
                            }
                        }
                    }
                }
            }

            Some(())
        });

        None
    }
}
