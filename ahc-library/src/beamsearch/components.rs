use crate::beamsearch::runner::BeamWidthPolicy;

pub struct FixedBeamWidthPolicy {
    width: usize,
}

impl FixedBeamWidthPolicy {
    pub fn new(width: usize) -> Self {
        FixedBeamWidthPolicy { width }
    }
}

impl BeamWidthPolicy for FixedBeamWidthPolicy {
    fn suggest_width(&self, _: f64, _: usize, _: usize) -> usize {
        self.width
    }
}

pub trait BeamUnitEstimator {
    fn estimate_remain_unit(
        &self,
        time_progress: f64,
        cur_beam_width: usize,
        cur_turn: usize,
        end_turn: usize,
    ) -> usize;
}

pub struct FixedBeamUnitEstimator;

impl BeamUnitEstimator for FixedBeamUnitEstimator {
    fn estimate_remain_unit(
        &self,
        _: f64,
        cur_beam_width: usize,
        cur_turn: usize,
        end_turn: usize,
    ) -> usize {
        (end_turn - cur_turn.min(end_turn)) * cur_beam_width
    }
}

pub struct DynamicBeamWidthPolicy<E>
where
    E: BeamUnitEstimator,
{
    desired_time_sec: f64,
    consumed_unit: usize,
    initial_width: usize,
    min_width: usize,
    max_width: usize,
    unit_estimator: E,
}

impl<E> DynamicBeamWidthPolicy<E>
where
    E: BeamUnitEstimator,
{
    pub fn new(
        desired_time_sec: f64,
        initial_width: usize,
        min_width: usize,
        max_width: usize,
        unit_estimator: E,
    ) -> Self {
        DynamicBeamWidthPolicy {
            desired_time_sec,
            consumed_unit: 0,
            initial_width,
            min_width,
            max_width,
            unit_estimator,
        }
    }
}

impl<E> DynamicBeamWidthPolicy<E>
where
    E: BeamUnitEstimator,
{
    pub fn estimate_remain_unit(
        &self,
        time_progress: f64,
        cur_beam_width: usize,
        cur_turn: usize,
        end_turn: usize,
    ) -> usize {
        self.unit_estimator
            .estimate_remain_unit(time_progress, cur_beam_width, cur_turn, end_turn)
    }
}

impl<E> BeamWidthPolicy for DynamicBeamWidthPolicy<E>
where
    E: BeamUnitEstimator,
{
    fn suggest_width(&self, time_progress: f64, cur_turn: usize, end_turn: usize) -> usize {
        const E: f64 = 0.1;

        if self.consumed_unit == 0 || time_progress < 1e-5 {
            return self.initial_width;
        };

        let available_unit =
            (self.consumed_unit as f64 * (1. - time_progress) / time_progress).floor() as usize;

        // remain_unit * (1 + E) < available_unit を満たす最大の`beam_width`を求める
        let width = {
            let (mut l, mut r) = (self.min_width, self.max_width + 1);
            while r - l > 1 {
                let w = (r + l) / 2;
                let remain_unit = self.estimate_remain_unit(time_progress, w, cur_turn, end_turn);

                if remain_unit as f64 * (1. + E) < available_unit as f64 {
                    l = w;
                } else {
                    r = w;
                }
            }

            l
        };

        width.clamp(self.min_width, self.max_width)
    }

    fn end_unit(&mut self) {
        self.consumed_unit += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::beamsearch::{
        components::{
            BeamUnitEstimator, DynamicBeamWidthPolicy, FixedBeamUnitEstimator, FixedBeamWidthPolicy,
        },
        runner::BeamWidthPolicy,
    };

    #[test]
    fn test_fixed_beam_unit_estimator() {
        const END_TURN: usize = 5;
        const W: usize = 5;
        let estimator = FixedBeamUnitEstimator;
        assert_eq!(estimator.estimate_remain_unit(0.0, W, 0, END_TURN), 25);
        assert_eq!(estimator.estimate_remain_unit(0.5, W, 2, END_TURN), 15);
        assert_eq!(estimator.estimate_remain_unit(1.0, W, 5, END_TURN), 0);
        assert_eq!(estimator.estimate_remain_unit(1.0, W, 10, END_TURN), 0);
    }

    #[test]
    fn test_fixed_beam_width_policy() {
        let policy = FixedBeamWidthPolicy::new(5);
        assert_eq!(policy.suggest_width(0.0, 0, 10), 5);
        assert_eq!(policy.suggest_width(0.5, 5, 10), 5);
        assert_eq!(policy.suggest_width(1.0, 10, 10), 5);
        assert_eq!(policy.suggest_width(1.1, 10, 10), 5);
    }

    #[test]
    fn test_dynamic_beam_width_policy() {
        const W: usize = 5;
        const END_TURN: usize = 4;
        let estimator = FixedBeamUnitEstimator;
        let mut policy = DynamicBeamWidthPolicy::new(1.0, W, 1, 10, estimator);
        assert_eq!(policy.suggest_width(0.0, 0, END_TURN), W);
        for _ in 0..W {
            policy.end_unit();
        }
        assert_eq!(policy.suggest_width(0.25, 1, END_TURN), 4);
        assert_eq!(policy.suggest_width(0.01, 1, END_TURN), 10);
        for _ in 0..W {
            policy.end_unit();
        }
        assert_eq!(policy.suggest_width(1.0, 2, END_TURN), 1);
        assert_eq!(policy.suggest_width(1.1, 2, END_TURN), 1);
    }
}
