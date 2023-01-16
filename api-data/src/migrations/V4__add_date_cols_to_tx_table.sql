BEGIN;
SET check_function_bodies = false;

-- =============================
-- bpl_token_metadata
-- =============================
DROP VIEW IF EXISTS public.promo_transactions;

CREATE VIEW public.promo_transactions AS
    SELECT
        mp.signature, 'mint' as transaction_type,
        mp.payer,
        mp.promo_group,
        mp.mint,
        mp.authority,
        mp.promo,
        mp.token_account,
        mp.memo,
        mp.slot,
        mp.created_at,
        mp.modified_at
    FROM mint_promo_token mp
    UNION ALL
        SELECT
            bdp.signature, 'burn_delegated' as transaction_type,
            bdp.payer,
            bdp.promo_group,
            bdp.mint,
            bdp.authority,
            bdp.promo,
            bdp.token_account,
            bdp.memo,
            bdp.slot,
            bdp.created_at,
            bdp.modified_at
        FROM burn_delegated_promo_token bdp;

COMMIT;