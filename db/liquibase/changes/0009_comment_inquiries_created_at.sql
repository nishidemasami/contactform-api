--liquibase formatted sql
--changeset copilot:0009 context:local,develop
COMMENT ON COLUMN public.inquiries.created_at IS '作成日時';
