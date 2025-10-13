from ahc_utils.autotune import generate_input_partitions
from ahc_utils.core.input_param import InputParameter
from ahc_utils.core.partition import NumericalInputPartition


def test_generate_input_partitions() -> None:
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
            max=30,
            parser={"type": "constant", "params": {"index": 1}},
            partitions=[10, 20, 30],
        ),
    ]

    partitions = generate_input_partitions(input_params)
    assert len(partitions) == 4
    assert all(
        str(p)
        in {
            "n=0-10_m=10-20",
            "n=0-10_m=20-30",
            "n=10-20_m=10-20",
            "n=10-20_m=20-30",
        }
        for p in partitions
    )


def test_numerical_input_partition_is_included() -> None:
    partition = NumericalInputPartition(
        name="n",
        min_value=0,
        max_value=20,
    )
    assert partition.is_included({"n": 0})
    assert partition.is_included({"n": 5})
    assert partition.is_included({"n": 20})
    assert not partition.is_included({"n": -1})
    assert not partition.is_included({"n": 21})
