use nih_plug_vizia::vizia::{prelude::*, vg};
use std::sync::Arc;

use crate::FuririParams;

pub struct Waveform<V>
where
    V: Lens<Target = Arc<FuririParams>>,
{
    data: V,
}

impl<V> Waveform<V>
where
    V: Lens<Target = Arc<FuririParams>>,
{
    pub fn new(cx: &mut Context, data: V) -> Handle<Self> {
        Self { data }.build(cx, |_| {})
    }
}

impl<V> View for Waveform<V>
where
    V: Lens<Target = Arc<FuririParams>>,
{
    fn element(&self) -> Option<&'static str> {
        Some("waveform")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let mut max: f32 = 0.0;
        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();
                let binding = self.data.get(cx);

                let overtones: [f32; 8] = [
                    binding.overtones.overtone1.value(),
                    binding.overtones.overtone2.value(),
                    binding.overtones.overtone3.value(),
                    binding.overtones.overtone4.value(),
                    binding.overtones.overtone5.value(),
                    binding.overtones.overtone6.value(),
                    binding.overtones.overtone7.value(),
                    binding.overtones.overtone8.value(),
                ];
                const STEP_SIZE: f32 = 2.0;
                path.move_to(bounds.x, bounds.y + bounds.h / 2.0);
                let mut x = bounds.x;
                while x < bounds.x + bounds.w {
                    let p = (x - bounds.x) / bounds.w;
                    let s = overtones
                        .iter()
                        .enumerate()
                        .map(|(i, v)| v * (std::f32::consts::TAU * (1.0 + i as f32) * p).sin())
                        .sum::<f32>();
                    max = max.max(s.abs());
                    path.line_to(x, bounds.y + (1.0 - s / 8.0) * bounds.h / 2.0);
                    x += STEP_SIZE;
                }
                path
            },
            &vg::Paint::color(if max > 4.0 {
                vg::Color::rgb(255, 0, 0)
            } else {
                cx.font_color().into()
            })
            .with_line_width(2.0),
        );
    }
}
