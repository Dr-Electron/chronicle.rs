use clap::{
    load_yaml,
    App,
    ArgMatches,
};
use futures::SinkExt;
use permanode::{
    ConfigCommand,
    SocketMsg,
};
use permanode_broker::application::PermanodeBrokerThrough;
use permanode_common::config::{
    Config,
    MqttType,
};
use scylla::application::ScyllaThrough;
use std::process::Command;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
};
use url::Url;

#[tokio::main]
async fn main() {
    let yaml = load_yaml!("../cli.yaml");
    let app = App::from_yaml(yaml).version(std::env!("CARGO_PKG_VERSION"));
    let matches = app.get_matches();

    match matches.subcommand() {
        ("start", Some(matches)) => {
            // Assume the permanode exe is in the same location as this one
            let current_exe = std::env::current_exe().unwrap();
            let parent_dir = current_exe.parent().unwrap();
            let permanode_exe = if cfg!(target_os = "windows") {
                parent_dir.join("permanode.exe")
            } else {
                parent_dir.join("permanode")
            };
            if permanode_exe.exists() {
                if cfg!(target_os = "windows") {
                    let mut command = Command::new("cmd");
                    command.args(&["/c", "start"]);
                    if matches.is_present("service") {
                        command.arg("/B");
                    }
                    command.arg("powershell");
                    if matches.is_present("noexit") {
                        command.arg("-noexit");
                    }
                    command
                        .arg(permanode_exe.to_str().unwrap())
                        .spawn()
                        .expect("failed to execute process")
                } else {
                    if matches.is_present("service") {
                        Command::new(permanode_exe.to_str().unwrap())
                            .arg("&")
                            .spawn()
                            .expect("failed to execute process")
                    } else {
                        Command::new("bash")
                            .arg(permanode_exe.to_str().unwrap())
                            .spawn()
                            .expect("failed to execute process")
                    }
                };
            } else {
                panic!("No chronicle exe in the current directory: {}", parent_dir.display());
            }
        }
        ("stop", Some(_matches)) => {
            let config = Config::load(None).expect("No config file found for Chronicle!");
            let (mut stream, _) = connect_async(Url::parse(&format!("ws://{}/", config.websocket_address)).unwrap())
                .await
                .unwrap();
            let message =
                Message::text(serde_json::to_string(&SocketMsg::Broker(PermanodeBrokerThrough::ExitProgram)).unwrap());
            stream.send(message).await.unwrap();
        }
        ("rebuild", Some(_matches)) => {
            let config = Config::load(None).expect("No config file found for Chronicle!");
            let (mut stream, _) = connect_async(Url::parse(&format!("ws://{}/", config.websocket_address)).unwrap())
                .await
                .unwrap();
            let message = Message::text(
                serde_json::to_string(&SocketMsg::Scylla(ScyllaThrough::Topology(
                    scylla::application::Topology::BuildRing(1),
                )))
                .unwrap(),
            );
            stream.send(message).await.unwrap();
        }
        ("config", Some(matches)) => {
            let config = Config::load(None).expect("No config file found for Chronicle!");
            if matches.is_present("print") {
                println!("{:#?}", config);
            }
            if matches.is_present("rollback") {
                let (mut stream, _) =
                    connect_async(Url::parse(&format!("ws://{}/", config.websocket_address)).unwrap())
                        .await
                        .unwrap();
                let message =
                    Message::text(serde_json::to_string(&SocketMsg::General(ConfigCommand::Rollback)).unwrap());
                stream.send(message).await.unwrap();
            }
        }
        ("nodes", Some(matches)) => nodes(matches).await,
        ("brokers", Some(matches)) => brokers(matches).await,
        ("archive", Some(matches)) => archive(matches).await,
        _ => (),
    }
}

