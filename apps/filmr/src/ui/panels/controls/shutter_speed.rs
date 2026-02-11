use egui::RichText;

pub struct ShutterSpeed(pub f64);

impl Default for ShutterSpeed {
    fn default() -> Self {
        Self(1.0 / 125.0)
    }
}

impl ShutterSpeed {
    #[allow(clippy::eq_op)]
    const STOPS: &[f64] = &[
        1.0 / 8000.0,
        1.0 / 4000.0,
        1.0 / 2000.0,
        1.0 / 1000.0,
        1.0 / 500.0,
        1.0 / 250.0,
        1.0 / 125.0,
        1.0 / 60.0,
        1.0 / 30.0,
        1.0 / 15.0,
        1.0 / 8.0,
        1.0 / 4.0,
        1.0 / 2.0,
        1.0,
        1.0 + 1.0 / 3.0,
        1.0 + 2.0 / 3.0,
        2.0,
        2.0 + 1.0 / 3.0,
        2.0 + 2.0 / 3.0,
        2.0 + 3.0 / 3.0,
        2.0 + 4.0 / 3.0,
        2.0 + 4.0 / 3.0,
        2.0 + 5.0 / 3.0,
        4.0,
        4.0 + 1.0 / 3.0,
        4.0 + 2.0 / 3.0,
        4.0 + 3.0 / 3.0,
        4.0 + 4.0 / 3.0,
        4.0 + 5.0 / 3.0,
        4.0 + 6.0 / 3.0,
        4.0 + 7.0 / 3.0,
        4.0 + 8.0 / 3.0,
        4.0 + 9.0 / 3.0,
        4.0 + 10.0 / 3.0,
        4.0 + 11.0 / 3.0,
        8.0,
        8.0 + 1.0 / 3.0,
        8.0 + 2.0 / 3.0,
        8.0 + 3.0 / 3.0,
        8.0 + 4.0 / 3.0,
        8.0 + 5.0 / 3.0,
        8.0 + 6.0 / 3.0,
        8.0 + 7.0 / 3.0,
        8.0 + 8.0 / 3.0,
        8.0 + 9.0 / 3.0,
        8.0 + 9.0 / 3.0,
        8.0 + 10.0 / 3.0,
        8.0 + 11.0 / 3.0,
        8.0 + 12.0 / 3.0,
        8.0 + 13.0 / 3.0,
        8.0 + 14.0 / 3.0,
        8.0 + 15.0 / 3.0,
        8.0 + 16.0 / 3.0,
        8.0 + 17.0 / 3.0,
        8.0 + 18.0 / 3.0,
        8.0 + 19.0 / 3.0,
        8.0 + 20.0 / 3.0,
        15.0,
        20.0,
        25.0,
        30.0,
    ];

    fn idx(&self) -> usize {
        Self::STOPS
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (*a - self.0)
                    .abs()
                    .partial_cmp(&(*b - self.0).abs())
                    .unwrap()
            })
            .map(|(i, _)| i)
            .unwrap()
    }

    pub fn display(&self) -> String {
        if self.0 < 1.0 {
            format!("1/{}", (1.0 / self.0).round())
        } else {
            format!("{:.1}\"", self.0)
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut idx = self.idx() as f64;

        let resp = ui
            .horizontal(|ui| {
                let slider = egui::Slider::new(&mut idx, 0.0..=(Self::STOPS.len() - 1) as f64)
                    .step_by(1.0)
                    .show_value(false)
                    .trailing_fill(true);

                let resp = ui.add(slider);
                ui.label(RichText::new(self.display()).size(18.0).monospace());
                resp
            })
            .inner;

        if resp.changed() {
            self.0 = Self::STOPS[(idx.round() as usize).clamp(0, Self::STOPS.len() - 1)];
        }
        resp
    }
}
