use crate::message::{Message, NodeInfo};
use pipewire as pw;
use pipewire::node::NodeState;
use pipewire::proxy::{Listener, ProxyListener, ProxyT};
use std::cell::RefCell;
use std::collections::HashMap;
use std::{rc::Rc, u32};
use tokio::sync::mpsc::Sender;

use pipewire::{context::Context, main_loop::MainLoop, node::Node, types::ObjectType};

struct Proxies {
    proxies_t: HashMap<u32, Box<dyn ProxyT>>,
    listeners: HashMap<u32, Vec<Box<dyn Listener>>>,
}

impl Proxies {
    fn new() -> Self {
        Self {
            proxies_t: HashMap::new(),
            listeners: HashMap::new(),
        }
    }

    fn add_proxy_t(&mut self, proxy_t: Box<dyn ProxyT>, listener: Box<dyn Listener>) {
        let proxy_id = {
            let proxy = proxy_t.upcast_ref();
            proxy.id()
        };

        self.proxies_t.insert(proxy_id, proxy_t);

        let v = self.listeners.entry(proxy_id).or_default();
        v.push(listener);
    }

    fn add_proxy_listener(&mut self, proxy_id: u32, listener: ProxyListener) {
        let v = self.listeners.entry(proxy_id).or_default();
        v.push(Box::new(listener));
    }

    fn remove(&mut self, proxy_id: u32) {
        self.proxies_t.remove(&proxy_id);
        self.listeners.remove(&proxy_id);
    }
}

pub fn listen(tx: Sender<Message>) -> anyhow::Result<()> {
    pw::init();

    let mainloop = MainLoop::new(None)?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;
    let registry = Rc::new(core.get_registry()?);
    let registry_weak = Rc::downgrade(&registry);

    let proxies = Rc::new(RefCell::new(Proxies::new()));
    let _reg_listener = registry
        .add_listener_local()
        .global(move |global| {
            if global.type_ != ObjectType::Node {
                return;
            }

            if let Some(registry) = registry_weak.upgrade() {
                if let Some(props) = &global.props {
                    if props.get("media.class") == Some("Video/Source") {
                        // Bind node
                        let node: Node = registry.bind(global).unwrap();
                        let proxy_id = node.upcast_ref().id();

                        let tx_clone = tx.clone();
                        let listener = node
                            .add_listener_local()
                            .info(move |info| {
                                let is_live = info
                                    .props()
                                    .unwrap()
                                    .get("stream.is-live")
                                    .map(|v| v == "true")
                                    .unwrap_or(false);

                                tx_clone
                                    .blocking_send(Message::NodeInfo(NodeInfo {
                                        id: proxy_id,
                                        is_live,
                                        running: matches!(&info.state(), NodeState::Running),
                                    }))
                                    .unwrap();
                            })
                            .register();

                        let node = Box::new(node);
                        let proxy = node.upcast_ref();
                        let listener = Box::new(listener);
                        let proxies_weak = Rc::downgrade(&proxies);
                        let tx_clone = tx.clone();
                        let remove_listener = proxy
                            .add_listener_local()
                            .removed(move || {
                                tx_clone
                                    .blocking_send(Message::NodeRemoved(proxy_id))
                                    .unwrap();
                                if let Some(proxies) = proxies_weak.upgrade() {
                                    proxies.borrow_mut().remove(proxy_id);
                                }
                            })
                            .register();
                        proxies.borrow_mut().add_proxy_t(node, listener);
                        proxies
                            .borrow_mut()
                            .add_proxy_listener(proxy_id, remove_listener);
                    }
                }
            }
        })
        .register();

    mainloop.run();

    Ok(())
}