async fn nodes<'a>(matches: &ArgMatches<'a>) {
    let mut config = Config::load(None).expect("No config file found for Chronicle!");
    let add_address = matches
        .value_of("add")
        .map(|address| address.parse().expect("Invalid address provided!"));
    let rem_address = matches
        .value_of("remove")
        .map(|address| address.parse().expect("Invalid address provided!"));
    if !matches.is_present("skip-connection") {
        let (mut stream, _) = connect_async(Url::parse(&format!("ws://{}/", config.websocket_address)).unwrap())
            .await
            .unwrap();

        if matches.is_present("list") {
            todo!("Print list of nodes");
        }
        if let Some(address) = add_address {
            let message = SocketMsg::Scylla(ScyllaThrough::Topology(scylla::application::Topology::AddNode(address)));
            let message = Message::text(serde_json::to_string(&message).unwrap());
            stream.send(message).await.unwrap();
        }
        if let Some(address) = rem_address {
            let message = SocketMsg::Scylla(ScyllaThrough::Topology(scylla::application::Topology::RemoveNode(
                address,
            )));
            let message = Message::text(serde_json::to_string(&message).unwrap());
            stream.send(message).await.unwrap();
        }
    } else {
        if let Some(address) = add_address {
            config.storage_config.nodes.insert(address);
            config.save(None).expect("Failed to save config!");
        }
        if let Some(address) = rem_address {
            config.storage_config.nodes.remove(&address);
            config.save(None).expect("Failed to save config!");
        }
    }
}

async fn brokers<'a>(matches: &ArgMatches<'a>) {
    let mut config = Config::load(None).expect("No config file found for Chronicle!");
    match matches.subcommand() {
        ("add", Some(subcommand)) => {
            let mqtt_addresses = subcommand
                .values_of("mqtt-address")
                .unwrap()
                .map(|mqtt_address| Url::parse(mqtt_address).unwrap());
            let endpoint_addresses = subcommand.values_of("endpoint-address");
            // TODO add endpoints

            if !matches.is_present("skip-connection") {
                let mut messages = mqtt_addresses.clone().fold(Vec::new(), |mut list, mqtt_address| {
                    list.push(Message::text(
                        serde_json::to_string(&SocketMsg::Broker(PermanodeBrokerThrough::Topology(
                            permanode_broker::application::Topology::AddMqttMessages(mqtt_address.clone()),
                        )))
                        .unwrap(),
                    ));
                    list.push(Message::text(
                        serde_json::to_string(&SocketMsg::Broker(PermanodeBrokerThrough::Topology(
                            permanode_broker::application::Topology::AddMqttMessagesReferenced(mqtt_address),
                        )))
                        .unwrap(),
                    ));
                    list
                });
                let (mut stream, _) =
                    connect_async(Url::parse(&format!("ws://{}/", config.websocket_address)).unwrap())
                        .await
                        .unwrap();
                for message in messages.drain(..) {
                    stream.send(message).await.unwrap();
                }
            } else {
                config
                    .broker_config
                    .mqtt_brokers
                    .get_mut(&MqttType::Messages)
                    .map(|m| m.extend(mqtt_addresses.clone()));
                config
                    .broker_config
                    .mqtt_brokers
                    .get_mut(&MqttType::MessagesReferenced)
                    .map(|m| m.extend(mqtt_addresses));
                config.save(None).expect("Failed to save config!");
            }
        }
        ("remove", Some(subcommand)) => {
            let id = subcommand.value_of("id").unwrap();
            todo!("Send message");
        }
        ("list", Some(subcommand)) => {
            todo!("List brokers");
        }
        _ => (),
    }
}

async fn archive<'a>(matches: &ArgMatches<'a>) {
    let config = Config::load(None).expect("No config file found for Chronicle!");
    match matches.subcommand() {
        ("import", Some(subcommand)) => {
            let dir = subcommand.value_of("directory");
            let range = subcommand.value_of("range");
            todo!("Send message");
        }
        _ => (),
    }
}
