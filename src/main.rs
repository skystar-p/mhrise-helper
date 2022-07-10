use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Armor {
    name: String,
    part: ArmorPart,
    rarity: isize,
    skills: Vec<Skill>,
    slots: Vec<isize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Skill {
    name: String,
    level: isize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ArmorPart {
    Head,
    Body,
    Hands,
    Waist,
    Legs,
    Unknown,
}

impl ArmorPart {
    fn from_name(keywords: &PartKeywords, s: &str) -> Self {
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
struct PartKeywords {
    head: Vec<String>,
    body: Vec<String>,
    hands: Vec<String>,
    waist: Vec<String>,
    legs: Vec<String>,
}

impl PartKeywords {
    fn add_keyword(&mut self, part: &ArmorPart, keyword: &str) {
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // read part keywords from file
    let part_keywords = tokio::fs::read_to_string("assets/part_keywords.json").await?;
    let mut part_keywords: PartKeywords = serde_json::from_str(&part_keywords)?;

    let client = reqwest::Client::new();
    let mut armors = Vec::new();
    for rarity in 0..10 {
        let response = client
            .get(format!(
                "https://mhrise.kiranico.com/ko/data/armors?view={}",
                rarity
            ))
            .send()
            .await?;

        let body = response.text().await?;

        let doc = scraper::Html::parse_document(&body);

        // select main table
        let table_selector = scraper::Selector::parse("table[x-data=categoryFilter]").unwrap();
        let table = if let Some(t) = doc.select(&table_selector).next() {
            t
        } else {
            bail!("table not found");
        };

        // select each table row
        let row_selector = scraper::Selector::parse("tr").unwrap();
        let rows = table.select(&row_selector);

        for row in rows {
            // select each td
            let td_selector = scraper::Selector::parse("td").unwrap();
            let tds = row.select(&td_selector);

            let mut tds = tds.skip(2);
            let name = if let Some(t) = tds.next() {
                let a_selector = scraper::Selector::parse("a").unwrap();
                let a = if let Some(a) = t.select(&a_selector).next() {
                    a
                } else {
                    bail!("a not found in name td");
                };
                let s = a.text().collect::<String>();
                s.trim().to_string()
            } else {
                bail!("name td not found in tr");
            };

            let slots = if let Some(t) = tds.next() {
                // this td contains multiple imgs, and each img tag contains information about slot size
                let img_selector = scraper::Selector::parse("img").unwrap();
                let imgs = t.select(&img_selector);
                let mut slots = Vec::new();
                for img in imgs {
                    let src = img.value().attr("src").unwrap_or("");
                    let (_, image_name) = src.rsplit_once("/").unwrap_or(("", ""));
                    let (image_name, _) = image_name.split_once(".").unwrap_or(("", ""));
                    let s = image_name
                        .trim_start_matches("deco")
                        .parse::<isize>()
                        .unwrap();
                    slots.push(s);
                }
                slots.sort_unstable();
                slots
            } else {
                bail!("slots td not found in tr");
            };

            let mut tds = tds.skip(2);
            let skills = if let Some(t) = tds.next() {
                // this td contains multiple divs, each div contains a skill name and level
                let div_selector = scraper::Selector::parse("div").unwrap();
                let divs = t.select(&div_selector);
                let mut skills = Vec::new();
                for div in divs {
                    let div_text = div.text().collect::<String>().trim().to_string();
                    // TODO: handle errors
                    let (name, level) = div_text.rsplit_once(" ").unwrap();
                    skills.push(Skill {
                        name: name.to_string(),
                        level: level.trim_start_matches("Lv").parse::<isize>().unwrap(),
                    });
                }
                skills
            } else {
                bail!("skills td not found in tr");
            };

            let part = ArmorPart::from_name(&part_keywords, &name);
            let armor = Armor {
                name,
                part,
                rarity,
                skills,
                slots,
            };
            println!("{:?}", armor);

            armors.push(armor);
        }
    }

    println!("total armors: {}", armors.len());

    // iterate each unknown parts inquery which part is it
    for armor in armors.as_mut_slice() {
        if armor.part != ArmorPart::Unknown {
            continue;
        }
        let part = ArmorPart::from_name(&part_keywords, &armor.name);
        if part != ArmorPart::Unknown {
            armor.part = part;
            println!("armor {} tagged as {}", armor.name, armor.part);
            continue;
        }
        println!("armor name is {}, which part is it?", armor.name);
        println!("1. head");
        println!("2. body");
        println!("3. hands");
        println!("4. waist");
        println!("5. legs");

        while armor.part == ArmorPart::Unknown {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            let input = input.parse::<usize>().unwrap();
            match input {
                1 => armor.part = ArmorPart::Head,
                2 => armor.part = ArmorPart::Body,
                3 => armor.part = ArmorPart::Hands,
                4 => armor.part = ArmorPart::Waist,
                5 => armor.part = ArmorPart::Legs,
                _ => {
                    println!("invalid input, skipping...");
                    break;
                }
            }
        }

        if armor.part == ArmorPart::Unknown {
            break;
        }

        println!("what keyword does make it as {}?", armor.part);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        let input = input.to_string();
        part_keywords.add_keyword(&armor.part, &input);
        let serialized = serde_json::to_string(&part_keywords)?;
        tokio::fs::write("assets/part_keywords.json", serialized).await?;
    }

    // save to file
    let serialized = serde_json::to_string(&armors)?;
    tokio::fs::write("assets/armors.json", serialized).await?;

    Ok(())
}
