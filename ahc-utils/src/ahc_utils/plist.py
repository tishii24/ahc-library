import datetime
import json
import pathlib
import textwrap
from typing import Optional

import classopt
import polars as pl

PAHCER_BEST_SCORES_FILE = "best_scores.json"
PAHCER_JSON_DIR = "json"


RESET = "\033[0m"
GREEN = "\033[32m"


@classopt.classopt(default_long=True)
class Args:
    pahcer_path: pathlib.Path = classopt.config(
        "--pahcer_path",
        help="Path to the pahcer directory containing json and best_scores.json",
        default=pathlib.Path("pahcer"),
    )
    input_parameter_path: pathlib.Path = classopt.config(
        "--input_parameter_path",
        short="-i",
        help="Path to the input parameter json file",
        default=pathlib.Path("input.json"),
    )
    pivot_parameter_name: str = classopt.config(
        "--parameter_name",
        short="-p",
        help="Parameter name to use for pivoting",
        default=None,
    )
    is_maximize: bool = classopt.config(
        "--maximize",
        help="Whether higher scores are better (default: True, i.e. higher is better)",
        default=True,
    )
    n_rows: int = classopt.config(
        "--n_rows",
        short="-n",
        help="Number of recent rows to display",
        default=10,
    )
    case_count: int = classopt.config(
        "--case_count",
        short="-c",
        help="Filter runs by case count",
        default=None,
    )
    max_comment_width: int = classopt.config(
        "--max_comment_width",
        help="Maximum width of comment column (in characters)",
        default=40,
    )


def load_best_scores(best_score_path: pathlib.Path) -> pl.DataFrame:
    with open(best_score_path) as f:
        best_scores = json.load(f)
    return pl.DataFrame([{"seed": k, "best_score": v} for k, v in best_scores.items()])


def load_cases(
    pahcer_cases_dir: pathlib.Path, case_count: Optional[int]
) -> pl.DataFrame:
    data = []
    for path in pahcer_cases_dir.glob("*.json"):
        with path.open() as f:
            d = json.load(f)
            if case_count is not None and d["case_count"] != case_count:
                continue
            if not d["comment"]:
                continue

            for case in d["cases"]:
                data.append(
                    {
                        "start_time": datetime.datetime.fromisoformat(d["start_time"]),
                        "comment": d["comment"],
                        "tag": d["tag_name"] if d["tag_name"] else "",
                        "seed": f"{case['seed']:04}",
                        "score": case["score"],
                        "execution_time": case["execution_time"],
                    }
                )
    return pl.DataFrame(data, infer_schema_length=10_000)


def build_summary(
    cases_df: pl.DataFrame,
    best_scores_df: pl.DataFrame,
    input_df: pl.DataFrame,
    pivot_parameter_name: str | None,
) -> pl.DataFrame:
    df = (
        cases_df.join(best_scores_df, on="seed")
        .with_columns((pl.col("score") / pl.col("best_score")).alias("rel"))
        .sort("rel", descending=True)
        .join(input_df, on="seed")
    )

    summary = df.group_by("start_time").agg(
        pl.mean("score").alias("abs"),
        pl.mean("rel"),
        pl.first("tag"),
        pl.first("comment"),
    )

    if pivot_parameter_name is not None:
        pivot_df = df.pivot(
            index="start_time",
            on=pivot_parameter_name,
            values="rel",
            aggregate_function="mean",
            sort_columns=True,
        ).sort("start_time", descending=True)
        summary = summary.join(pivot_df, on="start_time")

    # 列順: start_time, absolute, relative, [pivot cols], tag, comment
    str_tail = ["tag", "comment"]
    mid_cols = [c for c in summary.columns if c not in {"start_time"} | set(str_tail)]
    summary = summary.select(["start_time"] + mid_cols + str_tail)

    return summary


