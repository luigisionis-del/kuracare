//! Look of the app: themes (Zafferano, Salvia, Alba and the four seasons of
//! central Sardinia), day/evening mode, bundled fonts.
//!
//! Every theme is only colours, fonts and shapes, turned into CSS custom
//! properties here; `assets/app.css` uses nothing else. A test checks the
//! contrast of every pair that carries text (WCAG AA).
//!
//! The choice is kept in `look.json` in the app-private directory, outside the
//! encrypted vault, so that the lock screen already looks right. It holds
//! only the theme name and the mode: nothing personal.

use std::sync::OnceLock;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::platform::data_dir;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    /// Follows the calendar: spring, summer, autumn, winter.
    #[default]
    Seasons,
    Zafferano,
    Salvia,
    Alba,
    Primavera,
    Estate,
    Autunno,
    Inverno,
}

impl Theme {
    pub const CHOICES: [Theme; 8] = [
        Self::Seasons,
        Self::Zafferano,
        Self::Salvia,
        Self::Alba,
        Self::Primavera,
        Self::Estate,
        Self::Autunno,
        Self::Inverno,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Seasons => "Le stagioni",
            Self::Zafferano => "Zafferano",
            Self::Salvia => "Salvia",
            Self::Alba => "Alba",
            Self::Primavera => "Primavera",
            Self::Estate => "Estate",
            Self::Autunno => "Autunno",
            Self::Inverno => "Inverno",
        }
    }

    /// The concrete theme shown on `month` (1-12): seasons change on the 1st of
    /// March, June, September and December (meteorological seasons).
    pub fn resolve(self, month: i8) -> Theme {
        match self {
            Self::Seasons => match month {
                3..=5 => Self::Primavera,
                6..=8 => Self::Estate,
                9..=11 => Self::Autunno,
                _ => Self::Inverno,
            },
            other => other,
        }
    }

    pub fn css_name(self) -> &'static str {
        match self {
            Self::Seasons => "seasons",
            Self::Zafferano => "zafferano",
            Self::Salvia => "salvia",
            Self::Alba => "alba",
            Self::Primavera => "primavera",
            Self::Estate => "estate",
            Self::Autunno => "autunno",
            Self::Inverno => "inverno",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    /// Day or evening as the phone says.
    #[default]
    Auto,
    Light,
    Dark,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Self::Auto, Self::Light, Self::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Come il telefono",
            Self::Light => "Sempre chiaro",
            Self::Dark => "Sempre scuro",
        }
    }

    pub fn css_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// For the Android bars: 0 auto, 1 light, 2 dark.
    pub fn code(self) -> i32 {
        match self {
            Self::Auto => 0,
            Self::Light => 1,
            Self::Dark => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Look {
    pub theme: Theme,
    pub mode: Mode,
}

const LOOK_FILE: &str = "look.json";

impl Look {
    pub fn load() -> Self {
        std::fs::read(data_dir().join(LOOK_FILE))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(data_dir())?;
        let json = serde_json::to_vec(self).expect("look serialises");
        std::fs::write(data_dir().join(LOOK_FILE), json)
    }
}

// ---------------------------------------------------------------- palettes

/// Colours of one mode (day or evening), as `#rrggbb`.
#[derive(Clone, Copy, Debug)]
pub struct Colors {
    pub bg: &'static str,
    /// Bottom of the background gradient (same as `bg` for a flat one).
    pub bg2: &'static str,
    pub surface: &'static str,
    pub surface2: &'static str,
    pub text: &'static str,
    pub muted: &'static str,
    pub accent: &'static str,
    pub accent_text: &'static str,
    pub accent_soft: &'static str,
    pub icon: &'static str,
    /// Saffron: the moments that did good (a gesture, something done).
    pub bloom: &'static str,
    pub on_bloom: &'static str,
    pub border: &'static str,
    pub danger: &'static str,
    pub focus: &'static str,
    pub toast_bg: &'static str,
    pub toast_fg: &'static str,
}

/// Fonts and shapes, shared by a family of themes.
#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub body: &'static str,
    pub head: &'static str,
    pub head_weight: u16,
    pub greet: &'static str,
    pub greet_weight: u16,
    pub greet_size: &'static str,
    pub radius: &'static str,
    pub button_radius: &'static str,
    pub chip_radius: &'static str,
    /// Cards: soft shadow (true) or a thin border (false).
    pub shadow: bool,
}

pub struct Palette {
    pub theme: Theme,
    pub style: Style,
    pub day: Colors,
    pub night: Colors,
}

const ZAFFERANO_STYLE: Style = Style {
    body: "'Atkinson Hyperlegible'",
    head: "'Bricolage Grotesque'",
    head_weight: 700,
    greet: "'Bricolage Grotesque'",
    greet_weight: 700,
    greet_size: "2rem",
    radius: "20px",
    button_radius: "14px",
    chip_radius: "999px",
    shadow: true,
};

const SALVIA_STYLE: Style = Style {
    body: "'Figtree'",
    head: "'Figtree'",
    head_weight: 600,
    greet: "'Instrument Serif'",
    greet_weight: 400,
    greet_size: "2.45rem",
    radius: "14px",
    button_radius: "10px",
    chip_radius: "10px",
    shadow: false,
};

const ALBA_STYLE: Style = Style {
    body: "'Nunito Sans'",
    head: "'Outfit'",
    head_weight: 600,
    greet: "'Outfit'",
    greet_weight: 600,
    greet_size: "2rem",
    radius: "24px",
    button_radius: "999px",
    chip_radius: "999px",
    shadow: true,
};

const DANGER_DAY: &str = "#9a3b2e";
const DANGER_NIGHT: &str = "#f2a093";
const FOCUS_DAY: &str = "#2f5bd0";
const FOCUS_NIGHT: &str = "#9db8ff";

pub const PALETTES: [Palette; 7] = [
    Palette {
        theme: Theme::Zafferano,
        style: ZAFFERANO_STYLE,
        day: Colors {
            bg: "#f7f5f9",
            bg2: "#f7f5f9",
            surface: "#ffffff",
            surface2: "#efeaf4",
            text: "#2a2233",
            muted: "#5e5568",
            accent: "#5a3d8a",
            accent_text: "#ffffff",
            accent_soft: "#ece4f6",
            icon: "#93600a",
            bloom: "#e8a317",
            on_bloom: "#2a1d05",
            border: "#e2dcea",
            danger: DANGER_DAY,
            focus: FOCUS_DAY,
            toast_bg: "#2a2233",
            toast_fg: "#f7f2e6",
        },
        night: Colors {
            bg: "#17131d",
            bg2: "#17131d",
            surface: "#221c2a",
            surface2: "#2b2434",
            text: "#efe9f4",
            muted: "#b0a6bb",
            accent: "#c9b2f0",
            accent_text: "#1d1329",
            accent_soft: "#2f2640",
            icon: "#f0b53d",
            bloom: "#f0b53d",
            on_bloom: "#241703",
            border: "#3a3146",
            danger: DANGER_NIGHT,
            focus: FOCUS_NIGHT,
            toast_bg: "#f3ecf9",
            toast_fg: "#1d1329",
        },
    },
    Palette {
        theme: Theme::Salvia,
        style: SALVIA_STYLE,
        day: Colors {
            bg: "#f2f4f0",
            bg2: "#f2f4f0",
            surface: "#fcfdfb",
            surface2: "#e6ebe6",
            text: "#1f2a24",
            muted: "#4f5c54",
            accent: "#3d6a57",
            accent_text: "#ffffff",
            accent_soft: "#dce8e0",
            icon: "#3d6a57",
            bloom: "#e2a72e",
            on_bloom: "#2a1e06",
            border: "#d3dcd5",
            danger: DANGER_DAY,
            focus: FOCUS_DAY,
            toast_bg: "#1f2a24",
            toast_fg: "#f2f4f0",
        },
        night: Colors {
            bg: "#131a16",
            bg2: "#131a16",
            surface: "#1b2420",
            surface2: "#24302a",
            text: "#e6ece7",
            muted: "#a5b2aa",
            accent: "#8fc4aa",
            accent_text: "#0f2219",
            accent_soft: "#24372e",
            icon: "#8fc4aa",
            bloom: "#e9b44c",
            on_bloom: "#261a03",
            border: "#33413a",
            danger: DANGER_NIGHT,
            focus: FOCUS_NIGHT,
            toast_bg: "#e6ece7",
            toast_fg: "#131a16",
        },
    },
    Palette {
        theme: Theme::Alba,
        style: ALBA_STYLE,
        day: Colors {
            bg: "#fff3dc",
            bg2: "#f6f2f4",
            surface: "#fffaf3",
            surface2: "#f6ead9",
            text: "#2b2420",
            muted: "#62574f",
            accent: "#a3560b",
            accent_text: "#ffffff",
            accent_soft: "#fbe6c8",
            icon: "#a3560b",
            bloom: "#e8a317",
            on_bloom: "#2a1d05",
            border: "#efdcc4",
            danger: DANGER_DAY,
            focus: FOCUS_DAY,
            toast_bg: "#2b2420",
            toast_fg: "#fff3dc",
        },
        night: Colors {
            bg: "#2a2142",
            bg2: "#141219",
            surface: "#231e33",
            surface2: "#2d2740",
            text: "#f1ebf6",
            muted: "#b8aec6",
            accent: "#f2b24a",
            accent_text: "#2a1a02",
            accent_soft: "#3a2f4f",
            icon: "#f2b24a",
            bloom: "#f2b24a",
            on_bloom: "#2a1a02",
            border: "#3a3352",
            danger: DANGER_NIGHT,
            focus: FOCUS_NIGHT,
            toast_bg: "#f1ebf6",
            toast_fg: "#1b1828",
        },
    },
    // Primavera: il verde tenero dei pascoli, il giallo della ginestra.
    Palette {
        theme: Theme::Primavera,
        style: ZAFFERANO_STYLE,
        day: Colors {
            bg: "#f4f6ec",
            bg2: "#f4f6ec",
            surface: "#ffffff",
            surface2: "#e8eddc",
            text: "#25301f",
            muted: "#56624c",
            accent: "#4b7a2c",
            accent_text: "#ffffff",
            accent_soft: "#e2ecd3",
            icon: "#5c7a2e",
            bloom: "#f0c02c",
            on_bloom: "#2a1d05",
            border: "#dde4cf",
            danger: DANGER_DAY,
            focus: FOCUS_DAY,
            toast_bg: "#25301f",
            toast_fg: "#f4f6ec",
        },
        night: Colors {
            bg: "#131910",
            bg2: "#131910",
            surface: "#1c2418",
            surface2: "#253020",
            text: "#e9eee2",
            muted: "#a9b59e",
            accent: "#a8d07c",
            accent_text: "#142009",
            accent_soft: "#27341f",
            icon: "#c3dc8f",
            bloom: "#f2c94c",
            on_bloom: "#261c02",
            border: "#33402c",
            danger: DANGER_NIGHT,
            focus: FOCUS_NIGHT,
            toast_bg: "#e9eee2",
            toast_fg: "#131910",
        },
    },
    // Estate: le stoppie dorate, il blu del mare, il cielo di maestrale.
    Palette {
        theme: Theme::Estate,
        style: ZAFFERANO_STYLE,
        day: Colors {
            bg: "#faf5e8",
            bg2: "#faf5e8",
            surface: "#fffdf7",
            surface2: "#f1e9d3",
            text: "#2a2619",
            muted: "#625a44",
            accent: "#1d5c88",
            accent_text: "#ffffff",
            accent_soft: "#dde9f2",
            icon: "#8f6508",
            bloom: "#e8a317",
            on_bloom: "#2a1d05",
            border: "#ece2c8",
            danger: DANGER_DAY,
            focus: FOCUS_DAY,
            toast_bg: "#13324a",
            toast_fg: "#faf5e8",
        },
        night: Colors {
            bg: "#0e1923",
            bg2: "#0e1923",
            surface: "#16242f",
            surface2: "#1e2f3c",
            text: "#e8eef3",
            muted: "#a4b3c0",
            accent: "#8ac2e8",
            accent_text: "#0b1e2c",
            accent_soft: "#1e3344",
            icon: "#f0b53d",
            bloom: "#f0b53d",
            on_bloom: "#241703",
            border: "#2a3c4b",
            danger: DANGER_NIGHT,
            focus: FOCUS_NIGHT,
            toast_bg: "#e8eef3",
            toast_fg: "#0e1923",
        },
    },
    // Autunno: il rosso del corbezzolo, il bruno del sughero.
    Palette {
        theme: Theme::Autunno,
        style: ZAFFERANO_STYLE,
        day: Colors {
            bg: "#f7f0ea",
            bg2: "#f7f0ea",
            surface: "#fffaf6",
            surface2: "#efe3d9",
            text: "#2d211b",
            muted: "#65544b",
            accent: "#9a3a1b",
            accent_text: "#ffffff",
            accent_soft: "#f4dfd5",
            icon: "#7a5536",
            bloom: "#e8a317",
            on_bloom: "#2a1d05",
            border: "#eadbd0",
            danger: DANGER_DAY,
            focus: FOCUS_DAY,
            toast_bg: "#2d211b",
            toast_fg: "#f7f0ea",
        },
        night: Colors {
            bg: "#1a1310",
            bg2: "#1a1310",
            surface: "#251b16",
            surface2: "#30241d",
            text: "#f1e8e2",
            muted: "#bba99f",
            accent: "#f2956b",
            accent_text: "#2a1006",
            accent_soft: "#3a2419",
            icon: "#d9a77c",
            bloom: "#f0b53d",
            on_bloom: "#241703",
            border: "#3d2e26",
            danger: DANGER_NIGHT,
            focus: FOCUS_NIGHT,
            toast_bg: "#f1e8e2",
            toast_fg: "#1a1310",
        },
    },
    // Inverno: il grigio del granito, il verde scuro del leccio, la nebbia.
    Palette {
        theme: Theme::Inverno,
        style: ZAFFERANO_STYLE,
        day: Colors {
            bg: "#eef0f0",
            bg2: "#eef0f0",
            surface: "#fbfcfc",
            surface2: "#e1e5e5",
            text: "#1f2627",
            muted: "#536063",
            accent: "#2e5445",
            accent_text: "#ffffff",
            accent_soft: "#dbe7e2",
            icon: "#56656a",
            bloom: "#e8a317",
            on_bloom: "#2a1d05",
            border: "#d8dddd",
            danger: DANGER_DAY,
            focus: FOCUS_DAY,
            toast_bg: "#1f2627",
            toast_fg: "#eef0f0",
        },
        night: Colors {
            bg: "#111516",
            bg2: "#111516",
            surface: "#1a2021",
            surface2: "#232b2c",
            text: "#e6ebeb",
            muted: "#a3aeb0",
            accent: "#9cc8b6",
            accent_text: "#0f2219",
            accent_soft: "#22302c",
            icon: "#b5c2c5",
            bloom: "#f0b53d",
            on_bloom: "#241703",
            border: "#2f3839",
            danger: DANGER_NIGHT,
            focus: FOCUS_NIGHT,
            toast_bg: "#e6ebeb",
            toast_fg: "#111516",
        },
    },
];

/// The palette of a concrete theme (resolve `Seasons` first); Zafferano otherwise.
pub fn palette(t: Theme) -> &'static Palette {
    PALETTES
        .iter()
        .find(|p| p.theme == t)
        .unwrap_or(&PALETTES[0])
}

