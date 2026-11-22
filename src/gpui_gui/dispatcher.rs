//! Schedules GPUI tasks on the host UI thread and an owned background worker.

use std::collections::VecDeque;
use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::{self, JoinHandle, ThreadId};
use std::time::{Duration, Instant};

use gpui::{PlatformDispatcher, Priority, RunnableVariant};

/// Capacity per queue. Overflow stops the dispatcher instead of losing work silently.
const QUEUE_CAPACITY: usize = 1024;
/// Limit foreground work so each pump returns control to the host UI thread.
const PUMP_BUDGET: usize = 64;

pub(super) struct EmbeddedDispatcher {
    state: Arc<DispatcherState>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

struct DispatcherState {
    main_thread: ThreadId,
    stopped: AtomicBool,
    failed: AtomicBool,
    main_queue: Mutex<VecDeque<RunnableVariant>>,
    background_queue: Mutex<VecDeque<WorkItem>>,
    timers: Mutex<VecDeque<(Instant, RunnableVariant)>>,
    worker_wakeup: Condvar,
}

enum WorkItem {
    Runnable(RunnableVariant),
    Realtime(Box<dyn FnOnce() + Send>),
}

impl EmbeddedDispatcher {
    pub(super) fn new() -> io::Result<Arc<Self>> {
        let state = Arc::new(DispatcherState {
            main_thread: thread::current().id(),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            main_queue: Mutex::new(VecDeque::new()),
            background_queue: Mutex::new(VecDeque::new()),
            timers: Mutex::new(VecDeque::new()),
            worker_wakeup: Condvar::new(),
        });
        let worker_state = Arc::clone(&state);
        let worker = thread::Builder::new()
            .name("toybox-gpui-worker".into())
            .spawn(move || worker_state.run_worker())?;
        Ok(Arc::new(Self {
            state,
            worker: Mutex::new(Some(worker)),
        }))
    }

    /// Runs due timers and queued foreground tasks on the host UI thread.
    pub(super) fn pump(&self) {
        if !self.is_main_thread() || self.state.stopped() {
            return;
        }

        let now = Instant::now();
        let ready = self.state.timers.lock().map(|mut timers| {
            let mut ready = Vec::new();
            for _ in 0..timers.len() {
                let (deadline, runnable) = timers.pop_front().expect("one visit per queued timer");
                if deadline <= now {
                    ready.push(runnable);
                } else {
                    timers.push_back((deadline, runnable));
                }
            }
            ready
        });
        let ready = match ready {
            Ok(ready) => ready,
            Err(error) => {
                drop(error);
                self.state.fail();
                return;
            }
        };

        // Transfer the entire due batch or fail, preserving enqueue order.
        if !ready.is_empty() {
            let queued = match self.state.main_queue.lock() {
                Ok(mut queue)
                    if !self.state.stopped() && queue.len() + ready.len() <= QUEUE_CAPACITY =>
                {
                    queue.extend(ready);
                    true
                }
                _ => false,
            };
            if !queued {
                self.state.fail();
                return;
            }
        }

        for _ in 0..PUMP_BUDGET {
            if self.state.stopped() {
                break;
            }
            let runnable = self
                .state
                .main_queue
                .lock()
                .map(|mut queue| queue.pop_front());
            match runnable {
                Ok(Some(runnable)) if !self.state.stopped() => {
                    if panic::catch_unwind(AssertUnwindSafe(|| runnable.run())).is_err() {
                        self.state.fail();
                        break;
                    }
                }
                Ok(_) => break,
                Err(error) => {
                    drop(error);
                    self.state.fail();
                    break;
                }
            }
        }
    }

    pub(super) fn stop(&self) {
        self.state.signal_stop();
        let worker = {
            let mut slot = self.worker.lock().unwrap_or_else(PoisonError::into_inner);
            // A worker callback can request stop. Leave the handle in place
            // so the host thread can join it before unloading the plugin.
            if slot
                .as_ref()
                .is_some_and(|worker| worker.thread().id() != thread::current().id())
            {
                slot.take()
            } else {
                None
            }
        };
        if let Some(worker) = worker {
            let _ = worker.join();
        }

        // Take captures out of their locks before dropping them. A GPUI
        // task destructor may try to enqueue work on this dispatcher.
        self.state.discard(&self.state.main_queue);
        self.state.discard(&self.state.background_queue);
        self.state.discard(&self.state.timers);
    }

    pub(super) fn failed(&self) -> bool {
        self.state.failed.load(Ordering::Acquire)
    }
}

impl DispatcherState {
    fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }

    fn signal_stop(&self) {
        {
            // The worker checks the stop predicate and starts waiting while
            // holding this mutex. Changing it under the same lock prevents
            // a notification from arriving just before the worker waits.
            let _queue = self
                .background_queue
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            self.stopped.store(true, Ordering::Release);
        }
        self.worker_wakeup.notify_all();
    }

