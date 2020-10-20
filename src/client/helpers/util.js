const {
  Account,
  sendAndConfirmTransaction: _sendAndConfirmTransaction
} = require('@solana/web3.js');


function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function sendAndConfirmTransaction(connection, transaction, ...signers) {
  await _sendAndConfirmTransaction(
    connection,
    transaction,
    signers,
    {
      skipPreflight: true,
      commitment: 'recent',
    },
  );
}

async function newAccountWithLamports(connection, lamports = 1000000) {
  const account = new Account();
  let retries = 10;
  await connection.requestAirdrop(account.publicKey, lamports);
  while (true) {
    await sleep(500);
    if (lamports == (await connection.getBalance(account.publicKey)))
      return account;
    if (retries <= 0) break;
    console.log('Airdrop retry ' + retries);
    retries -= 1;
  }
  throw new Error(`Airdrop of ${lamports} failed`);
}

async function newSystemAccountWithAirdrop(connection, lamports = 1) {
  const account = new Account();
  await connection.requestAirdrop(account.publicKey, lamports);
  return account;
}

module.exports = {
  newAccountWithLamports,
  newSystemAccountWithAirdrop,
  sendAndConfirmTransaction,
}
