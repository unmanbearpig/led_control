use std::fmt;

struct BarBounds {
    start: char,
    end: char,
}

pub struct TermBarConfig {
    bar_len: usize,
    min: f32,
    max: f32,
    tick_char: char,
    empty_char: char,
    bounds: Option<BarBounds>,
}

pub struct TermBar<'a> {
    config: &'a TermBarConfig,
    val: f32,
}

impl TermBarConfig {
    #[allow(dead_code)]
    pub fn tick_char(mut self, tick_char: char) -> Self {
        self.tick_char = tick_char;
        self
    }

    #[allow(dead_code)]
    pub fn empty_char(mut self, empty_char: char) -> Self {
        self.empty_char = empty_char;
        self
    }

    #[allow(dead_code)]
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    #[allow(dead_code)]
    pub fn bounds(mut self, start: char, end: char) -> Self {
        self.bounds = Some(BarBounds { start, end });
        self
    }

    #[allow(dead_code)]
    pub fn no_bounds(mut self) -> Self {
        self.bounds = None;
        self
    }

    #[allow(dead_code)]
    pub fn len(mut self, len: usize) -> Self {
        self.bar_len = len;
        self
    }

    #[allow(dead_code)]
    pub fn val(&self, val: f32) -> TermBar {
        TermBar { config: self, val }
    }

    /// returns bar length including bounds chars if they're present
    pub fn whole_len(&self) -> usize {
        let mut len = self.bar_len;
        if self.bounds.is_some() {
            len += 2;
        }
        len
    }
}

pub fn config() -> TermBarConfig {
    TermBarConfig::default()
}

impl Default for TermBarConfig {
    fn default() -> Self {
        TermBarConfig {
            bar_len: 40,
            min: 0.0,
            max: 1.0,
            tick_char: '=',
            empty_char: ' ',
            bounds: Some(BarBounds {
                start: '|',
                end: '|',
            }),
        }
    }
}

impl TermBar<'_> {
    fn amount(&self) -> f32 {
        let val = self.val - self.config.min;
        val / (self.config.max - self.config.min)
    }

    fn bar_ticks(&self) -> usize {
        (self.amount() * self.config.bar_len as f32) as usize
    }
}

impl fmt::Display for TermBar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole_len = self.config.whole_len();
        let len = self.config.bar_len;
        let mut buf = String::with_capacity(whole_len);

        if let Some(bounds) = &self.config.bounds {
            buf.push(bounds.start);
        }

        let tick_char = self.config.tick_char;
        let ticks = self.bar_ticks();

        for _ in 0..ticks {
            buf.push(tick_char)
        }

        let empty_char = self.config.empty_char;

        for _ in ticks..len {
            buf.push(empty_char);
        }

        if let Some(bounds) = &self.config.bounds {
            buf.push(bounds.end);
        }

        write!(f, "{}", buf)
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;

    #[test]
    fn test_empty_small_no_bars() {
        let bar = config().len(2).no_bounds();
        assert_eq!(format!("{}", bar.val(0.0)), "  ");
    }

    #[test]
    fn test_empty_small_with_bars() {
        let bar = config().len(2).bounds('<', '>');
        assert_eq!(format!("{}", bar.val(0.0)), "<  >");
    }

    #[test]
    fn test_small_full_no_bars() {
        let bar = config().len(3).no_bounds().tick_char('-');
        assert_eq!(format!("{}", bar.val(1.0)), "---");
    }

    #[test]
    fn test_empty_char_config() {
        let bar = config().len(4).no_bounds().empty_char('-').tick_char('#');
        assert_eq!(format!("{}", bar.val(0.5)), "##--");
    }

    #[test]
    fn test_range() {
        let bar = config().len(4).range(1.0, 3.0);
        assert_eq!(format!("{}", bar.val(2.0)), "|==  |");
    }

    #[test]
    fn test_example() {
        let bar = config().len(40);
        assert_eq!(
            format!("bar: {}", bar.val(0.3)),
            "bar: |============                            |"
        );
    }
}
