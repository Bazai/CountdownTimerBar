//! The popover's colours as semantic roles. `Palette::DARK` is the reference
//! palette; a test pins its values.

use super::color::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Elevation {
    pub knob: u8,
    pub card: u8,
}

impl Elevation {
    pub fn knob_opacity(self) -> f32 {
        f32::from(self.knob) / 100.
    }

    pub fn card_opacity(self) -> f32 {
        f32::from(self.card) / 100.
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a percentage scaled by a factor below 1 stays within 0..=100"
    )]
    fn scaled(self, factor: f32) -> Self {
        let scale = |percent: u8| (f32::from(percent) * factor).round() as u8;
        Self {
            knob: scale(self.knob),
            card: scale(self.card),
        }
    }
}

/// Every colour the views use. No `Default`: a new role must get a value in each palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub popover_bg: Rgb,
    pub popover_border: Rgb,
    pub control: Rgb,
    pub control_hover: Rgb,
    pub control_hover_border: Rgb,
    pub control_border: Rgb,
    pub divider: Rgb,
    pub input_bg: Rgb,
    pub text: Rgb,
    pub text_muted: Rgb,
    pub text_subtle: Rgb,
    pub text_body: Rgb,
    /// Value on a running / edited / dragged circle (sits on an accent tint).
    pub text_active: Rgb,
    pub icon: Rgb,
    pub dashed: Rgb,
    pub card_bg: Rgb,
    pub card_border: Rgb,
    pub badge_bg: Rgb,
    pub badge_fg: Rgb,
    pub focus_ring: Rgb,
    /// The selected m/s toggle (inverted chip).
    pub inverse_bg: Rgb,
    pub inverse_fg: Rgb,
    pub accent: Rgb,
    pub danger: Rgb,
    pub danger_text: Rgb,
    pub shadow: Rgb,
    pub elevation: Elevation,
}

impl Palette {
    pub const DARK: Self = Self {
        popover_bg: Rgb::hex(0x111113),
        popover_border: Rgb::hex(0x2c2c31),
        control: Rgb::hex(0x1b1b1f),
        control_hover: Rgb::hex(0x27272d),
        control_hover_border: Rgb::hex(0x4a4a52),
        control_border: Rgb::hex(0x2e2e34),
        divider: Rgb::hex(0x222226),
        input_bg: Rgb::hex(0x0a0a0b),
        text: Rgb::hex(0xfafafa),
        text_muted: Rgb::hex(0xa1a1aa),
        text_subtle: Rgb::hex(0x8b8b94),
        text_body: Rgb::hex(0xd4d4d8),
        text_active: Rgb::hex(0xffffff),
        icon: Rgb::hex(0xe4e4e7),
        dashed: Rgb::hex(0x5f5f69),
        card_bg: Rgb::hex(0x1b1b1f),
        card_border: Rgb::hex(0x3a3a41),
        badge_bg: Rgb::hex(0x52525b),
        badge_fg: Rgb::hex(0xffffff),
        focus_ring: Rgb::hex(0xffffff),
        inverse_bg: Rgb::hex(0xfafafa),
        inverse_fg: Rgb::hex(0x0f0f10),
        accent: Rgb::hex(0xa78bfa),
        danger: Rgb::hex(0xef4444),
        danger_text: Rgb::hex(0xf87171),
        shadow: Rgb::hex(0x000000),
        elevation: Elevation { knob: 55, card: 60 },
    };

