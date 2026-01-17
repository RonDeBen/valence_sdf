//! 7-segment digit representation and segment logic.

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
    /// 7-segment bitmask, bits 0..6 correspond to SegmentId 0..6
    pub const fn mask(self) -> u8 {
        match self {
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

    /// Convert a 7-segment mask back to a Digit
    pub fn from_mask(mask: u8) -> Option<Self> {
        match mask {
            0b0111111 => Some(Digit::Zero),
            0b0000110 => Some(Digit::One),
            0b1011011 => Some(Digit::Two),
            0b1001111 => Some(Digit::Three),
            0b1100110 => Some(Digit::Four),
            0b1101101 => Some(Digit::Five),
            0b1111101 => Some(Digit::Six),
            0b0000111 => Some(Digit::Seven),
            0b1111111 => Some(Digit::Eight),
            0b1101111 => Some(Digit::Nine),
            _ => None,
        }
    }

    /// Convert digit to its numeric value (0-9)
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    // /// Convert a numeric value to a Digit
    // pub fn from_u8(n: u8) -> Option<Self> {
    //     match n {
    //         0 => Some(Digit::Zero),
    //         1 => Some(Digit::One),
    //         2 => Some(Digit::Two),
    //         3 => Some(Digit::Three),
    //         4 => Some(Digit::Four),
    //         5 => Some(Digit::Five),
    //         6 => Some(Digit::Six),
    //         7 => Some(Digit::Seven),
    //         8 => Some(Digit::Eight),
    //         9 => Some(Digit::Nine),
    //         _ => None,
    //     }
    // }
    //
    // /// Get an iterator over the active segments for this digit
    // pub fn active_segments(self) -> impl Iterator<Item = SegmentId> {
    //     let mask = self.mask();
    //     (0u8..7).filter_map(move |i| {
    //         if (mask & (1 << i)) != 0 {
    //             Some(match i {
    //                 0 => SegmentId::Top,
    //                 1 => SegmentId::TopRight,
    //                 2 => SegmentId::BottomRight,
    //                 3 => SegmentId::Bottom,
    //                 4 => SegmentId::BottomLeft,
    //                 5 => SegmentId::TopLeft,
    //                 6 => SegmentId::Middle,
    //                 _ => unreachable!(),
    //             })
    //         } else {
    //             None
    //         }
    //     })
    // }
}
