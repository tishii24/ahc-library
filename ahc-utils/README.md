## Install

```
uv tool install --no-cache ../../ahc-utils --force
```

## TODO

- 入力パラメータに応じたチューニング
- docker
- google cloudを使ったリモート実行

### input

```yaml
```

0. cp -R . workdir
1. generate inputs
	- seeds.txt=[0, n)で生成
	- cargo run --release --bin gen seeds.txt
2. init run
3. run-optuna
	- save best params
4. generate rust impls
