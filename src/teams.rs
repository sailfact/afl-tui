//! Per-club identity: a readable score-highlight colour and a block-art emblem.
//!
//! Logos are described declaratively as [`Emblem`]s (guernsey stripes/bands, a
//! sash, the Swans' V, a sun, an anchor, or a monogram letter) and rasterised by
//! [`crate::ui::logo`]. Dark club colours (navy/black/brown) are nudged brighter
//! so the emblem still reads on a dark terminal background.

/// RGB triple. Kept dependency-free so this module stays pure data.
pub type Rgb = (u8, u8, u8);

// Brightened club colours, tuned for legibility on a dark terminal.
const BLACK: Rgb = (30, 30, 34);
const WHITE: Rgb = (244, 244, 244);
const NAVY: Rgb = (14, 54, 110);
const NAVY_GEEL: Rgb = (18, 60, 120);
const ROYAL: Rgb = (30, 70, 160); // North Melbourne royal blue
const BLUE_WCE: Rgb = (0, 70, 170);
const BLUE_DOG: Rgb = (40, 95, 185);
const RED: Rgb = (226, 25, 55);
const RED_ESS: Rgb = (210, 38, 52);
const RED_STK: Rgb = (237, 27, 47);
const RED_DOG: Rgb = (200, 20, 55);
const MAROON: Rgb = (150, 12, 70);
const GOLD_BL: Rgb = (253, 190, 87);
const GOLD_HAW: Rgb = (251, 191, 21);
const GOLD_WCE: Rgb = (242, 169, 0);
const YELLOW: Rgb = (255, 210, 0); // Richmond / Adelaide / Gold Coast
const ORANGE: Rgb = (244, 121, 32);
const CHARCOAL: Rgb = (52, 52, 51);
const BROWN: Rgb = (108, 55, 22);
const PURPLE: Rgb = (78, 34, 124);
const TEAL: Rgb = (0, 140, 172);

/// How a club's logo tile is drawn. Stripe/band variants fill the whole tile;
/// the rest paint a coloured tile with a foreground motif.
pub enum Emblem {
    /// Vertical guernsey stripes, colours cycled left→right.
    VStripes(&'static [Rgb]),
    /// Horizontal guernsey bands, one per colour top→bottom.
    HBands(&'static [Rgb]),
    /// Diagonal sash (top-right to bottom-left) over a solid tile.
    Sash { bg: Rgb, stripe: Rgb },
    /// The Swans' white V on a red tile.
    Vee { bg: Rgb, fg: Rgb },
    /// Gold Coast sunburst.
    Sun { core: Rgb, ray: Rgb },
    /// Fremantle anchor.
    Anchor { bg: Rgb, fg: Rgb },
    /// A monogram letter centred on a tile.
    Letter { bg: Rgb, fg: Rgb, glyph: Glyph },
}

/// Hand-drawn monogram letters (only the few clubs that need one).
#[derive(Clone, Copy)]
pub enum Glyph {
    C,
    G,
}

pub struct Team {
    pub nickname: &'static str,
    pub name: &'static str,
    /// Bright, readable colour used to highlight this team's score.
    pub score: Rgb,
    pub emblem: Emblem,
}

/// All 18 clubs, keyed for lookup by the API `nickname` (with `name` fallback).
pub const TEAMS: &[Team] = &[
    Team {
        nickname: "Crows",
        name: "Adelaide Crows",
        score: YELLOW,
        emblem: Emblem::HBands(&[NAVY, RED, YELLOW]),
    },
    Team {
        nickname: "Lions",
        name: "Brisbane Lions",
        score: GOLD_BL,
        emblem: Emblem::HBands(&[MAROON, GOLD_BL, BLUE_WCE]),
    },
    Team {
        nickname: "Blues",
        name: "Carlton",
        score: (120, 170, 230),
        emblem: Emblem::Letter {
            bg: NAVY,
            fg: WHITE,
            glyph: Glyph::C,
        },
    },
    Team {
        nickname: "Magpies",
        name: "Collingwood",
        score: WHITE,
        emblem: Emblem::VStripes(&[BLACK, WHITE]),
    },
    Team {
        nickname: "Bombers",
        name: "Essendon",
        score: RED_ESS,
        emblem: Emblem::Sash {
            bg: BLACK,
            stripe: RED_ESS,
        },
    },
    Team {
        nickname: "Dockers",
        name: "Fremantle",
        score: (150, 110, 210),
        emblem: Emblem::Anchor {
            bg: PURPLE,
            fg: WHITE,
        },
    },
    Team {
        nickname: "Cats",
        name: "Geelong Cats",
        score: WHITE,
        emblem: Emblem::VStripes(&[NAVY_GEEL, WHITE]),
    },
    Team {
        nickname: "SUNS",
        name: "Gold Coast SUNS",
        score: YELLOW,
        emblem: Emblem::Sun {
            core: RED,
            ray: YELLOW,
        },
    },
    Team {
        nickname: "GIANTS",
        name: "GWS GIANTS",
        score: ORANGE,
        emblem: Emblem::Letter {
            bg: CHARCOAL,
            fg: ORANGE,
            glyph: Glyph::G,
        },
    },
    Team {
        nickname: "Hawks",
        name: "Hawthorn",
        score: GOLD_HAW,
        emblem: Emblem::VStripes(&[BROWN, GOLD_HAW]),
    },
    Team {
        nickname: "Demons",
        name: "Melbourne",
        score: RED,
        emblem: Emblem::Sash {
            bg: NAVY,
            stripe: RED,
        },
    },
    Team {
        nickname: "Kangaroos",
        name: "North Melbourne",
        score: (90, 140, 220),
        emblem: Emblem::VStripes(&[ROYAL, WHITE]),
    },
    Team {
        nickname: "Power",
        name: "Port Adelaide",
        score: TEAL,
        emblem: Emblem::VStripes(&[TEAL, WHITE, BLACK]),
    },
    Team {
        nickname: "Tigers",
        name: "Richmond",
        score: YELLOW,
        emblem: Emblem::Sash {
            bg: BLACK,
            stripe: YELLOW,
        },
    },
    Team {
        nickname: "Saints",
        name: "St Kilda",
        score: RED_STK,
        emblem: Emblem::HBands(&[RED_STK, WHITE, BLACK]),
    },
    Team {
        nickname: "Swans",
        name: "Sydney Swans",
        score: RED,
        emblem: Emblem::Vee { bg: RED, fg: WHITE },
    },
    Team {
        nickname: "Eagles",
        name: "West Coast Eagles",
        score: GOLD_WCE,
        emblem: Emblem::VStripes(&[BLUE_WCE, GOLD_WCE]),
    },
    Team {
        nickname: "Bulldogs",
        name: "Western Bulldogs",
        score: (70, 130, 220),
        emblem: Emblem::HBands(&[BLUE_DOG, WHITE, RED_DOG]),
    },
];

/// Resolve a club by API `nickname` (exact, case-insensitive), falling back to a
/// case-insensitive match on the full `name`.
pub fn lookup(nickname: &str, name: &str) -> Option<&'static Team> {
    TEAMS
        .iter()
        .find(|t| t.nickname.eq_ignore_ascii_case(nickname))
        .or_else(|| TEAMS.iter().find(|t| t.name.eq_ignore_ascii_case(name)))
}
