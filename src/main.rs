extern crate reqwest;

use regex::Regex;
use reqwest::Url;
use slack_hook::{
    AttachmentBuilder, PayloadBuilder, Slack, SlackText, SlackTextContent, SlackUserLink,
};
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
    let _slack_url = Url::parse(&slack).expect("SLACK_WEBHOOK must be a valid URL"); // Just validation
    let lag_exporter = env::var("KAFKA_LAG_EXPORTER").expect("Please set KAFKA_LAG_EXPORTER");
    let lag_exporter_url =
        Url::parse(&lag_exporter).expect("KAFKA_LAG_EXPORTER must be a valid URL");
    let threshold = env::var("THRESHOLD").expect("Please set THRESHOLD");
    let threshold_u32 = threshold
        .parse::<u32>()
        .expect("THRESHOLD must be a valid integer");
    let channel = env::var("SLACK_CHANNEL")
        .and_then(|c: String| match c.as_str() {
            s if s.starts_with("#") => Ok(s.to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .expect("Please set SLACK_CHANNEL, which must be prefixed with #");

    let by_count = env::var("BY_COUNT").map(|_| true).unwrap_or(false);

    let metric = match by_count {
        x if x => "kafka_consumergroup_group_lag",
        _ => "kafka_consumergroup_group_lag_seconds",
    };

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

    let ms: Vec<slack_hook::Attachment> = metrics?
        .iter()
        .map(|m| {
            let colour = "#ff0000";
            let text = format!("{} / {} / {}: {}", m.group, m.topic, m.partition, m.value);
            AttachmentBuilder::new(text).color(colour).build().unwrap()
        })
        .collect();

    let (ms2, text) = match ms {
        _ if ms.is_empty() => (
            vec![
                AttachmentBuilder::new(format!(":party-porg: - No lag above {}", threshold_u32))
                    .color("#00FF00")
                    .build()
                    .unwrap(),
            ],
            vec![Text("Kafka Lag".into())],
        ),
        x => (
            x,
            vec![Text("Kafka Lag".into()), User(SlackUserLink::new("!here"))],
        ),
    };

    use SlackTextContent::{Text, User};

    let slack = Slack::new(slack.as_str()).unwrap();

    let payload = PayloadBuilder::new()
        .text(SlackText::from(&text[..]))
        .icon_url("https://www.biography.com/.image/ar_1:1%2Cc_fill%2Ccs_srgb%2Cg_face%2Cq_auto:good%2Cw_300/MTIwNjA4NjMzODYxOTMyNTU2/franz-kafka-9359401-1-402.jpg")
        .channel(channel)
        .username("Franz Kafka")
        .attachments(ms2)
        .build()?;

    slack
        .send(&payload)
        .map_err(|_e| panic!("Could not send Slack message"))
}
