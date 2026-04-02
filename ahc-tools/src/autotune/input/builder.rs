use std::collections::{HashMap, HashSet};

use anyhow::Result;
use tracing::info;

use crate::autotune::input::{generator::InputGenerator, grouper::InputGrouper};

pub struct InputBuilder<GE: InputGenerator, GR: InputGrouper> {
    generator: GE,
    grouper: GR,
}

impl<GE: InputGenerator, GR: InputGrouper> InputBuilder<GE, GR> {
    pub fn new(generator: GE, grouper: GR) -> Self {
        Self { generator, grouper }
    }

    fn process_seed_range(
        &self,
        start_seed: u64,
        end_seed: u64,
        group_ids: &mut HashSet<String>,
        group_seeds: &mut HashMap<String, Vec<u64>>,
        case_num_per_group: usize,
        accept_new_groups: bool,
    ) -> Result<()> {
        info!("Processing seeds from {} to {}...", start_seed, end_seed);
        let seeds = (start_seed..end_seed).collect::<Vec<_>>();
        let inputs = self.generator.generate_inputs(&seeds)?;

        for (seed, input) in seeds.into_iter().zip(inputs.into_iter()) {
            let group_id = self.grouper.get_group_id(&input)?;
            if !accept_new_groups && !group_ids.contains(&group_id) {
                continue;
            }

            group_ids.insert(group_id.clone());
            let group = group_seeds.entry(group_id).or_default();
            if group.len() < case_num_per_group {
                group.push(seed);
            }
        }

        Ok(())
    }

