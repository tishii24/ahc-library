set -euo pipefail

cargo install --path ../ahc-tools
mkdir -p src/bin
expander_rs --ahc_library_path ../ahc-library > src/bin/expanded.rs

cargo check --bin expanded

echo "ok"
