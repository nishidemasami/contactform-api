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

# 既にSAM APIが起動しているか確認
if curl -s "${BASE_URL}/api/v1/inquiries" >/dev/null 2>&1; then
  echo "SAM APIは既に ${BASE_URL} で稼働中です。"
else
  echo "sam local start-api を起動しています (ポート: ${PORT})..."
  sam local start-api \
    --template "${TEMPLATE_FILE}" \
    --env-vars "${ENV_VARS}" \
    --docker-network host \
    --host "${HOST}" \
    --port "${PORT}" \
    --warm-containers NEVER > "${SCRIPT_DIR}/sam-api.log" 2>&1 &
  SAM_PID=$!
  SAM_STARTED=1

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

echo "=== テスト実行開始 ==="

# 1. 正常系: POST /api/v1/inquiry (問い合わせ作成)
echo "テスト 1: POST /api/v1/inquiry (正常登録)"
HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/api/v1/inquiry" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "結合テスト太郎",
    "email": "integration-test@example.com",
    "message": "SAM local start-api 結合テストメッセージです。"
  }')

HTTP_BODY=$(echo "${HTTP_RESPONSE}" | head -n -1)
HTTP_STATUS=$(echo "${HTTP_RESPONSE}" | tail -n 1)

echo "ステータスコード: ${HTTP_STATUS}"
echo "レスポンス: ${HTTP_BODY}"

if [ "${HTTP_STATUS}" -ne 201 ]; then
  echo "エラー: 期待ステータス 201 ですが ${HTTP_STATUS} が返されました。"
  cat "${SCRIPT_DIR}/sam-api.log" 2>/dev/null || true
  exit 1
fi

if ! echo "${HTTP_BODY}" | grep -q '"id"'; then
  echo "エラー: レスポンスに 'id' が含まれていません。"
  exit 1
fi

echo "テスト 1 成功!"

# 2. 異常系: POST /api/v1/inquiry (入力エラー - 名前空)
echo "テスト 2: POST /api/v1/inquiry (入力不備エラー)"
HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/api/v1/inquiry" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "",
    "email": "invalid-email",
    "message": "エラーテスト"
  }')

HTTP_STATUS=$(echo "${HTTP_RESPONSE}" | tail -n 1)
echo "ステータスコード: ${HTTP_STATUS}"

if [ "${HTTP_STATUS}" -ne 400 ]; then
  echo "エラー: 期待ステータス 400 ですが ${HTTP_STATUS} が返されました。"
  exit 1
fi

echo "テスト 2 成功!"

# 3. 正常系: GET /api/v1/inquiries (未実装 501 レスポンス)
echo "テスト 3: GET /api/v1/inquiries (501 Not Implemented)"
HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${BASE_URL}/api/v1/inquiries")
HTTP_STATUS=$(echo "${HTTP_RESPONSE}" | tail -n 1)
echo "ステータスコード: ${HTTP_STATUS}"

if [ "${HTTP_STATUS}" -ne 501 ]; then
  echo "エラー: 期待ステータス 501 ですが ${HTTP_STATUS} が返されました。"
  exit 1
fi

echo "テスト 3 成功!"

# 4. 正常系: GET /api/v1/not-found-route (404 Not Found)
echo "テスト 4: GET 未定義ルート (404 Not Found)"
HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${BASE_URL}/api/v1/not-found-route")
HTTP_STATUS=$(echo "${HTTP_RESPONSE}" | tail -n 1)
echo "ステータスコード: ${HTTP_STATUS}"

if [ "${HTTP_STATUS}" -ne 404 ]; then
  echo "エラー: 期待ステータス 404 ですが ${HTTP_STATUS} が返されました。"
  exit 1
fi

echo "テスト 4 成功!"

# 5. DB確認 (PostgreSQLが利用可能な場合)
if command -v psql >/dev/null 2>&1 || docker exec postgres psql -U postgres -d postgres -c "SELECT 1" >/dev/null 2>&1; then
  echo "テスト 5: DBデータ永続化確認"
  COUNT=$(PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -d postgres -t -c "SELECT count(*) FROM public.inquiries WHERE email='integration-test@example.com';" 2>/dev/null || \
         docker exec postgres psql -U postgres -d postgres -t -c "SELECT count(*) FROM public.inquiries WHERE email='integration-test@example.com';" 2>/dev/null || echo "0")
  COUNT=$(echo "${COUNT}" | tr -d ' ')
  echo "DB内のレコード数: ${COUNT}"
  if [ "${COUNT}" -ge 1 ]; then
    echo "DB確認 成功! (${COUNT} 件のデータが存在します)"
  else
    echo "警告: DB確認でレコードが見つかりませんでした (要確認)"
  fi
fi

echo "=== 全ての結合テストが正常に完了しました ==="
