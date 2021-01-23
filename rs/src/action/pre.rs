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
const LIFE_CX: f64 = (16.0 + 100.0) / 1920.0;
const LIFE_CY: f64 = 1.0 - ((2.0 + 100.0) / 1080.0);
const LIFE_RX: f64 = 100.0 / 1920.0;
const LIFE_RY: f64 = 100.0 / 1080.0;
const LIFE_Y_UNSAFE: f64 = 1.0 - (26.0 / 1080.0);

const MANA_CX: f64 = (1704.0 + 100.0) / 1920.0;
const MANA_CY: f64 = 1.0 - ((2.0 + 100.0) / 1080.0);
//const MANA_RX: f64 = 100.0 / 1920.0;
const MANA_RY: f64 = 100.0 / 1080.0;
const MANA_Y_UNSAFE: f64 = 1.0 - (16.0 / 1080.0);

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

#[derive(Clone, Debug)]
pub struct ScreenPoint {
    x: usize,
    y: usize,
    rgb: (u8, u8, u8),
    distance: i32,
}

pub enum PreCondition {
    ScreenChange { point: ScreenPoint },
    KeyPress { vk: u16 },
}

impl ScreenPoint {
    fn new(x: usize, y: usize) -> Self {
        let rgb = globals::get_cached_color(x, y);
        Self {
            x,
            y,
            rgb,
            distance: 1,
        }
    }

    pub fn new_life(percent: f64, width: usize, height: usize) -> Self {
        let y = LIFE_CY + LIFE_RY * 2.0 * (0.5 - percent);
        if y > LIFE_Y_UNSAFE {
            eprintln!(
                "\x07warning: the life percentage {}% is too low and may not work",
                (percent * 100.0) as usize
            );
        }
        Self::new(
            (width as f64 * LIFE_CX) as usize,
            (height as f64 * y) as usize,
        )
    }

    pub fn new_es(percent: f64, width: usize, height: usize) -> Self {
        // x²/a² + y²/b² = 1
        // x = √(a² * (1 - y²/b²))
        let a = LIFE_RX;
        let b = LIFE_RY;
        let y = b * 2.0 * (0.5 - percent);
        let x = f64::sqrt(a.powi(2) * (1.0 - y.powi(2) / b.powi(2)));

        let x = (width as f64 * (LIFE_CX + x)) as usize;
        let y = (height as f64 * (LIFE_CY + y)) as usize;
        let rgb = globals::get_cached_color(x, y);
        // Only ES needs a threshold because life can be reserved. The colors of everything else
        // must match exactly. It is risky to use the threshold anywhere else because the ground
        // may be close enough (e.g. mana).
        Self {
            x,
            y,
            rgb,
            distance: ES_COLOR_THRESHOLD_SQ,
        }
    }

    pub fn new_mana(percent: f64, width: usize, height: usize) -> Self {
        let y = MANA_CY + MANA_RY * 2.0 * (0.5 - percent);
        if y > MANA_Y_UNSAFE {
            eprintln!(
                "\x07warning: the mana percentage {}% is too low and may not work",
                (percent * 100.0) as usize
            );
        }
        Self::new(
            (width as f64 * MANA_CX) as usize,
            (height as f64 * y) as usize,
        )
    }

    pub fn new_deco1(width: usize, height: usize) -> Self {
        Self::new(
            (width as f64 * DECO_X0) as usize,
            (height as f64 * DECO_Y0) as usize,
        )
    }

    pub fn new_deco2(width: usize, height: usize) -> Self {
        Self::new(
            (width as f64 * DECO_X1) as usize,
            (height as f64 * DECO_Y1) as usize,
        )
    }

    pub fn changed(&self) -> bool {
        let rgb = globals::get_cached_color(self.x, self.y);
        self.rgb != rgb
    }

    fn different(&self) -> bool {
        if self.distance == 1 {
            self.changed()
        } else {
            let rgb = globals::get_cached_color(self.x, self.y);
            (self.rgb.0 as i32 - rgb.0 as i32).pow(2)
                + (self.rgb.1 as i32 - rgb.1 as i32).pow(2)
                + (self.rgb.2 as i32 - rgb.2 as i32).pow(2)
                >= self.distance
        }
    }
}

impl PreCondition {
    pub fn is_valid(&self) -> bool {
        match self {
            Self::ScreenChange { point } => point.different(),
            Self::KeyPress { vk } => win::keyboard::is_down(*vk),
        }
    }
}
