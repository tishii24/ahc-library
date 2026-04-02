set -euo pipefail

uv tool install --no-cache ../../ahc-utils --force

python input.py

plist -p n
plist -p m
