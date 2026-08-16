--liquibase formatted sql
--changeset copilot:0011 context:local,develop
COMMENT ON COLUMN public.inquiries.row_log IS '行ログ (JSON形式で生リクエストを保存)';
