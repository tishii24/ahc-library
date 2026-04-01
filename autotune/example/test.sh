set -euo pipefail

STUDY_PREFIX="example_$(date +%s)"
echo "Using study prefix: ${STUDY_PREFIX}"

pip install optuna
cargo install pahcer
uv tool install --no-cache ../../ahc-utils --force
cargo install --path ..

# 1. params_impl!を生成する
generate_impl --optuna_config_path optuna_config.yaml

# 2. 生成されたコードをexample/src/main.rsにコピーして、パラメータの定義を行う
# params_impl! {
#     n_coef: f64 = 5.0,
#     m_coef: f64 = 5.0,
# }

# pahcer-optunaを使う場合
pahcer-optuna --study_name ${STUDY_PREFIX}_p --optuna_config_path optuna_config.yaml
generate_impl --optuna_config_path optuna_config.yaml --study_name ${STUDY_PREFIX}_p

autotune --config_path autotune_config.yaml --optuna_study_prefix ${STUDY_PREFIX}

optuna best-trial --study-name ${STUDY_PREFIX}_p --storage sqlite:///optuna.db -f json
optuna best-trial --study-name ${STUDY_PREFIX}_0 --storage sqlite:///optuna.db -f json
optuna best-trial --study-name ${STUDY_PREFIX}_1 --storage sqlite:///optuna.db -f json
optuna best-trial --study-name ${STUDY_PREFIX}_2 --storage sqlite:///optuna.db -f json
optuna best-trial --study-name ${STUDY_PREFIX}_3 --storage sqlite:///optuna.db -f json