def _build_display_df(
    summary: pl.DataFrame, score_cols: list[str], n_rows: int
) -> pl.DataFrame:
    """最新n件 + 各スコア列でベストな行 (重複除去) を返す。先頭にbest行を付加。"""
    recent_times = set(
        summary.sort("start_time", descending=True).head(n_rows)["start_time"].to_list()
    )

    # 最新n件に含まれないベスト行を収集
    extra_times: set = set()
    extra_parts: list[pl.DataFrame] = []
    for col in score_cols:
        col_max = summary[col].max()
        if col_max is None:
            continue
        best_row = summary.filter(pl.col(col) == col_max).head(1)
        st = best_row["start_time"][0]
        if st not in recent_times and st not in extra_times:
            extra_parts.append(best_row)
            extra_times.add(st)

    recent_df = summary.sort("start_time", descending=True).head(n_rows)
    body_df = (
        pl.concat([recent_df] + extra_parts).sort("start_time", descending=True)
        if extra_parts
        else recent_df
    )

    # start_time を文字列化
    body_df = body_df.with_columns(
        pl.col("start_time").dt.strftime("%m/%d %H:%M").alias("start_time")
    )

    # best行: 各スコア列の全体最大値を1行にまとめて先頭に挿入
    str_tail = [c for c in summary.columns if c in {"tag", "comment"}]
    best_row_df = summary.select(
        [pl.lit("best").alias("start_time")]
        + [pl.col(c).max() for c in score_cols]
        + [pl.lit("").alias(c) for c in str_tail]
    )
    return pl.concat([best_row_df, body_df])


def _fmt(col: str, score_cols: list[str], val: object) -> str:
    """値を表示用文字列にフォーマットする。"""
    if col not in score_cols:
        return str(val)
    if val is None:
        return "None"
    assert isinstance(val, float)
    return f"{val:.0f}" if col == "abs" else f"{val:.4f}"


def _render_row(
    r: dict,
    cols: list[str],
    score_cols: list[str],
    col_widths: dict[str, int],
    best_vals: dict[str, float],
    max_comment_width: int,
) -> list[str]:
    """1行分のセルリストを返す"""
    comment_lines = (
        textwrap.wrap(str(r.get("comment") or ""), max_comment_width)
        if "comment" in r
        else []
    ) or [""]

    def make_line(line_idx: int) -> str:
        parts = []
        for col in cols:
            w = col_widths[col]
            if col == "comment":
                parts.append(
                    comment_lines[line_idx].ljust(w)
                    if line_idx < len(comment_lines)
                    else " " * w
                )
            elif line_idx > 0:
                parts.append(" " * w)
            else:
                text = _fmt(col, score_cols, r[col])
                cell = text.rjust(w) if col in score_cols else text.ljust(w)
                if col in best_vals and r[col] is not None and r[col] == best_vals[col]:
                    cell = GREEN + cell + RESET
                parts.append(cell)
        return "  ".join(parts)

    return [make_line(i) for i in range(len(comment_lines))]


def print_table(
    summary: pl.DataFrame, n_rows: int, max_comment_width: int, is_maximize: bool
) -> None:
    score_cols = [
        c for c in summary.columns if c not in {"start_time", "tag", "comment"}
    ]

    display_df = _build_display_df(summary, score_cols, n_rows)

    # 全summaryから各スコア列のベスト値を取得
    if is_maximize:
        best_vals: dict[str, float] = {
            col: v for col in score_cols if (v := summary[col].max()) is not None  # type: ignore
        }
    else:
        best_vals = {
            col: v for col in score_cols if (v := summary[col].min()) is not None  # type: ignore
        }

    rows = display_df.to_dicts()
    cols = display_df.columns
    col_widths: dict[str, int] = {
        c: (
            min(
                max_comment_width,
                max(
                    len(c),
                    max((len(_fmt(c, score_cols, r[c])) for r in rows), default=0),
                ),
            )
            if c == "comment"
            else max(
                len(c), max((len(_fmt(c, score_cols, r[c])) for r in rows), default=0)
            )
        )
        for c in cols
    }

    # ヘッダー
    header = "  ".join(
        c.rjust(col_widths[c]) if c in score_cols else c.ljust(col_widths[c])
        for c in cols
    )
    print(header)
    print("-" * len(header))

    # 各行
    for r in rows:
        for line in _render_row(
            r, cols, score_cols, col_widths, best_vals, max_comment_width
        ):
            print(line)


def main() -> None:
    args = Args.from_args()  # type: ignore

    best_scores_df = load_best_scores(args.pahcer_path / PAHCER_BEST_SCORES_FILE)
    cases_df = load_cases(args.pahcer_path / PAHCER_JSON_DIR, args.case_count)
    input_df = pl.read_json(args.input_parameter_path)

    summary = build_summary(
        cases_df, best_scores_df, input_df, args.pivot_parameter_name
    )
    if args.pivot_parameter_name is not None:
        print(f"Pivoted by '{args.pivot_parameter_name}'")
    print_table(summary, args.n_rows, args.max_comment_width, args.is_maximize)


if __name__ == "__main__":
    main()
