//! Colour value type and the colour maths the theme needs.

use gpui_kit::Hsla;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(u32);

impl Rgb {
    pub const fn hex(value: u32) -> Self {
        Self(value & 0x00ff_ffff)
    }

    #[cfg(test)]
    pub const fn value(self) -> u32 {
        self.0
    }

    pub fn hsla(self) -> Hsla {
        gpui_kit::rgb(self.0).into()
    }

    pub fn alpha(self, opacity: f32) -> Hsla {
        self.hsla().opacity(opacity)
    }

    fn channels(self) -> [f32; 3] {
        let [_, r, g, b] = self.0.to_be_bytes();
        [r, g, b].map(|byte| f32::from(byte) / 255.)
    }

    pub fn to_hsl(self) -> (f32, f32, f32) {
        let [r, g, b] = self.channels();
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let lightness = f32::midpoint(max, min);
        let delta = max - min;
        if delta.total_cmp(&0.).is_eq() {
            return (0., 0., lightness);
        }
        let saturation = delta / (1. - (2. * lightness - 1.).abs());
        let hue = if max.total_cmp(&r).is_eq() {
            ((g - b) / delta).rem_euclid(6.)
        } else if max.total_cmp(&g).is_eq() {
            (b - r) / delta + 2.
        } else {
            (r - g) / delta + 4.
        };
        (hue * 60., saturation, lightness)
    }

    #[expect(
        clippy::many_single_char_names,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "r/g/b are conventional; `sector` is within 0..6 after rem_euclid"
    )]
    pub fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> Self {
        let saturation = saturation.clamp(0., 1.);
        let lightness = lightness.clamp(0., 1.);
        let chroma = (1. - (2. * lightness - 1.).abs()) * saturation;
        let sector = hue.rem_euclid(360.) / 60.;
        let x = chroma * (1. - (sector % 2. - 1.).abs());
        let (r, g, b) = match sector as u32 {
            0 => (chroma, x, 0.),
            1 => (x, chroma, 0.),
            2 => (0., chroma, x),
            3 => (0., x, chroma),
            4 => (x, 0., chroma),
            _ => (chroma, 0., x),
        };
        let m = lightness - chroma / 2.;
        Self::from_channels([r + m, g + m, b + m])
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the value is clamped to 0..=255 before the cast"
    )]
    fn from_channels(channels: [f32; 3]) -> Self {
        let [r, g, b] = channels.map(|channel| (channel.clamp(0., 1.) * 255.).round() as u8);
        Self(u32::from_be_bytes([0, r, g, b]))
    }

    pub fn lightness(self) -> f32 {
        self.to_hsl().2
    }

    pub fn with_lightness(self, lightness: f32) -> Self {
        let (hue, saturation, _) = self.to_hsl();
        Self::from_hsl(hue, saturation, lightness)
    }

    #[cfg(test)]
    pub fn over(self, backdrop: Self, opacity: f32) -> Self {
        let (fg, bg) = (self.channels(), backdrop.channels());
        Self::from_channels([0, 1, 2].map(|i| fg[i] * opacity + bg[i] * (1. - opacity)))
    }

    /// Moves the lightness away from `against` in 1 % steps until the WCAG
    /// contrast reaches `minimum`; hue and saturation are kept.
    pub fn with_min_contrast(self, against: Self, minimum: f32) -> Self {
        let step = if against.lightness() > 0.5 {
            -0.01
        } else {
            0.01
        };
        let mut colour = self;
        let mut lightness = self.lightness();
        while colour.contrast(against) < minimum && (0.0..=1.0).contains(&(lightness + step)) {
            lightness += step;
            colour = self.with_lightness(lightness);
        }
        colour
    }

    pub fn relative_luminance(self) -> f32 {
        let [r, g, b] = self.channels().map(|channel| {
            if channel <= 0.03928 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        });
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    pub fn contrast(self, other: Self) -> f32 {
        let (a, b) = (self.relative_luminance(), other.relative_luminance());
        let (light, dark) = if a > b { (a, b) } else { (b, a) };
        (light + 0.05) / (dark + 0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_masks_to_24_bits_and_round_trips() {
        assert_eq!(Rgb::hex(0xff_a78bfa).value(), 0xa78bfa);
    }

    #[test]
    fn hsl_round_trips_for_representative_colours() {
        for value in [
            0x111113, 0xfafafa, 0xa78bfa, 0xef4444, 0x000000, 0xffffff, 0x52525b,
        ] {
            let colour = Rgb::hex(value);
            let (h, s, l) = colour.to_hsl();
            let back = Rgb::from_hsl(h, s, l);
            for (a, b) in colour
                .value()
                .to_be_bytes()
                .into_iter()
                .zip(back.value().to_be_bytes())
            {
                assert!(a.abs_diff(b) <= 1, "{value:06x} -> {:06x}", back.value());
            }
        }
    }

    #[test]
    fn lightness_of_known_colours() {
        assert!((Rgb::hex(0x000000).lightness() - 0.).abs() < 1e-6);
        assert!((Rgb::hex(0xffffff).lightness() - 1.).abs() < 1e-6);
        assert!((Rgb::hex(0x111113).lightness() - 0.0706).abs() < 1e-3);
    }

    #[test]
    fn contrast_matches_the_wcag_reference_values() {
        let black = Rgb::hex(0x000000);
        let white = Rgb::hex(0xffffff);
        assert!((white.contrast(black) - 21.).abs() < 1e-3);
        assert!((white.contrast(white) - 1.).abs() < 1e-6);
        assert!((white.contrast(black) - black.contrast(white)).abs() < 1e-6);
        // #767676 on white is the classic AA-boundary grey (4.54:1)
        assert!((Rgb::hex(0x767676).contrast(white) - 4.54).abs() < 0.02);
    }

    #[test]
    fn over_blends_in_srgb() {
        let (white, black) = (Rgb::hex(0xffffff), Rgb::hex(0x000000));
        assert_eq!(white.over(black, 0.), black);
        assert_eq!(white.over(black, 1.), white);
        assert_eq!(white.over(black, 0.5), Rgb::hex(0x808080));
    }

    #[test]
    fn min_contrast_darkens_on_light_and_lightens_on_dark() {
        let accent = Rgb::hex(0xa78bfa);
        let on_light = accent.with_min_contrast(Rgb::hex(0xf7f7f9), 3.);
        assert!(on_light.contrast(Rgb::hex(0xf7f7f9)) >= 3.);
        assert!(on_light.lightness() < accent.lightness());
        let on_dark = Rgb::hex(0x4040ff).with_min_contrast(Rgb::hex(0x111113), 4.5);
        assert!(on_dark.contrast(Rgb::hex(0x111113)) >= 4.5);
        assert_eq!(accent.with_min_contrast(Rgb::hex(0x111113), 3.), accent);
    }

    #[test]
    fn with_lightness_keeps_hue_and_saturation() {
        let accent = Rgb::hex(0xa78bfa);
        let darker = accent.with_lightness(0.4);
        let (h1, s1, _) = accent.to_hsl();
        let (h2, s2, l2) = darker.to_hsl();
        assert!((h1 - h2).abs() < 1.5 && (s1 - s2).abs() < 0.02);
        assert!((l2 - 0.4).abs() < 0.01);
    }
}
