--liquibase formatted sql
--changeset copilot:0007 context:local,develop
COMMENT ON COLUMN public.inquiries.name IS '氏名';
