from ahc_utils.core.generate_impl import generate_impl, generate_param_impl
from ahc_utils.core.input_param import InputParameter
from ahc_utils.core.optuna_param import OptunaParameter
from ahc_utils.core.partition import InputPartitionGroup, NumericalInputPartition


def test_generate_impl() -> None:
    params = [
        OptunaParameter(
            name="START_TEMP", type="float", default=1000.0, min=0.0, max=2000.0
        ),
        OptunaParameter(
            name="END_TEMP", type="float", default=10.0, min=0.0, max=100.0
        ),
    ]
    best_params = {
        "START_TEMP": 2000.0,
    }

    impl = generate_impl(best_params, params)
    expected_impl = """params_impl! {
    START_TEMP: f64 = 2000.0,
    END_TEMP: f64 = 10.0,
}

"""

    assert impl == expected_impl


def test_generate_param_impl() -> None:
    input_partitions_params = {
        InputPartitionGroup(
            partitions=[
                NumericalInputPartition(name="n", min_value=0, max_value=10),
                NumericalInputPartition(name="m", min_value=10, max_value=20),
            ]
        ): {
            "START_TEMP": 1000.0,
            "END_TEMP": 10.0,
        },
        InputPartitionGroup(
            partitions=[
                NumericalInputPartition(name="n", min_value=10, max_value=20),
                NumericalInputPartition(name="m", min_value=10, max_value=20),
            ]
        ): {
            "START_TEMP": 2000.0,
            "END_TEMP": 20.0,
        },
    }
    input_params = [
        InputParameter(
            name="n",
            type="usize",
            min=0,
            max=20,
            parser={"type": "constant", "params": {"index": 0}},
            partitions=[0, 10, 20],
        ),
        InputParameter(
            name="m",
            type="usize",
            min=10,
            max=20,
            parser={"type": "constant", "params": {"index": 1}},
            partitions=[10, 20],
        ),
    ]
    optuna_params = [
        OptunaParameter(
            name="START_TEMP", type="float", default=1000.0, min=0.0, max=2000.0
        ),
        OptunaParameter(
            name="END_TEMP", type="float", default=10.0, min=0.0, max=100.0
        ),
    ]

    impl = generate_param_impl(input_partitions_params, input_params, optuna_params)
    expected_impl = """params_impl! {
    { n: usize, m: usize },
    { START_TEMP: f64, END_TEMP: f64 },
    [
        ((0)..=(10), (10)..=(20)) => { START_TEMP: 1000.0, END_TEMP: 10.0 },
        ((10)..=(20), (10)..=(20)) => { START_TEMP: 2000.0, END_TEMP: 20.0 },
        _ => { START_TEMP: 1000.0, END_TEMP: 10.0 },
    ]
}

"""

    assert impl == expected_impl
