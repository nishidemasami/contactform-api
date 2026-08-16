--liquibase formatted sql
--changeset copilot:0008 context:local,develop
COMMENT ON COLUMN public.inquiries.email IS '連絡先メールアドレス';