    /// The light palette, derived from [`Palette::DARK`] (ADR 0004): neutral roles
    /// mirror their HSL lightness, `accent` and `danger` roles are darkened to a
    /// minimum contrast on the light body, the shadow gets weaker.
    pub fn light() -> Self {
        let dark = Self::DARK;
        let neutrals = dark.neutral_roles();
        let lightest = neutrals.iter().map(|c| c.lightness()).fold(0., f32::max);
        let darkest = neutrals.iter().map(|c| c.lightness()).fold(1., f32::min);
        let flip = |colour: Rgb| colour.with_lightness(darkest + lightest - colour.lightness());
        let popover_bg = flip(dark.popover_bg);
        Self {
            popover_bg,
            popover_border: flip(dark.popover_border),
            control: flip(dark.control),
            control_hover: flip(dark.control_hover),
            control_hover_border: flip(dark.control_hover_border),
            control_border: flip(dark.control_border),
            divider: flip(dark.divider),
            input_bg: flip(dark.input_bg),
            text: flip(dark.text),
            text_muted: flip(dark.text_muted),
            text_subtle: flip(dark.text_subtle),
            text_body: flip(dark.text_body),
            text_active: flip(dark.text_active),
            icon: flip(dark.icon),
            dashed: flip(dark.dashed),
            card_bg: flip(dark.card_bg),
            card_border: flip(dark.card_border),
            badge_bg: flip(dark.badge_bg),
            badge_fg: flip(dark.badge_fg),
            focus_ring: flip(dark.focus_ring),
            inverse_bg: flip(dark.inverse_bg),
            inverse_fg: flip(dark.inverse_fg),
            accent: dark.accent.with_min_contrast(popover_bg, 3.),
            danger: dark.danger.with_min_contrast(popover_bg, 3.),
            danger_text: dark.danger_text.with_min_contrast(popover_bg, 4.5),
            shadow: dark.shadow,
            elevation: dark.elevation.scaled(0.35),
        }
    }

    fn neutral_roles(&self) -> [Rgb; 22] {
        [
            self.popover_bg,
            self.popover_border,
            self.control,
            self.control_hover,
            self.control_hover_border,
            self.control_border,
            self.divider,
            self.input_bg,
            self.text,
            self.text_muted,
            self.text_subtle,
            self.text_body,
            self.text_active,
            self.icon,
            self.dashed,
            self.card_bg,
            self.card_border,
            self.badge_bg,
            self.badge_fg,
            self.focus_ring,
            self.inverse_bg,
            self.inverse_fg,
        ]
    }

