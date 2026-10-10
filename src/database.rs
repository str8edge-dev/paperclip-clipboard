use crate::{clipboard_object::ClipboardObject, config};

use gtk::glib::{Bytes, clone};
use rand::{TryRng, rng};
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use secret_service::{EncryptionType, blocking::SecretService};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
extern crate alloc;
use alloc::vec::Vec;
use chacha20poly1305::{
    self, ChaCha20Poly1305, KeyInit, Nonce,
    aead::{Aead, Generate},
};
use gtk::gio;
use postcard::to_allocvec;

const TIMELINE_TABLE: TableDefinition<i64, &[u8; 32]> = TableDefinition::new("timeline");
const CONTENT_TABLE: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("content");
const NONCE_LEN: usize = 12;

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub item_bytes: Vec<u8>,
    pub mime_type: String,
}

pub struct HistoryItem {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct DbState {
    pub db: Arc<Database>,
    pub hash_cache: Arc<Mutex<HashMap<[u8; 32], i64>>>,
    pub history_limit: usize,
    pub key: Arc<[u8; 32]>,
}

impl DbState {
    pub fn init(limit: usize) -> anyhow::Result<Self> {
        let db_path = config::app_config_dir()?.join("history.redb");
        let db = Database::create(db_path)?;
        let mut cache = HashMap::new();
        let read_txn = db.begin_read()?;
        if let Ok(timeline_table) = read_txn.open_table(TIMELINE_TABLE) {
            for item in timeline_table.iter()? {
                let (ts_guard, hash_guard) = item?;
                cache.insert(*hash_guard.value(), ts_guard.value());
            }
        }

        Ok(Self {
            db: Arc::new(db),
            hash_cache: Arc::new(Mutex::new(cache)),
            history_limit: limit,
            key: Arc::new(Self::load_or_create_key()?),
        })
    }

    pub async fn store(&self, clipboard_object: ClipboardObject) -> anyhow::Result<()> {
        let content = Content {
            item_bytes: clipboard_object.item_bytes().as_ref().to_vec(),
            mime_type: clipboard_object.mime_type(),
        };

        let serialized_bytes: Vec<u8> = to_allocvec(&content)?;
        let cipher = ChaCha20Poly1305::new(self.key.as_ref().into());
        let nonce = Nonce::generate();
        let ciphertext = cipher.encrypt(&nonce, serialized_bytes.as_ref())?;
        let mut encrypted_bytes = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        encrypted_bytes.extend_from_slice(&nonce);
        encrypted_bytes.extend_from_slice(&ciphertext);

        let hash_array: [u8; 32] = clipboard_object.hash().as_ref().try_into()?;
        let timestamp = clipboard_object.timestamp();

        let handle = gio::spawn_blocking(clone!(
            #[strong(rename_to = state)]
            self,
            move || -> anyhow::Result<()> {
                let write_txn = state.db.begin_write()?;
                {
                    let mut content_table = write_txn.open_table(CONTENT_TABLE)?;
                    let mut timeline_table = write_txn.open_table(TIMELINE_TABLE)?;

                    if content_table.get(&hash_array)?.is_none() {
                        content_table.insert(&hash_array, encrypted_bytes.as_slice())?;
                    }

                    let mut hash_cache = state
                        .hash_cache
                        .lock()
                        .map_err(|e| anyhow::anyhow!("hash_cache mutex poisoned: {:#}", e))?;
                    if let Some(old_ts) = hash_cache.get(&hash_array) {
                        timeline_table.remove(*old_ts)?;
                    }

                    timeline_table.insert(timestamp, &hash_array)?;
                    hash_cache.insert(hash_array, timestamp);

                    // Maintaining history limit
                    while timeline_table.len()? > state.history_limit as u64 {
                        let Some((_, hash_guard)) = timeline_table.pop_first()? else {
                            break;
                        };
                        content_table.remove(hash_guard.value())?;
                        hash_cache.remove(hash_guard.value());
                    }
                }
                write_txn.commit()?;
                Ok(())
            }
        ));

        match handle.await {
            Ok(result) => result,
            Err(e) => Err(anyhow::anyhow!("spawn_blocking panicked: {:?}", e)),
        }
    }

