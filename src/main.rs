use std::{collections::HashMap, sync::mpsc, thread};

use streamlistener::Message;
use tokio::process::Command;
use zbus::Connection;

mod notifications;
mod streamlistener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel::<Message>();
    let t_handle = thread::spawn(move || {
        streamlistener::listen(tx);
    });

    let mut nodes: HashMap<u32, streamlistener::VideoInfo> = HashMap::new();

    loop {
        let now = std::time::Instant::now();
        match rx.recv() {
            Ok(msg) => match msg {
                Message::VideoInformation(info) => {
                    // if !info.running {
                    //     nodes.remove(&info.id);
                    // }

                    if let Some(node) = nodes.get_mut(&info.id) {
                        // node.is_live = info.is_live;
                        node.running = info.running;
                    } else {
                        nodes.insert(info.id, info);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }

        // check if any nodes are live
        let any_live_nodes = nodes.values().any(|node| node.is_live && node.running);
        client(any_live_nodes).await?;
        if any_live_nodes {
            println!("Live!");
            Command::new("swww")
                .arg("img")
                .arg("--transition-duration=0")
                .arg("/home/linde/.local/share/wallpapers/totoro.png")
                .spawn()
                .expect("Failed to change bg");
        } else {
            println!("Not live!");
            Command::new("swww")
                .arg("img")
                .arg("--transition-duration=0")
                .arg("/home/linde/.local/share/wallpapers/wp6982689-uzaki-chan-wants-to-hang-out-wallpapers.png")
                .spawn()
                .expect("Failed to change bg");
        }
    }
    // client().await
    t_handle.join().unwrap();

    Ok(())
}

async fn client(flag: bool) -> anyhow::Result<()> {
    let connection = Connection::session().await?;
    let proxy = notifications::NotificationsProxy::new(&connection).await?;
    // let hints = HashMap::new();
    proxy.set_dont_disturb(flag).await?;
    // let reply = proxy
    //     .notify(
    //         "My Crusty App",
    //         0,
    //         "face-smile",
    //         "A notification kek",
    //         "WAOW! A notification",
    //         &[],
    //         hints,
    //         0,
    //     )
    // .await?;

    // println!("Notification ID: {}", reply);

    Ok(())
}
