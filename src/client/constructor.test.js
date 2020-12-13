const { sendAndConfirmTransaction, TransactionInstruction, Transaction, Account } = require('@solana/web3.js');
const soproxABI = require('soprox-abi');
const { init, info } = require('./helpers');

/**
 * Pool constructor
 */
const poolConstructor = async (pool, treasury, tokenPublicKey, tokenProgramId, programId, payer, connection) => {
  console.log('Pool constructor at', pool.publicKey.toBase58());
  console.log('Treasury constructor at', treasury.publicKey.toBase58());
  const schema = [
    { key: 'code', type: 'u8' },
  ];
  const layout = new soproxABI.struct(schema, {
    code: 0,
  });
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: pool.publicKey, isSigner: true, isWritable: true },
      { pubkey: treasury.publicKey, isSigner: true, isWritable: true },
      { pubkey: tokenPublicKey, isSigner: false, isWritable: false },
      { pubkey: tokenProgramId, isSigner: false, isWritable: false },
    ],
    programId,
    data: layout.toBuffer()
  });
  const transaction = new Transaction();
  transaction.add(instruction);
  await sendAndConfirmTransaction(
    connection,
    transaction,
    [
      payer,
      new Account(Buffer.from(pool.secretKey, 'hex')),
      new Account(Buffer.from(treasury.secretKey, 'hex'))
    ],
    { skipPreflight: true, commitment: 'recent' });
}

module.exports = async function () {
  console.log('\n\n*** Test constructor\n');
  const { connection, payer, token, tokenProgramId, programId, registers: [pool, treasury] } = await init();

  try {
    await poolConstructor(pool, treasury, token, tokenProgramId, programId, payer, connection);
  } catch (er) {
    console.log(er)
    // Token or Account is already initialized
    console.log('The token and accound may be created already');
  }
  return console.log('Pool info:', await info(pool, connection));
}