    pub async fn get_history(&self) -> anyhow::Result<Vec<HistoryItem>> {
        let handle = gio::spawn_blocking(clone!(
            #[strong(rename_to = state)]
            self,
            move || -> anyhow::Result<Vec<HistoryItem>> {
                let read_txn = state.db.begin_read()?;
                let timeline_table = read_txn.open_table(TIMELINE_TABLE)?;
                let content_table = read_txn.open_table(CONTENT_TABLE)?;
                let mut history = Vec::with_capacity(state.history_limit);
                for item in timeline_table.iter()?.rev().take(state.history_limit) {
                    let (ts_guard, hash_guard) = item?;
                    let timestamp = ts_guard.value();
                    let hash = hash_guard.value();

                    if let Some(content_guard) = content_table.get(hash)? {
                        let encrypted_bytes = content_guard.value();
                        let cipher = ChaCha20Poly1305::new(state.key.as_ref().into());
                        let (nonce, ciphertext) = encrypted_bytes.split_at(12);
                        let content_bytes = cipher.decrypt(&Nonce::try_from(nonce)?, ciphertext)?;

                        let content: Content = postcard::from_bytes(&content_bytes)?;
                        let item = HistoryItem {
                            bytes: content.item_bytes,
                            mime_type: content.mime_type,
                            timestamp,
                        };
                        history.push(item);
                    }
                }
                Ok(history)
            }
        ));
        match handle.await {
            Ok(result) => result,
            Err(e) => Err(anyhow::anyhow!("spawn_blocking panicked: {:?}", e)),
        }
    }

    pub async fn remove(&self, hash: &Bytes) -> anyhow::Result<()> {
        let handle = gio::spawn_blocking(clone!(
            #[strong(rename_to = state)]
            self,
            #[strong]
            hash,
            move || -> anyhow::Result<()> {
                let hash_array: [u8; 32] = hash.as_ref().try_into()?;
                let write_txn = state.db.begin_write()?;
                {
                    let mut content_table = write_txn.open_table(CONTENT_TABLE)?;
                    let mut timeline_table = write_txn.open_table(TIMELINE_TABLE)?;

                    let mut hash_cache = state
                        .hash_cache
                        .lock()
                        .map_err(|e| anyhow::anyhow!("hash_cache mutex poisoned: {:#}", e))?;
                    if let Some(timestamp) = hash_cache.remove(&hash_array) {
                        timeline_table.remove(timestamp)?;
                    }
                    content_table.remove(&hash_array)?;
                }
                write_txn.commit()?;
                Ok(())
            }
        ));
        match handle.await {
            Ok(result) => result,
            Err(e) => Err(anyhow::anyhow!("spawn_blocking panicked: {:?}", e)),
        }
    }

    pub async fn clear_all(&self) -> anyhow::Result<()> {
        let handle = gio::spawn_blocking(clone!(
            #[strong(rename_to = state)]
            self,
            move || -> anyhow::Result<()> {
                let write_txn = state.db.begin_write()?;
                {
                    let mut timeline_table = write_txn.open_table(TIMELINE_TABLE)?;
                    let mut content_table = write_txn.open_table(CONTENT_TABLE)?;

                    while timeline_table.pop_first()?.is_some() {}
                    while content_table.pop_first()?.is_some() {}

                    state
                        .hash_cache
                        .lock()
                        .map_err(|e| anyhow::anyhow!("hash_cache mutex poisoned: {:#}", e))?
                        .clear();
                }
                write_txn.commit()?;
                Ok(())
            }
        ));
        match handle.await {
            Ok(result) => result,
            Err(e) => Err(anyhow::anyhow!("spawn_blocking panicked: {:?}", e)),
        }
    }

    fn generate_key() -> anyhow::Result<[u8; 32]> {
        let mut key = [0; 32];
        rng().try_fill_bytes(&mut key)?;
        Ok(key)
    }

    fn load_or_create_key() -> anyhow::Result<[u8; 32]> {
        let ss = SecretService::connect(EncryptionType::Dh)?;
        let collection = ss.get_default_collection()?;
        let secret_attributes =
            HashMap::from([("app_name", config::app_id()), ("usage", "database")]);

        let search_items = collection.search_items(secret_attributes.clone())?;

        if let Some(item) = search_items.first() {
            let secret = item.get_secret()?;
            return Ok(secret.as_slice().try_into()?);
        }
        let key = Self::generate_key()?;
        collection.create_item(
            config::app_id(),
            secret_attributes,
            &key,
            false,
            "application/octet-stream",
        )?;
        Ok(key)
    }
}
