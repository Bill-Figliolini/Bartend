//! [`ColorField`]: one color of a palette, as a compact row (swatch, name,
//! hint and hex code) that opens to show hue, saturation and lightness
//! sliders.
#![expect(
    dead_code,
    reason = "the palette editor shows and uses it, from phase 4"
)]

use std::ops::RangeInclusive;

use iced::{
    Alignment::Center,
    Background, Border, Color, Degrees, Element, Gradient,
    Length::Fill,
    Padding, Theme,
    alignment::Horizontal,
    gradient::Linear,
    widget::{button, column, container, row, slider, space, stack, text, text_input},
};

use crate::{
    models::Rgb,
    presentation::{
        color::{Hsl, to_color},
        constants,
        styling::Styling,
        widget::{
            controls::{self, SLIDER_HANDLE},
            detail::{caption, hint, muted},
            frame,
            input::{Error, error_text},
        },
    },
};

/// Size of the square swatch showing the color.
const SWATCH: f32 = 28.0;
/// Width of the hex code box.
const HEX_WIDTH: f32 = 110.0;
/// Height of a slider, and of its handle.
const SLIDER_HEIGHT: f32 = 18.0;
/// Thickness of the colored track behind a slider.
const TRACK: f32 = 8.0;
/// Width of the name before each slider.
const CHANNEL_NAME_WIDTH: f32 = 80.0;
/// Width of the reading after each slider.
const CHANNEL_VALUE_WIDTH: f32 = 48.0;
/// Space before the sliders, so they sit under the name, past the swatch.
#[expect(
    clippy::cast_precision_loss,
    reason = "the spacing is a few pixels, which f32 holds exactly"
)]
const SLIDERS_INDENT: f32 = SWATCH + constants::SPACING as f32;

/// What can be done to a [`ColorField`].
#[derive(Debug, Clone, PartialEq)]
pub enum ColorMessage {
    /// Opens the sliders, or closes them if open.
    Toggle,
    /// The hex code box was edited.
    Hex(String),
    Hue(f32),
    Saturation(f32),
    Lightness(f32),
}

/// One color, edited by its hex code or by hue, saturation and lightness
/// sliders.
///
/// Keeps the hex code as typed, the slider positions, and the last valid
/// color, which the swatch shows while the code is being typed. The slider
/// positions are kept as they are rather than read back from the color,
/// which can't hold every position exactly: reading back would make the
/// sliders creep, and a gray would forget its hue.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorField {
    text: String,
    hsl: Hsl,
    color: Rgb,
    open: bool,
}

impl ColorField {
    /// A closed field showing `color`.
    pub fn new(color: Rgb) -> Self {
        Self {
            text: color.to_string(),
            hsl: Hsl::from(color),
            color,
            open: false,
        }
    }

    pub fn update(&mut self, message: ColorMessage) {
        match message {
            ColorMessage::Toggle => self.open = !self.open,
            ColorMessage::Hex(text) => {
                // Retyping the same color (in another case, say) keeps the
                // sliders where they are.
                if let Ok(color) = text.parse::<Rgb>()
                    && color != self.color
                {
                    self.color = color;
                    self.hsl = Hsl::from(color);
                }
                self.text = text;
            }
            ColorMessage::Hue(hue) => self.slide(Hsl { hue, ..self.hsl }),
            ColorMessage::Saturation(saturation) => self.slide(Hsl {
                saturation,
                ..self.hsl
            }),
            ColorMessage::Lightness(lightness) => self.slide(Hsl {
                lightness,
                ..self.hsl
            }),
        }
    }

    /// Moves the sliders to `hsl`, rewriting the hex code to match.
    fn slide(&mut self, hsl: Hsl) {
        self.hsl = hsl;
        self.color = hsl.to_rgb();
        self.text = self.color.to_string();
    }

    /// Closes the sliders, as when another field's open.
    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// The color the field last held a valid code for: what the swatch
    /// and any preview show.
    pub fn color(&self) -> Rgb {
        self.color
    }

    /// The color the hex code box holds.
    ///
    /// # Errors
    ///
    /// [`Error::ColorInvalid`] if the code isn't six hex digits.
    pub fn value(&self) -> Result<Rgb, Error> {
        self.text.parse().map_err(|_| Error::ColorInvalid)
    }

