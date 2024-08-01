use std::hash::Hash;
use std::num::NonZero;
use std::{cell::RefCell, rc::Rc};

use lru::LruCache;

pub trait Summoner<Obj> {
    type Id;
    type Err;
    fn summon(&self, id: Self::Id) -> Result<Obj, Self::Err>;
}

/// Supply LRU cache for any `Summoner`
pub struct CachedSummoner<Obj, S: Summoner<Obj>> {
    summoner: S,
    pub cache: RefCell<LruCache<S::Id, Rc<Obj>>>,
}

impl<Obj, S> CachedSummoner<Obj, S>
where
    S: Summoner<Obj>,
    S::Id: Hash + Eq,
{
    pub const DEFAULT_CACHE_SIZE: NonZero<usize> = unsafe { NonZero::new_unchecked(1024) };
    pub fn new(summoner: S) -> Self {
        Self {
            summoner,
            cache: RefCell::new(LruCache::new(Self::DEFAULT_CACHE_SIZE)),
        }
    }
}

impl<Obj, S: Summoner<Obj, Id: ?Sized>> Summoner<Rc<Obj>> for CachedSummoner<Obj, S>
where
    S::Id: Hash + Eq + Clone,
{
    type Id = S::Id;
    type Err = S::Err;

    fn summon(&self, id: Self::Id) -> Result<Rc<Obj>, Self::Err> {
        if let Some(obj) = self.cache.borrow_mut().get(&id) {
            return Ok(obj.clone());
        }

        let obj = Rc::new(self.summoner.summon(id.clone())?);
        self.cache.borrow_mut().put(id, obj.clone());
        Ok(obj)
    }
}