/// "#rrggbb" -> 0xAARRGGBB with full alpha (for Android).
pub fn argb(hex: &str) -> i32 {
    let rgb = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
    (0xFF00_0000u32 | rgb) as i32
}

// ---------------------------------------------------------------- CSS

fn colors_css(c: &Colors) -> String {
    format!(
        "--bg:{};--bg2:{};--surface:{};--surface-2:{};--text:{};--muted:{};--accent:{};\
         --accent-text:{};--accent-soft:{};--icon:{};--bloom:{};--on-bloom:{};--border:{};\
         --danger:{};--focus:{};--toast-bg:{};--toast-fg:{};",
        c.bg,
        c.bg2,
        c.surface,
        c.surface2,
        c.text,
        c.muted,
        c.accent,
        c.accent_text,
        c.accent_soft,
        c.icon,
        c.bloom,
        c.on_bloom,
        c.border,
        c.danger,
        c.focus,
        c.toast_bg,
        c.toast_fg
    )
}

fn style_css(s: &Style) -> String {
    let fallback = "system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif";
    let card = if s.shadow {
        "--card-border:transparent;--card-shadow:0 1px 0 var(--border), 0 10px 24px -20px rgb(0 0 0 / 0.45);"
    } else {
        "--card-border:var(--border);--card-shadow:none;"
    };
    format!(
        "--f-body:{body}, {fallback};--f-head:{head}, {fallback};--head-weight:{hw};\
         --f-greet:{greet}, {fallback};--greet-weight:{gw};--greet-size:{gs};\
         --radius:{r};--button-radius:{br};--chip-radius:{cr};{card}",
        body = s.body,
        head = s.head,
        hw = s.head_weight,
        greet = s.greet,
        gw = s.greet_weight,
        gs = s.greet_size,
        r = s.radius,
        br = s.button_radius,
        cr = s.chip_radius,
    )
}

