pub mod transformations;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SegmentId {
    Top = 0,
    TopRight = 1,
    BottomRight = 2,
    Bottom = 3,
    BottomLeft = 4,
    TopLeft = 5,
    Middle = 6,
}

#[derive(Clone, Copy, Debug)]
pub struct Flow {
    pub from: SegmentId,
    pub to: SegmentId,
    pub share: f32, // we can just set this to 1.0 for now if you don’t care
}

#[derive(Clone, Debug)]
pub struct TransitionSpec {
    pub from_digit: Digit,
    pub to_digit: Digit,
    pub flows: Vec<Flow>, // “this segment sends goo there”
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Digit {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
}

impl Digit {
    /// 7-seg bitmask, bits 0..6 correspond to SegmentId 0..6
    pub const fn mask(self) -> u8 {
        match self {
            // evil bitmagic lmao
            Digit::Zero => 0b0111111,
            Digit::One => 0b0000110,
            Digit::Two => 0b1011011,
            Digit::Three => 0b1001111,
            Digit::Four => 0b1100110,
            Digit::Five => 0b1101101,
            Digit::Six => 0b1111101,
            Digit::Seven => 0b0000111,
            Digit::Eight => 0b1111111,
            Digit::Nine => 0b1101111,
        }
    }

    pub fn active_segments(self) -> impl Iterator<Item = SegmentId> {
        let mask = self.mask();
        (0u8..7).filter_map(move |i| {
            if (mask & (1 << i)) != 0 {
                Some(match i {
                    0 => SegmentId::Top,
                    1 => SegmentId::TopRight,
                    2 => SegmentId::BottomRight,
                    3 => SegmentId::Bottom,
                    4 => SegmentId::BottomLeft,
                    5 => SegmentId::TopLeft,
                    6 => SegmentId::Middle,
                    _ => unreachable!(),
                })
            } else {
                None
            }
        })
    }
}
