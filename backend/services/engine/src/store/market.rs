use crate::types::market_types::{Market, MarketStatus, MarketSide, MarketMeta};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct MarketStore {
    inner: Arc<RwLock<HashMap<u64, Market>>>,
    event_markets: Arc<RwLock<HashMap<u64, Vec<u64>>>>,
}

impl MarketStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            event_markets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_market(&self, market: Market) -> Result<(), String> {
        let mut guard = self.inner.write().map_err(|_| "failed to register markets".to_string())?;
        guard.insert(market.market_id, market);
        Ok(())
    }

    pub fn register_market_pair(&self, meta: MarketMeta) -> Result<(), String> {
        let mut guard = self.inner.write().map_err(|_| "failed to register markets".to_string())?;
        
        let yes_market = Market {
            market_id: meta.yes_market_id,
            status: MarketStatus::Active,
            side: Some(MarketSide::Yes),
            paired_market_id: Some(meta.no_market_id),
            event_id: Some(meta.event_id),
            outcome_id: Some(meta.outcome_id),
        };
        
        let no_market = Market {
            market_id: meta.no_market_id,
            status: MarketStatus::Active,
            side: Some(MarketSide::No),
            paired_market_id: Some(meta.yes_market_id),
            event_id: Some(meta.event_id),
            outcome_id: Some(meta.outcome_id),
        };
        
        guard.insert(meta.yes_market_id, yes_market);
        guard.insert(meta.no_market_id, no_market);
        
        let mut event_guard = self.event_markets.write().map_err(|_| "failed to register event markets".to_string())?;
        event_guard.entry(meta.event_id).or_insert_with(Vec::new).push(meta.yes_market_id);
        event_guard.entry(meta.event_id).or_insert_with(Vec::new).push(meta.no_market_id);
        
        Ok(())
    }

    pub fn get_market(&self, market_id: u64) -> Option<Market> {
        self.inner.read().ok()?.get(&market_id).cloned()
    }

    pub fn get_markets_by_event(&self, event_id: u64) -> Vec<u64> {
        self.event_markets.read()
            .ok()
            .and_then(|g| g.get(&event_id).cloned())
            .unwrap_or_default()
    }

    pub fn update_status(&self, market_id: u64, status: MarketStatus) -> Result<(), String> {
        let mut guard = self.inner.write().map_err(|_| "market registry poisoned".to_string())?;
        let Some(meta) = guard.get_mut(&market_id) else {
            return Err("market not registered".into());
        };
        meta.status = status;
        Ok(())
    }

    pub fn update_status_bulk(&self, market_ids: Vec<u64>, status: MarketStatus) -> Result<(), String> {
        let mut guard = self.inner.write().map_err(|_| "market registry poisoned".to_string())?;
        for market_id in market_ids {
            if let Some(meta) = guard.get_mut(&market_id) {
                meta.status = status.clone();
            }
        }
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

    pub fn remove_markets_by_event(&self, event_id: u64) -> Result<(), String> {
        let market_ids = self.get_markets_by_event(event_id);
        let mut guard = self.inner.write().map_err(|_| "market registry poisoned".to_string())?;
        for market_id in market_ids {
            guard.remove(&market_id);
        }
        let mut event_guard = self.event_markets.write().map_err(|_| "failed to remove event markets".to_string())?;
        event_guard.remove(&event_id);
        Ok(())
    }
}
