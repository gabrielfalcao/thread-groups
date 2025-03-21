//! Thread Groups is a simple tool for spawing several threads and waiting for all to complete - i.e.: join - at once.
//!
//! It provides the [`ThreadGroup`] struct which does all the job for
//! you so you can wait and enjoy the silence of your life in
//! the real world.

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Display;
use std::thread::{Builder, JoinHandle, Thread};

/// `thread_id` returns a deterministic name for instances of [`std::thread::Thread`].
pub fn thread_id(thread: &Thread) -> String {
    format!(
        "{}:{}",
        std::process::id(),
        thread
            .name()
            .map(|a| a.to_string())
            .unwrap_or_else(|| format!("{:#?}", thread.id()))
            .to_string()
    )
}

/// `ThreadGroup` is allows spawning several threads and waiting for
/// their completion through the specialized methods.
pub struct ThreadGroup<T> {
    id: String,
    handles: VecDeque<JoinHandle<T>>,
    count: usize,
    errors: BTreeMap<String, Error>,
}
impl<T: Send + Sync + 'static> ThreadGroup<T> {
    /// `ThreadGroup::new` creates a new thread group
    pub fn new() -> ThreadGroup<T> {
        ThreadGroup::with_id(thread_id(&std::thread::current()))
    }

    /// `ThreadGroup::with_id` creates a new thread group with a specific id ([`String`])
    pub fn with_id(id: String) -> ThreadGroup<T> {
        ThreadGroup {
            id,
            handles: VecDeque::new(),
            errors: BTreeMap::new(),
            count: 0,
        }
    }

    /// `ThreadGroup::spawn` spawns a thread
    pub fn spawn<F: FnOnce() -> T + Send + 'static>(&mut self, func: F) -> Result<()> {
        self.count += 1;
        let name = format!("{}:{}", &self.id, self.count);
        self.handles.push_back(
            Builder::new().name(name.clone()).spawn(func).map_err(|e| {
                Error::ThreadJoinError(format!("spawning thread {}: {:#?}", name, e))
            })?,
        );
        Ok(())
    }

    /// `ThreadGroup::join` waits for the first thread to join in
    /// blocking fashion, returning the result of that threads
    /// [`FnOnce`]
    pub fn join(&mut self) -> Result<T> {
        let handle = self
            .handles
            .pop_front()
            .ok_or(Error::ThreadGroupError(format!(
                "no threads in group {}",
                &self
            )))?;

        let id = thread_id(&handle.thread());

        let end = match handle.join() {
            Ok(t) => Ok(t),
            Err(e) => {
                let e = Error::ThreadJoinError(format!("joining thread {}: {:#?}", id, e));
                self.errors.insert(id, e.clone());
                Err(e)
            }
        };
        self.count -= 1;
        end
    }

    /// `ThreadGroup::results` waits for the all threads to join in
    /// blocking fashion, returning all their results at once as a [`Vec<Result<T>>`]
    pub fn results(&mut self) -> Vec<Result<T>> {
        let mut val = Vec::<Result<T>>::new();
        while !self.handles.is_empty() {
            val.push(self.join());
        }
        val
    }

    /// `ThreadGroup::as_far_as_ok` waits for the all threads to join in
    /// blocking fashion, returning all the OK results at once as a [`Vec<T>`] but ignoring all errors.
    pub fn as_far_as_ok(&mut self) -> Vec<T> {
        let mut val = Vec::<T>::new();
        while !self.handles.is_empty() {
            if let Ok(g) = self.join() {
                val.push(g)
            }
        }
        val
    }

    /// `ThreadGroup::all_ok` waits for the all threads to join in
    /// blocking fashion, returning all the OK results at once as a [`Vec<T>`] if there are no errors.
    pub fn all_ok(&mut self) -> Result<Vec<T>> {
        let mut val = Vec::<T>::new();
        while !self.handles.is_empty() {
            val.push(self.join()?);
        }
        Ok(val)
    }

    /// `ThreadGroup::errors` returns a [`BTreeMap<String, Error>`] of errors whose keys are thread ids that panicked.
    pub fn errors(&self) -> BTreeMap<String, Error> {
        self.errors.clone()
    }
}

impl<T> std::fmt::Display for ThreadGroup<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}::ThreadGroup {}", module_path!(), &self.id)
    }
}

impl<T: Send + Sync + 'static> Default for ThreadGroup<T> {
    fn default() -> ThreadGroup<T> {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ThreadGroupError(String),
    ThreadJoinError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.prefix().unwrap_or_default(),
            match self {
                Self::ThreadGroupError(s) => format!("{}", s),
                Self::ThreadJoinError(s) => format!("{}", s),
            }
        )
    }
}

impl Error {
    pub fn variant(&self) -> String {
        match self {
            Error::ThreadGroupError(_) => "ThreadGroupError",
            Error::ThreadJoinError(_) => "ThreadJoinError",
        }
        .to_string()
    }

    fn prefix(&self) -> Option<String> {
        match self {
            _ => Some(format!("{}: ", self.variant())),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
