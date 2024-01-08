use futures::{executor::block_on, stream::StreamExt};
use paho_mqtt as mqtt;
use std::{ process, time::Duration};
use log::info;

// The topics to which we subscribe.
const TOPICS: &[&str] = &["central/report"];
const QOS: &[i32] = &[1];

/////////////////////////////////////////////////////////////////////////////

#[tokio::test]
pub async fn test_logger() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    println!("test:{}", "test");
    info!("info! :{}", "test");
    Ok(())
}


#[tokio::test]
pub async fn test() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    // Initialize the logger from the environment
    env_logger::init();

    let create_opts = mqtt::CreateOptionsBuilder::new_v3()
        .server_uri("mqtt://192.168.68.24:1883")
        .client_id("rust_async_subscribe")
        .finalize();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(create_opts)
        .unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    // Get message stream before connecting.
    let mut strm = cli.get_stream(25);

    // Define the set of options for the connection
    let lwt = mqtt::Message::new(
        "test/lwt",
        "[LWT] Async subscriber lost connection",
        mqtt::QOS_1,
    );

    // Create the connect options, explicitly requesting MQTT v3.x
    let conn_opts = mqtt::ConnectOptionsBuilder::new_v3()
        .keep_alive_interval(Duration::from_secs(30))
        .clean_session(false)
        // .will_message(lwt)
        .finalize();

    // Make the connection to the broker
    cli.connect(conn_opts).await.expect("连接失败");

    println!("Subscribing to topics: {:?}", TOPICS);
    cli.subscribe_many(TOPICS, QOS).await?;

    // Just loop on incoming messages.
    println!("Waiting for messages...");

    let mut rconn_attempt: usize = 0;

    while let Some(msg_opt) = strm.next().await {
        if let Some(msg) = msg_opt {
            info!("{}", msg);
        } else {
            // A "None" means we were disconnected. Try to reconnect...
            println!("Lost connection. Attempting reconnect...");
            while let Err(err) = cli.reconnect().await {
                rconn_attempt += 1;
                println!("Error reconnecting #{}: {}", rconn_attempt, err);
                // For tokio use: tokio::time::delay_for()
                // async_std::task::sleep(Duration::from_secs(1)).await;
            }
            println!("Reconnected.");
        }
    }
    Ok(())
}
