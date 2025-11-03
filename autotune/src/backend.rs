use std::path::PathBuf;

use anyhow::Result;

use crate::{
    input_param::{InputGenerator, InputGroupBuilder},
    model::AutotuneConfig,
};

trait OptimizerBackend {
    fn build(input_group_builder: InputGroupBuilder<impl InputGenerator>) -> Self;
    fn optimize(&self) -> Result<()>;
}

struct LocalBackend {
    work_dir: PathBuf,
}

impl LocalBackend {
    fn new(config: &AutotuneConfig) {}
}

impl OptimizerBackend for LocalBackend {
    fn build(input_group_builder: InputGroupBuilder<impl InputGenerator>) -> Self {
        todo!()
    }

    fn optimize(&self) -> Result<()> {
        todo!()
    }
}
