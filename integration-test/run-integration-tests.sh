#!/usr/bin/env bash
set -euo pipefail

# stdout (fd 1) を fd 3 に退避し、stdout を stderr (fd 2) にリダイレクト
exec 3>&1
exec 1>&2

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

TEST_NAMES=()
TEST_STATUSES=()
TEST_TIMES=()
TEST_ERRORS=()

get_time() {
  local t
  t=$(date +%s.%N 2>/dev/null || true)
  if [[ "$t" =~ ^[0-9]+\.[0-9]+$ ]]; then
    echo "$t"
  else
    date +%s
  fi
}

calc_duration() {
  local start="$1"
  local end="$2"
  awk -v s="$start" -v e="$end" 'BEGIN { printf "%.3f", e - s }'
}

xml_escape() {
  local s="$1"
  s="${s//&/&amp;}"
  s="${s//</&lt;}"
  s="${s//>/&gt;}"
  s="${s//\"/&quot;}"
  s="${s//\'/&apos;}"
  echo "$s"
}

record_test() {
  local name="$1"
  local status="$2"
  local duration="$3"
  local err_msg="${4:-}"

  TEST_NAMES+=("$name")
  TEST_STATUSES+=("$status")
  TEST_TIMES+=("$duration")
  TEST_ERRORS+=("$err_msg")
}

generate_junit_xml() {
  local total_tests=${#TEST_NAMES[@]}
  local total_failures=0
  local total_time=0.0

  for ((i=0; i<total_tests; i++)); do
    if [ "${TEST_STATUSES[$i]}" = "failure" ]; then
      total_failures=$((total_failures + 1))
    fi
    total_time=$(awk -v t="$total_time" -v duration="${TEST_TIMES[$i]}" 'BEGIN { printf "%.3f", t + duration }')
  done

  cat <<EOF >&3
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="integration-tests" tests="${total_tests}" failures="${total_failures}" errors="0" time="${total_time}">
  <testsuite name="integration-tests" tests="${total_tests}" failures="${total_failures}" errors="0" time="${total_time}">
EOF

  for ((i=0; i<total_tests; i++)); do
    local name_escaped
    name_escaped=$(xml_escape "${TEST_NAMES[$i]}")
    local time_val="${TEST_TIMES[$i]}"

    if [ "${TEST_STATUSES[$i]}" = "success" ]; then
      cat <<EOF >&3
    <testcase name="${name_escaped}" classname="integration_test" time="${time_val}"/>
EOF
    else
      local err_escaped
      err_escaped=$(xml_escape "${TEST_ERRORS[$i]}")
      cat <<EOF >&3
    <testcase name="${name_escaped}" classname="integration_test" time="${time_val}">
      <failure message="${err_escaped}">${err_escaped}</failure>
    </testcase>
EOF
    fi
  done

  cat <<EOF >&3
  </testsuite>
</testsuites>
EOF
}

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
    --docker-network integration-test-network \
    --host "${HOST}" \
    --port "${PORT}" \
    --warm-containers LAZY > "${SCRIPT_DIR}/sam-api.log" 2>&1 &
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
      record_test "APIサーバー起動" "failure" "0.0" "sam local start-api の起動タイムアウト"
      generate_junit_xml
      exit 1
    fi
    sleep 2
  done
  echo "APIサーバーが正常に起動しました。"
fi

echo "=== テスト実行開始 ==="

HAS_FAILURE=0

# 1. 正常系: POST /api/v1/inquiry (問い合わせ作成)
TEST1_NAME="POST /api/v1/inquiry (正常登録)"
echo "テスト 1: ${TEST1_NAME}"
START_T=$(get_time)

HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/api/v1/inquiry" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "結合テスト太郎",
    "email": "integration-test@example.com",
    "message": "SAM local start-api 結合テストメッセージです。"
  }')

HTTP_BODY=$(echo "${HTTP_RESPONSE}" | head -n -1)
HTTP_STATUS=$(echo "${HTTP_RESPONSE}" | tail -n 1)
END_T=$(get_time)
DUR=$(calc_duration "$START_T" "$END_T")

echo "ステータスコード: ${HTTP_STATUS}"
echo "レスポンス: ${HTTP_BODY}"

if [ "${HTTP_STATUS}" -ne 201 ]; then
  ERR="エラー: 期待ステータス 201 ですが ${HTTP_STATUS} が返されました。"
  echo "${ERR}"
  cat "${SCRIPT_DIR}/sam-api.log" 2>/dev/null || true
  record_test "${TEST1_NAME}" "failure" "$DUR" "${ERR}"
  HAS_FAILURE=1
elif ! echo "${HTTP_BODY}" | grep -q '"id"'; then
  ERR="エラー: レスポンスに 'id' が含まれていません。"
  echo "${ERR}"
  record_test "${TEST1_NAME}" "failure" "$DUR" "${ERR}"
  HAS_FAILURE=1