    /// The row: a swatch, `name` with `hint` under it, and the hex code,
    /// with the sliders under it while open. Pressing the swatch or name
    /// opens or closes the sliders. `on_change` builds the message for each
    /// change. With `show_error`, a bad hex code's error is shown under it.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn view<'a, M: Clone + 'a>(
        &'a self,
        styling: &Styling,
        name: &str,
        hint_text: &'a str,
        on_change: impl Fn(ColorMessage) -> M + Clone + 'a,
        show_error: bool,
    ) -> Element<'a, M> {
        let swatch = container(space().width(SWATCH).height(SWATCH))
            .style(swatch_style(self.color, styling.radius));
        let names = column![caption(styling, name), hint(hint_text)].spacing(2);
        let toggle = button(
            row![swatch, names]
                .spacing(constants::SPACING)
                .align_y(Center),
        )
        .width(Fill)
        .style(frame::entry(styling, self.open, None))
        .on_press(on_change(ColorMessage::Toggle));

        let on_hex = on_change.clone();
        let hex = text_input("#RRGGBB", &self.text)
            .on_input(move |text| on_hex(ColorMessage::Hex(text)))
            .width(HEX_WIDTH)
            .style(controls::text_field(styling));
        let error = if show_error { self.value().err() } else { None };
        let top = row![toggle, hex]
            .spacing(constants::SPACING)
            .align_y(Center);
        // The error goes under the whole row, so the box stays put when
        // the error is wider than it. The row is there even when empty, so
        // the box keeps its focus as errors come and go.
        let top = column![top, row(error.map(|error| error_text(error).into()))]
            .spacing(2)
            .align_x(Horizontal::Right);
        if !self.open {
            return top.into();
        }

        let sliders = Channel::ALL.map(|channel| {
            let on_slide = on_change.clone();
            channel_row(styling, channel, self.hsl, move |value| {
                on_slide(channel.message(value))
            })
        });
        let sliders = container(column(sliders).spacing(constants::SPACING / 2))
            .padding(Padding::ZERO.left(SLIDERS_INDENT));
        column![top, sliders].spacing(constants::SPACING).into()
    }
}

/// One slider row: the channel's name, its slider over a track showing the
/// colors it runs through, and its reading.
#[cfg_attr(coverage_nightly, coverage(off))]
fn channel_row<'a, M: Clone + 'a>(
    styling: &Styling,
    channel: Channel,
    hsl: Hsl,
    on_slide: impl Fn(f32) -> M + 'a,
) -> Element<'a, M> {
    let track = container(space().width(Fill).height(TRACK))
        .style(track_style(channel.stops(hsl), styling.radius));
    // Inset by half a handle, so the track's ends sit under the handle's
    // middle at either end of the slider.
    let track = container(track)
        .height(SLIDER_HEIGHT)
        .align_y(Center)
        .padding(Padding::ZERO.horizontal(f32::from(SLIDER_HANDLE) / 2.0));
    let slide = slider(channel.range(), channel.value(hsl), on_slide)
        .step(channel.step())
        .height(SLIDER_HEIGHT)
        .style(over_track(styling));
    row![
        text(channel.name())
            .size(constants::SMALL_TEXT)
            .style(muted)
            .width(CHANNEL_NAME_WIDTH),
        stack![track, slide].width(Fill),
        text(channel.reading(hsl))
            .size(constants::SMALL_TEXT)
            .width(CHANNEL_VALUE_WIDTH)
            .align_x(Horizontal::Right),
    ]
    .spacing(constants::SPACING)
    .align_y(Center)
    .into()
}

/// The square showing a field's color, edged in the shade of cards'
/// borders so a color close to the panel's still shows.
fn swatch_style(color: Rgb, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |theme| container::Style {
        background: Some(to_color(color).into()),
        border: Border {
            width: 1.0,
            color: theme.extended_palette().background.weak.color,
            radius: radius.into(),
        },
        ..container::Style::default()
    }
}

/// The track behind a slider: `stops` (offsets from 0 at the left to 1
/// at the right) as a left-to-right gradient, edged like a swatch.
fn track_style(stops: Vec<(f32, Rgb)>, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let gradient = stops
            .iter()
            .fold(Linear::new(Degrees(90.0)), |gradient, &(offset, color)| {
                gradient.add_stop(offset, to_color(color))
            });
        container::Style {
            background: Some(Background::Gradient(Gradient::Linear(gradient))),
            border: Border {
                width: 1.0,
                color: theme.extended_palette().background.weak.color,
                radius: radius.min(TRACK / 2.0).into(),
            },
            ..container::Style::default()
        }
    }
}

/// A [slider](controls::slider) with no rail of its own, laid over a
/// track that shows its colors.
fn over_track(styling: &Styling) -> impl Fn(&Theme, slider::Status) -> slider::Style + use<> {
    let base = controls::slider(styling);
    move |theme, status| {
        let mut style = base(theme, status);
        style.rail.backgrounds = (Color::TRANSPARENT.into(), Color::TRANSPARENT.into());
        style.rail.border = Border::default();
        style
    }
}

