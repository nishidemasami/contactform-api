--liquibase formatted sql
--changeset copilot:0012 context:local
CREATE INDEX idx_inquiries_created_at ON public.inquiries (created_at) INCLUDE (id);
