use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use warp::filters::ws::Ws;
use crate::app::{PusherQuery, Pusher, generate_socket_id, ServerEvent, ConnectionInfo, Subscription, CustomEvent, Result};
use crate::app::ClientEvent::{ChannelEvent, Subscribe, Unsubscribe, Ping};

pub(crate) async fn ws(pusher: Pusher, ws: Ws) -> Result<impl warp::Reply> {
    Ok(ws.on_upgrade(|w| async move {
        let (mut tx, mut rx) = w.split();
        let (response_tx, mut response_rx) = mpsc::channel::<ServerEvent>(1024);

        let socket_id = generate_socket_id();

        let response_stream = async {
            while let Some(event) = response_rx.recv().await {
                let msg = serde_json::to_string(&event).unwrap();
                if let Err(err) = tx.send(warp::filters::ws::Message::text(msg)).await {
                    eprintln!("Send error: {}", err);
                    break;
                }
            }
        };

        let connection_established = ServerEvent::ConnectionEstablished {
            data: ConnectionInfo {
                socket_id: socket_id.clone(),
                activity_timeout: 120,
            },
        };

        if let Err(err) = response_tx.send(connection_established).await {
            eprintln!("Failed handshake: {}", err);
            return ();
        }

        let reader = async {
            while let Some(payload) = rx.next().await {
                let msg = match payload.ok().and_then(|msg| msg.to_str().map(|s| s.to_owned()).ok()) {
                    Some(msg) => msg,
                    None => {
                        eprintln!("invalid websocket payload");
                        break;
                    }
                };

                match serde_json::from_str(&msg) {
                    Ok(Subscribe {
                        ref channel,
                        channel_data,
                        ..
                    }) => {
                        async {
                            let mut channels = pusher.channels.write().await;
                            let channel =
                                channels.entry(channel.to_owned()).or_insert(channel.to_owned().into());
                            channel.add_subscription(
                                &socket_id,
                                Subscription {
                                    sender: response_tx.clone(),
                                    data: channel_data,
                                    user_id: None,
                                },
                            );
                        }
                            .await;

                        let success = ServerEvent::SubscriptionSucceeded {
                            channel: channel.to_owned(),
                            data: None,
                        };

                        if let Err(err) = response_tx.send(success).await {
                            eprintln!("Failed subscribe: {}", err);
                        }
                    }

                    Ok(Unsubscribe { ref channel }) => {
                        let mut channels = pusher.channels.write().await;
                        if let Some(channel) = channels.get_mut(channel) {
                            channel.remove_subscription(&socket_id);
                        } else {
                            response_tx
                                .send(ServerEvent::Error {
                                    message: format!(
                                        "No current subscription to channel {}, or subscription in progress",
                                        channel
                                    ),
                                    code: None,
                                })
                                .await
                                .unwrap();
                        }
                    }

                    Ok(Ping) => {
                        response_tx.send(ServerEvent::Pong).await.unwrap();
                    }

                    Ok(ChannelEvent {
                        event,
                        ref channel,
                        data,
                    }) => {
                        let event = ServerEvent::ChannelEvent(CustomEvent {
                            event,
                            channel: channel.to_owned(),
                            data,
                            user_id: None,
                        });

                        let channels = pusher.channels.read().await;
                        if let Some(channel) = channels.get(channel) {
                            channel.publish(event).await.unwrap();
                        } else {
                            eprintln!("Channel not found: {}", channel);
                        }
                    }

                    Err(err) => {
                        eprintln!("Invalid message: {}", err);
                        continue;
                    }
                }
            }
        };

        tokio::select! {
                    _ = response_stream => {
                        eprintln!("Response finished");
                    },
                    _ = reader => {
                        eprintln!("Reader finished");
                    },
                }
        ;

        let mut channels = pusher.channels.write().await;
        for (_, channel) in channels.iter_mut() {
            channel.remove_subscription(&socket_id);
        }

        eprintln!("client disconnected");
    }))
}
