#![allow(
    clippy::unwrap_used,
    clippy::unimplemented,
    clippy::missing_panics_doc,
    reason = "Test utilities can panic"
)]

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::conversation::{
    Conversation, ConversationCursor, ConversationId, ConversationInboxItem, ConversationInboxPage,
    ConversationPair, Message, MessageCursor, MessageId, MessagePage, NewMessage, OfferInquirer,
    OfferInquirersPage,
};
use crate::image::{ImageDimensions, ImageId, ImagePosition, ImagePublicId, OfferImage};
use crate::offer::{CreateOffer, Offer, OfferId, UpdateOffer, UserId};
use crate::ports::{
    AccountRepository, ConversationRepository, CreationStatus, IdempotencyKey, MagicLinkRepository,
    OfferImageRepository, OfferRepository, Registry, RepoError, SessionRepository, UserRepository,
};
use crate::saved::{CollectionCursor, FollowPage, SavedOffer, SavedOfferPage};
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
        Ok(lock
            .iter()
            .filter(|o| o.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_owned(&self, id: OfferId, user_id: UserId) -> Result<Option<Offer>, RepoError> {
        let lock = self.offers.lock().unwrap();
        Ok(lock
            .iter()
            .find(|o| o.id == id && o.user_id == user_id)
            .cloned())
    }

    async fn find_by_slug(
        &self,
        slug: &crate::offer::OfferSlug,
    ) -> Result<Option<Offer>, RepoError> {
        let lock = self.offers.lock().unwrap();
        Ok(lock.iter().find(|o| o.slug == *slug).cloned())
    }

    async fn list_public(
        &self,
        query: &crate::offer::PublicFeedQuery,
    ) -> Result<crate::offer::PublicOfferPage, RepoError> {
        let lock = self.offers.lock().unwrap();
        let mut sorted = lock.clone();
        sorted.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.id.as_uuid().cmp(&a.id.as_uuid()))
        });

        let capped_limit = query.limit.clamp(1, 50);

        let filtered: Vec<Offer> = sorted
            .into_iter()
            .filter(|o| {
                if !o.is_public() {
                    return false;
                }
                if let Some(ref title_q) = query.filter.title {
                    let needle = title_q.as_str().to_lowercase();
                    if !o.title.to_lowercase().contains(&needle) {
                        return false;
                    }
                }
                if let Some(ref pf) = query.filter.price {
                    if o.currency != pf.currency {
                        return false;
                    }
                    if let Some(min_p) = pf.min
                        && o.price.as_i32() < min_p.as_i32()
                    {
                        return false;
                    }
                    if let Some(max_p) = pf.max
                        && o.price.as_i32() > max_p.as_i32()
                    {
                        return false;
                    }
                }
                if let Some(ref c) = query.cursor
                    && (o.created_at, o.id.as_uuid()) >= (c.created_at, c.id.as_uuid())
                {
                    return false;
                }
                true
            })
            .collect();

        let has_more = filtered.len() > capped_limit;
        let items: Vec<Offer> = filtered.into_iter().take(capped_limit).collect();
        let next_cursor = if has_more {
            items
                .last()
                .map(|item| crate::offer::OfferCursor::new(item.created_at, item.id).encode())
        } else {
            None
        };

        let public_items: Vec<crate::offer::PublicOffer> = items
            .into_iter()
            .map(|offer| crate::offer::PublicOffer {
                offer,
                maker_display_name: crate::user::DisplayName::parse("Test Maker").unwrap(),
                maker_handle: crate::user::ProfileHandle::parse("test-maker").unwrap(),
            })
            .collect();

        Ok(crate::offer::PublicOfferPage {
            items: public_items,
            next_cursor,
        })
    }

    async fn list_by_user(
        &self,
        user_id: UserId,
        cursor: Option<&crate::offer::OfferCursor>,
        limit: usize,
    ) -> Result<crate::offer::OfferPage, RepoError> {
        let lock = self.offers.lock().unwrap();
        let mut sorted: Vec<Offer> = lock
            .iter()
            .filter(|o| o.user_id == user_id && o.image_count >= 1)
            .cloned()
            .collect();
        sorted.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.id.as_uuid().cmp(&a.id.as_uuid()))
        });

        let capped_limit = limit.clamp(1, 50);

        let filtered: Vec<Offer> = if let Some(c) = cursor {
            sorted
                .into_iter()
                .filter(|o| (o.created_at, o.id.as_uuid()) < (c.created_at, c.id.as_uuid()))
                .collect()
        } else {
            sorted
        };

        let has_more = filtered.len() > capped_limit;
        let items: Vec<Offer> = filtered.into_iter().take(capped_limit).collect();
        let next_cursor = if has_more {
            items
                .last()
                .map(|item| crate::offer::OfferCursor::new(item.created_at, item.id).encode())
        } else {
            None
        };

        Ok(crate::offer::OfferPage { items, next_cursor })
    }

    async fn create(
        &self,
        user_id: UserId,
        idempotency_key: IdempotencyKey,
        req: &CreateOffer,
    ) -> Result<CreationStatus<Offer>, RepoError> {
        let mut idemp = self.idemp.lock().unwrap();
        let key = (user_id, idempotency_key.as_uuid());
        if let Some(offer) = idemp.get(&key) {
            return Ok(CreationStatus::Replayed(offer.clone()));
        }

        let offer_id = OfferId::generate();
        let suffix = &offer_id.as_uuid().to_string()[..8];
        let slug = crate::offer::OfferSlug::from_title_and_suffix(&req.title, suffix)
            .map_err(|e| RepoError::Corrupt(e.to_string()))?;

        let offer = Offer {
            id: offer_id,
            user_id,
            slug,
            title: req.title.clone(),
            description: req.description.clone(),
            price: req.price,
            currency: req.currency,
            image_count: 0,
            images: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.offers.lock().unwrap().push(offer.clone());
        idemp.insert(key, offer.clone());
        Ok(CreationStatus::Created(offer))
    }

    async fn update(
        &self,
        id: OfferId,
        user_id: UserId,
        req: &UpdateOffer,
    ) -> Result<Offer, RepoError> {
        let mut lock = self.offers.lock().unwrap();
        let Some(offer) = lock.iter_mut().find(|o| o.id == id && o.user_id == user_id) else {
            return Err(RepoError::NotFound {
                entity: "offer",
                id: id.to_string(),
            });
        };
        if let Some(ref t) = req.title {
            offer.title.clone_from(t);
        }
        if let Some(ref d) = req.description {
            offer.description.clone_from(d);
        }
        if let Some(p) = req.price {
            offer.price = p;
        }
        if let Some(c) = req.currency {
            offer.currency = c;
        }
        offer.updated_at = Utc::now();
        Ok(offer.clone())
    }

    async fn delete(&self, id: OfferId, user_id: UserId) -> Result<(), RepoError> {
        let mut lock = self.offers.lock().unwrap();
        let pos = lock
            .iter()
            .position(|o| o.id == id && o.user_id == user_id)
            .ok_or_else(|| RepoError::NotFound {
                entity: "offer",
                id: id.to_string(),
            })?;
        lock.remove(pos);
        Ok(())
    }
}

