use ::store::RealStore;
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::state::State;

#[derive(NewMiddleware, Copy, Clone, Default)]
pub struct StoreMiddleware;

impl Middleware for StoreMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
        where Chain: FnOnce(State) -> Box<HandlerFuture> + 'static
    {
        state.put(RealStore::new().expect("store"));
        chain(state)
    }
}


