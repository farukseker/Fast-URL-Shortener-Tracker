use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub(crate) struct UrlCache {
    entries: Arc<RwLock<HashMap<String, CachedUrl>>>,
}

#[derive(Clone)]
pub(crate) struct CachedUrl {
    pub(crate) id: i32,
    pub(crate) url: String,
}

impl UrlCache {
    pub(crate) fn new(entries: HashMap<String, CachedUrl>) -> Self {
        Self {
            entries: Arc::new(RwLock::new(entries)),
        }
    }

    pub(crate) async fn get(&self, code: &str) -> Option<CachedUrl> {
        self.entries.read().await.get(code).cloned()
    }

    pub(crate) async fn insert(&self, code: String, url: CachedUrl) {
        self.entries.write().await.insert(code, url);
    }

    pub(crate) async fn replace(&self, old_code: &str, code: String, url: CachedUrl) {
        let mut entries = self.entries.write().await;
        entries.remove(old_code);
        entries.insert(code, url);
    }

    pub(crate) async fn remove(&self, code: &str) {
        self.entries.write().await.remove(code);
    }
}