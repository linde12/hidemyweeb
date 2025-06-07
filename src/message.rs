#[derive(Debug)]
pub struct NodeInfo {
    pub id: u32,
    /// Whether this node is a live-stream node.
    pub is_live: bool,
    /// Whether this node is currently running.
    pub running: bool,
}
pub enum Message {
    NodeInfo(NodeInfo),
    /// Node ID removed.
    NodeRemoved(u32),
}
