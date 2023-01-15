INSERT INTO sign_memo (
    signature,
    payer,
    signer,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5)
ON CONFLICT ON CONSTRAINT sign_memo_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        signer = EXCLUDED.signer,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > sign_memo.slot
RETURNING created_at = modified_at
