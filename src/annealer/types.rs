pub trait State<E>
where
    E: Env,
{
    fn calc_score(&self, env: &E) -> f64;
}

pub trait Env {}

pub trait NeighborHandler {
    type State: State<Self::Env>;
    type Env: Env;

    fn tag(&self) -> &'static str;
    fn apply(&self, state: &mut Self::State, env: &Self::Env);
    fn revert(&self, state: &mut Self::State, env: &Self::Env);
}

pub trait NeighborType {
    type H: NeighborHandler;
    fn generate(&self) -> Self::H;
}

pub trait NeighborGenerator<N>
where
    N: NeighborType,
{
    fn generate(&self, progress: f64) -> N::H;
}

#[macro_export]
macro_rules! neighbor_impl {
    ($state_type:ident, $env_type:ident, $($variant:ident),+) => {
        enum NeighborHandlerImpl {
            $($variant($variant),)+
        }

        impl crate::annealer::types::NeighborHandler for NeighborHandlerImpl {
            type Env = $env_type;
            type State = $state_type;

            fn tag(&self) -> &'static str {
                match self {
                    $(Self::$variant(inner) => inner.tag(),)+
                }
            }

            fn apply(&self, state: &mut $state_type, env: &$env_type) {
                match self {
                    $(Self::$variant(inner) => inner.apply(state, env),)+
                }
            }

            fn revert(&self, state: &mut $state_type, env: &$env_type) {
                match self {
                    $(Self::$variant(inner) => inner.revert(state, env),)+
                }
            }
        }

        enum Neighbor {
            $($variant,)+
        }

        impl crate::annealer::types::NeighborType for Neighbor {
            type H = NeighborHandlerImpl;

            fn generate(&self) -> Self::H {
                match self {
                    $(Self::$variant => NeighborHandlerImpl::$variant($variant::generate()),)+
                }
            }
        }
    };
}
