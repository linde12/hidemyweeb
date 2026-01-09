use rand::seq::IndexedRandom;
mod message;
use std::collections::HashMap;

use message::{Message, NodeInfo};
use tokio::{process::Command, select, sync::mpsc};
use zbus::Connection;

mod config;
mod dconf;
mod notifications;
mod streamlistener;
mod tray;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::Config::new()?;
    let mut tray = tray::Tray::new();
    let (tx, mut rx) = mpsc::channel::<Message>(1);
    let t_handle = std::thread::spawn(move || {
        streamlistener::listen(tx.clone()).unwrap();
    });

    let mut nodes: HashMap<u32, NodeInfo> = HashMap::new();
    let mut debounce_deadline = None;

    loop {
        select! {
            msg = rx.recv() => {
                match msg {
                    Some(Message::NodeInfo(info)) => {
                        if let Some(node) = nodes.get_mut(&info.id) {
                             // we never unset is_live, it is not sent along with all NodeInfo messages
                             // as long as we've gotten it once and the node is running we count it as live
                            if info.is_live && info.running {
                                node.is_live = true;
                            }
                            node.running = info.running;
                        } else {
                            nodes.insert(info.id, info);
                        }
                        debounce_deadline = Some(tokio::time::Instant::now() + tokio::time::Duration::from_millis(500));
                    },
                    Some(Message::NodeRemoved(id)) => {
                        nodes.remove(&id);
                    },
                    None => {
                        eprintln!("Error receiving message");
                        break;
                    }
                }
            }

            Some(_) = sleep(debounce_deadline) => {
                debounce_deadline = None;
                // check if any nodes are live
                let any_live_nodes = nodes.values().any(|node| node.running && node.is_live);
                set_dnd(any_live_nodes).await;
                set_wallpaper(&mut tray, any_live_nodes, &cfg);
            }
        }
    }
    t_handle.join().unwrap();

    Ok(())
}

async fn sleep(t: Option<tokio::time::Instant>) -> Option<()> {
    match t {
        Some(timer) => Some(tokio::time::sleep_until(timer).await),
        None => None,
    }
}

async fn set_dnd(flag: bool) {
    if let Err(e) = dconf::set_bool("/io/astal/notifd/dont-disturb", flag) {
        eprintln!("Failed to set dconf DND: {}", e);
    }
}

fn set_wallpaper(tray: &mut tray::Tray, any_live_nodes: bool, cfg: &config::Config) {
    if any_live_nodes {
        tray.set_icon(tray::IconType::Recording);
        if let Some(wall) = cfg.wallpaper_whitelist.choose(&mut rand::rng()) {
            Command::new("swww")
                .arg("img")
                .arg("--transition-type=none")
                .arg(cfg.wallpaper_directory.join(wall))
                .spawn()
                .expect("Failed to change bg");
        }
    } else {
        tray.set_icon(tray::IconType::Idle);
    }
}
