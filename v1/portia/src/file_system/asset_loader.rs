use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;

/// Simple FSM for loading assets.
#[derive(PartialEq, Copy, Clone)]
pub enum AssetState {
    // The data is being loaded from the file system and prepared
    Buffering,
    // The data is ready to use
    Ready,
    // The asset has been consumed and can't be used anymore.
    Consumed,
}

/// An asset that may be created from the file system.
pub trait LoadableAsset: Send + Sync + 'static {
    /// Loads an asset from the disk.
    fn asset_from_file(file: &'static str) -> Self;
}

/// A asset file. Attempts to load an and process on a background thread.
pub struct Asset<TAsset>
where
    TAsset: LoadableAsset,
{
    file: &'static str,
    kind: AssetState,
    value: Option<Box<TAsset>>,
    receiver: Option<Receiver<TAsset>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl<TAsset> Drop for Asset<TAsset>
where
    TAsset: LoadableAsset,
{
    fn drop(&mut self) {
        self.consume();
    }
}

impl<TAsset> Asset<TAsset>
where
    TAsset: LoadableAsset,
{
    /// Starts loading the asset in a background thread
    pub fn load(file: &'static str) -> Self {
        // Spawn a new thread and load the file from that
        let (tx, rx): (Sender<TAsset>, Receiver<TAsset>) = mpsc::channel();

        let join_handle = thread::spawn(move || {
            let f = TAsset::asset_from_file(file);
            tx.send(f).unwrap();
        });

        Self {
            file,
            receiver: Some(rx),
            kind: AssetState::Buffering,
            join_handle: Some(join_handle),
            value: None,
        }
    }

    /// The current state of the asset.
    pub fn state(&self) -> AssetState {
        self.kind
    }

    /// The file of the asset
    pub fn file(&self) -> &'static str {
        self.file
    }

    /// Tries to read the asset from disk. Returns the current state after executing.
    pub fn try_receive(&mut self) -> AssetState {
        return self.try_receive_intermediate_asset();
    }

    /// Retrieve a mutable handle to the asset
    pub fn asset_mut<'b>(&mut self) -> Option<&mut TAsset> {
        match self.kind {
            AssetState::Buffering => match self.try_receive() {
                AssetState::Ready => self.asset_mut(),
                _ => None,
            },
            AssetState::Ready => self.value.as_deref_mut(),
            _ => None,
        }
    }

    /// Retrieve a mutable handle to the asset
    pub fn asset<'b>(&self) -> Option<&TAsset> {
        match self.kind {
            AssetState::Ready => self.value.as_deref(),
            _ => None,
        }
    }

    /// Consumes the asset, dropping it from the Asset struct and returning the reference to it. Cleans up all thread operations.
    pub fn consume(&mut self) -> Option<Box<TAsset>> {
        self.kind = AssetState::Consumed;
        let value = self.value.take();
        self.join_handle.take().map(|j| j.join());

        value
    }

    /// Attempts to receive the intermediate asset from the file thread.
    fn try_receive_intermediate_asset(&mut self) -> AssetState {
        if self.kind != AssetState::Buffering {
            return self.kind;
        }

        // Try to receive the intermediate asset
        let result = {
            match &self.receiver {
                Some(rx) => {
                    match rx.try_recv() {
                        Ok(result) => Some(result),
                        Err(e) => {
                            match e {
                                TryRecvError::Empty => {
                                    // Hasn't finished yet, keep going.
                                    None
                                }
                                TryRecvError::Disconnected => {
                                    panic!("Error loading file '{:?}'! Disconnected!", self.file);
                                }
                            }
                        }
                    }
                }
                None => None,
            }
        };

        // If it's ready, switch to prepare state
        match result {
            Some(result) => {
                self.receiver = None;
                self.value = Some(Box::new(result));
                self.kind = AssetState::Ready;
            }
            None => {}
        }

        self.kind
    }
}
