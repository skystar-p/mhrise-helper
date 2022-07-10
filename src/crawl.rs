use crate::types::{Armor, ArmorPart, Deco, PartKeywords, Skill};
use anyhow::bail;

pub async fn crawl_and_parse_armors() -> anyhow::Result<Vec<Armor>> {
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
        let serialized = serde_json::to_string_pretty(&part_keywords)?;
        tokio::fs::write("assets/part_keywords.json", serialized).await?;
    }

    // save to file
    let serialized = serde_json::to_string_pretty(&armors)?;
    tokio::fs::write("assets/armors.json", serialized).await?;

    Ok(armors)
}

pub async fn crawl_and_parse_decos() -> anyhow::Result<Vec<Deco>> {
    let client = reqwest::Client::new();
    let mut decos = Vec::new();
    let response = client
        .get("https://mhrise.kiranico.com/ko/data/decorations")
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
        let mut tds = row.select(&td_selector);

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

        let skill = if let Some(t) = tds.next() {
            let text = t.text().collect::<String>().trim().to_string();
            let (name, level) = text.rsplit_once(" ").unwrap();
            Skill {
                name: name.trim().to_string(),
                level: level.trim_start_matches("Lv").parse::<isize>().unwrap(),
            }
        } else {
            bail!("skills td not found in tr");
        };

        // push to decos
        decos.push(Deco { name, skill });
    }

    // save to file
    let serialized = serde_json::to_string_pretty(&decos)?;
    tokio::fs::write("assets/decos.json", serialized).await?;

    Ok(decos)
}