pub struct FakeRegistry {
    offers: Box<FakeOfferRepository>,
    saved_offers: Box<FakeSavedOfferRepository>,
    follows: Box<FakeFollowRepository>,
}

impl Default for FakeRegistry {
    fn default() -> Self {
        let offers = Box::new(FakeOfferRepository::default());
        Self {
            saved_offers: Box::new(FakeSavedOfferRepository {
                pairs: Arc::new(Mutex::new(HashMap::new())),
                offers: offers.offers.clone(),
            }),
            offers,
            follows: Box::new(FakeFollowRepository::default()),
        }
    }
}

pub type SavedOfferMap = HashMap<(UserId, OfferId), chrono::DateTime<Utc>>;
pub type FollowMap = HashMap<(UserId, UserId), chrono::DateTime<Utc>>;

#[derive(Default)]
pub struct FakeSavedOfferRepository {
    pub pairs: Arc<Mutex<SavedOfferMap>>,
    offers: Arc<Mutex<Vec<Offer>>>,
}
#[async_trait]
impl crate::ports::SavedOfferRepository for FakeSavedOfferRepository {
    async fn is_saved(&self, user_id: UserId, offer_id: OfferId) -> Result<bool, RepoError> {
        Ok(self
            .pairs
            .lock()
            .unwrap()
            .contains_key(&(user_id, offer_id)))
    }
    async fn saved_among(
        &self,
        user_id: UserId,
        offer_ids: &[OfferId],
    ) -> Result<std::collections::HashSet<OfferId>, RepoError> {
        let pairs = self.pairs.lock().unwrap();
        Ok(offer_ids
            .iter()
            .copied()
            .filter(|id| pairs.contains_key(&(user_id, *id)))
            .collect())
    }
    async fn save(&self, user_id: UserId, offer_id: OfferId) -> Result<(), RepoError> {
        self.pairs
            .lock()
            .unwrap()
            .entry((user_id, offer_id))
            .or_insert_with(Utc::now);
        Ok(())
    }
    async fn unsave(&self, user_id: UserId, offer_id: OfferId) -> Result<(), RepoError> {
        self.pairs.lock().unwrap().remove(&(user_id, offer_id));
        Ok(())
    }
    async fn list(
        &self,
        user_id: UserId,
        cursor: Option<CollectionCursor>,
        limit: usize,
    ) -> Result<SavedOfferPage, RepoError> {
        let pairs = self.pairs.lock().unwrap();
        let offers = self.offers.lock().unwrap();
        let mut entries = pairs
            .iter()
            .filter(|((owner, _), _)| *owner == user_id)
            .filter(|((_, id), time)| {
                cursor.is_none_or(|c| (**time, id.as_uuid()) < (c.created_at, c.id))
            })
            .map(|((_, id), time)| (*id, *time))
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| b.0.as_uuid().cmp(&a.0.as_uuid()))
        });
        let page_size = limit.clamp(1, 50);
        let has_more = entries.len() > page_size;
        let items = entries
            .into_iter()
            .take(page_size)
            .map(|(offer_id, created_at)| SavedOffer {
                offer_id,
                offer: offers
                    .iter()
                    .find(|o| o.id == offer_id && o.is_public())
                    .cloned(),
                created_at,
                id: offer_id.as_uuid(),
            })
            .collect::<Vec<_>>();
        let next_cursor = if has_more {
            items
                .last()
                .map(|item| CollectionCursor::new(item.created_at, item.id).encode())
        } else {
            None
        };
        Ok(SavedOfferPage { items, next_cursor })
    }
}

