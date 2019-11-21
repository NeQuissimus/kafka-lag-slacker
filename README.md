# kafka-lag-slacker

## Environment

- `KAFKA_LAG_EXPORTER`: [Kafka Lag Exporter](https://github.com/lightbend/kafka-lag-exporter) endpoint
- `SLACK_CHANNEL`: Slack channel to post to
- `SLACK_WEBHOOK`: [Slack webhook](https://api.slack.com/incoming-webhooks) endpoint
- `THRESHOLD` Minimum seconds to report


## Debug build

`nix-shell -p cargo -p pkgconfig -p openssl -p rustfmt --command "rustfmt ./src/main.rs; SLACK_WEBHOOK='https://localhost/slack' KAFKA_LAG_EXPORTER='http://localhost:8000/lag' SLACK_CHANNEL='#general' THRESHOLD='30' cargo run"`

## Release build

`nix-build .`
