use serde::{Deserialize, Serialize};

pub const CATEGORIES: [&str; 19] = [
    "components",
    "visualization",
    "model",
    "layout",
    "text",
    "languages",
    "scripting",
    "integration",
    "utility",
    "fun",
    "book",
    "report",
    "paper",
    "thesis",
    "poster",
    "flyer",
    "presentation",
    "cv",
    "office",
];

pub const DISCIPLINES: [&str; 36] = [
    "agriculture",
    "anthropology",
    "archaeology",
    "architecture",
    "biology",
    "business",
    "chemistry",
    "communication",
    "computer-science",
    "design",
    "drawing",
    "economics",
    "education",
    "engineering",
    "fashion",
    "film",
    "geography",
    "geology",
    "history",
    "journalism",
    "law",
    "linguistics",
    "literature",
    "mathematics",
    "medicine",
    "music",
    "painting",
    "philosophy",
    "photography",
    "physics",
    "politics",
    "psychology",
    "sociology",
    "theater",
    "theology",
    "transportation",
];

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub tokens: RegistryTokens,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RegistryTokens {
    pub universe: Option<String>,
}