    #[cfg(test)]
    pub fn roles(&self) -> [(&'static str, Rgb); 26] {
        [
            ("popover_bg", self.popover_bg),
            ("popover_border", self.popover_border),
            ("control", self.control),
            ("control_hover", self.control_hover),
            ("control_hover_border", self.control_hover_border),
            ("control_border", self.control_border),
            ("divider", self.divider),
            ("input_bg", self.input_bg),
            ("text", self.text),
            ("text_muted", self.text_muted),
            ("text_subtle", self.text_subtle),
            ("text_body", self.text_body),
            ("text_active", self.text_active),
            ("icon", self.icon),
            ("dashed", self.dashed),
            ("card_bg", self.card_bg),
            ("card_border", self.card_border),
            ("badge_bg", self.badge_bg),
            ("badge_fg", self.badge_fg),
            ("focus_ring", self.focus_ring),
            ("inverse_bg", self.inverse_bg),
            ("inverse_fg", self.inverse_fg),
            ("accent", self.accent),
            ("danger", self.danger),
            ("danger_text", self.danger_text),
            ("shadow", self.shadow),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palettes() -> [(&'static str, Palette); 2] {
        [("dark", Palette::DARK), ("light", Palette::light())]
    }

    #[test]
    fn light_is_derived_deterministically() {
        assert_eq!(Palette::light(), Palette::light());
    }

    #[test]
    fn light_differs_from_dark_but_keeps_the_shadow_colour() {
        assert_ne!(Palette::light(), Palette::DARK);
        assert_eq!(Palette::light().shadow, Palette::DARK.shadow);
    }

    #[test]
    fn light_has_lighter_shadows_than_dark() {
        let (dark, light) = (Palette::DARK.elevation, Palette::light().elevation);
        assert!(light.card < dark.card && light.knob < dark.knob);
    }

    #[test]
    fn dark_shadow_opacities_are_the_reference_ones() {
        let dark = Palette::DARK.elevation;
        assert_eq!((dark.card_opacity(), dark.knob_opacity()), (0.6, 0.55));
    }

    #[test]
    fn light_flips_the_stacking_order_of_the_neutral_layers() {
        let (dark, light) = (Palette::DARK, Palette::light());
        let order = |p: &Palette| {
            [p.popover_bg, p.control, p.control_hover].map(super::super::color::Rgb::lightness)
        };
        let (d, l) = (order(&dark), order(&light));
        assert!(d[0] < d[1] && d[1] < d[2], "dark layers get lighter: {d:?}");
        assert!(l[0] > l[1] && l[1] > l[2], "light layers get darker: {l:?}");
        assert!(dark.text.lightness() > 0.9 && light.text.lightness() < 0.1);
        assert!(dark.popover_bg.lightness() < 0.1 && light.popover_bg.lightness() > 0.9);
    }

    #[test]
    fn text_and_ui_colours_keep_their_contrast_in_both_themes() {
        for (name, p) in palettes() {
            // Running circle: accent tint over the popover (text_active sits on it).
            let active = p
                .accent
                .over(p.popover_bg, crate::theme::metrics::accent::FILL);
            let checks = [
                ("text on popover", p.text, p.popover_bg, 12.),
                ("text_muted on popover", p.text_muted, p.popover_bg, 4.5),
                ("text_body on popover", p.text_body, p.popover_bg, 7.),
                ("text_subtle on popover", p.text_subtle, p.popover_bg, 3.5),
                ("text on control", p.text, p.control, 10.),
                ("text_active on running circle", p.text_active, active, 10.),
                ("icon on popover", p.icon, p.popover_bg, 7.),
                ("accent on popover", p.accent, p.popover_bg, 3.),
                ("danger on popover", p.danger, p.popover_bg, 3.),
                ("danger_text on popover", p.danger_text, p.popover_bg, 4.5),
                ("badge_fg on badge_bg", p.badge_fg, p.badge_bg, 4.),
                ("inverse_fg on inverse_bg", p.inverse_fg, p.inverse_bg, 12.),
                ("focus ring on popover", p.focus_ring, p.popover_bg, 7.),
            ];
            for (what, fg, bg, minimum) in checks {
                let ratio = fg.contrast(bg);
                assert!(
                    ratio >= minimum,
                    "{name}: {what} is {ratio:.2}:1, needs {minimum}:1"
                );
            }
        }
    }

    /// Prints both palettes as a table: `cargo test print_palettes -- --ignored --nocapture`.
    #[test]
    #[ignore = "prints the palettes for docs and the design mock"]
    fn print_palettes() {
        let (dark, light) = (Palette::DARK.roles(), Palette::light().roles());
        println!("{:<22} {:<8} {:<8}", "role", "dark", "light");
        for ((name, d), (_, l)) in dark.iter().zip(light.iter()) {
            println!("{name:<22} #{:06x} #{:06x}", d.value(), l.value());
        }
    }

    #[test]
    fn roles_lists_every_field() {
        // Rgb is a u32, so the struct is exactly one word per role. A role
        // added to the struct but forgotten in `roles()` breaks this.
        let roles = Palette::DARK.roles();
        assert_eq!(
            std::mem::size_of::<Palette>(),
            (roles.len() * std::mem::size_of::<Rgb>() + std::mem::size_of::<Elevation>())
                .next_multiple_of(std::mem::align_of::<Palette>())
        );
        let mut names: Vec<_> = roles.iter().map(|(name, _)| *name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), roles.len(), "role names must be unique");
    }

    #[test]
    fn dark_palette_has_the_reference_values() {
        let dark = Palette::DARK;
        let expected = [
            ("popover_bg", dark.popover_bg, 0x111113),
            ("popover_border", dark.popover_border, 0x2c2c31),
            ("control", dark.control, 0x1b1b1f),
            ("control_hover", dark.control_hover, 0x27272d),
            ("dashed", dark.dashed, 0x5f5f69),
            ("danger", dark.danger, 0xef4444),
            ("control_border", dark.control_border, 0x2e2e34),
            ("divider", dark.divider, 0x222226),
            ("input_bg", dark.input_bg, 0x0a0a0b),
            ("text", dark.text, 0xfafafa),
            ("text_muted", dark.text_muted, 0xa1a1aa),
            ("text_subtle", dark.text_subtle, 0x8b8b94),
            ("accent", dark.accent, 0xa78bfa),
        ];
        for (name, role, value) in expected {
            assert_eq!(role.value(), value, "`{name}` drifted from the reference");
        }
    }
}
