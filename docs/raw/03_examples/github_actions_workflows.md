# GitHub Actions ワークフロー YAML 実装例

## DB CI/CD ワークフロー (`.github/workflows/db-cicd.yaml`)抜粋

```yaml
name: contactform-api DB CI/CD

on:
  push:
    branches:
      - develop
      - release
    paths:
      - "db/**"
  pull_request:
    paths:
      - "db/**"

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

jobs:
  validate:
    name: validate
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_DB: postgres
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - name: リポジトリのチェックアウト
        uses: actions/checkout@v7

      - name: Liquibaseでローカルマイグレーション
        run: |
          docker run --rm \
            --network host \
            -v "${{ github.workspace }}:/workspace" \
            liquibase/liquibase:4.33.0 \
            --search-path=/workspace \
            update \
            --changelog-file=db/liquibase/changelog.xml \
            --context-filter=local \
            --url=jdbc:postgresql://localhost:5432/postgres \
            --username=postgres \
            --password=postgres \
            -DLAMBDA_ROLE=local \
            -DAWS_ACCOUNT_ID=123456789012
```
