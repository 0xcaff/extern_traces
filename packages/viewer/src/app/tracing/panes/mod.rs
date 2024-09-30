mod search;
mod span_detail;
mod symbol_detail;

use crate::app::tracing::panes::search::SearchPane;
use crate::app::tracing::panes::span_detail::SpanDetailPane;
use crate::app::tracing::panes::symbol_detail::SymbolDetailPane;
use crate::app::tracing::view_state::ViewState;
use eframe::egui;
use eframe::egui::{vec2, Align2, Color32, Frame, Id, Response, Sense, Stroke, TextStyle, Ui};
use egui_tiles::{SimplificationOptions, TabState, Tabs, TileId, Tiles, Tree};
use ps4libdoc::LoadedDocumentation;

pub struct TreeBehaviorArgs<'a> {
    pub view_state: &'a mut ViewState,
    pub docs: &'a LoadedDocumentation,
    pub last_width: Option<f32>,
}

pub struct TreeBehavior<'a> {
    pub args: TreeBehaviorArgs<'a>,
    pub pane_response: Option<PaneResponse>,
}

pub enum Pane {
    CurrentlySelectedSpanDetail(SpanDetailPane),
    SearchPane(SearchPane),
    CurrentSymbolDetailsPane(SymbolDetailPane),
}

impl Pane {
    pub fn key(&self) -> PaneKey {
        match self {
            Pane::CurrentlySelectedSpanDetail(_) => PaneKey::CurrentlySelectedSpanDetail,
            Pane::SearchPane(_) => PaneKey::SearchPane,
            Pane::CurrentSymbolDetailsPane(_) => PaneKey::CurrentSymbolDetailsPane,
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum PaneKey {
    CurrentlySelectedSpanDetail,
    SearchPane,
    CurrentSymbolDetailsPane,
}

pub enum PaneResponse {
    OpenPane(Pane),
    FocusPane(PaneKey),
}

impl<'a> egui_tiles::Behavior<Pane> for TreeBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut Ui,
        _tile_id: TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        let frame = Frame::side_top_panel(ui.style());

        frame.show(ui, |ui| {
            self.pane_response = match pane {
                Pane::CurrentlySelectedSpanDetail(pane) => pane.pane_ui(&mut self.args, ui),
                Pane::SearchPane(pane) => pane.pane_ui(&mut self.args, ui),
                Pane::CurrentSymbolDetailsPane(pane) => pane.pane_ui(&mut self.args, ui),
            };
        });

        egui_tiles::UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::CurrentlySelectedSpanDetail(pane) => pane.title(),
            Pane::SearchPane(pane) => pane.title(),
            Pane::CurrentSymbolDetailsPane(pane) => pane.title(),
        }
    }

    fn tab_ui(
        &mut self,
        tiles: &mut Tiles<Pane>,
        ui: &mut Ui,
        id: Id,
        tile_id: TileId,
        state: &TabState,
    ) -> Response {
        let text = self.tab_title_for_tile(tiles, tile_id);

        let font_id = TextStyle::Button.resolve(ui.style());
        let galley = text.into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, font_id);

        let x_margin = 8.;

        let button_width = galley.size().x + 2. * x_margin;
        let (_, tab_rect) = ui.allocate_space(vec2(button_width, ui.available_height()));

        let tab_response = ui
            .interact(tab_rect, id, Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::Grab);

        // Show a gap when dragged
        if ui.is_rect_visible(tab_rect) && !state.is_being_dragged {
            let bg_color = if state.active {
                ui.visuals().panel_fill
            } else {
                Color32::TRANSPARENT
            };

            ui.painter().rect(tab_rect, 0.0, bg_color, Stroke::NONE);

            let text_color = self.tab_text_color(ui.visuals(), tiles, tile_id, state);
            let text_position = Align2::LEFT_CENTER
                .align_size_within_rect(galley.size(), tab_rect.shrink(x_margin))
                .min;

            ui.painter().galley(text_position, galley, text_color);
        }

        tab_response
    }

    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            prune_empty_containers: true,
            prune_empty_tabs: true,
            ..Default::default()
        }
    }
}

pub fn create_tree() -> Tree<Pane> {
    let mut tiles = Tiles::default();

    let currently_selected_span_detail_pane =
        tiles.insert_pane(Pane::CurrentlySelectedSpanDetail(SpanDetailPane::init()));

    let search_pane = tiles.insert_pane(Pane::SearchPane(SearchPane::init()));

    let symbol_pane = tiles.insert_pane(Pane::CurrentSymbolDetailsPane(SymbolDetailPane {
        last_matching: None,
    }));

    let root = tiles.insert_container(Tabs::new(vec![
        currently_selected_span_detail_pane,
        symbol_pane,
        search_pane,
    ]));

    Tree::new("detail_tree", root, tiles)
}
