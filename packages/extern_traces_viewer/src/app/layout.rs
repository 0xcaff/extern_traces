use eframe::egui::{vec2, Align, Color32, FontSelection, Galley, Rect, Ui, Vec2, WidgetText};
use eframe::{egui, epaint};
use std::sync::Arc;

pub trait Draw {
    fn draw(self, ui: &mut Ui);
}

trait DrawDyn {
    fn display(self: Box<Self>, ui: &mut Ui);
}

impl<T: Draw + 'static> DrawDyn for T {
    fn display(self: Box<Self>, ui: &mut Ui) {
        (*self).draw(ui);
    }
}

pub trait Measure {
    type Measured: Draw;

    fn measure(self, ui: &Ui) -> (Vec2, Self::Measured);
}

trait MeasureDyn {
    fn measure(self: Box<Self>, ui: &Ui) -> (Vec2, Box<dyn DrawDyn>);
}

impl<T: Measure + 'static> MeasureDyn for T {
    fn measure(self: Box<Self>, ui: &Ui) -> (Vec2, Box<dyn DrawDyn>) {
        let (vec, measured) = (*self).measure(ui);
        (vec, Box::new(measured))
    }
}

pub enum LayoutDirection {
    Row,
    Column,
}

pub enum Alignment {
    Start,
    End,
    Center,
}

pub struct LayoutParams {
    pub direction: LayoutDirection,
    pub main_axis_alignment: Alignment,
}

pub struct Layout {
    params: LayoutParams,
    children: Vec<Box<dyn MeasureDyn>>,
}

impl Layout {
    pub fn new(params: LayoutParams) -> Self {
        Self {
            children: vec![],
            params,
        }
    }

    pub fn with_child(mut self, child: impl Measure + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
}

impl Measure for Layout {
    type Measured = MeasuredLayout;

    fn measure(self, ui: &Ui) -> (Vec2, Self::Measured) {
        let measured_children: Vec<_> =
            self.children.into_iter().map(|it| it.measure(ui)).collect();

        let bounding_box = match self.params.direction {
            LayoutDirection::Row => vec2(
                measured_children.iter().map(|it| it.0.x).sum(),
                max_partial(measured_children.iter().map(|it| it.0.y)).unwrap(),
            ),
            LayoutDirection::Column => vec2(
                max_partial(measured_children.iter().map(|it| it.0.x)).unwrap(),
                measured_children.iter().map(|it| it.0.y).sum(),
            ),
        };

        (
            bounding_box,
            MeasuredLayout {
                params: self.params,
                children: measured_children,
            },
        )
    }
}

pub struct MeasuredLayout {
    params: LayoutParams,
    children: Vec<(Vec2, Box<dyn DrawDyn>)>,
}

impl Draw for MeasuredLayout {
    fn draw(mut self, ui: &mut Ui) {
        let available_space = ui.available_rect_before_wrap();

        match self.params.main_axis_alignment {
            Alignment::Start => {
                let mut current_x = 0.;

                for (measurement, child) in self.children {
                    let mut ui = ui.child_ui(
                        Rect::from_min_size(
                            available_space.min + vec2(current_x, 0.),
                            measurement.clone(),
                        ),
                        egui::Layout::default(),
                        None,
                    );

                    child.display(&mut ui);

                    current_x += measurement.x;
                }
            }
            Alignment::End => {
                let consumed_width: f32 = self.children.iter().map(|it| it.0.x).sum();
                let spacing_width = available_space.width() - consumed_width;

                let mut current_x = 0.;

                for (measurement, child) in self.children {
                    let mut ui = ui.child_ui(
                        Rect::from_min_size(
                            available_space.min + vec2(current_x + spacing_width, 0.),
                            measurement.clone(),
                        ),
                        egui::Layout::default(),
                        None,
                    );

                    child.display(&mut ui);

                    current_x += measurement.x;
                }
            }
            Alignment::Center => {
                let consumed_width: f32 = self.children.iter().map(|it| it.0.x).sum();
                let spacing_width = available_space.width() - consumed_width;

                let mut current_x = 0.;

                for (measurement, child) in self.children {
                    let mut ui = ui.child_ui(
                        Rect::from_min_size(
                            available_space.min + vec2(current_x + spacing_width / 2., 0.),
                            measurement.clone(),
                        ),
                        egui::Layout::default(),
                        None,
                    );

                    child.display(&mut ui);

                    current_x += measurement.x;
                }
            }
        }
    }
}

pub struct Text {
    text: WidgetText,
}

impl Text {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self { text: text.into() }
    }
}

fn max_partial<T: PartialOrd>(values: impl Iterator<Item = T>) -> Option<T> {
    values.fold(None, |max, x| match max {
        Some(m) if m.partial_cmp(&x) == Some(std::cmp::Ordering::Greater) => Some(m),
        _ => Some(x),
    })
}

impl Measure for Text {
    type Measured = MeasuredText;

    fn measure(self, ui: &Ui) -> (Vec2, MeasuredText) {
        let layout_job = self
            .text
            .into_layout_job(ui.style(), FontSelection::Default, Align::Min);
        let galley = ui.fonts(|it| it.layout_job(layout_job));

        (galley.size(), MeasuredText { galley })
    }
}

pub struct MeasuredText {
    galley: Arc<Galley>,
}

impl Draw for MeasuredText {
    fn draw(self, ui: &mut Ui) {
        let position = ui.next_widget_position();
        ui.painter()
            .add(epaint::TextShape::new(position, self.galley, Color32::RED));
    }
}
