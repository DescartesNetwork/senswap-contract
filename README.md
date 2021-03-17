# SenSwap Contracts

## Validation checklist

Initialize Network: `[owner, network, primary_token, vault, splt, sysvar_rent, mint x 31]`
- [ ] network_acc.owner == program_id
- [ ] !network_data.is_initialized
- [ ] owner.is_signer
- [ ] network_acc.is_signer
- [ ] vault_acc.is_signer

Initialize Pool: `(reserve, lpt) [owner, network, pool, treasury, lpt, src, mint, treasurer, splt, sysvar_rent]`

- [ ] network_acc.owner == pool_acc.owner == lpt_acc.owner == program_id
- [ ] network_data.is_initialized
- [ ] !pool_data.is_initialized
- [ ] !lpt_data.is_initilized
- [ ] owner.is_signer
- [ ] pool_acc.is_signer
- [ ] lpt_acc.is_signer
- [ ] program_address(seed(pool_acc), program_id) == treasurer
- [ ] network_data.is_approved(mint_acc)
- [ ] if mint != sen { network_data.is_activated }
- [ ] if mint == sen { !network_data.is_activated }
- [ ] reserve != 0 && lpt != 0

Initialized LPT: `[owner, pool, lpt]`

- [ ] pool_acc.owner == lpt_acc.owner == program_id
- [ ] !lpt_data.is_initialized
- [ ] owner.is_signer
- [ ] lpt_acc.is_signer

Add Liquidity: `(reserve) [owner, pool, treasury, lpt, src, splt]`

- [ ] pool_acc.owner == lpt_acc.owner == program_id
- [ ] pool_data.is_initialized
- [ ] lpt_data.is_initialized
- [ ] owner.is_signer
- [ ] pool_data.treasury = treasury_acc
- [ ] lpt_data.owner == owner
- [ ] lpt_data.pool == pool_acc
- [ ] reserve != 0

Remove Liquidity: `(lpt) [owner, pool, treasury, lpt, dst, treasurer, splt]`

- [ ] pool_acc.owner ==  lpt_acc.owner == program_id
- [ ] pool_data.is_initialized
- [ ] lpt_data.is_initialized
- [ ] owner.is_signer
- [ ] pool_data.treasury = treasury_acc
- [ ] lpt_data.owner == owner
- [ ] lpt_data.pool == pool_acc
- [ ] program_address(seed(pool_acc), program_id) == treasurer
- [ ] lpt != 0
- [ ] lpt <= lpt_data.lpt

Swap: `(amount) [owner, network, bid_pool, bid_treasury, src, ask_pool, ask_treasury, dst, ask_treasurer, sen_pool, sen_treasury, vault, sen_treasurer, splt]`

- [ ] bid_pool_acc.owner == ask_pool_acc.owner == sen_pool_acc.owner ==program_id
- [ ] bid_pool_data.is_initialized
- [ ] ask_pool_data.is_initialized
- [ ] sen_pool_data.is_initialized
- [ ] owner.is_signer
- [ ] bid_pool_data.treasury = bid_treasury_acc
- [ ] ask_pool_data.treasury = ask_treasury_acc
- [ ] sen_pool_data.treasury = sen_treasury_acc
- [ ] program_address(seed(ask_pool_acc), program_id) == ask_treasurer
- [ ] program_address(seed(sen_pool_acc), program_id) == sen_treasurer
- [ ] network_acc = bid_pool_data.network = ask_pool_data.network = sen_pool_data.network
- [ ] amount != 0
- [ ] if bid_pool_acc == ask_pool_acc { Return }

Transfer: `(lpt) [owner, src_lpt, dst_lpt]`

- [ ] src_lpt_acc.owner == dst_lpt_acc.owner == program_id
- [ ] src_lpt_data.is_initialized
- [ ] dst_lpt_data.is_initialized
- [ ] owner.is_signer
- [ ] src_lpt_data.owner == owner
- [ ] src_lpt_data.pool == dst_lpt_data.pool
- [ ] if src_lpt_acc == dst_lpt_acc { Return }
- [ ] lpt != 0
- [ ] lpt <= src_lpt_data.lpt

Close LPT: `[owner, lpt, dst]`

- [ ] lpt_acc.owner == program_id
- [ ] owner.is_signer
- [ ] lpt_data.owner == owner
- [ ] lpt_data.lpt == 0

Close Pool: `[owner, pool, treasury, dst, treasurer, splt]`

- [ ] pool_acc.owner == program_id
- [ ] pool_data.is_initialized
- [ ] owner.is_signer
- [ ] program_address(seed(pool_acc), program_id) == treasurer
- [ ] pool_data.owner == owner
- [ ] pool_data.treasury == treasury_acc
- [ ] pool_data.lpt == 0 && pool_data.reserve == 0