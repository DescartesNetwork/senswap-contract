# SenSwap Contracts

## Validation checklist

Initialize Pool: `[owner, network, pool, treasury, lpt, src, mint, treasurer, splt, sysvar_rent]`

- [ ] pool_acc.owner == lpt_acc.owner == program_id
- [ ] owner.is_signer
- [ ] pool_acc.is_signer
- [ ] treasury_acc.is_signer
- [ ] lpt_acc.is_signer
- [ ] !pool_data.is_initialized
- [ ] !lpt_data.is_initilized
- [ ] program_address(seed(pool_acc), program_id) == treasurer
- [ ] if !network_data.is_initialized { network_acc.is_signer }

Initialized LPT: `[owner, pool, lpt]`

- [ ] pool_acc.owner == lpt_acc.owner == program_id
- [ ] owner_acc.is_signer
- [ ] lpt_acc.is_signer
- [ ] !lpt_data.is_initialized

Add Liquidity: `(reserve) [owner, network, prev_pool, pool, treasury, lpt, src, splt]`

- [ ] network_acc.owner == prev_pool_acc.owner == pool_acc.owner == lpt_acc.owner == progrma_id
- [ ] network_data.is_initialized
- [ ] prev_pool_data.is_initialized
- [ ] pool_data.is_initialized
- [ ] lpt_data.is_initialized
- [ ] owner_acc.is_signer
- [ ] lpt_data.owner == owner_acc
- [ ] pool_data.network == prev_pool_data.network == network
- [ ] if pool_data.is_approved { Add lpt to network & Recheck approval condition }
- [ ] reserve != 0

Remove Liquidity: `(lpt) [owner, network, prev_pool, pool, treasury, lpt, dst, treasurer, splt]`

- [ ] network_acc.owner == prev_pool_acc.owner == pool_acc.owner ==  lpt_acc.owner == program_id
- [ ] network_data.is_initialized
- [ ] prev_pool_data.is_initialized
- [ ] pool_data.is_initialized
- [ ] lpt_data.is_initialized
- [ ] owner.is_signer
- [ ] lpt_data.owner == owner
- [ ] program_address(seed(pool_acc), program_id) == treasurer
- [ ] lpt <= lpt_data.lpt
- [ ] if pool_data.is_approved { Remove lpt to network }

Swap: `(amount) [owner, bid_pool, bid_treasury, src, ask_pool, ask_treasury, dst, ask_treasurer, splt]`

- [ ] bid_pool_acc.owner == ask_pool_acc.owner == program_id
- [ ] bid_pool_data.is_initialized
- [ ] bid_treasury_data.is_initialized
- [ ] ask_pool_data.is_initialized
- [ ] ask_treasury_data.is_initialized
- [ ] owner.is_signer
- [ ] program_address(seed(ask_pool_acc), program_id) == ask_treasurer
- [ ] bid_pool_data.network == ask_pool_data.network
- [ ] bid_pool_data.is_approved
- [ ] ask_pool_data.is_approved
- [ ] amount != 0
- [ ] if bid_pool_acc == ask_pool_acc { Return }

Vote: `[owner, network, prev_pool, next_pool, pool, lpt]`

- [ ] network_acc.owner == prev_pool_acc.owner == next_pool_acc.owner == pool_acc.owner == lpt_acc.owner == program_id
- [ ] network_data.is_initialized
- [ ] prev_pool_data.is_initialized
- [ ] next_pool_data.is_initialized
- [ ] pool_data.is_initialized
- [ ] lpt_data.is_initialized
- [ ] owner.is_signer
- [ ] lpt_data.owner == owner
- [ ] prev_pool_data.network == next_pool_data.network == pool_data.network == network_acc
- [ ] pool_data.is_approved
- [ ] if prev_pool_acc == next_pool_acc { Return }

Close LPT: `[owner, lpt]`

- [ ] lpt_acc,owner == program_id
- [ ] owner.is_signer
- [ ] lpt_data.owner == owner
- [ ] lpt_data.lpt == 0