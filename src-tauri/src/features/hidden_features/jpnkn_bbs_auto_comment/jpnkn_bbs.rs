use std::time::Duration;

use anyhow::{anyhow, Result};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;
use tokio::{spawn, sync::mpsc};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};

#[derive(Deserialize)]
struct JpnknBbsMqttItem {
    body: String,
    #[allow(unused)]
    no: String,
    #[allow(unused)]
    bbsid: String,
    #[allow(unused)]
    threadkey: String,
}

struct JpnknBbsMqttItemBody {
    #[allow(unused)]
    name: String,
    #[allow(unused)]
    email: String,
    #[allow(unused)]
    date: String,
    message: String,
    _unknown: String,
}

impl JpnknBbsMqttItem {
    pub fn body(&self) -> JpnknBbsMqttItemBody {
        let data = self.body.split("<>").collect::<Vec<_>>();
        JpnknBbsMqttItemBody {
            name: data[0].into(),
            email: data[1].into(),
            date: data[2].into(),
            message: data[3].into(),
            _unknown: data[4].into(),
        }
    }
}

fn mqtt_stream(board_name: &str) -> impl Stream<Item = String> {
    let (tx, rx) = mpsc::channel::<String>(1);

    let topic = format!("bbs/{}", board_name);
    spawn(async move {
        let mut mqtt_options = MqttOptions::new("rumqtt-async", "a.mq.jpnkn.com", 1884);
        mqtt_options.set_keep_alive(Duration::from_secs(5));
        mqtt_options.set_credentials("genkai", "7144");
        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
        client.subscribe(topic, QoS::AtMostOnce).await.unwrap();

        while let Ok(event) = eventloop.poll().await {
            let Event::Incoming(packet) = event else {
                continue;
            };
            let Packet::Publish(publish) = packet else {
                if let Packet::PingResp = packet {
                    continue;
                }
                continue;
            };
            tx.send(String::from_utf8_lossy(&publish.payload).to_string())
                .await
                .unwrap();
        }
    });

    ReceiverStream::new(rx)
}

pub fn jpnkn_new_message_stream(
    board_name: impl Into<String>,
) -> impl Stream<Item = Result<String>> {
    let (tx, rx) = mpsc::channel::<Result<String>>(1);

    let board_name = board_name.into();
    spawn(async move {
        let mut stream = mqtt_stream(&board_name);
        while let Some(item) = stream.next().await {
            let json = serde_json::from_str::<JpnknBbsMqttItem>(&item);
            let json = match json {
                Err(err) => {
                    tx.send(Err(anyhow!("{}", err))).await.unwrap();
                    continue;
                }
                Ok(ok) => ok,
            };
            tx.send(Ok(json.body().message)).await.unwrap();
        }
    });

    ReceiverStream::new(rx)
}
