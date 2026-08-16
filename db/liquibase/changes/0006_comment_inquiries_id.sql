--liquibase formatted sql
--changeset copilot:0006 context:local,develop
COMMENT ON COLUMN public.inquiries.id IS '問い合わせID (UUID v7)';
