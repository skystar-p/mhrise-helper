use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Armor {
    name: String,
    rarity: isize,
    skills: Vec<Skill>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Skill {
    name: String,
    level: isize,
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

            let mut tds = tds.skip(3);
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

            let armor = Armor {
                name,
                rarity,
                skills,
            };
            println!("{:?}", armor);

            armors.push(armor);
        }
    }

    println!("total armors: {}", armors.len());

    let serialized = serde_json::to_string(&armors)?;
    // save to file
    tokio::fs::write("armors.json", serialized).await?;

    Ok(())
}
