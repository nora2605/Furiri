use nih_plug::{prelude::*, util::db_to_gain_fast};
use std::sync::Arc;

mod editor;
use nih_plug_vizia::ViziaState;

#[derive(Params)]
pub struct FuririParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    #[id = "basepitch"]
    basepitch: FloatParam,
    #[id = "basenote"]
    basenote: IntParam,
    #[id = "tuning"]
    tuning: EnumParam<Tuning>,
    #[id = "gain"]
    gain: FloatParam,
    #[nested]
    overtones: OvertoneParams,
    #[nested]
    envelope: EnvelopeParams,
}

#[derive(Enum, PartialEq)]
enum Tuning {
    Equal,
    Just, // relative to base note
    Pythagorean,
}

#[derive(Params)]
struct EnvelopeParams {
    #[id = "attack"]
    attack: FloatParam,
    #[id = "decay"]
    decay: FloatParam,
    #[id = "sustain"]
    sustain: FloatParam,
    #[id = "release"]
    release: FloatParam,
}

#[derive(Params)]
struct OvertoneParams {
    #[id = "overtone1"]
    overtone1: FloatParam,
    #[id = "overtone2"]
    overtone2: FloatParam,
    #[id = "overtone3"]
    overtone3: FloatParam,
    #[id = "overtone4"]
    overtone4: FloatParam,
    #[id = "overtone5"]
    overtone5: FloatParam,
    #[id = "overtone6"]
    overtone6: FloatParam,
    #[id = "overtone7"]
    overtone7: FloatParam,
    #[id = "overtone8"]
    overtone8: FloatParam,
}

impl Default for FuririParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            basepitch: FloatParam::new(
                "Base Pitch",
                440.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: 0.25,
                },
            )
            .with_step_size(0.01)
            .with_unit(" Hz"),
            basenote: IntParam::new("Base Note", 69, IntRange::Linear { min: 0, max: 127 }),
            tuning: EnumParam::new("Tuning", Tuning::Equal),
            gain: FloatParam::new(
                "Gain",
                -6.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_step_size(0.1)
            .with_unit(" dB"),
            overtones: OvertoneParams::default(),
            envelope: EnvelopeParams::default(),
        }
    }
}