#[derive(Default)]
pub struct FakeFollowRepository {
    pub pairs: Arc<Mutex<FollowMap>>,
    pub people: Arc<Mutex<HashMap<UserId, crate::user::User>>>,
}
impl FakeFollowRepository {
    /// Adds identity data for a followed person in repository tests.
    pub fn add_person(&self, person: crate::user::User) {
        self.people.lock().unwrap().insert(person.id, person);
    }
}
#[async_trait]
impl crate::ports::FollowRepository for FakeFollowRepository {
    async fn is_following(&self, user_id: UserId, followed_id: UserId) -> Result<bool, RepoError> {
        Ok(self
            .pairs
            .lock()
            .unwrap()
            .contains_key(&(user_id, followed_id)))
    }
    async fn follow(&self, user_id: UserId, followed_id: UserId) -> Result<(), RepoError> {
        if user_id == followed_id {
            return Err(RepoError::Conflict {
                entity: "follow",
                key: "cannot follow yourself".into(),
            });
        }
        self.pairs
            .lock()
            .unwrap()
            .entry((user_id, followed_id))
            .or_insert_with(Utc::now);
        Ok(())
    }
    async fn unfollow(&self, user_id: UserId, followed_id: UserId) -> Result<(), RepoError> {
        self.pairs.lock().unwrap().remove(&(user_id, followed_id));
        Ok(())
    }
    async fn list(
        &self,
        user_id: UserId,
        cursor: Option<CollectionCursor>,
        limit: usize,
    ) -> Result<FollowPage, RepoError> {
        let pairs = self.pairs.lock().unwrap();
        let people = self.people.lock().unwrap();
        let mut entries = pairs
            .iter()
            .filter(|((owner, person), _)| *owner == user_id && people.contains_key(person))
            .filter(|((_, person), time)| {
                cursor.is_none_or(|c| (**time, person.as_uuid()) < (c.created_at, c.id))
            })
            .map(|((_, person), time)| (*person, *time))
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| b.0.as_uuid().cmp(&a.0.as_uuid()))
        });
        let page_size = limit.clamp(1, 50);
        let has_more = entries.len() > page_size;
        let items = entries
            .into_iter()
            .take(page_size)
            .filter_map(|(id, created_at)| {
                people
                    .get(&id)
                    .cloned()
                    .map(|person| crate::saved::FollowedPerson {
                        person,
                        created_at,
                        id: id.as_uuid(),
                    })
            })
            .collect::<Vec<_>>();
        let next_cursor = if has_more {
            items
                .last()
                .map(|item| CollectionCursor::new(item.created_at, item.id).encode())
        } else {
            None
        };
        Ok(FollowPage { items, next_cursor })
    }
}

