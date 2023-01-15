BEGIN;
SET check_function_bodies = false;

-- =============================
-- bpl_token_metadata
-- =============================

CREATE TABLE public.sign_memo (
    signature text NOT NULL,
    payer text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.sign_memo
    ADD CONSTRAINT sign_memo_pkey PRIMARY KEY (signature);

COMMIT;