/// Every theme as CSS: `:root[data-theme=…]` for the day,
/// `[data-mode="dark"]` and (in auto) `prefers-color-scheme: dark` for the evening.
pub fn themes_css() -> &'static str {
    static CSS: OnceLock<String> = OnceLock::new();
    CSS.get_or_init(|| {
        let mut out = String::new();
        for p in &PALETTES {
            let n = p.theme.css_name();
            let night = colors_css(&p.night);
            out += &format!(
                ":root[data-theme='{n}']{{{}{}color-scheme:light;}}\n",
                style_css(&p.style),
                colors_css(&p.day)
            );
            out += &format!(":root[data-theme='{n}'][data-mode='dark']{{{night}color-scheme:dark;}}\n");
            out += &format!(
                "@media (prefers-color-scheme: dark){{:root[data-theme='{n}'][data-mode='auto']{{{night}color-scheme:dark;}}}}\n"
            );
        }
        out
    })
}

/// (family, weight, file) of every bundled font: OFL, see assets/fonts/.
const FONTS: [(&str, u16, &[u8]); 12] = [
    (
        "Atkinson Hyperlegible",
        400,
        include_bytes!("../assets/fonts/atkinson-hyperlegible-400.woff2"),
    ),
    (
        "Atkinson Hyperlegible",
        700,
        include_bytes!("../assets/fonts/atkinson-hyperlegible-700.woff2"),
    ),
    (
        "Bricolage Grotesque",
        500,
        include_bytes!("../assets/fonts/bricolage-grotesque-500.woff2"),
    ),
    (
        "Bricolage Grotesque",
        700,
        include_bytes!("../assets/fonts/bricolage-grotesque-700.woff2"),
    ),
    (
        "Instrument Serif",
        400,
        include_bytes!("../assets/fonts/instrument-serif-400.woff2"),
    ),
    (
        "Figtree",
        400,
        include_bytes!("../assets/fonts/figtree-400.woff2"),
    ),
    (
        "Figtree",
        600,
        include_bytes!("../assets/fonts/figtree-600.woff2"),
    ),
    (
        "Outfit",
        500,
        include_bytes!("../assets/fonts/outfit-500.woff2"),
    ),
    (
        "Outfit",
        600,
        include_bytes!("../assets/fonts/outfit-600.woff2"),
    ),
    (
        "Nunito Sans",
        400,
        include_bytes!("../assets/fonts/nunito-sans-400.woff2"),
    ),
    (
        "Nunito Sans",
        600,
        include_bytes!("../assets/fonts/nunito-sans-600.woff2"),
    ),
    (
        "Nunito Sans",
        700,
        include_bytes!("../assets/fonts/nunito-sans-700.woff2"),
    ),
];