/// One of the three sliders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channel {
    Hue,
    Saturation,
    Lightness,
}

impl Channel {
    const ALL: [Channel; 3] = [Channel::Hue, Channel::Saturation, Channel::Lightness];

    fn name(self) -> &'static str {
        match self {
            Channel::Hue => "Hue",
            Channel::Saturation => "Saturation",
            Channel::Lightness => "Lightness",
        }
    }

    /// What the slider runs over: degrees round the color wheel for hue,
    /// fractions for the others.
    fn range(self) -> RangeInclusive<f32> {
        match self {
            Channel::Hue => 0.0..=360.0,
            Channel::Saturation | Channel::Lightness => 0.0..=1.0,
        }
    }

    /// How far one step of the slider moves: a degree, or half a percent.
    fn step(self) -> f32 {
        match self {
            Channel::Hue => 1.0,
            Channel::Saturation | Channel::Lightness => 0.005,
        }
    }

    fn value(self, hsl: Hsl) -> f32 {
        match self {
            Channel::Hue => hsl.hue,
            Channel::Saturation => hsl.saturation,
            Channel::Lightness => hsl.lightness,
        }
    }

    /// The reading beside the slider: whole degrees or a whole percent.
    fn reading(self, hsl: Hsl) -> String {
        let value = self.value(hsl);
        match self {
            Channel::Hue => format!("{value:.0}°"),
            Channel::Saturation | Channel::Lightness => format!("{:.0}%", value * 100.0),
        }
    }

    /// The message for moving this slider to `value`.
    fn message(self, value: f32) -> ColorMessage {
        match self {
            Channel::Hue => ColorMessage::Hue(value),
            Channel::Saturation => ColorMessage::Saturation(value),
            Channel::Lightness => ColorMessage::Lightness(value),
        }
    }

    /// The colors the slider runs through from `hsl`, as gradient stops:
    /// every hue at full strength; from gray to full saturation in this
    /// hue and lightness; or from black, through this hue and saturation,
    /// to white. RGB changes in straight lines between these stops, so
    /// the gradient shows exactly what each position gives.
    fn stops(self, hsl: Hsl) -> Vec<(f32, Rgb)> {
        match self {
            Channel::Hue => (0..=6_u8)
                .map(|sixth| {
                    let hue = Hsl {
                        hue: 60.0 * f32::from(sixth),
                        saturation: 1.0,
                        lightness: 0.5,
                    };
                    (f32::from(sixth) / 6.0, hue.to_rgb())
                })
                .collect(),
            Channel::Saturation => [0.0, 1.0]
                .map(|saturation| (saturation, Hsl { saturation, ..hsl }.to_rgb()))
                .to_vec(),
            Channel::Lightness => [0.0, 0.5, 1.0]
                .map(|lightness| (lightness, Hsl { lightness, ..hsl }.to_rgb()))
                .to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{styling::every_styling, theme::Themes};

    const NAVY: Rgb = Rgb::new(0x0F, 0x16, 0x26);
    const RED: Rgb = Rgb::new(255, 0, 0);

    fn field(color: Rgb) -> ColorField {
        ColorField::new(color)
    }

    /// Compares floats by bit pattern, which avoids `float_cmp`.
    fn same(a: f32, b: f32) -> bool {
        a.to_bits() == b.to_bits()
    }

    mod field {
        use super::*;

        #[test]
        fn starts_closed_showing_its_color() {
            let navy = field(NAVY);
            assert_eq!(navy.color(), NAVY);
            assert_eq!(navy.value(), Ok(NAVY));
            assert_eq!(navy.text, "#0F1626");
            assert_eq!(navy.hsl, Hsl::from(NAVY));
            assert!(!navy.is_open());
        }

        #[test]
        fn toggles_open_and_closes() {
            let mut navy = field(NAVY);
            navy.update(ColorMessage::Toggle);
            assert!(navy.is_open());
            navy.update(ColorMessage::Toggle);
            assert!(!navy.is_open());

            navy.update(ColorMessage::Toggle);
            navy.close();
            assert!(!navy.is_open());
            navy.close();
            assert!(!navy.is_open());
        }

        #[test]
        fn a_valid_hex_code_changes_the_color_and_moves_the_sliders() {
            let mut navy = field(NAVY);
            navy.update(ColorMessage::Hex("ff0000".to_string()));

            assert_eq!(navy.color(), RED);
            assert_eq!(navy.value(), Ok(RED));
            assert_eq!(navy.hsl, Hsl::from(RED));
            assert_eq!(navy.text, "ff0000", "the code stays as typed");
        }

        #[test]
        fn a_half_typed_hex_code_keeps_the_last_color() {
            let mut navy = field(NAVY);
            navy.update(ColorMessage::Hex("#FF00".to_string()));

            assert_eq!(navy.color(), NAVY);
            assert_eq!(navy.hsl, Hsl::from(NAVY));
            assert_eq!(navy.value(), Err(Error::ColorInvalid));
            assert_eq!(navy.text, "#FF00");
        }

        #[test]
        fn retyping_the_same_color_leaves_the_sliders_alone() {
            let mut gray = field(Rgb::new(0x80, 0x80, 0x80));
            gray.update(ColorMessage::Hue(200.0));
            gray.update(ColorMessage::Hex("#808080".to_string()));

            assert!(same(gray.hsl.hue, 200.0), "{:?}", gray.hsl);
        }

        #[test]
        fn sliding_rewrites_the_hex_code() {
            let mut red = field(RED);
            red.update(ColorMessage::Hue(240.0));
            assert_eq!(red.color(), Rgb::new(0, 0, 255));
            assert_eq!(red.text, "#0000FF");

            red.update(ColorMessage::Saturation(0.0));
            assert_eq!(red.color(), Rgb::new(128, 128, 128));

            red.update(ColorMessage::Lightness(1.0));
            assert_eq!(red.color(), Rgb::new(255, 255, 255));
            assert_eq!(red.value(), Ok(Rgb::new(255, 255, 255)));
        }

        #[test]
        fn sliding_fixes_a_half_typed_hex_code() {
            let mut red = field(RED);
            red.update(ColorMessage::Hex("#12".to_string()));
            red.update(ColorMessage::Lightness(0.25));

            assert_eq!(red.value(), Ok(red.color()));
            assert_eq!(red.color(), Rgb::new(128, 0, 0));
        }

        #[test]
        fn sliders_do_not_creep() {
            let mut navy = field(NAVY);
            let start = navy.hsl;
            for hue in [10.0, 300.0, 47.0, 221.0] {
                navy.update(ColorMessage::Hue(hue));
            }
            assert!(same(navy.hsl.saturation, start.saturation));
            assert!(same(navy.hsl.lightness, start.lightness));
        }

        #[test]
        fn a_gray_keeps_its_hue() {
            let mut red = field(RED);
            red.update(ColorMessage::Saturation(0.0));
            red.update(ColorMessage::Saturation(1.0));

            assert_eq!(red.color(), RED);
        }
    }

    mod channel {
        use super::*;

        const HSL: Hsl = Hsl {
            hue: 221.7,
            saturation: 0.434,
            lightness: 0.104,
        };

        #[test]
        fn read_and_set_their_own_part() {
            assert!(same(Channel::Hue.value(HSL), 221.7));
            assert!(same(Channel::Saturation.value(HSL), 0.434));
            assert!(same(Channel::Lightness.value(HSL), 0.104));
            assert_eq!(Channel::Hue.message(1.0), ColorMessage::Hue(1.0));
            assert_eq!(
                Channel::Saturation.message(1.0),
                ColorMessage::Saturation(1.0)
            );
            assert_eq!(
                Channel::Lightness.message(1.0),
                ColorMessage::Lightness(1.0)
            );
        }

        #[test]
        fn have_names_ranges_and_steps() {
            let names = Channel::ALL.map(Channel::name);
            assert_eq!(names, ["Hue", "Saturation", "Lightness"]);
            assert_eq!(Channel::Hue.range(), 0.0..=360.0);
            assert_eq!(Channel::Lightness.range(), 0.0..=1.0);
            assert!(same(Channel::Hue.step(), 1.0));
            assert!(same(Channel::Saturation.step(), 0.005));
        }

        #[test]
        fn read_in_degrees_and_percent() {
            assert_eq!(Channel::Hue.reading(HSL), "222°");
            assert_eq!(Channel::Saturation.reading(HSL), "43%");
            assert_eq!(Channel::Lightness.reading(HSL), "10%");
        }

        #[test]
        fn hue_runs_round_the_wheel_at_full_strength() {
            let stops = Channel::Hue.stops(HSL);
            let colors: Vec<Rgb> = stops.iter().map(|&(_, color)| color).collect();
            assert_eq!(
                colors,
                [
                    RED,
                    Rgb::new(255, 255, 0),
                    Rgb::new(0, 255, 0),
                    Rgb::new(0, 255, 255),
                    Rgb::new(0, 0, 255),
                    Rgb::new(255, 0, 255),
                    RED,
                ]
            );
            assert!(same(stops[0].0, 0.0) && same(stops[6].0, 1.0));
            assert!(stops.windows(2).all(|pair| pair[0].0 < pair[1].0));
        }

        #[test]
        fn saturation_runs_from_gray_in_this_hue_and_lightness() {
            let red = Hsl::from(RED);
            assert_eq!(
                Channel::Saturation.stops(red),
                [(0.0, Rgb::new(128, 128, 128)), (1.0, RED)]
            );
        }

        #[test]
        fn lightness_runs_from_black_through_the_color_to_white() {
            let red = Hsl::from(RED);
            assert_eq!(
                Channel::Lightness.stops(red),
                [
                    (0.0, Rgb::new(0, 0, 0)),
                    (0.5, RED),
                    (1.0, Rgb::new(255, 255, 255))
                ]
            );
        }

        #[test]
        fn every_slider_position_matches_its_track() {
            // Between stops the track blends in straight lines; check the
            // middle of each span against what the slider would give. The
            // hue track shows each hue at full strength, so it matches the
            // slider only for a color at full strength.
            let champagne = Hsl::from(Rgb::new(0xD8, 0xC0, 0x8A));
            for channel in Channel::ALL {
                let hsl = match channel {
                    Channel::Hue => Hsl {
                        saturation: 1.0,
                        lightness: 0.5,
                        ..champagne
                    },
                    Channel::Saturation | Channel::Lightness => champagne,
                };
                let stops = channel.stops(hsl);
                let (start, end) = (*channel.range().start(), *channel.range().end());
                for pair in stops.windows(2) {
                    let (low, high) = (pair[0], pair[1]);
                    let middle = f32::midpoint(low.0, high.0);
                    let given = match channel.message(start + middle * (end - start)) {
                        ColorMessage::Hue(hue) => Hsl { hue, ..hsl },
                        ColorMessage::Saturation(saturation) => Hsl { saturation, ..hsl },
                        ColorMessage::Lightness(lightness) => Hsl { lightness, ..hsl },
                        other => panic!("unexpected {other:?}"),
                    }
                    .to_rgb();
                    let blend = |a: u8, b: u8| {
                        u8::try_from((u16::from(a) + u16::from(b)).div_ceil(2)).unwrap()
                    };
                    let blended = Rgb::new(
                        blend(low.1.r, high.1.r),
                        blend(low.1.g, high.1.g),
                        blend(low.1.b, high.1.b),
                    );
                    let close = |a: u8, b: u8| a.abs_diff(b) <= 1;
                    assert!(
                        close(given.r, blended.r)
                            && close(given.g, blended.g)
                            && close(given.b, blended.b),
                        "{channel:?} at {middle}: slider gives {given}, track shows {blended}"
                    );
                }
            }
        }
    }

    mod styles {
        use super::*;

        #[test]
        fn a_swatch_shows_its_color_edged_like_a_card() {
            for theme in Themes::new().options() {
                let style = swatch_style(NAVY, 3.0)(theme);
                assert_eq!(style.background, Some(to_color(NAVY).into()));
                assert_eq!(
                    style.border.color,
                    theme.extended_palette().background.weak.color
                );
                assert_eq!(style.border.radius, 3.0.into());
            }
        }

        #[test]
        fn a_track_is_its_stops_left_to_right() {
            let stops = vec![(0.0, NAVY), (1.0, RED)];
            let style = track_style(stops, 0.0)(&Theme::Dark);
            let expected = Linear::new(Degrees(90.0))
                .add_stop(0.0, to_color(NAVY))
                .add_stop(1.0, to_color(RED));
            assert_eq!(
                style.background,
                Some(Background::Gradient(Gradient::Linear(expected)))
            );
        }

        #[test]
        fn a_track_rounds_no_more_than_half_its_thickness() {
            let style = track_style(Vec::new(), 99.0)(&Theme::Dark);
            assert_eq!(style.border.radius, (TRACK / 2.0).into());
        }

        #[test]
        fn a_slider_over_a_track_has_no_rail_but_keeps_its_handle() {
            for styling in every_styling() {
                for status in [slider::Status::Active, slider::Status::Dragged] {
                    let theme = Theme::Dark;
                    let style = over_track(&styling)(&theme, status);
                    let clear = Color::TRANSPARENT.into();
                    assert_eq!(style.rail.backgrounds, (clear, clear));
                    assert_eq!(style.rail.border, Border::default());
                    assert_eq!(
                        style.handle,
                        controls::slider(&styling)(&theme, status).handle
                    );
                }
            }
        }
    }
}
