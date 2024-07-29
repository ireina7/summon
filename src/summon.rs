use std::hash::Hash;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub trait Summoner<Obj> {
    type Id;
    type Err;
    fn summon(&self, id: Self::Id) -> Result<Obj, Self::Err>;
}

pub struct CachedSummoner<Obj, S: Summoner<Obj>> {
    summoner: S,
    pub cache: RefCell<HashMap<S::Id, Rc<Obj>>>,
}

impl<Obj, S: Summoner<Obj, Id: ?Sized>> Summoner<Rc<Obj>> for CachedSummoner<Obj, S>
where
    S::Id: Hash + Eq + Clone,
{
    type Id = S::Id;
    type Err = S::Err;

    fn summon(&self, id: Self::Id) -> Result<Rc<Obj>, Self::Err> {
        if let Some(obj) = self.cache.borrow().get(&id) {
            return Ok(obj.clone());
        }

        let obj = Rc::new(self.summoner.summon(id.clone())?);
        self.cache.borrow_mut().insert(id, obj.clone());
        Ok(obj)
    }
}
