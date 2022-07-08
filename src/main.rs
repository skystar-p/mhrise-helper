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

const HEAD_NAME_KEYWORDS: [&str; 14] = [
    "머리",
    "헬름",
    "복면",
    "상투",
    "가면",
    "후드",
    "이어링",
    "귀걸이",
    "두건",
    "헤드",
    "페이크",
    "갑주",
    "깃털장식",
    "크라운",
];

const BODY_NAME_KEYWORDS: [&str; 6] = ["상의", "재킷", "베스트", "메일", "갑옷", "슈트"];

const HAND_NAME_KEYWORDS: [&str; 5] = ["암", "장갑", "손목", "글러브", "의팔"];

const WAIST_NAME_KEYWORDS: [&str; 3] = ["벨트", "코일", "허리"];

const LEG_NAME_KEYWORDS: [&str; 5] = ["그리브", "풋", "다리", "각", "팬츠"];

impl ArmorPart {
    fn from_name(s: &str) -> Self {
        if HEAD_NAME_KEYWORDS.iter().any(|&kw| s.contains(kw)) {
            ArmorPart::Head
        } else if BODY_NAME_KEYWORDS.iter().any(|&kw| s.contains(kw)) {
            ArmorPart::Body
        } else if HAND_NAME_KEYWORDS.iter().any(|&kw| s.contains(kw)) {
            ArmorPart::Hands
        } else if WAIST_NAME_KEYWORDS.iter().any(|&kw| s.contains(kw)) {
            ArmorPart::Waist
        } else if LEG_NAME_KEYWORDS.iter().any(|&kw| s.contains(kw)) {
            ArmorPart::Legs
        } else {
            ArmorPart::Unknown
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

            let part = ArmorPart::from_name(&name);
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

    // print unknown part armors
    let unknowns = armors
        .iter()
        .filter(|a| a.part == ArmorPart::Unknown)
        .collect::<Vec<_>>();

    let serialized = serde_json::to_string(&armors)?;
    // save to file
    tokio::fs::write("armors.json", serialized).await?;

    let serialized = serde_json::to_string(&unknowns)?;
    // save to file
    tokio::fs::write("unknowns.json", serialized).await?;

    Ok(())
}
