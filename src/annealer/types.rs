use crate::utils::rnd::Rnd;

pub trait State<E>
where
    E: Env,
{
    /// `annealer`では最初のイテレーション以外はスコアの取得にこちらを使用される
    /// 差分計算などでスコアを効率的に計算できる場合は、この関数を再定義する
    fn get_score(&mut self, env: &E) -> f64 {
        self.calc_score(env)
    }

    /// スコアの完全な計算を行う
    fn calc_score(&mut self, env: &E) -> f64;
}

pub trait Env {}

pub trait NeighborHandler {
    type State: State<Self::Env>;
    type Env: Env;

    fn tag(&self) -> &'static str;
    fn apply(&mut self, state: &mut Self::State, env: &Self::Env, rnd: &mut Rnd);
    fn revert(&mut self, state: &mut Self::State, env: &Self::Env, rnd: &mut Rnd);
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

pub trait Scheduler {
    fn get_temp(&self, progress: f64) -> f64;
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

            fn apply(&mut self, state: &mut $state_type, env: &$env_type, rnd: &mut Rnd) {
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
