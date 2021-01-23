use crate::{globals, win};

// Measured in a 1920x1080 screen, life and mana fit in a 205px box.
// The bottom right corners are (16, 2) for life and (1704, 2) for mana.
// There is some decoration near the bottom in both (20px and 15px).
// It doesn't seem to consider the area, only the height to indicate values.
//
// These values start at bottom-left, but we need origin to be in top-left
// which is why we do `1.0 - (...)` for the Y coordinates.
//
// The unsafe zone contains decoration so points below it may not work.
//
// To obtain the `PERCENT_UNSAFE`, consider:
//     y = CY + RY * 2.0 * (0.5 - percent)
//     y > Y_UNSAFE
// Then:
//     CY + RY * 2.0 * (0.5 - percent) = Y_UNSAFE
//     CY + 2RY * (0.5 - percent) = Y_UNSAFE
//     CY + RY - 2RY * percent = Y_UNSAFE
//     -2RY * percent = Y_UNSAFE - CY - RY
//     2RY * percent = CY + RY - Y_UNSAFE
//     percent = (CY + RY - Y_UNSAFE) / 2RY
const LIFE_CX: f64 = (16.0 + 100.0) / 1920.0;
const LIFE_CY: f64 = 1.0 - ((2.0 + 100.0) / 1080.0);
const LIFE_RX: f64 = 100.0 / 1920.0;
const LIFE_RY: f64 = 100.0 / 1080.0;
const LIFE_Y_UNSAFE: f64 = 1.0 - (26.0 / 1080.0);
pub const LIFE_PERCENT_UNSAFE: f64 = (LIFE_CY + LIFE_RY - LIFE_Y_UNSAFE) / (2.0 * LIFE_RY);

const MANA_CX: f64 = (1704.0 + 100.0) / 1920.0;
const MANA_CY: f64 = 1.0 - ((2.0 + 100.0) / 1080.0);
//const MANA_RX: f64 = 100.0 / 1920.0;
const MANA_RY: f64 = 100.0 / 1080.0;
const MANA_Y_UNSAFE: f64 = 1.0 - (16.0 / 1080.0);
pub const MANA_PERCENT_UNSAFE: f64 = (MANA_CY + MANA_RY - MANA_Y_UNSAFE) / (2.0 * MANA_RY);

// There are plenty of places where we can look for decorations,
// but we just pick a few around the bottom-left side of the screen.
const DECO_X0: f64 = 8.0 / 1920.0;
const DECO_Y0: f64 = 1.0 - (130.0 / 1080.0);

const DECO_X1: f64 = 69.0 / 1920.0;
const DECO_Y1: f64 = 1.0 - (44.0 / 1080.0);

// The color distance threshold after which we consider it to have changed.
// Tested on all ES ranges with all life reserved (30 disconnects, 40 doesn't),
// going in and out of town (having no life works fine too).
const ES_COLOR_THRESHOLD_SQ: i32 = 40 * 40;

pub trait Checker {
    fn can_check(&self) -> bool;
    fn life_below(&self, percent: f64) -> bool;
    fn es_below(&self, percent: f64) -> bool;
    fn mana_below(&self, percent: f64) -> bool;
}

pub struct ScreenChecker {
    // Somewhat wasteful, but we don't know the precise points a precondition might ask, so...
    orig_colors: win::screen::Screenshot,
    decorations: [(u8, u8, u8); 2],
}

pub enum PreCondition {
    LifeBelow { percent: f64 },
    EnergyBelow { percent: f64 },
    ManaBelow { percent: f64 },
    KeyPress { vk: u16 },
}

impl ScreenChecker {
    pub fn new() -> Self {
        let orig_colors = globals::last_screenshot().clone();
        Self {
            decorations: [
                orig_colors.color(DECO_X0, DECO_Y0),
                orig_colors.color(DECO_X1, DECO_Y1),
            ],
            orig_colors,
        }
    }
}

impl Checker for ScreenChecker {
    fn can_check(&self) -> bool {
        self.decorations[0] == globals::get_cached_color(DECO_X0, DECO_Y0)
            && self.decorations[1] == globals::get_cached_color(DECO_X1, DECO_Y1)
    }

    fn life_below(&self, percent: f64) -> bool {
        let x = LIFE_CX;
        let y = LIFE_CY + LIFE_RY * 2.0 * (0.5 - percent);

        globals::get_cached_color(x, y) != self.orig_colors.color(x, y)
    }

    fn es_below(&self, percent: f64) -> bool {
        // x²/a² + y²/b² = 1
        // x = √(a² * (1 - y²/b²))
        let a = LIFE_RX;
        let b = LIFE_RY;
        let y = LIFE_CY + (b * 2.0 * (0.5 - percent));
        let x = LIFE_CX + f64::sqrt(a.powi(2) * (1.0 - y.powi(2) / b.powi(2)));

        // Only ES needs a threshold because life can be reserved. The colors of everything else
        // must match exactly. It is risky to use the threshold anywhere else because the ground
        // may be close enough (e.g. mana).
        let rgb = globals::get_cached_color(x, y);
        let last_rgb = self.orig_colors.color(x, y);
        (last_rgb.0 as i32 - rgb.0 as i32).pow(2)
            + (last_rgb.1 as i32 - rgb.1 as i32).pow(2)
            + (last_rgb.2 as i32 - rgb.2 as i32).pow(2)
            >= ES_COLOR_THRESHOLD_SQ
    }

    fn mana_below(&self, percent: f64) -> bool {
        let x = MANA_CX;
        let y = MANA_CY + MANA_RY * 2.0 * (0.5 - percent);
        globals::get_cached_color(x, y) != self.orig_colors.color(x, y)
    }
}

impl PreCondition {
    pub fn is_valid<C: Checker>(&self, checker: &C) -> bool {
        match self {
            Self::LifeBelow { percent } => checker.life_below(*percent),
            Self::EnergyBelow { percent } => checker.es_below(*percent),
            Self::ManaBelow { percent } => checker.mana_below(*percent),
            Self::KeyPress { vk } => win::keyboard::is_down(*vk),
        }
    }
}