impl Default for OvertoneParams {
    fn default() -> Self {
        Self {
            overtone1: FloatParam::new(
                "Overtone 1",
                1.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_step_size(0.01),
            overtone2: FloatParam::new(
                "Overtone 2",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_step_size(0.01),
            overtone3: FloatParam::new(
                "Overtone 3",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_step_size(0.01),
            overtone4: FloatParam::new(
                "Overtone 4",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_step_size(0.01),
            overtone5: FloatParam::new(
                "Overtone 5",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_step_size(0.01),
            overtone6: FloatParam::new(
                "Overtone 6",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_step_size(0.01),
            overtone7: FloatParam::new(
                "Overtone 7",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_step_size(0.01),
            overtone8: FloatParam::new(
                "Overtone 8",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_step_size(0.01),
        }
    }
}

impl Default for EnvelopeParams {
    fn default() -> Self {
        Self {
            attack: FloatParam::new(
                "Attack",
                1.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 500.0,
                },
            )
            .with_step_size(0.1)
            .with_unit(" ms"),
            decay: FloatParam::new(
                "Decay",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 500.0,
                },
            )
            .with_step_size(0.1)
            .with_unit(" ms"),
            sustain: FloatParam::new("Sustain", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_step_size(0.01),
            release: FloatParam::new(
                "Release",
                20.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 5000.0,
                    factor: 0.5,
                },
            )
            .with_step_size(0.1)
            .with_unit(" ms"),
        }
    }
}

const PITCH_RANGE: f32 = 2.0; // semitones
const MAX_VOICES: usize = 64;

pub struct Furiri {
    params: Arc<FuririParams>,
    current_notes: Vec<Note>,
    sample_rate: f32,
    pitch_bend: f32,
    sustain_pedal: bool,
}

struct Note {
    note: u8,
    velocity: u8,
    phase: f32,
    samples_since_event: usize, // updated per block
    release_envelope: f32,      // envelope value at note off
    off: bool,
    sustaining: bool,
}

impl Note {
    fn get_frequency(&self, basepitch: f32, basenote: u8, tuning: Tuning, pitch_bend: f32) -> f32 {
        let note = self.note as i32 - basenote as i32;
        (match tuning {
            Tuning::Equal => 2.0f32.powf(note as f32 / 12.0) * basepitch,
            Tuning::Just => {
                let ratio = match note.rem_euclid(12) {
                    0 => 1.0,
                    1 => 16.0 / 15.0,
                    2 => 9.0 / 8.0,
                    3 => 6.0 / 5.0,
                    4 => 5.0 / 4.0,
                    5 => 4.0 / 3.0,
                    6 => 45.0 / 32.0,
                    7 => 3.0 / 2.0,
                    8 => 8.0 / 5.0,
                    9 => 5.0 / 3.0,
                    10 => 16.0 / 9.0,
                    11 => 15.0 / 8.0,
                    _ => unreachable!(),
                };
                let octave = (note as f32 / 12.0).floor();
                2.0f32.powf(octave) * basepitch * ratio
            }
            Tuning::Pythagorean => {
                let ratio = match note.rem_euclid(12) {
                    0 => 1.0,
                    1 => 256.0 / 243.0,
                    2 => 9.0 / 8.0,
                    3 => 32.0 / 27.0,
                    4 => 81.0 / 64.0,
                    5 => 4.0 / 3.0,
                    6 => 729.0 / 512.0,
                    7 => 3.0 / 2.0,
                    8 => 128.0 / 81.0,
                    9 => 27.0 / 16.0,
                    10 => 16.0 / 9.0,
                    11 => 243.0 / 128.0,
                    _ => unreachable!(),
                };
                let octave = (note as f32 / 12.0).floor();
                2.0f32.powf(octave) * basepitch * ratio
            }
        }) * 2.0f32.powf(pitch_bend / 12.0)
    }

    fn calculate_envelope(&self, envelope_time: f32, envelope: &[f32; 4]) -> f32 {
        if self.off {
            (self.release_envelope * (1.0 - envelope_time / envelope[3])).max(0.0)
        } else if envelope_time <= envelope[0] {
            envelope_time / envelope[0]
        } else if envelope_time <= envelope[0] + envelope[1] {
            let s1 = 1.0 - envelope[2];
            1.0 - s1 * (envelope_time - envelope[0]) / envelope[1]
        } else {
            envelope[2]
        }
    }

    fn calculate_sample(
        &self,
        envelope_time: f32,
        overtones: &[f32; 8],
        envelope: &[f32; 4],
    ) -> f32 {
        let sample = overtones
            .iter()
            .enumerate()
            .map(|(i, v)| v * (std::f32::consts::TAU * (1.0 + i as f32) * self.phase).sin())
            .sum::<f32>();
        sample * (self.velocity as f32 / 127.0) * self.calculate_envelope(envelope_time, envelope)
    }
}

impl Default for Furiri {
    fn default() -> Self {
        Self {
            params: Arc::new(FuririParams::default()),
            current_notes: Vec::with_capacity(MAX_VOICES),
            sample_rate: 1.0,
            pitch_bend: 0.0,
            sustain_pedal: false,
        }
    }
}

impl Plugin for Furiri {
    const NAME: &'static str = "Furiri";
    const VENDOR: &'static str = "Nora2605";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "nora.ja2605@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let overtones: [f32; 8] = [
            self.params.overtones.overtone1.value(),
            self.params.overtones.overtone2.value(),
            self.params.overtones.overtone3.value(),
            self.params.overtones.overtone4.value(),
            self.params.overtones.overtone5.value(),
            self.params.overtones.overtone6.value(),
            self.params.overtones.overtone7.value(),
            self.params.overtones.overtone8.value(),
        ];
        let envelope: [f32; 4] = [
            self.params.envelope.attack.value() / 1000.0,
            self.params.envelope.decay.value() / 1000.0,
            self.params.envelope.sustain.value(),
            self.params.envelope.release.value() / 1000.0,
        ];

        let mut next_event = context.next_event();

        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        if self.current_notes.len() >= MAX_VOICES {
                            self.current_notes.swap_remove(0);
                        }
                        self.current_notes.push(Note {
                            note,
                            velocity: (velocity * 127.0) as u8,
                            phase: 0.0,
                            samples_since_event: 0,
                            release_envelope: 0.0,
                            off: false,
                            sustaining: false,
                        });
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        if self.sustain_pedal {
                            for n in self.current_notes.iter_mut().filter(|n| n.note == note) {
                                n.sustaining = true;
                            }
                        } else {
                            for n in self.current_notes.iter_mut().filter(|n| n.note == note) {
                                n.release_envelope = n.calculate_envelope(
                                    n.samples_since_event as f32 / self.sample_rate,
                                    &envelope,
                                );
                                n.off = true;
                                n.samples_since_event = 0;
                            }
                        }
                    }
                    NoteEvent::MidiPitchBend { value, .. } => {
                        self.pitch_bend = PITCH_RANGE * 2.0 * (value - 0.5);
                    }
                    NoteEvent::MidiCC { cc, value, .. } => {
                        if cc == 64 {
                            self.sustain_pedal = value > 0.5;
                            if !self.sustain_pedal {
                                for n in self.current_notes.iter_mut().filter(|n| n.sustaining) {
                                    n.release_envelope = n.calculate_envelope(
                                        n.samples_since_event as f32 / self.sample_rate,
                                        &envelope,
                                    );
                                    n.off = true;
                                    n.samples_since_event = 0;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                next_event = context.next_event();
            }

            let val = self
                .current_notes
                .iter_mut()
                .map(|note| {
                    let freq = note.get_frequency(
                        self.params.basepitch.value(),
                        self.params.basenote.value() as u8,
                        self.params.tuning.value(),
                        self.pitch_bend,
                    );
                    note.phase = (note.phase + freq / self.sample_rate).fract();

                    note.samples_since_event += 1;
                    let envelope_time = note.samples_since_event as f32 / self.sample_rate;
                    note.calculate_sample(envelope_time, &overtones, &envelope)
                })
                .sum::<f32>();

            for sample in channel_samples {
                *sample = val * db_to_gain_fast(self.params.gain.value());
            }
        }

        self.current_notes.retain(|n| {
            !n.off || (n.samples_since_event as f32 / self.sample_rate < envelope[3])
        });

        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for Furiri {
    const CLAP_ID: &'static str = "com.nojufe.furiri";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("waves if they were cool");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Stereo,
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
    ];
}

impl Vst3Plugin for Furiri {
    const VST3_CLASS_ID: [u8; 16] = *b"nojufepluhfuriri";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Synth,
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(Furiri);
nih_export_vst3!(Furiri);
