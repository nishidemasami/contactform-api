--liquibase formatted sql
--changeset copilot:0003 context:develop
AWS IAM GRANT insertonly TO '${LAMBDA_ROLE}';
