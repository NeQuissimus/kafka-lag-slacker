extern crate reqwest;

use regex::Regex;
use reqwest::Url;
use std::env;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Metric {
    group: String,
    topic: String,
    partition: u32,
    value: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let group_regex = Regex::new(".*group=\"([^\"]+)\".*").unwrap();
    let topic_regex = Regex::new(".*topic=\"([^\"]+)\".*").unwrap();
    let partition_regex = Regex::new(".*partition=\"([0-9]+)\".*").unwrap();

    let slack = env::var("SLACK_WEBHOOK").expect("Please set SLACK_WEBHOOK");
    let slack_url = Url::parse(&slack).expect("SLACK_WEBHOOK must be a valid URL");
    let lag_exporter = env::var("KAFKA_LAG_EXPORTER").expect("Please set KAFKA_LAG_EXPORTER");
    let lag_exporter_url =
        Url::parse(&lag_exporter).expect("KAFKA_LAG_EXPORTER must be a valid URL");
    let threshold = env::var("THRESHOLD").expect("Please set THRESHOLD");
    let threshold_u32 = threshold
        .parse::<u32>()
        .expect("THRESHOLD must be a valid integer");

    let metric = "kafka_consumergroup_group_lag_seconds";
    let metric_prefix = format!("{}{{", metric);

    let metrics = reqwest::get(lag_exporter_url)
        .and_then(|mut resp| resp.text())
        .map(|s| {
            let mut ms = s
                .lines()
                .filter(|s| s.starts_with(&metric_prefix))
                .filter(|s| !s.ends_with("NaN"))
                .filter(|s| s.contains(" "))
                .map(|s| s.trim_start_matches(metric).trim())
                .map(|s| {
                    let idx = s.rfind(" ");
                    idx.and_then(|i| {
                        let (m, v) = s.split_at(i);
                        let vf = v.trim().parse::<f32>().ok().map(|f| f as u32);

                        let group = group_regex
                            .captures(m)
                            .and_then(|c| c.get(1))
                            .map(|c| c.as_str().to_string());
                        let topic = topic_regex
                            .captures(m)
                            .and_then(|c| c.get(1))
                            .map(|c| c.as_str().to_string());
                        let partition = partition_regex
                            .captures(m)
                            .and_then(|c| c.get(1))
                            .and_then(|c| c.as_str().parse::<u32>().ok());

                        group.and_then(|g| {
                            topic.and_then(|t| {
                                partition.and_then(|p| {
                                    vf.map(|f| Metric {
                                        group: g,
                                        topic: t,
                                        partition: p,
                                        value: f,
                                    })
                                })
                            })
                        })
                    })
                })
                .into_iter()
                .flatten()
                .filter(|m| m.value >= threshold_u32)
                .collect::<Vec<Metric>>();

            ms.sort();
            ms.to_vec()
        });

    println!("{:#?}", metrics);

    // Format for Slack
    // Send to Slack

    Ok(())
}
