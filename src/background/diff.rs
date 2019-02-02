use hashbrown::HashSet;
use reqwest::Client;
use std::hash::Hash;
use std::sync::mpsc::{Receiver, SendError, Sender, channel};

use crate::IO;

pub type DiffReceiver<K> = Receiver<(K, bool)>;
pub type DiffSender<K>   = Sender<(K, bool)>;
pub type DiffResult<K>   = Result<(), SendError<(K, bool)>>;

pub trait Diff<K: Clone + Eq + Hash + Send + Sync + 'static> {
    fn new(sender: DiffSender<K>) -> Self;
    fn cache(&self) -> &HashSet<K>;
    fn send(&self, k: K, v: bool) -> DiffResult<K>;
    fn refresh(&self, cli: &Client) -> IO<HashSet<K>>;
    fn update(&mut self, new: HashSet<K>);

    fn build(cli: &Client) -> IO<(Self, DiffReceiver<K>)> where Self: Sized {
        let (sender, receiver) = channel();
        let mut new = Self::new(sender);
        new.update(new.refresh(cli)?);
        Ok((new, receiver))
    }

    fn diff(&mut self, cli: &Client) -> IO<()> {
        let old = self.cache();
        let new = self.refresh(cli)?;
        for added in new.difference(&old) {
            self.send(added.clone(), true)?;
        }
        for deleted in old.difference(&new) {
            self.send(deleted.clone(), false)?;
        }
        self.update(new);
        Ok(())
    }
}
