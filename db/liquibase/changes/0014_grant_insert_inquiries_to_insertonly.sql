--liquibase formatted sql
--changeset copilot:0014 context:local,develop
GRANT INSERT ON public.inquiries TO insertonly;
