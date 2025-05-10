use crate::{PendingSignals, SignalAction, SignalInfo, SignalSet, Signo};
use axtask::WaitQueue;
use core::{
    array,
    ops::{Index, IndexMut},
};
use lock_api::{Mutex, RawMutex};

/// Signal actions for a process.
pub struct SignalActions(pub(crate) [SignalAction; 64]);
impl Default for SignalActions {
    fn default() -> Self {
        Self(array::from_fn(|_| SignalAction::default()))
    }
}
impl Index<Signo> for SignalActions {
    type Output = SignalAction;
    fn index(&self, signo: Signo) -> &SignalAction {
        &self.0[signo as usize - 1]
    }
}
impl IndexMut<Signo> for SignalActions {
    fn index_mut(&mut self, signo: Signo) -> &mut SignalAction {
        &mut self.0[signo as usize - 1]
    }
}

/// Process-level signal manager.
pub struct ProcessSignalManager<M> {
    /// The process-level shared pending signals
    pending: Mutex<M, PendingSignals>,
    /// The signal actions
    pub actions: Mutex<M, SignalActions>,
    /// The wait queue for signal.
    pub(crate) wq: WaitQueue,
    /// The default restorer function.
    pub(crate) default_restorer: usize,
}

impl<M: RawMutex> ProcessSignalManager<M> {
    /// Creates a new process signal manager.
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(PendingSignals::new()),
            actions: Mutex::new(SignalActions::default()),
            wq: WaitQueue::new(),
            default_restorer: 0,
        }
    }

    pub(crate) fn dequeue_signal(&self, mask: &SignalSet) -> Option<SignalInfo> {
        self.pending.lock().dequeue_signal(mask)
    }

    /// Sends a signal to the process.
    ///
    /// See [`ThreadSignalManager::send_signal`] for the thread-level version.
    pub fn send_signal(&self, sig: SignalInfo) {
        self.pending.lock().put_signal(sig);
        self.wq.notify_one(false);
    }

    /// Gets currently pending signals.
    pub fn pending(&self) -> SignalSet {
        self.pending.lock().set
    }
}
