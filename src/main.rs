extern crate reqwest;

use reqwest::Url;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let slack = env::var("SLACK_WEBHOOK").expect("Please set SLACK_WEBHOOK");
    let slack_url = Url::parse(&slack).expect("SLACK_WEBHOOK must be a valid URL");
    let lag_exporter = env::var("KAFKA_LAG_EXPORTER").expect("Please set KAFKA_LAG_EXPORTER");
    let lag_exporter_url =
        Url::parse(&lag_exporter).expect("KAFKA_LAG_EXPORTER must be a valid URL");

    // Threshold

    println!("{}", slack_url);
    println!("{}", lag_exporter_url);

    let metric = "kafka_consumergroup_group_max_lag";
    let metric_prefix = format!("{}{{", metric);

    let response = reqwest::get(lag_exporter_url)
        .and_then(|mut resp| resp.text())
        .map(|s| {
            s.lines()
                .filter(|s| s.starts_with(&metric_prefix))
                .map(|s| s.trim_start_matches(metric))
                .map(str::to_owned)
                .collect::<Vec<String>>()
        });

    println!("{:#?}", response);

    Ok(())
}
