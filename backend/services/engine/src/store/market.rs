use crate::types::market_types::{Market, MarketStatus};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct MarketStore {
    inner: Arc<RwLock<HashMap<u64, Market>>>,
}

impl MarketStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_market(&self, market: Market) -> Result<(), String> {
        let mut guard = self.inner.write().map_err(|_| "failed to register markets".to_string())?;
        guard.insert(market.market_id, market);
        Ok(())
    }

    pub fn get_market(&self, market_id: u64) -> Option<Market> {
        self.inner.read().ok()?.get(&market_id).cloned()
    }

    pub fn update_status(&self, market_id: u64, status: MarketStatus) -> Result<(), String> {
        let mut guard = self.inner.write().map_err(|_| "market registry poisoned".to_string())?;
        let Some(meta) = guard.get_mut(&market_id) else {
            return Err("market not registered".into());
        };
        meta.status = status;
        Ok(())
    }

    pub fn list_markets(&self) -> Vec<Market> {
        self.inner
            .read()
            .map(|g| g.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn remove_market(&self, market_id: u64) -> Result<(), String> {
        let mut guard = self.inner.write().map_err(|_| "market registry poisoned".to_string())?;
        guard
            .remove(&market_id)
            .map(|_| ())
            .ok_or_else(|| "market not registered".into())
    }
}
