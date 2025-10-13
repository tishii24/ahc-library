# ahc-library

## Usage

```sh
cargo add --path $PATH_TO_AHC_LIBRARY
uv tool install $PATH_TO_AHC_LIBRARY/ahc_utils
```

```rust
// ahcXXX/src/main.rs
use ahc_library::utils::time;

fn main() {
    time::start_clock();
    println!("Hello with elapsed_time: {:.5}", time::elapsed_seconds());
}
```

```shell
cd ahcXXX
PATH_AHC_LIBRARY=path/to/ahc-library
uv run $PATH_AHC_LIBRARY/scripts/expander/expander.py --ahc_library_path=$PATH_AHC_LIBRARY --solution_path=.
```


## TODO:

- Loggerの実装
    - スコア遷移
    - 解の遷移
- キック
    - callback + kick近傍
- デバッグモード
    - 差分計算のチェック
- PhantomDataを使ってtype Stateあたりを綺麗にする
    - Neighborマクロを綺麗にする
- 時間ごとの近傍の受諾確率の出力
- 近傍確率の時間に応じた調整
- 近傍ごとの受諾確率調整
- [焼きなまし法での評価関数の打ち切り](https://qiita.com/not522/items/cd20b87157d15850d31c)
- Op

- 時間の取得間隔を設定
- 温度自動調整

- beamsearch.rs
- pqueue.rs
    - https://github.com/tidwall/pqueue/blob/master/src/lib.rs

- パラメータ調整をNNでやってくれるやつ
- stopwatch

