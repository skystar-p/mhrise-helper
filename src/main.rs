use std::collections::HashMap;

use itertools::{iproduct, Itertools};
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

    let wanted_skills = Skills::from_skill_vec(&vec![
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

        let skills = Skills::from_skill_vec(&armor.skills);
        if !skills.has_intersection(&wanted_skills) {
            continue;
        }

        let av = armors_map.entry(armor.part.clone()).or_insert(Vec::new());
        av.push(armor.clone());
    }

    // filter available deco
    let decos = decos
        .into_iter()
        .filter(|deco| wanted_skills.has_skill(&deco.skill.name))
        .sorted_by(|a, b| b.size.cmp(&a.size))
        .collect::<Vec<_>>();

    for (head, body, hands, waist, legs) in iproduct!(
        armors_map.get(&ArmorPart::Head).unwrap(),
        armors_map.get(&ArmorPart::Body).unwrap(),
        armors_map.get(&ArmorPart::Hands).unwrap(),
        armors_map.get(&ArmorPart::Waist).unwrap(),
        armors_map.get(&ArmorPart::Legs).unwrap()
    ) {
        let mut skills = Skills::new();
        let head_skills = Skills::from_skill_vec(&head.skills);
        let body_skills = Skills::from_skill_vec(&body.skills);
        let hands_skills = Skills::from_skill_vec(&hands.skills);
        let waist_skills = Skills::from_skill_vec(&waist.skills);
        let legs_skills = Skills::from_skill_vec(&legs.skills);

        // merge all
        let skills = skills
            .merge(&head_skills)
            .merge(&body_skills)
            .merge(&hands_skills)
            .merge(&waist_skills)
            .merge(&legs_skills);

        if skills.is_superset_of(&wanted_skills) {
            println!("{:?}", skills);
            println!(
                "{} {} {} {} {}",
                head.name, body.name, hands.name, waist.name, legs.name
            );
        } else {
            let mut wanted_skills = wanted_skills.clone();
            wanted_skills.subtract(&skills);
        }
    }

    Ok(())
}
