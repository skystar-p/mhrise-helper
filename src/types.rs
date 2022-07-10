use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Armor {
    pub name: String,
    pub part: ArmorPart,
    pub rarity: isize,
    pub skills: Vec<Skill>,
    pub slots: Vec<isize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skill {
    pub name: String,
    pub level: isize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ArmorPart {
    Head,
    Body,
    Hands,
    Waist,
    Legs,
    Unknown,
}

impl ArmorPart {
    pub fn from_name(keywords: &PartKeywords, s: &str) -> Self {
        if keywords.head.iter().find(|p| s.contains(&**p)).is_some() {
            return ArmorPart::Head;
        }
        if keywords.body.iter().find(|p| s.contains(&**p)).is_some() {
            return ArmorPart::Body;
        }
        if keywords.hands.iter().find(|p| s.contains(&**p)).is_some() {
            return ArmorPart::Hands;
        }
        if keywords.waist.iter().find(|p| s.contains(&**p)).is_some() {
            return ArmorPart::Waist;
        }
        if keywords.legs.iter().find(|p| s.contains(&**p)).is_some() {
            return ArmorPart::Legs;
        }
        return ArmorPart::Unknown;
    }
}

impl std::fmt::Display for ArmorPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArmorPart::Head => write!(f, "head"),
            ArmorPart::Body => write!(f, "body"),
            ArmorPart::Hands => write!(f, "hands"),
            ArmorPart::Waist => write!(f, "waist"),
            ArmorPart::Legs => write!(f, "legs"),
            ArmorPart::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartKeywords {
    pub head: Vec<String>,
    pub body: Vec<String>,
    pub hands: Vec<String>,
    pub waist: Vec<String>,
    pub legs: Vec<String>,
}

impl PartKeywords {
    pub fn add_keyword(&mut self, part: &ArmorPart, keyword: &str) {
        match part {
            ArmorPart::Head => self.head.push(keyword.to_string()),
            ArmorPart::Body => self.body.push(keyword.to_string()),
            ArmorPart::Hands => self.hands.push(keyword.to_string()),
            ArmorPart::Waist => self.waist.push(keyword.to_string()),
            ArmorPart::Legs => self.legs.push(keyword.to_string()),
            ArmorPart::Unknown => {}
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deco {
    pub name: String,
    pub size: isize,
    pub skill: Skill,
}
