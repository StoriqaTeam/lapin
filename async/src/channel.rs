pub use amq_protocol::protocol::BasicProperties;

use std::collections::{HashMap, HashSet, VecDeque};

use crate::api::{Answer, ChannelState};
use crate::queue::*;

#[derive(Debug)]
pub struct Channel {
  pub id:             u16,
  pub state:          ChannelState,
  pub send_flow:      bool,
  pub receive_flow:   bool,
  pub queues:         HashMap<String, Queue>,
  pub prefetch_size:  u32,
  pub prefetch_count: u16,
  pub awaiting:       VecDeque<Answer>,
  pub confirm:        bool,
  pub delivery_tag:   u64,
  pub acked:          HashSet<u64>,
  pub nacked:         HashSet<u64>,
  pub unacked:        HashSet<u64>,
}

impl Channel {
  pub fn new(channel_id: u16) -> Channel {
    Channel {
      id:             channel_id,
      state:          ChannelState::Initial,
      send_flow:      true,
      receive_flow:   true,
      queues:         HashMap::new(),
      prefetch_size:  0,
      prefetch_count: 0,
      awaiting:       VecDeque::new(),
      confirm:        false,
      delivery_tag:   1,
      acked:          HashSet::new(),
      nacked:         HashSet::new(),
      unacked:        HashSet::new(),
    }
  }

  pub fn global() -> Channel {
    Channel::new(0)
  }

  pub fn is_connected(&self) -> bool {
    self.state != ChannelState::Initial && self.state != ChannelState::Closed && self.state != ChannelState::Error
  }

  pub fn next_delivery_tag(&mut self) -> u64 {
    let tag = self.delivery_tag;
    self.delivery_tag += 1;
    if self.delivery_tag == 0 {
      self.delivery_tag = 1;
    }
    tag
  }
}
