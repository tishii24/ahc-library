# uv tool install --no-cache ../../ahc-utils --force

# 1. params_impl!を生成する
# generate-impl --config optuna_config.yaml

# 2. 生成されたコードをexample/src/main.rsにコピーして、パラメータの定義を行う
# params_impl! {
#     n_coef: f64 = 5.0,
#     m_coef: f64 = 5.0,
# }

# pahcer-optunaを使う場合
# pahcer-optuna --study_name study-optuna0 --config_path optuna_config.yaml
# generate-impl --config optuna_config.yaml --study_name study-optuna0

autotune --config_path autotune_config.yaml --optuna_study_prefix s0