#[async_trait]
impl OfferImageRepository for FakeOfferRepository {
    async fn find_by_offer(&self, offer_id: OfferId) -> Result<Vec<OfferImage>, RepoError> {
        let lock = self.offers.lock().unwrap();
        let images = lock
            .iter()
            .find(|o| o.id == offer_id)
            .map(|o| o.images.clone())
            .unwrap_or_default();
        Ok(images)
    }

    async fn add_image(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        public_id: &ImagePublicId,
        dimensions: ImageDimensions,
    ) -> Result<OfferImage, RepoError> {
        let mut lock = self.offers.lock().unwrap();
        let offer = lock
            .iter_mut()
            .find(|o| o.id == offer_id && o.user_id == user_id)
            .ok_or_else(|| RepoError::NotFound {
                entity: "offer",
                id: offer_id.to_string(),
            })?;

        if offer.images.len() >= 5 {
            return Err(RepoError::Conflict {
                entity: "offer_image",
                key: "maximum 5 images per offer".into(),
            });
        }

        let pos_u8 = u8::try_from(offer.images.len().saturating_add(1))
            .map_err(|_| RepoError::Corrupt("image position exceeds valid bounds".into()))?;
        let position = ImagePosition::new(pos_u8).map_err(|e| RepoError::Corrupt(e.to_string()))?;

        let image = OfferImage {
            id: ImageId::generate(),
            offer_id,
            public_id: public_id.clone(),
            position,
            dimensions,
            created_at: Utc::now(),
        };

        offer.images.push(image.clone());
        offer.image_count = u8::try_from(offer.images.len())
            .map_err(|_| RepoError::Corrupt("image count exceeds valid bounds".into()))?;
        Ok(image)
    }

    async fn remove_image(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        image_id: ImageId,
    ) -> Result<ImagePublicId, RepoError> {
        let mut lock = self.offers.lock().unwrap();
        let offer = lock
            .iter_mut()
            .find(|o| o.id == offer_id && o.user_id == user_id)
            .ok_or_else(|| RepoError::NotFound {
                entity: "offer",
                id: offer_id.to_string(),
            })?;

        if offer.images.len() <= 1 {
            return Err(RepoError::Conflict {
                entity: "offer_image",
                key: "an offer needs at least one image".into(),
            });
        }

        let idx = offer
            .images
            .iter()
            .position(|img| img.id == image_id)
            .ok_or_else(|| RepoError::NotFound {
                entity: "offer_image",
                id: image_id.to_string(),
            })?;

        let removed = offer.images.remove(idx);
        for (i, img) in offer.images.iter_mut().enumerate() {
            let pos_u8 = u8::try_from(i.saturating_add(1))
                .map_err(|_| RepoError::Corrupt("image position exceeds valid bounds".into()))?;
            img.position =
                ImagePosition::new(pos_u8).map_err(|e| RepoError::Corrupt(e.to_string()))?;
        }
        offer.image_count = u8::try_from(offer.images.len())
            .map_err(|_| RepoError::Corrupt("image count exceeds valid bounds".into()))?;
        Ok(removed.public_id)
    }

    async fn reorder_images(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        order: &[ImageId],
    ) -> Result<Vec<OfferImage>, RepoError> {
        let mut lock = self.offers.lock().unwrap();
        let offer = lock
            .iter_mut()
            .find(|o| o.id == offer_id && o.user_id == user_id)
            .ok_or_else(|| RepoError::NotFound {
                entity: "offer",
                id: offer_id.to_string(),
            })?;

        if order.len() != offer.images.len() {
            return Err(RepoError::Conflict {
                entity: "offer_image",
                key: "reorder list length mismatch".into(),
            });
        }

        let mut reordered = Vec::with_capacity(order.len());
        for target_id in order {
            let found = offer
                .images
                .iter()
                .find(|img| img.id == *target_id)
                .cloned()
                .ok_or_else(|| RepoError::NotFound {
                    entity: "offer_image",
                    id: target_id.to_string(),
                })?;
            reordered.push(found);
        }

        for (i, img) in reordered.iter_mut().enumerate() {
            let pos_u8 = u8::try_from(i.saturating_add(1))
                .map_err(|_| RepoError::Corrupt("image position exceeds valid bounds".into()))?;
            img.position =
                ImagePosition::new(pos_u8).map_err(|e| RepoError::Corrupt(e.to_string()))?;
        }

        offer.images.clone_from(&reordered);
        Ok(reordered)
    }
}

