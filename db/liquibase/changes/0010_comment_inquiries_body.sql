--liquibase formatted sql
--changeset copilot:0010 context:local,develop
COMMENT ON COLUMN public.inquiries.body IS '問い合わせ本文';
