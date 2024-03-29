BEGIN;
SET check_function_bodies = false;

-- =============================
-- bpl_token_metadata
-- =============================

-- Accounts
-- ----------------------------
CREATE TABLE public.admin_settings (
    id text NOT NULL,
    platform text NOT NULL,
    create_promo_lamports bigint NOT NULL,
    burn_promo_token_lamports bigint NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.admin_settings
    ADD CONSTRAINT admin_settings_pkey PRIMARY KEY (id);

CREATE TABLE public.merchant (
    id text NOT NULL,
    owner text NOT NULL,
    name text NOT NULL,
    uri text NOT NULL,
    metadata_json jsonb,
    active boolean NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.merchant
    ADD CONSTRAINT merchant_pkey PRIMARY KEY (id);

CREATE TABLE public.location (
    id text NOT NULL,
    merchant text NOT NULL,
    name text NOT NULL,
    uri text NOT NULL,
    metadata_json jsonb,
    active boolean NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.location
    ADD CONSTRAINT location_pkey PRIMARY KEY (id);

CREATE TABLE public.device (
    id text NOT NULL,
    owner text NOT NULL,
    location text NOT NULL,
    name text NOT NULL,
    uri text NOT NULL,
    metadata_json jsonb,
    active boolean NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.device
    ADD CONSTRAINT device_pkey PRIMARY KEY (id);

CREATE TABLE public.campaign (
    id text NOT NULL,
    merchant text NOT NULL,
    name text NOT NULL,
    uri text NOT NULL,
    metadata_json jsonb,
    active boolean NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.campaign
    ADD CONSTRAINT campaign_pkey PRIMARY KEY (id);

CREATE TABLE public.campaign_location (
    id text NOT NULL,
    campaign text NOT NULL,
    location text NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.campaign_location
    ADD CONSTRAINT campaign_location_pkey PRIMARY KEY (id);

CREATE TABLE public.promo (
    id text NOT NULL,
    campaign text NOT NULL,
    mint text NOT NULL,
    metadata text NOT NULL,
    mint_count int NOT NULL,
    burn_count int NOT NULL,
    max_mint int,
    max_burn int,
    active boolean NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.promo
    ADD CONSTRAINT promo_pkey PRIMARY KEY (id);

-- Transactions
-- ----------------------------
CREATE TABLE public.create_admin_settings (
    signature text NOT NULL,
    payer text NOT NULL,
    admin_settings text NOT NULL,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.create_admin_settings
    ADD CONSTRAINT create_admin_settings_pkey PRIMARY KEY (signature);

CREATE TABLE public.create_merchant (
    signature text NOT NULL,
    payer text NOT NULL,
    owner text NOT NULL,
    merchant text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.create_merchant
    ADD CONSTRAINT create_merchant_pkey PRIMARY KEY (signature);

CREATE TABLE public.create_location (
    signature text NOT NULL,
    payer text NOT NULL,
    owner text NOT NULL,
    merchant text NOT NULL,
    location text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.create_location
    ADD CONSTRAINT create_location_pkey PRIMARY KEY (signature);

CREATE TABLE public.create_device (
    signature text NOT NULL,
    payer text NOT NULL,
    merchant_owner text NOT NULL,
    merchant text NOT NULL,
    location text NOT NULL,
    device text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.create_device
    ADD CONSTRAINT create_device_pkey PRIMARY KEY (signature);

CREATE TABLE public.create_campaign (
    signature text NOT NULL,
    payer text NOT NULL,
    owner text NOT NULL,
    merchant text NOT NULL,
    campaign text NOT NULL,
    lamports bigint NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.create_campaign
    ADD CONSTRAINT create_campaign_pkey PRIMARY KEY (signature);

CREATE TABLE public.create_campaign_location (
    signature text NOT NULL,
    payer text NOT NULL,
    owner text NOT NULL,
    merchant text NOT NULL,
    campaign text NOT NULL,
    location text NOT NULL,
    campaign_location text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.create_campaign_location
    ADD CONSTRAINT create_campaign_location_pkey PRIMARY KEY (signature, campaign_location);

CREATE TABLE public.create_promo (
    signature text NOT NULL,
    payer text NOT NULL,
    payer_balance bigint NOT NULL,
    owner text NOT NULL,
    owner_balance bigint NOT NULL,
    merchant text NOT NULL,
    campaign text NOT NULL,
    campaign_balance bigint NOT NULL,
    mint text NOT NULL,
    metadata text NOT NULL,
    authority text NOT NULL,
    promo text NOT NULL,
    platform text NOT NULL,
    platform_balance bigint NOT NULL,
    admin_settings text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.create_promo
    ADD CONSTRAINT create_promo_pkey PRIMARY KEY (signature);

CREATE TABLE public.mint_promo_token (
    signature text NOT NULL,
    payer text NOT NULL,
    device_owner text NOT NULL,
    device text NOT NULL,
    campaign text NOT NULL,
    campaign_location text NOT NULL,
    token_owner text NOT NULL,
    mint text NOT NULL,
    authority text NOT NULL,
    promo text NOT NULL,
    token_account text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.mint_promo_token
    ADD CONSTRAINT mint_promo_token_pkey PRIMARY KEY (signature);

CREATE TABLE public.delegate_promo_token (
    signature text NOT NULL,
    payer text NOT NULL,
    device_owner text NOT NULL,
    device text NOT NULL,
    campaign text NOT NULL,
    campaign_location text NOT NULL,
    token_owner text NOT NULL,
    mint text NOT NULL,
    promo text NOT NULL,
    token_account text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.delegate_promo_token
    ADD CONSTRAINT delegate_promo_token_pkey PRIMARY KEY (signature);

CREATE TABLE public.burn_delegated_promo_token (
    signature text NOT NULL,
    payer text NOT NULL,
    payer_balance bigint NOT NULL,
    device_owner text NOT NULL,
    device text NOT NULL,
    campaign text NOT NULL,
    campaign_balance bigint NOT NULL,
    campaign_location text NOT NULL,
    mint text NOT NULL,
    authority text NOT NULL,
    promo text NOT NULL,
    platform text NOT NULL,
    platform_balance bigint NOT NULL,
    admin_settings text NOT NULL,
    token_account text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.burn_delegated_promo_token
    ADD CONSTRAINT burn_delegated_promo_token_pkey PRIMARY KEY (signature);

CREATE TABLE public.sign_memo (
    signature text NOT NULL,
    payer text NOT NULL,
    signer text NOT NULL,
    memo jsonb,
    slot bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.sign_memo
    ADD CONSTRAINT sign_memo_pkey PRIMARY KEY (signature);

CREATE VIEW public.promo_transactions AS
    SELECT
        mp.signature, 'mint' as transaction_type,
        mp.payer,
        mp.device_owner,
        mp.device,
        mp.campaign,
        mp.campaign_location,
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
            bdp.device_owner,
            bdp.device,
            bdp.campaign,
            bdp.campaign_location,
            bdp.mint,
            bdp.authority,
            bdp.promo,
            bdp.token_account,
            bdp.memo,
            bdp.slot,
            bdp.created_at,
            bdp.modified_at
        FROM burn_delegated_promo_token bdp;

-- =============================
-- mpl_auction_house
-- =============================

CREATE TABLE public.auction_house (
    id text NOT NULL,
    auction_house_fee_account text NOT NULL,
    auction_house_treasury text NOT NULL,
    treasury_withdrawal_destination text NOT NULL,
    fee_withdrawal_destination text NOT NULL,
    treasury_mint text NOT NULL,
    authority text NOT NULL,
    creator text NOT NULL,
    bump int NOT NULL,
    treasury_bump int NOT NULL,
    fee_payer_bump int NOT NULL,
    seller_fee_basis_points int NOT NULL,
    requires_sign_off boolean NOT NULL,
    can_change_sale_price boolean NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.auction_house
    ADD CONSTRAINT auction_house_pkey PRIMARY KEY (id);

CREATE TABLE public.bid_receipt (
    id text NOT NULL,
    trade_state text NOT NULL,
    bookkeeper text NOT NULL,
    auction_house text NOT NULL,
    buyer text NOT NULL,
    metadata text NOT NULL,
    token_account text,
    purchase_receipt text,
    price bigint NOT NULL,
    token_size bigint NOT NULL,
    bump int NOT NULL,
    trade_state_bump int NOT NULL,
    created_at_on_chain bigint NOT NULL,
    canceled_at_on_chain bigint,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.bid_receipt
    ADD CONSTRAINT bid_receipt_pkey PRIMARY KEY (id);

CREATE TABLE public.listing_receipt (
    id text NOT NULL,
    trade_state text NOT NULL,
    bookkeeper text NOT NULL,
    auction_house text NOT NULL,
    seller text NOT NULL,
    metadata text NOT NULL,
    purchase_receipt text,
    price bigint NOT NULL,
    token_size bigint NOT NULL,
    bump int NOT NULL,
    trade_state_bump int NOT NULL,
    created_at_on_chain bigint NOT NULL,
    canceled_at_on_chain bigint,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.listing_receipt
    ADD CONSTRAINT listing_receipt_pkey PRIMARY KEY (id);

CREATE TABLE public.purchase_receipt (
    id text NOT NULL,
    bookkeeper text NOT NULL,
    buyer text NOT NULL,
    seller text NOT NULL,
    auction_house text NOT NULL,
    metadata text NOT NULL,
    token_size bigint NOT NULL,
    price bigint NOT NULL,
    bump int NOT NULL,
    created_at_on_chain bigint NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.purchase_receipt
    ADD CONSTRAINT purchase_receipt_pkey PRIMARY KEY (id);

-- =============================
-- mpl_token_metadata
-- =============================

CREATE TABLE public.creator (
    metadata text NOT NULL,
    address text NOT NULL,
    verified boolean NOT NULL,
    share integer NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.creator
    ADD CONSTRAINT creator_pkey PRIMARY KEY (metadata, address);

CREATE TABLE public.metadata (
    id text NOT NULL,
    key text NOT NULL,
    update_authority text NOT NULL,
    mint text NOT NULL,
    name text NOT NULL,
    symbol text NOT NULL,
    uri text NOT NULL,
    metadata_json jsonb,
    seller_fee_basis_points int NOT NULL,
    primary_sale_happened boolean NOT NULL,
    is_mutable boolean NOT NULL,
    edition_nonce int,
    token_standard text,
    collection_key text,
    collection_verified boolean,
    uses_use_method text,
    uses_remaining bigint,
    uses_total bigint,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.metadata
    ADD CONSTRAINT metadata_pkey PRIMARY KEY (id);
    
-- =============================
-- spl_token
-- =============================

CREATE TABLE public.mint (
    id text NOT NULL,
    mint_authority text,
    supply bigint NOT NULL,
    decimals integer NOT NULL,
    is_initialized boolean NOT NULL,
    freeze_authority text,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    modified_at timestamp with time zone DEFAULT now()
);
ALTER TABLE ONLY public.mint
    ADD CONSTRAINT mint_pkey PRIMARY KEY (id);

CREATE TABLE public.token_account (
    id text NOT NULL,
    mint text NOT NULL,
    owner text NOT NULL,
    amount bigint NOT NULL,
    delegate text,
    state text NOT NULL,
    is_native bigint,
    delegated_amount bigint,
    close_authority text,
    slot bigint NOT NULL,
    write_version bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    modified_at timestamp with time zone DEFAULT now() NOT NULL
);
ALTER TABLE ONLY public.token_account
    ADD CONSTRAINT token_account_pkey PRIMARY KEY (id);

CREATE VIEW public.mint_relation AS
    WITH t AS (
        SELECT promo.mint, promo.created_at, ta.owner, ta.amount
        FROM promo
        LEFT JOIN token_account ta ON promo.mint = ta.mint
    )
    SELECT
        t1.mint as mint, t2.mint as related_mint,
        SUM(
            CASE
                WHEN
                    t1.owner IS NOT NULL
                    AND t2.owner IS NOT NULL
                    AND t1.owner = t2.owner
                THEN t2.amount
                ELSE 0
            END
        ) as amount_sum,
        COUNT(
            CASE
                WHEN
                    t1.owner IS NOT NULL
                    AND t2.owner IS NOT NULL
                    AND t1.owner = t2.owner
                THEN t2.amount
                ELSE NULL
            END
        ) as owner_count,
        MIN(t2.created_at) AS created_at
    FROM t AS t1
    JOIN t as t2
        ON t1.mint != t2.mint
    GROUP BY t1.mint, related_mint
    ORDER BY t1.mint, amount_sum DESC, created_at DESC;

-- CREATE VIEW public.receipt AS
--     SELECT pr.id, 'secondary' as receipt_type, metadata.mint, pr.buyer,
--         pr.seller, pr.token_size, pr.price, pr.created_at
--     FROM purchase_receipt pr JOIN metadata ON pr.metadata = metadata.id
--     UNION ALL
--         SELECT id, 'primary' as receipt_type, mint, buyer,
--             NULL as seller, token_size, price, created_at
--         FROM primary_receipt ORDER BY mint, created_at;

CREATE VIEW public.listing_with_token AS
    SELECT lr.id, metadata.mint, lr.metadata, ta.id token_account, lr.seller,
        lr.price, lr.token_size, lr.created_at_on_chain, lr.slot
    FROM listing_receipt lr
    JOIN metadata ON lr.metadata = metadata.id
    JOIN token_account ta
        ON ta.mint = metadata.mint
        AND ta.owner = lr.seller
    WHERE
        lr.canceled_at_on_chain IS NULL
        AND lr.purchase_receipt IS NULL
        AND ta.delegated_amount > 0;

CREATE VIEW public.floor_price AS
    SELECT mint, MIN(price / token_size) floor
    FROM listing_with_token
    GROUP BY mint;
COMMIT;
