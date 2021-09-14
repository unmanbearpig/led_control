use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Tag(String);

const RGB_EMOJI: &str = "🌈";
const WINDOW_EMOJI: &str = "🌇";

const RED_EMOJI: &str = "🔴";
const GREEN_EMOJI: &str = "🟢";
const BLUE_EMOJI: &str = "🔵";
const WHITE_EMOJI: &str = "⚪";

const WALL_EMOJI: &str = "🧱";
const WALL_TOP_EMOJI: &str = "📈";
const WALL_BOTTOM_EMOJI: &str = "📉";
const SCORPION_EMOJI: &str = "🦂";
const DOOR_EMOJI: &str = "🚪";
const BED_EMOJI: &str = "🛏️";

const CEILING_EMOJI: &str = "🌌";

const UNUSED_EMOJI: &str = "🔧";

impl Tag {
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        let name = name.as_ref();
        Tag(name.to_string())
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn humanized(&self) -> &str {
        match self.as_ref() {
            "r" => RED_EMOJI,
            "red" => RED_EMOJI,
            "g" => GREEN_EMOJI,
            "green" => GREEN_EMOJI,
            "b" => BLUE_EMOJI,
            "blue" => BLUE_EMOJI,
            "w" => WHITE_EMOJI,
            "white" => WHITE_EMOJI,
            "rgb" => RGB_EMOJI,
            "window" => WINDOW_EMOJI,
            "top" => WALL_TOP_EMOJI,
            "bottom" => WALL_BOTTOM_EMOJI,
            "wall" => WALL_EMOJI,
            "scorpion" => SCORPION_EMOJI,
            "bed" => BED_EMOJI,
            "door" => DOOR_EMOJI,
            "ceiling" => CEILING_EMOJI,
            "unused" => UNUSED_EMOJI,
            other => other,
        }
    }
}

pub fn tags_to_str(tags: &[Tag]) -> String {
    let mut out = String::new();
    for t in tags.iter().rev() {
        out += format!("{} ", t).as_ref();
    }
    out
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }

}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.humanized())
    }
}
