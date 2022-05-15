use std::fmt;

pub struct BarBounds {
    pub start: char,
    pub end: char,
}

pub struct TermBarConfig {
    pub bar_len: usize,
    pub min: f32,
    pub max: f32,
    pub tick_char: char,
    pub empty_char: char,
    pub bounds: Option<BarBounds>,

    /// If Some, then print the value after the bar with this number of digits
    pub print_val_digits: Option<usize>,
}

pub struct TermBar<'a> {
    config: &'a TermBarConfig,
    val: f32,
}

impl TermBarConfig {
    pub const fn default_config() -> Self {
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
            print_val_digits: None,
        }
    }

    // Configuration

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
    pub fn print_val_digits(mut self, digits: usize) -> Self {
        self.print_val_digits = Some(digits);
        self
    }

    #[allow(dead_code)]
    pub fn no_print_val_digits(mut self) -> Self {
        self.print_val_digits = None;
        self
    }

    // value

    #[allow(dead_code)]
    pub fn val(&self, val: f32) -> TermBar {
        TermBar { config: self, val }
    }

    /// Returns bar length including bounds chars if they're present
    pub fn whole_len(&self) -> usize {
        let mut len = self.bar_len;
        if self.bounds.is_some() {
            len += 2;
        }
        len
    }
}

pub const fn config() -> TermBarConfig {
    TermBarConfig::default_config()
}

impl Default for TermBarConfig {
    fn default() -> Self {
        TermBarConfig::default_config()
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

    pub fn print(&self) {
        println!("{}", self)
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

        if let Some(digits) = self.config.print_val_digits {
            buf.push_str(format!(" {1:.0$}", digits, self.val).as_ref());
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
