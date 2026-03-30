# Autotune: Automatically tune hyperparameters for Atcoder Heuristic Contest

## 1. 準備

```bash
pip install optuna
cargo install pahcer
uv tool install --no-cache ahc-utils --force
cargo install .
```

## 2. 設定ファイルの作成

- autotune_config.yaml
- optuna_config.yaml

## 3. 実行

```shell
# 1. params_impl!を生成する
generate-impl --config optuna_config.yaml

# 2. 解法のsrcに実装をコピーする

# 3.a pahcer-optunaを使う場合
pahcer-optuna --study-name study-optuna --config optuna_config.yaml --storage_path=sqlite:///optuna.db

# 3.b autotuneを使う場合
autotune --config_path autotune_config.yaml --optuna_study_prefix study-autotune

# 4. 得られた出力を解法にコピーする
```
