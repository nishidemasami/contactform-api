#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

HOST="127.0.0.1"
PORT="${PORT:-3000}"
BASE_URL="http://${HOST}:${PORT}"
ENV_VARS="${SCRIPT_DIR}/env.json"
TEMPLATE_FILE="${PROJECT_ROOT}/.aws-sam/build/template.yaml"

if [ ! -f "${TEMPLATE_FILE}" ]; then
  TEMPLATE_FILE="${PROJECT_ROOT}/api/template.yaml"
fi

echo "=== 結合テスト (sam local start-api) の準備 ==="

SAM_STARTED=0
SAM_PID=""

cleanup() {
  echo "=== 終了処理 ==="
  if [ "${SAM_STARTED}" -eq 1 ] && [ -n "${SAM_PID}" ]; then
    echo "sam local start-api (PID: ${SAM_PID}) を停止しています..."
    kill "${SAM_PID}" 2>/dev/null || true
    wait "${SAM_PID}" 2>/dev/null || true
  fi
}

trap cleanup EXIT

# 依存パッケージのインストール
pip install -r "${SCRIPT_DIR}/requirements.txt" >/dev/null 2>&1 || true

# 既にSAM APIが起動しているか確認
if curl -s "${BASE_URL}/api/v1/inquiries" >/dev/null 2>&1; then
  echo "SAM APIは既に ${BASE_URL} で稼働中です。"
else
  echo "sam local start-api を起動しています (ポート: ${PORT})..."
  cd "${PROJECT_ROOT}/api"
  sam local start-api \
    --template "${TEMPLATE_FILE}" \
    --env-vars "${ENV_VARS}" \
    --docker-network integration-test-network \
    --host "${HOST}" \
    --port "${PORT}" \
    --warm-containers LAZY > "${SCRIPT_DIR}/sam-api.log" 2>&1 &
  SAM_PID=$!
  SAM_STARTED=1

  # 元の作業ディレクトリに戻る
  cd "${SCRIPT_DIR}"

  echo "APIサーバーの起動を待機しています..."
  MAX_RETRIES=30
  RETRY_COUNT=0
  until curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/api/v1/inquiries" | grep -qE "^(200|400|404|405|500|501)$"; do
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ "${RETRY_COUNT}" -gt "${MAX_RETRIES}" ]; then
      echo "エラー: sam local start-api の起動タイムアウト"
      echo "=== sam-api.log ==="
      cat "${SCRIPT_DIR}/sam-api.log" 2>/dev/null || true
      exit 1
    fi
    sleep 2
  done
  echo "APIサーバーが正常に起動しました。"
fi

echo "=== Pytestによる結合テスト実行開始 ==="
export BASE_URL="${BASE_URL}"
pytest "${SCRIPT_DIR}/test_api.py" --alluredir="${PROJECT_ROOT}/allure-results"
echo "=== 全ての結合テストが正常に完了しました ==="
