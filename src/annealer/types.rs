use crate::utils::rnd::Rnd;

pub trait State<E>: Clone
where
    E: Env,
{
    /// スコアを取得する関数
    fn get_score(&mut self, env: &E, progress: f64) -> f64;
}

pub trait Env {}

pub trait NeighborHandler {
    type State: State<Self::Env>;
    type Env: Env;

    fn tag(&self) -> &'static str;
    fn apply(&mut self, state: &mut Self::State, env: &Self::Env, rnd: &mut Rnd) -> bool;
    fn revert(&mut self, state: &mut Self::State, env: &Self::Env, rnd: &mut Rnd);
}

pub trait NeighborType {
    type H: NeighborHandler;
    fn setup(&self) -> Self::H;
}

pub trait NeighborGenerator<N>
where
    N: NeighborType,
{
    fn generate(&self, progress: f64, rnd: &mut Rnd) -> N::H;
}

pub trait Criterion {
    fn adopt(
        &self,
        cur_score: f64,
        new_score: f64,
        cur_temp: f64,
        progress: f64,
        rnd: &mut Rnd,
    ) -> bool;
}

pub trait TemperatureScheduler {
    fn get_temp(&self, progress: f64) -> f64;
}

pub trait ProgressScheduler {
    fn start(&mut self) {}
    fn step(&mut self) {}
    fn get_progress(&self) -> f64;
}

pub trait Callback<S, E>
where
    S: State<E>,
    E: Env,
{
    fn on_before_step(&mut self, _step: usize, _progreess: f64, _state: &mut S, _env: &E) {}
    fn on_after_step(&mut self, _step: usize, _progreess: f64, _state: &mut S, _env: &E) {}
}

#[macro_export]
macro_rules! neighbor_impl {
    ($state_type:ident, $env_type:ident, $($variant:ident),+) => {
        enum NeighborHandlerImpl {
            $($variant($variant),)+
        }

        impl $crate::annealer::types::NeighborHandler for NeighborHandlerImpl {
            type Env = $env_type;
            type State = $state_type;

            fn tag(&self) -> &'static str {
                match self {
                    $(Self::$variant(inner) => inner.tag(),)+
                }
            }

            fn apply(&mut self, state: &mut $state_type, env: &$env_type, rnd: &mut Rnd) -> bool {
                match self {
                    $(Self::$variant(inner) => inner.apply(state, env, rnd),)+
                }
            }

            fn revert(&mut self, state: &mut $state_type, env: &$env_type, rnd: &mut Rnd) {
                match self {
                    $(Self::$variant(inner) => inner.revert(state, env, rnd),)+
                }
            }
        }

        enum Neighbor {
            $($variant,)+
        }

        impl $crate::annealer::types::NeighborType for Neighbor {
            type H = NeighborHandlerImpl;

            fn setup(&self) -> Self::H {
                match self {
                    $(Self::$variant => NeighborHandlerImpl::$variant($variant::setup()),)+
                }
            }
        }
    };
}
