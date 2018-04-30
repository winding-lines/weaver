use ::weaver_db::RealStore;
use gotham::handler::HandlerFuture;
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::State;
use std::io;
use std::sync::Arc;
use super::StoreData;

pub struct StoreMiddleware {
    store: Arc<RealStore>
}


pub struct StoreMiddlewareImpl {
    data: StoreData,
}

impl StoreMiddleware {
    pub fn new(store: Arc<RealStore>) -> StoreMiddleware {
        StoreMiddleware {
            store
        }
    }
}

impl Middleware for StoreMiddlewareImpl {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
        where Chain: FnOnce(State) -> Box<HandlerFuture> + 'static
    {
        state.put::<StoreData>(self.data);
        chain(state)
    }
}

impl NewMiddleware for StoreMiddleware {
    type Instance = StoreMiddlewareImpl;

    fn new_middleware(&self) -> io::Result<StoreMiddlewareImpl> {
        let epic = self.store.epic()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "bad epic"))?;
        Ok(StoreMiddlewareImpl {
            data: StoreData {
                destination: self.store.destination(),
                epic,
            }
        })
    }
}