/// `@font-face` rules with the fonts inlined (data: URIs): nothing is fetched.
pub fn fonts_css() -> &'static str {
    static CSS: OnceLock<String> = OnceLock::new();
    CSS.get_or_init(|| {
        let b64 = base64::engine::general_purpose::STANDARD;
        FONTS
            .iter()
            .map(|(family, weight, bytes)| {
                format!(
                    "@font-face{{font-family:'{family}';font-weight:{weight};font-style:normal;\
                     font-display:block;src:url(data:font/woff2;base64,{}) format('woff2');}}\n",
                    b64.encode(bytes)
                )
            })
            .collect()
    })
}

/// JS that applies `look` (resolved for `month`) to the page.
pub fn apply_js(look: Look, month: i8) -> String {
    format!(
        "document.documentElement.dataset.theme='{}';document.documentElement.dataset.mode='{}';",
        look.theme.resolve(month).css_name(),
        look.mode.css_name()
    )
}

/// Applies `look` to the page and to the Android status and navigation bars,
/// resolving "Le stagioni" for the current month.
pub fn apply(look: Look) {
    let month = kuracare_core::time::now().month();
    let p = palette(look.theme.resolve(month));
    dioxus::prelude::document::eval(&apply_js(look, month));
    crate::notify::set_bars(argb(p.day.bg), argb(p.night.bg), look.mode.code());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lum(hex: &str) -> f64 {
        let v = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap();
        let ch = |s: u32| {
            let c = f64::from((v >> s) & 0xff) / 255.0;
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * ch(16) + 0.7152 * ch(8) + 0.0722 * ch(0)
    }

    fn contrast(a: &str, b: &str) -> f64 {
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn every_theme_is_readable_day_and_evening() {
        let mut bad = vec![];
        for p in &PALETTES {
            for (mode, c) in [("giorno", &p.day), ("sera", &p.night)] {
                // Text: 4.5 (WCAG AA). Icons, borders of controls, accents on the page: 3.
                let pairs: [(&str, &str, &str, f64); 14] = [
                    ("text/bg", c.text, c.bg, 4.5),
                    ("text/bg2", c.text, c.bg2, 4.5),
                    ("text/surface", c.text, c.surface, 4.5),
                    ("text/surface2", c.text, c.surface2, 4.5),
                    ("text/accent-soft", c.text, c.accent_soft, 4.5),
                    ("muted/bg", c.muted, c.bg, 4.5),
                    ("muted/surface", c.muted, c.surface, 4.5),
                    ("accent-text/accent", c.accent_text, c.accent, 4.5),
                    ("on-bloom/bloom", c.on_bloom, c.bloom, 4.5),
                    ("toast", c.toast_fg, c.toast_bg, 4.5),
                    ("danger/surface", c.danger, c.surface, 4.5),
                    ("accent/bg", c.accent, c.bg, 3.0),
                    ("icon/bg", c.icon, c.bg, 3.0),
                    ("focus/bg", c.focus, c.bg, 3.0),
                ];
                for (name, fg, bg, min) in pairs {
                    let r = contrast(fg, bg);
                    if r < min {
                        bad.push(format!("{} {mode} {name}: {r:.2} < {min}", p.theme.label()));
                    }
                }
            }
        }
        assert!(bad.is_empty(), "\n{}", bad.join("\n"));
    }

    #[test]
    fn seasons_follow_the_calendar() {
        let s = |m| Theme::Seasons.resolve(m);
        assert_eq!(s(2), Theme::Inverno);
        assert_eq!(s(3), Theme::Primavera);
        assert_eq!(s(5), Theme::Primavera);
        assert_eq!(s(6), Theme::Estate);
        assert_eq!(s(9), Theme::Autunno);
        assert_eq!(s(11), Theme::Autunno);
        assert_eq!(s(12), Theme::Inverno);
        assert_eq!(Theme::Salvia.resolve(7), Theme::Salvia);
    }

    #[test]
    fn every_choice_has_a_palette_and_css() {
        for t in Theme::CHOICES {
            let concrete = t.resolve(4);
            assert!(PALETTES.iter().any(|p| p.theme == concrete), "{t:?}");
            assert!(themes_css().contains(&format!("data-theme='{}'", concrete.css_name())));
        }
        assert!(fonts_css().matches("@font-face").count() == FONTS.len());
        assert_eq!(argb("#3f6b5a"), 0xFF3F6B5Au32 as i32);
    }

    #[test]
    fn look_round_trips_and_tolerates_old_files() {
        let l = Look {
            theme: Theme::Estate,
            mode: Mode::Dark,
        };
        let back: Look = serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
        assert_eq!(back, l);
        let partial: Look = serde_json::from_str(r#"{"theme":"salvia"}"#).unwrap();
        assert_eq!(partial.mode, Mode::Auto);
    }
}
