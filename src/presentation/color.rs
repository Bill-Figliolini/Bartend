//! Color maths for palettes: converting between the saved [`Rgb`] and
//! iced's [`Color`], hue/saturation/lightness for the palette editor's
//! sliders, and contrast between two colors.

use iced::Color;

use crate::models::Rgb;

/// The contrast ratio below which text is hard to read on its background,
/// from WCAG 2 (level AA, normal-size text).
pub const MIN_CONTRAST: f32 = 4.5;

/// `rgb` as an iced color.
pub fn to_color(rgb: Rgb) -> Color {
    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
}

/// The nearest [`Rgb`] to `color`, ignoring its alpha.
pub fn to_rgb(color: Color) -> Rgb {
    Rgb::new(channel(color.r), channel(color.g), channel(color.b))
}

/// A channel from 0 to 1 as the nearest byte.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to 0..=255 and rounded first, so it always fits a u8"
)]
fn channel(unit: f32) -> u8 {
    (unit.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// A color as hue, saturation and lightness: one slider for which
/// color, one for how vivid, one for how light.
///
/// The palette editor keeps these as it goes, rather than reading them back
/// from an [`Rgb`] on every change: an [`Rgb`] can't hold every [`Hsl`]
/// exactly, so reading back would make the sliders creep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    /// The hue in degrees, from 0 up to (not including) 360: red at 0,
    /// green at 120, blue at 240.
    pub hue: f32,
    /// From 0 (gray) to 1 (as vivid as the hue gets).
    pub saturation: f32,
    /// From 0 (black) to 1 (white); 0.5 is the hue at its purest.
    pub lightness: f32,
}

impl Hsl {
    /// The nearest [`Rgb`]. Hues outside 0–360 wrap round; saturation and
    /// lightness are held to 0–1.
    pub fn to_rgb(self) -> Rgb {
        let hue = self.hue.rem_euclid(360.0);
        let saturation = self.saturation.clamp(0.0, 1.0);
        let lightness = self.lightness.clamp(0.0, 1.0);

        let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
        let sector = hue / 60.0;
        let second = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
        let (r, g, b) = match sector {
            s if s < 1.0 => (chroma, second, 0.0),
            s if s < 2.0 => (second, chroma, 0.0),
            s if s < 3.0 => (0.0, chroma, second),
            s if s < 4.0 => (0.0, second, chroma),
            s if s < 5.0 => (second, 0.0, chroma),
            _ => (chroma, 0.0, second),
        };
        let lift = lightness - chroma / 2.0;
        Rgb::new(channel(r + lift), channel(g + lift), channel(b + lift))
    }
}

impl From<Rgb> for Hsl {
    fn from(rgb: Rgb) -> Self {
        let max = rgb.r.max(rgb.g).max(rgb.b);
        let min = rgb.r.min(rgb.g).min(rgb.b);
        let unit = |byte: u8| f32::from(byte) / 255.0;
        let (high, low) = (unit(max), unit(min));
        let lightness = f32::midpoint(high, low);
        if max == min {
            // A gray: no hue, no saturation.
            return Self {
                hue: 0.0,
                saturation: 0.0,
                lightness,
            };
        }
        let spread = high - low;
        let saturation = spread / (1.0 - (2.0 * lightness - 1.0).abs());
        let (r, g, b) = (unit(rgb.r), unit(rgb.g), unit(rgb.b));
        // Which channel is highest decides the sixth of the wheel the hue
        // falls in; compared as bytes, so there's no float equality.
        let sixths = if max == rgb.r {
            ((g - b) / spread).rem_euclid(6.0)
        } else if max == rgb.g {
            (b - r) / spread + 2.0
        } else {
            (r - g) / spread + 4.0
        };
        Self {
            hue: (sixths * 60.0).rem_euclid(360.0),
            saturation,
            lightness,
        }
    }
}

/// How much `a` and `b` stand apart, from 1 (the same) to 21 (black and
/// white), as WCAG 2 measures it. The order doesn't matter.
pub fn contrast_ratio(a: Rgb, b: Rgb) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    let (light, dark) = if a >= b { (a, b) } else { (b, a) };
    (light + 0.05) / (dark + 0.05)
}

