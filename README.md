# senswap-contract

## Versioning

There has been some understandable confusion around deploying programs and the versions of the various Solana tools.  To help clarify and give current status:
- A new BPF loader is in the process of being deployed, it's called BPFLoader2
- BPFLoader2 is enabled on testnet only (NOT on devnet or mainnet yet)
- solana_sdk rust crate 1.3.5 switches to using BPFLoader2 by default
- solana-web3.js v0.71.9 requires you to specify which loader to use when loading a program
Recommendation:  The network you choose to deploy determines which loader you will be using:
- If devnet or mainnet, build your program against solana_sdk v1.3.4 and deploy use the latest solana-web3.js v0.71.9 specifying BPFLoader, or deploy using solana-cli v1.3.4.
- If testnet, build your program against solana-sdk v1.3.5 or later and deploy using solana-web3.js v0.71.9 specifying BPFLoader2 or deploy using solana-cli v1.3.5 or later.