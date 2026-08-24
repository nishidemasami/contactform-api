import os
import pytest
import requests
import psycopg2

BASE_URL = os.environ.get("BASE_URL", "http://127.0.0.1:3000")
DB_HOST = os.environ.get("POSTGRES_HOST", "127.0.0.1")
DB_PORT = os.environ.get("POSTGRES_PORT", "5432")
DB_NAME = os.environ.get("POSTGRES_DB", "postgres")
DB_USER = os.environ.get("POSTGRES_USER", "postgres")
DB_PASS = os.environ.get("POSTGRES_PASSWORD", "postgres")


def test_create_inquiry_success():
    """1. 正常系: POST /api/v1/inquiry (問い合わせ作成)"""
    url = f"{BASE_URL}/api/v1/inquiry"
    payload = {
        "name": "結合テスト太郎",
        "email": "integration-test@example.com",
        "message": "SAM local start-api 結合テストメッセージです。",
    }
    headers = {"Content-Type": "application/json"}
    response = requests.post(url, json=payload, headers=headers)
    assert response.status_code == 201, f"Expected 201, got {response.status_code}: {response.text}"
    body = response.json()
    assert "id" in body, "Response body does not contain 'id'"


def test_create_inquiry_validation_error():
    """2. 異常系: POST /api/v1/inquiry (入力エラー - 名前空)"""
    url = f"{BASE_URL}/api/v1/inquiry"
    payload = {
        "name": "",
        "email": "invalid-email",
        "message": "エラーテスト",
    }
    headers = {"Content-Type": "application/json"}
    response = requests.post(url, json=payload, headers=headers)
    assert response.status_code == 400, f"Expected 400, got {response.status_code}: {response.text}"


def test_get_inquiries_not_implemented():
    """3. 正常系: GET /api/v1/inquiries (未実装 501 レスポンス)"""
    url = f"{BASE_URL}/api/v1/inquiries"
    response = requests.get(url)
    assert response.status_code == 501, f"Expected 501, got {response.status_code}: {response.text}"


def test_get_not_found_route():
    """4. 正常系: GET 未定義ルート (404 Not Found)"""
    url = f"{BASE_URL}/api/v1/not-found-route"
    response = requests.get(url)
    assert response.status_code == 404, f"Expected 404, got {response.status_code}: {response.text}"


def test_db_persistence():
    """5. DB確認: 問い合わせデータがDBに正常に登録されているか"""
    try:
        conn = psycopg2.connect(
            host=DB_HOST,
            port=DB_PORT,
            dbname=DB_NAME,
            user=DB_USER,
            password=DB_PASS,
        )
    except Exception as e:
        pytest.skip(f"DB Connection failed: {e}")

    try:
        with conn.cursor() as cur:
            cur.execute(
                "SELECT count(*) FROM public.inquiries WHERE email=%s;",
                ("integration-test@example.com",),
            )
            count = cur.fetchone()[0]
            assert count >= 1, f"Expected at least 1 record in DB, found {count}"
    finally:
        conn.close()
