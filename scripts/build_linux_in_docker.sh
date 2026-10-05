#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

OUTPUT_DIR="$PROJECT_ROOT/target/linux-dist"
mkdir -p "$OUTPUT_DIR"

echo "=== Building Jericho Spreadsheet Linux packages via Docker ==="
cd "$PROJECT_ROOT"

# Use BuildKit to extract artifacts directly to host directory
DOCKER_BUILDKIT=1 docker build \
    --file Dockerfile.linux \
    --target artifacts \
    --output type=local,dest="$OUTPUT_DIR" \
    .

echo ""
echo "=== Linux Build Complete! ==="
echo "Artifacts placed in: $OUTPUT_DIR"
ls -lh "$OUTPUT_DIR"
