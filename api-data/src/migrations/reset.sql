BEGIN;
SET check_function_bodies = false;

-- =============================
-- bpl_token_metadata
-- =============================

DROP TABLE IF EXISTS public.admin_settings CASCADE;
DROP TABLE IF EXISTS public.burn_delegated_promo_token CASCADE;
DROP TABLE IF EXISTS public.campaign CASCADE;
DROP TABLE IF EXISTS public.create_campaign CASCADE;
DROP TABLE IF EXISTS public.create_device CASCADE;
DROP TABLE IF EXISTS public.create_location CASCADE;
DROP TABLE IF EXISTS public.create_merchant CASCADE;
DROP TABLE IF EXISTS public.create_promo CASCADE;
DROP TABLE IF EXISTS public.create_promo_group CASCADE;
DROP TABLE IF EXISTS public.delegate_promo_token CASCADE;
DROP TABLE IF EXISTS public.device CASCADE;
DROP TABLE IF EXISTS public.location CASCADE;
DROP TABLE IF EXISTS public.merchant CASCADE;
DROP TABLE IF EXISTS public.mint_promo_token CASCADE;
DROP TABLE IF EXISTS public.promo CASCADE;
DROP TABLE IF EXISTS public.promo_group CASCADE;
DROP TABLE IF EXISTS public.sign_memo CASCADE;

-- =============================
-- mpl_auction_house
-- =============================

DROP TABLE IF EXISTS public.auction_house;
DROP TABLE IF EXISTS public.bid_receipt;
DROP TABLE IF EXISTS public.listing_receipt CASCADE;
DROP TABLE IF EXISTS public.purchase_receipt;

-- =============================
-- mpl_token_metadata
-- =============================

DROP TABLE IF EXISTS public.creator;
DROP TABLE IF EXISTS public.metadata;

-- =============================
-- spl_token
-- =============================

DROP TABLE IF EXISTS public.mint;
DROP TABLE IF EXISTS public.token_account CASCADE;

-- =============================
-- refinery migrations
-- =============================

DROP TABLE IF EXISTS public.refinery_schema_history;

COMMIT;
