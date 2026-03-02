-- Add Canadian Registered Account Types to Asset Types
-- Migration: Extends asset types to include TFSA, RRSP, LIRA, RESP, RRIF, FHSA

-- Drop the existing constraint
ALTER TABLE survey_assets DROP CONSTRAINT survey_assets_type_valid;

-- Add new constraint with Canadian asset types
ALTER TABLE survey_assets ADD CONSTRAINT survey_assets_type_valid CHECK (
    asset_type IN (
        -- Original types
        'liquid',
        'investment',
        'retirement',
        'real_estate',
        'other',
        -- Canadian registered accounts
        'tfsa',    -- Tax-Free Savings Account
        'rrsp',    -- Registered Retirement Savings Plan
        'lira',    -- Locked-In Retirement Account
        'resp',    -- Registered Education Savings Plan
        'rrif',    -- Registered Retirement Income Fund
        'fhsa'     -- First Home Savings Account
    )
);

-- Update comment
COMMENT ON COLUMN survey_assets.asset_type IS 'Asset type: liquid, investment, retirement, real_estate, tfsa, rrsp, lira, resp, rrif, fhsa, other';
