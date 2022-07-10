use std::collections::HashMap;

use itertools::iproduct;
use types::ArmorPart;

use crate::types::{Skill, Skills};

mod crawl;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let armors = if tokio::fs::metadata("assets/armors.json").await.is_err() {
        crawl::crawl_and_parse_armors().await?
    } else {
        let s = tokio::fs::read_to_string("assets/armors.json").await?;
        serde_json::from_str(&s)?
    };

    let decos = if tokio::fs::metadata("assets/decos.json").await.is_err() {
        crawl::crawl_and_parse_decos().await?
    } else {
        let s = tokio::fs::read_to_string("assets/decos.json").await?;
        serde_json::from_str(&s)?
    };

    let wanted_skills = Skills::from_skills(&vec![
        Skill {
            name: "체술".to_string(),
            level: 5,
        },
        Skill {
            name: "스태미나 급속 회복".to_string(),
            level: 3,
        },
        Skill {
            name: "약점 특효".to_string(),
            level: 3,
        },
        Skill {
            name: "슈퍼회심".to_string(),
            level: 2,
        },
    ]);

    // create hashmap from armors, keyed by armor part
    let mut armors_map = HashMap::new();
    for armor in armors {
        if armor.rarity < 7 {
            continue;
        }

        let skills = Skills::from_skills(&armor.skills);
        if !skills.has_intersection(&wanted_skills) {
            continue;
        }

        let av = armors_map.entry(armor.part.clone()).or_insert(Vec::new());
        av.push(armor.clone());
    }

    for (head, body, hands, waist, legs) in iproduct!(
        armors_map.get(&ArmorPart::Head).unwrap(),
        armors_map.get(&ArmorPart::Body).unwrap(),
        armors_map.get(&ArmorPart::Hands).unwrap(),
        armors_map.get(&ArmorPart::Waist).unwrap(),
        armors_map.get(&ArmorPart::Legs).unwrap()
    ) {
        let skills = Skills::new();
        let head_skills = Skills::from_skills(&head.skills);
        let body_skills = Skills::from_skills(&body.skills);
        let hands_skills = Skills::from_skills(&hands.skills);
        let waist_skills = Skills::from_skills(&waist.skills);
        let legs_skills = Skills::from_skills(&legs.skills);

        // merge all
        let skills = skills.merge(&head_skills);
        let skills = skills.merge(&body_skills);
        let skills = skills.merge(&hands_skills);
        let skills = skills.merge(&waist_skills);
        let skills = skills.merge(&legs_skills);

        if skills.is_superset_of(&wanted_skills) {
            println!("{:?}", skills);
            println!(
                "{} {} {} {} {}",
                head.name, body.name, hands.name, waist.name, legs.name
            );
        }
    }

    Ok(())
}
