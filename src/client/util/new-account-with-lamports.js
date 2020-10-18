const { Account } = require('@solana/web3.js');
const { sleep } = require('./sleep');

async function newAccountWithLamports(connection, lamports = 1000000) {
  const account = new Account();

  let retries = 10;
  await connection.requestAirdrop(account.publicKey, lamports);
  for (; ;) {
    await sleep(500);
    if (lamports == (await connection.getBalance(account.publicKey))) {
      return account;
    }
    if (--retries <= 0) {
      break;
    }
    console.log('Airdrop retry ' + retries);
  }
  throw new Error(`Airdrop of ${lamports} failed`);
}

module.exports = { newAccountWithLamports }