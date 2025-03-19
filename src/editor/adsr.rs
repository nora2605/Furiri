use nih_plug_vizia::vizia::{prelude::*, vg};
use std::sync::Arc;

use crate::FuririParams;

pub struct Adsr<V>
where
    V: Lens<Target = Arc<FuririParams>>,
{
    data: V,
}

impl<V> Adsr<V>
where
    V: Lens<Target = Arc<FuririParams>>,
{
    pub fn new(cx: &mut Context, data: V) -> Handle<Self> {
        Self { data }.build(cx, |_| {})
    }
}

impl<V> View for Adsr<V>
where
    V: Lens<Target = Arc<FuririParams>>,
{
    fn element(&self) -> Option<&'static str> {
        Some("adsr")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();
                let binding = self.data.get(cx);

                let attack = binding.envelope.attack.value() / 1000.0;
                let decay = binding.envelope.decay.value() / 1000.0;
                let sustain = binding.envelope.sustain.value();
                let release = binding.envelope.release.value() / 1000.0;

                path.move_to(bounds.x, bounds.y + bounds.h);
                path.line_to(bounds.x + bounds.w * attack / 8.0, bounds.y);
                path.line_to(
                    bounds.x + bounds.w * (attack + decay) / 8.0,
                    bounds.y + bounds.h * (1.0 - sustain),
                );
                path.line_to(
                    bounds.x + bounds.w * 3. / 8.,
                    bounds.y + bounds.h * (1.0 - sustain),
                );
                path.line_to(
                    bounds.x + bounds.w * (3. + release) / 8.,
                    bounds.y + bounds.h,
                );

                path
            },
            &vg::Paint::color(cx.font_color().into()).with_line_width(2.0),
        );
    }
}
