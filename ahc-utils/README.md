## Install

```
uv tool install --no-cache ../../ahc-utils --force
```

## 使い方

このパッケージでは主に `generate-impl`、`pahcer-optuna`、`list` を使います。

### 前提

- `config_example.yaml` のような設定ファイルを用意する
- `optuna.settings.storage_path` に Optuna の SQLite ファイルを指定する
- `pahcer-optuna` と `list` を使う場合は `pahcer` コマンドと、その実行結果一式を利用できる状態にしておく

設定例:

```yaml
optuna:
  settings:
    storage_path: optuna.db
    direction: maximize
    score_type: relative
  pruner:
    threshold: 0.02
  params:
    - name: START_TEMP
      type: float
      default: 10.0
      min: 1.0
      max: 10000.0
    - name: END_TEMP
      type: float
      default: 1e-1
      min: 1e-3
      max: 1e-1
```

### `generate_impl`

CLI コマンドは `generate-impl` です。

Optuna のベストパラメータ、または設定ファイル内の `default` 値から `params_impl!` を生成します。

#### 実行例

設定ファイルの `default` 値だけで生成する場合:

```
generate-impl --config config.yaml
```

既存 Study のベスト値を使って生成する場合:

```
generate-impl --config config.yaml --study-name my-study
```

#### 出力例

```rust
params_impl! {
    START_TEMP: f64 = 1234.5,
    END_TEMP: f64 = 0.01,
}
```

`--study-name` を指定しても Study が見つからない場合は、各パラメータの `default` 値が使われます。

### `pahcer_optuna`

CLI コマンドは `pahcer-optuna` です。

`pahcer run --json` を呼び出しながら Optuna でパラメータ探索を行います。終了時には、その時点のベスト値から生成した `params_impl!` も表示します。

#### 実行例

```
pahcer-optuna --study-name my-study --config config.yaml
```

タイムアウトを 30 分にする場合:

```
pahcer-optuna --study-name my-study --config config.yaml --timeout 1800
```

#### 主な挙動

- `optuna.params` に書かれたパラメータを探索する
- `optuna.ignore_params` に含まれるパラメータは固定値 `default` のまま使う
- `score_type` に応じて `score` / `relative_score` / `log10(score)` を最適化する
- 初回 Trial 以外は `pahcer run` に `--no-compile` を付ける
- Prune された Trial も途中までの平均スコアを Optuna に反映する

#### よくある流れ

1. `pahcer-optuna` で探索する
2. その Study 名を指定して `generate-impl` を実行する
3. 出力された `params_impl!` を Rust 側に貼り付ける

### `list`

`list` は `pahcer` の実行結果を集計して見やすく表示するコマンドです。

デフォルトでは以下を読みます。

- `pahcer/best_scores.json`
- `pahcer/json/*.json`
- `input.json`

#### 実行例

最新 10 件を表示:

```
list
```

入力パラメータ `N` ごとにピボットして表示:

```
list -p N
```

最新 30 件だけ表示:

```
list -n 30
```

ケース数 100 の結果だけ表示:

```
list -c 100
```

`pahcer` ディレクトリと `input.json` の場所を変える場合:

```
list --pahcer_path ./tmp/pahcer --input_parameter_path ./tmp/input.json
```

#### 表示内容

- `abs`: 平均絶対スコア
- `rel`: `best_score` で割った平均相対スコア
- `tag`: pahcer のタグ
- `comment`: 実行時コメント

`-p/--parameter_name` を指定すると、その入力パラメータ値ごとの平均 `rel` も横持ちで表示します。
