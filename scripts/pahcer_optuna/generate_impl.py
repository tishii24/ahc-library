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
        raise ValueError(f"Unknown type: {t}")


def main() -> None:
    args = Args.from_args()  # type: ignore

    with open(args.config_path, "r") as file:
        config = yaml.safe_load(file)

    content = ""

    content += "#[allow(non_snake_case)]\n"
    content += "struct Params {\n"
    for d in config["params"]:
        t = type_to_rust_type(d["type"])
        content += f'\t{d["name"]}: {t},\n'
    content += "}\n"

    content += "\n"
    content += "impl Params {\n"
    content += "\tfn from_env() -> Self {\n"
    content += "\t\tSelf {\n"
    for d in config["params"]:
        t = type_to_rust_type(d["type"])
        content += (
            f'\t\t\t{d["name"]}: std::env::var("{d["name"]}")'
            f'.unwrap_or("{d["default"]}".to_owned()).parse().unwrap(),\n'
        )

    content += "\t\t}\n"
    content += "\t}\n"
    content += "}\n"

    print(content)


if __name__ == "__main__":
    main()
