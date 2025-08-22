## アルゴリズム

- annealer.rs
- beamsearch.rs

## データ構造

- indexset.rs
- fastcleararray.rs
    - 1d, 2d
- 2darray.rs
- pqueue.rs
    - https://github.com/tidwall/pqueue/blob/master/src/lib.rs

## TODO:

- Op
- Callbackの実装
    - ベスト解出力
- Loggerの実装
    - スコア遷移
- キック
    - callback + kick近傍
- デバッグモード
    - 差分計算のチェック
    - ロガーの無効化
- 時間の取得間隔を設定
- PhantomDataを使ってtype Stateあたりを綺麗にする
- Neighborマクロを綺麗にする

- 近傍確率の時間に応じた調整
- 近傍ごとの受諾確率調整
- [焼きなまし法での評価関数の打ち切り](https://qiita.com/not522/items/cd20b87157d15850d31c)

- 温度自動調整