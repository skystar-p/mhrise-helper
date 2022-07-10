mod armor;
mod parse;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let armors = if tokio::fs::metadata("assets/armors.json").await.is_err() {
        parse::crawl_and_parse().await?
    } else {
        let s = tokio::fs::read_to_string("assets/armors.json").await?;
        serde_json::from_str(&s)?
    };

    Ok(())
}