impl FakeRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

pub type ConversationParticipantMap = HashMap<(ConversationId, UserId), chrono::DateTime<Utc>>;
pub type UserInfoMap = HashMap<
    UserId,
    (
        crate::user::DisplayName,
        crate::user::ProfileHandle,
        Option<String>,
    ),
>;

#[derive(Default, Clone)]
pub struct FakeConversationRepository {
    pub conversations: Arc<Mutex<Vec<Conversation>>>,
    pub messages: Arc<Mutex<Vec<Message>>>,
    pub participants: Arc<Mutex<ConversationParticipantMap>>,
    pub users_info: Arc<Mutex<UserInfoMap>>,
    pub offer_owners: Arc<Mutex<HashMap<OfferId, UserId>>>,
}

#[async_trait]
impl ConversationRepository for FakeConversationRepository {
    async fn get_or_create(&self, pair: ConversationPair) -> Result<Conversation, RepoError> {
        let mut convs = self.conversations.lock().unwrap();
        if let Some(c) = convs.iter().find(|c| c.pair == pair) {
            return Ok(c.clone());
        }
        let now = Utc::now();
        let conv = Conversation {
            id: ConversationId::generate(),
            pair,
            created_at: now,
            last_message_at: now,
        };
        convs.push(conv.clone());
        let mut parts = self.participants.lock().unwrap();
        parts.insert((conv.id, pair.low()), now);
        parts.insert((conv.id, pair.high()), now);
        Ok(conv)
    }

    async fn find_by_id(&self, id: ConversationId) -> Result<Option<Conversation>, RepoError> {
        let convs = self.conversations.lock().unwrap();
        Ok(convs.iter().find(|c| c.id == id).cloned())
    }

    async fn append(
        &self,
        conversation_id: ConversationId,
        new_message: NewMessage,
    ) -> Result<Message, RepoError> {
        let mut convs = self.conversations.lock().unwrap();
        let conv = convs
            .iter_mut()
            .find(|c| c.id == conversation_id)
            .ok_or_else(|| RepoError::NotFound {
                entity: "conversation",
                id: conversation_id.to_string(),
            })?;

        if !conv.pair.contains(new_message.sender_id) {
            return Err(RepoError::NotFound {
                entity: "conversation",
                id: conversation_id.to_string(),
            });
        }

        let now = Utc::now();
        conv.last_message_at = now;

        let message = Message {
            id: MessageId::generate(),
            conversation_id,
            sender_id: new_message.sender_id,
            body: new_message.body,
            offer_id: new_message.offer_id,
            created_at: now,
        };

        let mut msgs = self.messages.lock().unwrap();
        msgs.push(message.clone());

        let mut parts = self.participants.lock().unwrap();
        parts.insert((conversation_id, new_message.sender_id), now);

        Ok(message)
    }

