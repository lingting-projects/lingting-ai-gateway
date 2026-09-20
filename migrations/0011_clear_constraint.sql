ALTER TABLE provider_model
DROP CONSTRAINT IF EXISTS provider_model_provider_id_fkey;

ALTER TABLE request_sub
DROP CONSTRAINT IF EXISTS request_sub_main_request_id_fkey;