/// How bright `rgb` looks, from 0 (black) to 1 (white): WCAG 2's relative
/// luminance.
fn luminance(rgb: Rgb) -> f32 {
    let linear = |byte: u8| {
        let unit = f32::from(byte) / 255.0;
        if unit <= 0.040_45 {
            unit / 12.92
        } else {
            ((unit + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(rgb.r) + 0.7152 * linear(rgb.g) + 0.0722 * linear(rgb.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether `a` and `b` agree to within `tolerance`.
    fn near(a: f32, b: f32, tolerance: f32) -> bool {
        (a - b).abs() <= tolerance
    }

    fn assert_hsl(rgb: Rgb, hue: f32, saturation: f32, lightness: f32) {
        let hsl = Hsl::from(rgb);
        assert!(
            near(hsl.hue, hue, 0.5)
                && near(hsl.saturation, saturation, 0.005)
                && near(hsl.lightness, lightness, 0.005),
            "{rgb} gave {hsl:?}, expected ({hue}, {saturation}, {lightness})"
        );
    }

    mod rgb_and_color {
        use super::*;

        #[test]
        fn convert_both_ways() {
            let champagne = Rgb::new(0xD8, 0xC0, 0x8A);
            let color = to_color(champagne);
            assert_eq!(color, Color::from_rgb8(0xD8, 0xC0, 0x8A));
            assert_eq!(to_rgb(color), champagne);
        }

        #[test]
        fn every_byte_survives_the_round_trip() {
            for byte in 0..=u8::MAX {
                let gray = Rgb::new(byte, byte, byte);
                assert_eq!(to_rgb(to_color(gray)), gray);
            }
        }

        #[test]
        fn out_of_range_channels_are_held_to_the_ends() {
            let color = Color {
                r: -0.5,
                g: 1.5,
                b: 0.5,
                a: 1.0,
            };
            assert_eq!(to_rgb(color), Rgb::new(0, 255, 128));
        }
    }

    mod hsl {
        use super::*;

        #[test]
        fn reads_the_primaries_and_secondaries() {
            assert_hsl(Rgb::new(255, 0, 0), 0.0, 1.0, 0.5);
            assert_hsl(Rgb::new(255, 255, 0), 60.0, 1.0, 0.5);
            assert_hsl(Rgb::new(0, 255, 0), 120.0, 1.0, 0.5);
            assert_hsl(Rgb::new(0, 255, 255), 180.0, 1.0, 0.5);
            assert_hsl(Rgb::new(0, 0, 255), 240.0, 1.0, 0.5);
            assert_hsl(Rgb::new(255, 0, 255), 300.0, 1.0, 0.5);
        }

        #[test]
        fn reads_grays_as_unsaturated() {
            assert_hsl(Rgb::new(0, 0, 0), 0.0, 0.0, 0.0);
            assert_hsl(Rgb::new(128, 128, 128), 0.0, 0.0, 0.502);
            assert_hsl(Rgb::new(255, 255, 255), 0.0, 0.0, 1.0);
        }

        #[test]
        fn reads_the_navy_palette_as_one_hue_getting_lighter() {
            assert_hsl(Rgb::new(0x0F, 0x16, 0x26), 221.7, 0.434, 0.104);
            assert_hsl(Rgb::new(0x17, 0x20, 0x36), 222.6, 0.403, 0.151);
            assert_hsl(Rgb::new(0x2D, 0x3A, 0x57), 221.4, 0.3182, 0.259);
        }

        #[test]
        fn a_hue_just_below_red_stays_in_range() {
            let hsl = Hsl::from(Rgb::new(255, 0, 1));
            assert!((359.0..360.0).contains(&hsl.hue), "{hsl:?}");
        }

        #[test]
        fn writes_the_primaries_and_secondaries() {
            let hsl = |hue| Hsl {
                hue,
                saturation: 1.0,
                lightness: 0.5,
            };
            assert_eq!(hsl(0.0).to_rgb(), Rgb::new(255, 0, 0));
            assert_eq!(hsl(60.0).to_rgb(), Rgb::new(255, 255, 0));
            assert_eq!(hsl(120.0).to_rgb(), Rgb::new(0, 255, 0));
            assert_eq!(hsl(180.0).to_rgb(), Rgb::new(0, 255, 255));
            assert_eq!(hsl(240.0).to_rgb(), Rgb::new(0, 0, 255));
            assert_eq!(hsl(300.0).to_rgb(), Rgb::new(255, 0, 255));
        }

        #[test]
        fn wraps_the_hue_and_holds_the_rest_to_range() {
            let wild = Hsl {
                hue: -120.0,
                saturation: 2.0,
                lightness: 0.5,
            };
            assert_eq!(wild.to_rgb(), Rgb::new(0, 0, 255));
            let over = Hsl {
                hue: 480.0,
                saturation: 1.0,
                lightness: 7.0,
            };
            assert_eq!(over.to_rgb(), Rgb::new(255, 255, 255));
            let under = Hsl {
                hue: 0.0,
                saturation: -1.0,
                lightness: -1.0,
            };
            assert_eq!(under.to_rgb(), Rgb::new(0, 0, 0));
        }

        #[test]
        fn every_color_on_a_grid_survives_the_round_trip() {
            let steps = (0..=255).step_by(15).chain([1, 254]);
            for r in steps.clone() {
                for g in steps.clone() {
                    for b in steps.clone() {
                        let rgb = Rgb::new(r, g, b);
                        assert_eq!(Hsl::from(rgb).to_rgb(), rgb);
                    }
                }
            }
        }
    }

    mod contrast {
        use super::*;

        const BLACK: Rgb = Rgb::new(0, 0, 0);
        const WHITE: Rgb = Rgb::new(255, 255, 255);

        #[test]
        fn runs_from_one_to_twenty_one() {
            assert!(near(contrast_ratio(BLACK, WHITE), 21.0, 0.01));
            assert!(near(contrast_ratio(WHITE, WHITE), 1.0, 0.0001));
            assert!(near(contrast_ratio(BLACK, BLACK), 1.0, 0.0001));
        }

        #[test]
        fn does_not_depend_on_order() {
            let a = Rgb::new(0xD8, 0xC0, 0x8A);
            let b = Rgb::new(0x0F, 0x16, 0x26);
            assert!(near(contrast_ratio(a, b), contrast_ratio(b, a), 0.0001));
        }

        #[test]
        fn matches_known_values() {
            // #777777 on white is the classic just-too-light gray: 4.48.
            let gray = Rgb::new(0x77, 0x77, 0x77);
            assert!(near(contrast_ratio(gray, WHITE), 4.48, 0.01));
            assert!(contrast_ratio(gray, WHITE) < MIN_CONTRAST);
            // #767676 is the lightest gray that passes on white: 4.54.
            let passing = Rgb::new(0x76, 0x76, 0x76);
            assert!(near(contrast_ratio(passing, WHITE), 4.54, 0.01));
            assert!(contrast_ratio(passing, WHITE) >= MIN_CONTRAST);
        }

        #[test]
        fn shows_why_hsl_lightness_is_not_brightness() {
            let half_lit = |hue| {
                Hsl {
                    hue,
                    saturation: 1.0,
                    lightness: 0.5,
                }
                .to_rgb()
            };
            let yellow = contrast_ratio(half_lit(60.0), BLACK);
            let blue = contrast_ratio(half_lit(240.0), BLACK);
            assert!(near(yellow, 19.56, 0.05), "{yellow}");
            assert!(near(blue, 2.44, 0.05), "{blue}");
        }
    }
}