    async fn list_for_user(
        &self,
        user_id: UserId,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<ConversationInboxPage, RepoError> {
        let parsed_cursor = match cursor {
            Some(c) => ConversationCursor::decode(c).ok(),
            None => None,
        };

        let convs = self.conversations.lock().unwrap();
        let msgs = self.messages.lock().unwrap();
        let parts = self.participants.lock().unwrap();
        let users_info = self.users_info.lock().unwrap();

        let mut user_convs: Vec<Conversation> = convs
            .iter()
            .filter(|c| c.pair.contains(user_id))
            .cloned()
            .collect();

        user_convs.sort_by(|a, b| {
            b.last_message_at
                .cmp(&a.last_message_at)
                .then_with(|| b.id.as_uuid().cmp(&a.id.as_uuid()))
        });

        if let Some(cur) = parsed_cursor {
            user_convs.retain(|c| {
                c.last_message_at < cur.last_message_at
                    || (c.last_message_at == cur.last_message_at
                        && c.id.as_uuid() < cur.id.as_uuid())
            });
        }

        let has_more = user_convs.len() > limit;
        let paged: Vec<Conversation> = user_convs.into_iter().take(limit).collect();

        let mut items = Vec::new();
        for conv in &paged {
            let other_id = conv.pair.other(user_id).unwrap();
            let last_msg = msgs.iter().rfind(|m| m.conversation_id == conv.id);

            let last_read = parts
                .get(&(conv.id, user_id))
                .copied()
                .unwrap_or(conv.created_at);

            let is_unread = match last_msg {
                Some(m) => m.sender_id != user_id && m.created_at > last_read,
                None => false,
            };

            let default_name = crate::user::DisplayName::parse("User").unwrap();
            let default_handle = crate::user::ProfileHandle::parse("user").unwrap();
            let (name, handle, avatar) =
                users_info
                    .get(&other_id)
                    .cloned()
                    .unwrap_or((default_name, default_handle, None));

            items.push(ConversationInboxItem {
                id: conv.id,
                other_user_id: other_id,
                other_display_name: name,
                other_handle: handle,
                other_avatar_public_id: avatar,
                last_message_body: last_msg
                    .map(|m| m.body.as_str().to_string())
                    .unwrap_or_default(),
                last_message_at: conv.last_message_at,
                is_unread,
            });
        }

        let next_cursor = if has_more {
            items
                .last()
                .map(|it| ConversationCursor::new(it.last_message_at, it.id).encode())
        } else {
            None
        };

        Ok(ConversationInboxPage { items, next_cursor })
    }

    async fn messages(
        &self,
        conversation_id: ConversationId,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<MessagePage, RepoError> {
        let parsed_cursor = match cursor {
            Some(c) => MessageCursor::decode(c).ok(),
            None => None,
        };

        let msgs = self.messages.lock().unwrap();
        let mut conv_msgs: Vec<Message> = msgs
            .iter()
            .filter(|m| m.conversation_id == conversation_id)
            .cloned()
            .collect();

        conv_msgs.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.id.as_uuid().cmp(&a.id.as_uuid()))
        });

        if let Some(cur) = parsed_cursor {
            conv_msgs.retain(|m| {
                m.created_at < cur.created_at
                    || (m.created_at == cur.created_at && m.id.as_uuid() < cur.id.as_uuid())
            });
        }

        let has_more = conv_msgs.len() > limit;
        let mut items: Vec<Message> = conv_msgs.into_iter().take(limit).collect();
        items.reverse();

        let next_cursor = if has_more {
            items
                .first()
                .map(|m| MessageCursor::new(m.created_at, m.id).encode())
        } else {
            None
        };

