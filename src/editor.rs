use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;

use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;

use crate::FuririParams;

mod adsr;
use adsr::Adsr;

mod waveform;
use waveform::Waveform;

#[derive(Lens)]
struct Data {
    params: Arc<FuririParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 400))
}

pub(crate) fn create(
    params: Arc<FuririParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        nih_plug_vizia::assets::register_noto_sans_light(cx);

        Data {
            params: params.clone(),
        }
        .build(cx);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, "Furiri.")
                    .font_size(20.0)
                    .font_weight(FontWeightKeyword::Bold);
                Label::new(cx, "Envelope").height(Pixels(20.0));
                ParamSlider::new(cx, Data::params, |params| &params.envelope.attack);
                ParamSlider::new(cx, Data::params, |params| &params.envelope.decay);
                ParamSlider::new(cx, Data::params, |params| &params.envelope.sustain);
                ParamSlider::new(cx, Data::params, |params| &params.envelope.release);
                Label::new(cx, "Gain").height(Pixels(20.0));
                ParamSlider::new(cx, Data::params, |params| &params.gain);
                Adsr::new(cx, Data::params).height(Pixels(50.0));
            })
            .row_between(Pixels(10.0));

            VStack::new(cx, |cx| {
                Label::new(cx, "Base Freq & Midi Note")
                    .height(Pixels(20.0))
                    .top(Pixels(20.0));
                ParamSlider::new(cx, Data::params, |params| &params.basepitch);
                ParamSlider::new(cx, Data::params, |params| &params.basenote);
                Label::new(cx, "Tuning")
                    .height(Pixels(20.0));
                ParamSlider::new(cx, Data::params, |params| &params.tuning);
                Waveform::new(cx, Data::params).height(Pixels(100.0));
            })
            .row_between(Pixels(10.0))
            .top(Pixels(25.0));

            VStack::new(cx, |cx| {
                Label::new(cx, "Overtones");
                ParamSlider::new(cx, Data::params, |params| &params.overtones.overtone1);
                ParamSlider::new(cx, Data::params, |params| &params.overtones.overtone2);
                ParamSlider::new(cx, Data::params, |params| &params.overtones.overtone3);
                ParamSlider::new(cx, Data::params, |params| &params.overtones.overtone4);
                ParamSlider::new(cx, Data::params, |params| &params.overtones.overtone5);
                ParamSlider::new(cx, Data::params, |params| &params.overtones.overtone6);
                ParamSlider::new(cx, Data::params, |params| &params.overtones.overtone7);
                ParamSlider::new(cx, Data::params, |params| &params.overtones.overtone8);
            })
            .row_between(Pixels(10.0))
            .top(Pixels(20.0));
        })
        .top(Pixels(10.0))
        .col_between(Pixels(50.0))
        .left(Pixels(20.0));
    })
}
