## 要件

- コード生成

- gitとの連携
- 入力パラメータに応じたチューニング
- docker
- google cloudを使ったリモート実行

```
pahcer-optuna
    --solution_dir=SOLUTION_DIR
    --work_dir=WORK_DIR
    --config=CONFIG_FILE
```

## 実装

```sh
cp $SOLUTION_DIR $WORK_DIR

uv run optimize.py
```