        Ok(MessagePage { items, next_cursor })
    }

    async fn mark_read(
        &self,
        conversation_id: ConversationId,
        user_id: UserId,
    ) -> Result<(), RepoError> {
        let mut parts = self.participants.lock().unwrap();
        parts.insert((conversation_id, user_id), Utc::now());
        Ok(())
    }

    async fn unread_exists(&self, user_id: UserId) -> Result<bool, RepoError> {
        let convs = self.conversations.lock().unwrap();
        let msgs = self.messages.lock().unwrap();
        let parts = self.participants.lock().unwrap();

        for conv in convs.iter().filter(|c| c.pair.contains(user_id)) {
            let last_read = parts
                .get(&(conv.id, user_id))
                .copied()
                .unwrap_or(conv.created_at);

            let last_msg = msgs.iter().rfind(|m| m.conversation_id == conv.id);

            if let Some(m) = last_msg
                && m.sender_id != user_id
                && m.created_at > last_read
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn list_offer_inquirers(
        &self,
        offer_id: OfferId,
        _cursor: Option<&str>,
        _limit: usize,
    ) -> Result<OfferInquirersPage, RepoError> {
        let msgs = self.messages.lock().unwrap();
        let owners = self.offer_owners.lock().unwrap();
        let maybe_owner = owners.get(&offer_id).copied();
        let mut map: HashMap<UserId, (ConversationId, chrono::DateTime<Utc>, u64)> = HashMap::new();

        for m in msgs.iter().filter(|m| m.offer_id == Some(offer_id)) {
            if Some(m.sender_id) == maybe_owner {
                continue;
            }
            let entry = map
                .entry(m.sender_id)
                .or_insert((m.conversation_id, m.created_at, 0));
            entry.2 = entry.2.saturating_add(1);
            if m.created_at > entry.1 {
                entry.1 = m.created_at;
            }
        }

        let items = map
            .into_iter()
            .map(
                |(user_id, (conversation_id, last_inquiry_at, message_count))| OfferInquirer {
                    user_id,
                    conversation_id,
                    last_inquiry_at,
                    message_count,
                },
            )
            .collect();

        Ok(OfferInquirersPage {
            items,
            next_cursor: None,
        })
    }
}

impl Registry for FakeRegistry {
    fn offers(&self) -> &(dyn OfferRepository + 'static) {
        self.offers.as_ref()
    }
    fn accounts(&self) -> &(dyn AccountRepository + 'static) {
        unimplemented!()
    }
    fn users(&self) -> &(dyn UserRepository + 'static) {
        unimplemented!()
    }
    fn magic_links(&self) -> &(dyn MagicLinkRepository + 'static) {
        unimplemented!()
    }
    fn sessions(&self) -> &(dyn SessionRepository + 'static) {
        unimplemented!()
    }
    fn offer_images(&self) -> &(dyn OfferImageRepository + 'static) {
        self.offers.as_ref()
    }
    fn conversations(&self) -> &(dyn ConversationRepository + 'static) {
        unimplemented!()
    }
    fn saved_offers(&self) -> &(dyn crate::ports::SavedOfferRepository + 'static) {
        self.saved_offers.as_ref()
    }
    fn follows(&self) -> &(dyn crate::ports::FollowRepository + 'static) {
        self.follows.as_ref()
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "test assertions and indexing"
)]
mod tests {
    use super::*;
    use crate::ports::{FollowRepository, SavedOfferRepository};
    use futures::future::join_all;

    #[tokio::test]
    async fn saved_and_follow_toggles_are_idempotent_and_reject_self_follow() {
        let user = UserId::generate();
        let offer = OfferId::generate();
        let saved = FakeSavedOfferRepository::default();
        saved.save(user, offer).await.unwrap();
        let other_offer = OfferId::generate();
        assert_eq!(
            saved
                .saved_among(user, &[offer, other_offer])
                .await
                .unwrap(),
            [offer].into_iter().collect()
        );
        assert!(saved.saved_among(user, &[]).await.unwrap().is_empty());
        let first_time = saved.pairs.lock().unwrap()[&(user, offer)];
        saved.save(user, offer).await.unwrap();
        assert_eq!(saved.pairs.lock().unwrap().len(), 1);
        assert_eq!(saved.pairs.lock().unwrap()[&(user, offer)], first_time);
        saved.unsave(user, offer).await.unwrap();
        saved.unsave(user, offer).await.unwrap();
        assert!(saved.pairs.lock().unwrap().is_empty());

        let follows = FakeFollowRepository::default();
        assert!(follows.follow(user, user).await.is_err());
        let other = UserId::generate();
        follows.follow(user, other).await.unwrap();
        follows.follow(user, other).await.unwrap();
        assert_eq!(follows.pairs.lock().unwrap().len(), 1);
        follows.unfollow(user, other).await.unwrap();
        follows.unfollow(user, other).await.unwrap();
        assert!(follows.pairs.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_fake_offer_repository_idempotency_concurrent() {
        let repo = FakeOfferRepository::default();
        let user_id = UserId::generate();
        let idempotency_key = IdempotencyKey::new(uuid::Uuid::new_v4());
        let req = CreateOffer {
            title: "Test Offer".to_string(),
            description: "A concurrent test offer description.".to_string(),
            price: crate::offer::Price::ZERO,
            currency: crate::offer::CurrencyCode::default_code(),
        };

        // 20 concurrent creates
        let futures: Vec<_> = (0..20)
            .map(|_| {
                let repo = repo.clone();
                let key = idempotency_key;
                let req = req.clone();
                tokio::spawn(async move { repo.create(user_id, key, &req).await.unwrap() })
            })
            .collect();

        let results = join_all(futures).await;

        let first_id = results[0].as_ref().unwrap().clone().into_inner().id;
        let mut new_count = 0;
        for res in &results {
            let status = res.as_ref().unwrap();
            if status.is_new() {
                new_count += 1;
            }
            assert_eq!(status.clone().into_inner().id, first_id);
        }
        assert_eq!(new_count, 1);

        let offers = repo.offers.lock().unwrap();
        assert_eq!(offers.len(), 1);
    }
}
