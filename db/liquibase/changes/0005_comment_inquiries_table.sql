--liquibase formatted sql
--changeset copilot:0005 context:local,develop
COMMENT ON TABLE public.inquiries IS '問い合わせテーブル';
