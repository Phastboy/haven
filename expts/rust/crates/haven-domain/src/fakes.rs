#![allow(clippy::unwrap_used, clippy::unimplemented, clippy::missing_panics_doc, reason = "Test utilities can panic")]

use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use std::collections::HashMap;

use crate::ports::{
    AccountRepository, MagicLinkRepository, OfferRepository, Registry, SessionRepository, UserRepository, RepoError, IdempotencyKey
};
use crate::offer::{Offer, OfferId, UserId, CreateOffer, UpdateOffer};
use chrono::Utc;

#[derive(Default, Clone)]
pub struct FakeOfferRepository {
    pub offers: Arc<Mutex<Vec<Offer>>>,
    // (user_id, idempotency_key) -> Offer
    pub idemp: Arc<Mutex<HashMap<(UserId, uuid::Uuid), Offer>>>,
}

#[async_trait]
impl OfferRepository for FakeOfferRepository {
    async fn find_by_user(&self, user_id: UserId) -> Result<Vec<Offer>, RepoError> {
        let lock = self.offers.lock().unwrap();
        Ok(lock.iter().filter(|o| o.user_id == user_id).cloned().collect())
    }

    async fn find_owned(&self, id: OfferId, user_id: UserId) -> Result<Option<Offer>, RepoError> {
        let lock = self.offers.lock().unwrap();
        Ok(lock.iter().find(|o| o.id == id && o.user_id == user_id).cloned())
    }

    async fn create(&self, user_id: UserId, idempotency_key: IdempotencyKey, req: &CreateOffer) -> Result<Offer, RepoError> {
        let mut idemp = self.idemp.lock().unwrap();
        let key = (user_id, idempotency_key.0);
        if let Some(offer) = idemp.get(&key) {
            return Ok(offer.clone());
        }

        let offer = Offer {
            id: OfferId(uuid::Uuid::new_v4()),
            user_id,
            title: req.title.clone(),
            description: req.description.clone(),
            price: req.price,
            currency: req.currency.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.offers.lock().unwrap().push(offer.clone());
        idemp.insert(key, offer.clone());
        Ok(offer)
    }

    async fn update(&self, id: OfferId, user_id: UserId, req: &UpdateOffer) -> Result<Offer, RepoError> {
        let mut lock = self.offers.lock().unwrap();
        let Some(offer) = lock.iter_mut().find(|o| o.id == id && o.user_id == user_id) else {
            return Err(RepoError::NotFound);
        };
        offer.title.clone_from(&req.title);
        offer.description.clone_from(&req.description);
        if let Some(p) = req.price {
            offer.price = p;
        }
        if let Some(ref c) = req.currency {
            offer.currency = c.clone();
        }
        offer.updated_at = Utc::now();
        Ok(offer.clone())
    }

    async fn delete(&self, id: OfferId, user_id: UserId) -> Result<(), RepoError> {
        let mut lock = self.offers.lock().unwrap();
        let pos = lock.iter().position(|o| o.id == id && o.user_id == user_id).ok_or(RepoError::NotFound)?;
        lock.remove(pos);
        Ok(())
    }
}

pub struct FakeRegistry {
    offers: Box<dyn OfferRepository>,
}

impl Default for FakeRegistry {
    fn default() -> Self {
        Self {
            offers: Box::new(FakeOfferRepository::default()),
        }
    }
}

impl FakeRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Registry for FakeRegistry {
    fn offers(&self) -> &(dyn OfferRepository + 'static) { self.offers.as_ref() }
    fn accounts(&self) -> &(dyn AccountRepository + 'static) { unimplemented!() }
    fn users(&self) -> &(dyn UserRepository + 'static) { unimplemented!() }
    fn magic_links(&self) -> &(dyn MagicLinkRepository + 'static) { unimplemented!() }
    fn sessions(&self) -> &(dyn SessionRepository + 'static) { unimplemented!() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::join_all;

    #[tokio::test]
    async fn test_fake_offer_repository_idempotency_concurrent() {
        let repo = FakeOfferRepository::default();
        let user_id = UserId(uuid::Uuid::new_v4());
        let idempotency_key = IdempotencyKey(uuid::Uuid::new_v4());
        let req = CreateOffer {
            title: "Test Offer".to_string(),
            description: None,
            price: crate::offer::Price::ZERO,
            currency: crate::offer::CurrencyCode::default_code(),
        };

        // 20 concurrent creates
        let futures: Vec<_> = (0..20).map(|_| {
            let repo = repo.clone();
            let key = idempotency_key.clone();
            let req = req.clone();
            tokio::spawn(async move {
                repo.create(user_id, key, &req).await.unwrap()
            })
        }).collect();

        let results = join_all(futures).await;
        
        let first_id = results[0].as_ref().unwrap().id;
        for res in results.iter() {
            let offer = res.as_ref().unwrap();
            assert_eq!(offer.id, first_id);
        }

        let offers = repo.offers.lock().unwrap();
        assert_eq!(offers.len(), 1);
    }
}
