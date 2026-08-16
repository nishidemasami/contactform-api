--liquibase formatted sql
--changeset copilot:0013 context:develop
CREATE INDEX ASYNC idx_inquiries_created_at ON public.inquiries (created_at) INCLUDE (id);