    fn fail(&self) {
        self.failed.store(true, Ordering::Release);
        self.signal_stop();
    }

    /// Enqueues one item and releases the lock before dropping rejected work.
    fn enqueue<T>(&self, queue: &Mutex<VecDeque<T>>, item: T) -> bool {
        match queue.lock() {
            Ok(_) if self.stopped() => return false,
            Ok(mut queue) if queue.len() < QUEUE_CAPACITY => {
                queue.push_back(item);
                return true;
            }
            _ => {}
        }
        self.fail();
        false
    }

    fn enqueue_background(&self, item: WorkItem) {
        if self.enqueue(&self.background_queue, item) {
            self.worker_wakeup.notify_one();
        }
    }

    fn discard<T>(&self, queue: &Mutex<VecDeque<T>>) {
        let discarded = {
            let mut queue = queue.lock().unwrap_or_else(|error| {
                self.failed.store(true, Ordering::Release);
                error.into_inner()
            });
            std::mem::take(&mut *queue)
        };
        drop(discarded);
    }

    fn run_worker(&self) {
        while let Some(item) = self.next_work() {
            if self.stopped() {
                break;
            }
            let result = panic::catch_unwind(AssertUnwindSafe(|| match item {
                WorkItem::Runnable(runnable) => {
                    let _ = runnable.run();
                }
                WorkItem::Realtime(callback) => callback(),
            }));
            if result.is_err() {
                self.fail();
                break;
            }
        }
    }

    fn next_work(&self) -> Option<WorkItem> {
        let queue = self.background_queue.lock().and_then(|queue| {
            self.worker_wakeup
                .wait_while(queue, |queue| queue.is_empty() && !self.stopped())
        });
        match queue {
            Ok(mut queue) if !self.stopped() => queue.pop_front(),
            Ok(_) => None,
            Err(error) => {
                drop(error);
                self.fail();
                None
            }
        }
    }
}

impl Drop for EmbeddedDispatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

impl PlatformDispatcher for EmbeddedDispatcher {
    fn is_main_thread(&self) -> bool {
        thread::current().id() == self.state.main_thread
    }

    fn dispatch(&self, runnable: RunnableVariant, _priority: Priority) {
        self.state.enqueue_background(WorkItem::Runnable(runnable));
    }

    fn dispatch_on_main_thread(&self, runnable: RunnableVariant, _priority: Priority) {
        self.state.enqueue(&self.state.main_queue, runnable);
    }

    fn dispatch_after(&self, duration: Duration, runnable: RunnableVariant) {
        if let Some(deadline) = Instant::now().checked_add(duration) {
            self.state.enqueue(&self.state.timers, (deadline, runnable));
        } else {
            self.state.fail();
        }
    }

    fn spawn_realtime(&self, callback: Box<dyn FnOnce() + Send>) {
        self.state.enqueue_background(WorkItem::Realtime(callback));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn idle_workers_stop_without_waiting_for_more_work() {
        let (finished, done) = mpsc::channel();
        let owner = thread::spawn(move || {
            for _ in 0..32 {
                let dispatcher = EmbeddedDispatcher::new().expect("test dispatcher starts");
                dispatcher.stop();
                assert!(!dispatcher.failed());
            }
            finished.send(()).expect("test receiver is alive");
        });
        done.recv_timeout(Duration::from_millis(750))
            .expect("stopping an idle dispatcher wakes its worker");
        owner.join().expect("dispatcher owner finished");
    }

    #[test]
    fn realtime_worker_is_joined_when_dispatcher_stops() {
        let dispatcher = EmbeddedDispatcher::new().expect("test dispatcher starts");
        let (sender, receiver) = mpsc::channel();
        dispatcher.spawn_realtime(Box::new(move || {
            sender.send(()).expect("test receiver is alive");
        }));
        receiver
            .recv_timeout(Duration::from_millis(500))
            .expect("dispatcher worker ran realtime work");
        dispatcher.stop();
        assert!(dispatcher.worker.lock().unwrap().is_none());
        assert!(!dispatcher.failed());
    }

    #[test]
    fn worker_can_request_stop_before_host_joins_it() {
        let dispatcher = EmbeddedDispatcher::new().expect("test dispatcher starts");
        let worker_dispatcher = Arc::clone(&dispatcher);
        let (sender, receiver) = mpsc::channel();
        dispatcher.spawn_realtime(Box::new(move || {
            worker_dispatcher.stop();
            sender.send(()).expect("test receiver is alive");
        }));
        receiver
            .recv_timeout(Duration::from_millis(500))
            .expect("worker returned from stop");
        assert!(dispatcher.worker.lock().unwrap().is_some());
        dispatcher.stop();
        assert!(dispatcher.worker.lock().unwrap().is_none());
        assert!(!dispatcher.failed());
    }
}
