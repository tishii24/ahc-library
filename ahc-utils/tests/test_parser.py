import pathlib
import tempfile

from ahc_utils.core.parser import ConstantParser


def test_constant_parser() -> None:
    parser = ConstantParser(name="N", index=1, type="usize")
    with tempfile.NamedTemporaryFile(mode="w", delete=True) as f:
        f.write("10 20 30")
        f.flush()

        result = parser.parse(pathlib.Path(f.name))
        assert result == {"N": 20}
