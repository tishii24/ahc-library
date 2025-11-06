import os
import pathlib
import re
import subprocess
import tempfile

import classopt


@classopt.classopt(default_long=True)
class Args:
    ahc_library_path: str = classopt.config(
        help="Path to the AHC library directory",
    )
    solution_path: str = classopt.config(
        help="Path to the solution directory",
    )


def extract_modules(lib_file: pathlib.Path) -> list[str]:
    modules = []
    with open(lib_file, "r") as f:
        for line in f.readlines():
            regex = r"pub mod (\w+);"
            captures = re.search(regex, line)

            if not captures:
                continue

            mod_name = captures.group(1)
            if mod_name == "test":
                continue

            modules.append(mod_name)
    return modules


def read_modules_without_test(mod_name: str, file_path: pathlib.Path) -> str:
    content = ""
    content += f"pub mod {mod_name} {{\n"

    with open(file_path, "r") as f:
        is_test = False
        for line in f:
            if (line.startswith("#[test]")) or (line.startswith("#[cfg(test)]")):
                is_test = True
            elif is_test:
                if len(line) >= 1 and line[:1] == "}":
                    is_test = False
            else:
                content += "\t" + line

    content += "}\n\n"

    return content


def replace_macros(content: str) -> str:
    # マクロはcrate::*に置かれるので、特別に置き換える
    # マクロが増えるたびにここに書く必要がある、直したいでござる
    macros = [
        "neighbor_impl",
        "params_impl",
    ]
    for macro in macros:
        content = content.replace(
            f"crate::ahc_library::{macro}",
            f"crate::{macro}",
        )

    return content


def expand_ahc_library(ahc_library_path: pathlib.Path) -> str:
    src_dir = ahc_library_path / "src"
    content = ""

    lib_file = src_dir / "lib.rs"
    modules = extract_modules(lib_file)

    content += "\npub mod ahc_library {"
    for module in modules:
        sub_dir = src_dir / module
        content += f"\npub mod {module} {{"
        mod_file = sub_dir / "mod.rs"

        with open(mod_file, "r") as f:
            for line in f.readlines():
                regex = r"pub mod (\w+);"
                captures = re.search(regex, line)

                if not captures:
                    content += line
                    continue

                mod_name = captures.group(1)
                if mod_name == "test":
                    continue

                content += read_modules_without_test(
                    mod_name, sub_dir / f"{mod_name}.rs"
                )

        content += "}\n"
    content += "}\n"

    content = content.replace("crate::", "crate::ahc_library::")

    return content


def expand_solution(solution_dir: pathlib.Path) -> str:
    src_dir = solution_dir / "src"
    main_file = src_dir / "main.rs"
    content = ""

    with open(main_file, "r") as f:
        for line in f:
            regex = r"mod (\w+);"
            captures = re.search(regex, line)

            if not captures:
                content += line
                continue

            mod_name = captures.group(1)
            content += read_modules_without_test(mod_name, src_dir / f"{mod_name}.rs")

    return content


def apply_rustfmt(content: str) -> str:
    with tempfile.NamedTemporaryFile(
        mode="w+", suffix=".rs", encoding="utf-8"
    ) as temp_file:
        temp_file.write(content)
        temp_file_path = pathlib.Path(temp_file.name)

        subprocess.run(["rustfmt", str(temp_file_path)])
        with open(temp_file_path, "r", encoding="utf-8") as f:
            return f.read()


def main() -> None:
    args = Args.from_args()  # type: ignore

    if args.ahc_library_path is not None:
        ahc_library_path = pathlib.Path(args.ahc_library_path)
    elif os.environ.get("AHC_LIBRARY_PATH") is not None:
        ahc_library_path = pathlib.Path(os.environ["AHC_LIBRARY_PATH"])
    else:
        raise ValueError("AHC library path is not specified")

    if args.solution_path is not None:
        solution_path = pathlib.Path(args.solution_path)
    else:
        solution_path = pathlib.Path(".")

    library = expand_ahc_library(ahc_library_path)

    solution = expand_solution(solution_path)
    solution = solution.replace("use ahc_library::", "use crate::ahc_library::")

    submission = solution + library
    submission = replace_macros(content=submission)
    submission = apply_rustfmt(submission)

    print(submission)


if __name__ == "__main__":
    main()