else
  echo "テスト 1 成功!"
  record_test "${TEST1_NAME}" "success" "$DUR"
fi

# 2. 異常系: POST /api/v1/inquiry (入力エラー - 名前空)
TEST2_NAME="POST /api/v1/inquiry (入力不備エラー)"
echo "テスト 2: ${TEST2_NAME}"
START_T=$(get_time)

HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/api/v1/inquiry" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "",
    "email": "invalid-email",
    "message": "エラーテスト"
  }')

HTTP_STATUS=$(echo "${HTTP_RESPONSE}" | tail -n 1)
END_T=$(get_time)
DUR=$(calc_duration "$START_T" "$END_T")
echo "ステータスコード: ${HTTP_STATUS}"

if [ "${HTTP_STATUS}" -ne 400 ]; then
  ERR="エラー: 期待ステータス 400 ですが ${HTTP_STATUS} が返されました。"
  echo "${ERR}"
  record_test "${TEST2_NAME}" "failure" "$DUR" "${ERR}"
  HAS_FAILURE=1
else
  echo "テスト 2 成功!"
  record_test "${TEST2_NAME}" "success" "$DUR"
fi

# 3. 正常系: GET /api/v1/inquiries (未実装 501 レスポンス)
TEST3_NAME="GET /api/v1/inquiries (501 Not Implemented)"
echo "テスト 3: ${TEST3_NAME}"
START_T=$(get_time)

HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${BASE_URL}/api/v1/inquiries")
HTTP_STATUS=$(echo "${HTTP_RESPONSE}" | tail -n 1)
END_T=$(get_time)
DUR=$(calc_duration "$START_T" "$END_T")
echo "ステータスコード: ${HTTP_STATUS}"

if [ "${HTTP_STATUS}" -ne 501 ]; then
  ERR="エラー: 期待ステータス 501 ですが ${HTTP_STATUS} が返されました。"
  echo "${ERR}"
  record_test "${TEST3_NAME}" "failure" "$DUR" "${ERR}"
  HAS_FAILURE=1
else
  echo "テスト 3 成功!"
  record_test "${TEST3_NAME}" "success" "$DUR"
fi

# 4. 正常系: GET /api/v1/not-found-route (404 Not Found)
TEST4_NAME="GET 未定義ルート (404 Not Found)"
echo "テスト 4: ${TEST4_NAME}"
START_T=$(get_time)

HTTP_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${BASE_URL}/api/v1/not-found-route")
HTTP_STATUS=$(echo "${HTTP_RESPONSE}" | tail -n 1)
END_T=$(get_time)
DUR=$(calc_duration "$START_T" "$END_T")
echo "ステータスコード: ${HTTP_STATUS}"

if [ "${HTTP_STATUS}" -ne 404 ]; then
  ERR="エラー: 期待ステータス 404 ですが ${HTTP_STATUS} が返されました。"
  echo "${ERR}"
  record_test "${TEST4_NAME}" "failure" "$DUR" "${ERR}"
  HAS_FAILURE=1
else
  echo "テスト 4 成功!"
  record_test "${TEST4_NAME}" "success" "$DUR"
fi

# 5. DB確認 (PostgreSQLが利用可能な場合)
TEST5_NAME="DBデータ永続化確認"
if command -v psql >/dev/null 2>&1 || docker exec postgres psql -U postgres -d postgres -c "SELECT 1" >/dev/null 2>&1; then
  echo "テスト 5: ${TEST5_NAME}"
  START_T=$(get_time)
  COUNT=$(PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -d postgres -t -c "SELECT count(*) FROM public.inquiries WHERE email='integration-test@example.com';" 2>/dev/null || \
         docker exec postgres psql -U postgres -d postgres -t -c "SELECT count(*) FROM public.inquiries WHERE email='integration-test@example.com';" 2>/dev/null || echo "0")
  COUNT=$(echo "${COUNT}" | tr -d ' ')
  END_T=$(get_time)
  DUR=$(calc_duration "$START_T" "$END_T")
  echo "DB内のレコード数: ${COUNT}"
  if [ "${COUNT}" -ge 1 ]; then
    echo "DB確認 成功! (${COUNT} 件のデータが存在します)"
    record_test "${TEST5_NAME}" "success" "$DUR"
  else
    ERR="エラー: DB確認でレコードが見つかりませんでした (0件)"
    echo "${ERR}"
    record_test "${TEST5_NAME}" "failure" "$DUR" "${ERR}"
    HAS_FAILURE=1
  fi
fi

echo "=== 結合テスト完了 ==="

generate_junit_xml

if [ "${HAS_FAILURE}" -ne 0 ]; then
  echo "エラー: 結合テストで失敗が発生しました。"
  exit 1
fi

echo "=== 全ての結合テストが正常に完了しました ==="
