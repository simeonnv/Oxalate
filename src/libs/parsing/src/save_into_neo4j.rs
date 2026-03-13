use exn::{Result, ResultExt};
use itertools::Itertools;
use neo4rs::{Graph, query};
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to start the neo4j transaction")]
    StartTxn,

    #[error("failed to run the queries in the neo4j transaction")]
    RunQueries,

    #[error("failed to commit the neo4j transaction")]
    Commit,
}

pub async fn save_into_neo4j(
    neo4j_pool: &Graph,
    keywords: &[String],
    url: &Url,
    window_size: usize,
) -> Result<(), Error> {
    let mut rel_data = Vec::new();
    for window in keywords.windows(window_size) {
        for (word_1, word_2) in window.iter().tuple_combinations() {
            let (first, second) = if word_1 < word_2 {
                (word_1, word_2)
            } else {
                (word_2, word_1)
            };
            rel_data.push(vec![first.clone(), second.clone()]);
        }
    }

    let website_word_query = query(
        "
        MERGE (site:Website {url: $url})
        WITH site
        UNWIND $words AS word_text
        MERGE (w:Word {text: word_text})
          ON CREATE SET w.usage = 1
          ON MATCH SET w.usage = coalesce(w.usage, 0) + 1
        MERGE (site)-[r:CONTAINS]->(w)
          ON CREATE SET r.weight = 1
          ON MATCH SET r.weight = r.weight + 1
        ",
    )
    .param("url", url.as_str())
    .param("words", keywords.to_owned());

    let rel_query = query(
        "
        UNWIND $pairs AS pair
        MERGE (w1:Word {text: pair[0]})
        MERGE (w2:Word {text: pair[1]})
        MERGE (w1)-[r:RELATED]->(w2)
          ON CREATE SET r.weight = 1
          ON MATCH SET r.weight = r.weight + 1
        ",
    )
    .param("pairs", rel_data);

    log::info!("Creating neo4j txn");
    let mut txn = neo4j_pool.start_txn().await.or_raise(|| Error::StartTxn)?;

    log::info!("run neo4j rel query");
    txn.run(rel_query).await.or_raise(|| Error::RunQueries)?;

    log::info!("run neo4j word query");
    txn.run(website_word_query)
        .await
        .or_raise(|| Error::RunQueries)?;

    log::info!("run neo4j commit");
    txn.commit().await.or_raise(|| Error::Commit)?;

    Ok(())
}