    pub fn build_inputs(
        &self,
        case_num_per_group: usize,
        start_seed: u64,
        trial_count: u64,
    ) -> Result<HashMap<String, Vec<u64>>> {
        const CHUNK_SIZE: u64 = 100;

        if case_num_per_group == 0 || trial_count == 0 {
            return Ok(HashMap::new());
        }

        let mut group_ids = HashSet::new();
        let mut group_seeds: HashMap<String, Vec<u64>> = HashMap::new();

        let mut next_seed = start_seed;
        let trial_end_seed = start_seed + trial_count;
        while next_seed < trial_end_seed {
            let chunk_end = (next_seed + CHUNK_SIZE).min(trial_end_seed);
            self.process_seed_range(
                next_seed,
                chunk_end,
                &mut group_ids,
                &mut group_seeds,
                case_num_per_group,
                true,
            )?;

            next_seed = chunk_end;
        }

        info!(
            "Initial processing done. Discovered groups: {:?}",
            group_ids
        );

        while group_ids
            .iter()
            .any(|group_id| group_seeds.get(group_id).map_or(0, Vec::len) < case_num_per_group)
        {
            info!(
                "Some groups have less than {} seeds, continuing to generate more inputs...",
                case_num_per_group
            );
            info!(
                "Current group counts: {:?}",
                group_seeds
                    .iter()
                    .map(|(k, v)| (k, v.len()))
                    .collect::<HashMap<_, _>>()
            );

            let chunk_end = next_seed + CHUNK_SIZE;
            self.process_seed_range(
                next_seed,
                chunk_end,
                &mut group_ids,
                &mut group_seeds,
                case_num_per_group,
                false,
            )?;

            next_seed = chunk_end;
        }

        Ok(group_seeds
            .into_iter()
            .filter(|(group_id, seeds)| {
                group_ids.contains(group_id) && seeds.len() >= case_num_per_group
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    struct MockInputGenerator {
        calls: Rc<RefCell<Vec<Vec<u64>>>>,
    }

    impl InputGenerator for MockInputGenerator {
        fn generate_inputs(&self, seeds: &Vec<u64>) -> Result<Vec<String>> {
            self.calls.borrow_mut().push(seeds.clone());
            Ok(seeds.iter().map(|seed| seed.to_string()).collect())
        }
    }

    struct MockInputGrouper;

    impl InputGrouper for MockInputGrouper {
        fn get_group_id(&self, input: &str) -> Result<String> {
            let seed: u64 = input.parse().unwrap();
            let group = match seed {
                0 | 2 => "even-small",
                1 | 3 => "odd-small",
                _ => "outside-trial",
            };
            Ok(group.to_string())
        }
    }

    struct ParityInputGrouper;

    impl InputGrouper for ParityInputGrouper {
        fn get_group_id(&self, input: &str) -> Result<String> {
            let seed: u64 = input.parse().unwrap();
            let group = if seed % 2 == 0 { "even" } else { "odd" };
            Ok(group.to_string())
        }
    }

    struct SparseInputGrouper;

    impl InputGrouper for SparseInputGrouper {
        fn get_group_id(&self, input: &str) -> Result<String> {
            let seed: u64 = input.parse().unwrap();
            let group = match seed {
                0 | 4 | 7 => "slow",
                1 | 2 | 3 => "fast",
                _ => "ignored",
            };
            Ok(group.to_string())
        }
    }

    #[test]
    fn test_build_inputs_collects_seeds_per_group_and_excludes_insufficient_groups() {
        let calls = Rc::new(RefCell::new(vec![]));
        let generator = MockInputGenerator {
            calls: Rc::clone(&calls),
        };
        let grouper = MockInputGrouper;
        let builder = InputBuilder::new(generator, grouper);

        let inputs = builder.build_inputs(2, 0, 4).unwrap();

        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs.get("even-small").unwrap(), &vec![0, 2]);
        assert_eq!(inputs.get("odd-small").unwrap(), &vec![1, 3]);
        assert!(!inputs.contains_key("outside-trial"));
    }

    #[test]
    fn test_build_inputs_starts_from_seed_zero_and_only_reads_first_1000_seeds() {
        let calls = Rc::new(RefCell::new(vec![]));
        let generator = MockInputGenerator {
            calls: Rc::clone(&calls),
        };
        let grouper = ParityInputGrouper;
        let builder = InputBuilder::new(generator, grouper);

        let inputs = builder.build_inputs(3, 0, 1000).unwrap();

        let generated_seeds = calls.borrow().concat();
        assert_eq!(generated_seeds.len(), 1000);
        assert_eq!(generated_seeds.first(), Some(&0));
        assert_eq!(generated_seeds.last(), Some(&999));
        assert_eq!(inputs.get("even").unwrap(), &vec![0, 2, 4]);
        assert_eq!(inputs.get("odd").unwrap(), &vec![1, 3, 5]);
    }

    #[test]
    fn test_build_inputs_continues_past_trial_count_until_discovered_groups_are_filled() {
        let calls = Rc::new(RefCell::new(vec![]));
        let generator = MockInputGenerator {
            calls: Rc::clone(&calls),
        };
        let grouper = SparseInputGrouper;
        let builder = InputBuilder::new(generator, grouper);

        let inputs = builder.build_inputs(3, 0, 5).unwrap();

        assert_eq!(inputs.get("fast").unwrap(), &vec![1, 2, 3]);
        assert_eq!(inputs.get("slow").unwrap(), &vec![0, 4, 7]);

        let generated_seeds = calls.borrow().concat();
        assert!(generated_seeds.len() > 5);
        assert_eq!(generated_seeds.first(), Some(&0));
        assert!(generated_seeds.contains(&7));
    }

    #[test]
    fn test_build_inputs_with_nonzero_start_seed() {
        let calls = Rc::new(RefCell::new(vec![]));
        let generator = MockInputGenerator {
            calls: Rc::clone(&calls),
        };
        let grouper = ParityInputGrouper;
        let builder = InputBuilder::new(generator, grouper);

        let inputs = builder.build_inputs(2, 10, 10).unwrap();

        let generated_seeds = calls.borrow().concat();
        assert_eq!(generated_seeds.len(), 10);
        assert_eq!(generated_seeds.first(), Some(&10));
        assert_eq!(generated_seeds.last(), Some(&19));
        assert_eq!(inputs.get("even").unwrap(), &vec![10, 12]);
        assert_eq!(inputs.get("odd").unwrap(), &vec![11, 13]);
    }
}
