# /// script
# dependencies = [
#   "pyyaml==6.0.1",
#   "classopt==0.2.1",
# ]
# ///

import classopt
import yaml


@classopt.classopt(default_long=True)
class Args:
    config_path: str = classopt.config(
        "--config", required=True, help="Path to config file"
    )


def type_to_rust_type(t: str) -> str:
    if t == "int":
        return "i64"
    elif t == "float":
        return "f64"
    else:
        raise ValueError(f"Unsupported type: {t}")


def main() -> None:
    args = Args.from_args()  # type: ignore

    with open(args.config_path, "r") as file:
        config = yaml.safe_load(file)

    content = ""
    content += "params_impl! {\n"
    for d in config["params"]:
        t = type_to_rust_type(d["type"])
        content += f'\t{d["name"]}: {t} = {d["default"]},\n'
    content += "}\n\n"

    print(content)


if __name__ == "__main__":
    main()
