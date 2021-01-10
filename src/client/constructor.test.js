const {
  sendAndConfirmTransaction, TransactionInstruction, Transaction,
  Account, PublicKey,
} = require('@solana/web3.js');
const soproxABI = require('soprox-abi');
const { init, info } = require('./helpers');

/**
 * Pool constructor
 */
const poolConstructor = async (pool, treasury, lpt, srcPublicKey, tokenPublicKey, tokenProgramId, programId, payer, connection) => {
  console.log('Pool constructor at', pool.publicKey.toBase58());
  console.log('Treasury constructor at', treasury.publicKey.toBase58());
  const seeds = [pool.publicKey.toBuffer()];
  const tokenOwnerPublicKey = await PublicKey.createProgramAddress(seeds, programId);
  console.log('Token Owner constructor at', tokenOwnerPublicKey.toBase58());
  const schema = [
    { key: 'code', type: 'u8' },
    { key: 'reserve', type: 'u64' },
    { key: 'lpt', type: 'u64' },
  ];
  const layout = new soproxABI.struct(schema, {
    code: 0, reserve: 5000n, lpt: 1000n
  });
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: pool.publicKey, isSigner: true, isWritable: true },
      { pubkey: treasury.publicKey, isSigner: true, isWritable: true },
      { pubkey: lpt.publicKey, isSigner: true, isWritable: true },
      { pubkey: srcPublicKey, isSigner: false, isWritable: true },
      { pubkey: tokenPublicKey, isSigner: false, isWritable: false },
      { pubkey: tokenOwnerPublicKey, isSigner: false, isWritable: false },
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
      new Account(Buffer.from(treasury.secretKey, 'hex')),
      new Account(Buffer.from(lpt.secretKey, 'hex')),
    ],
    { skipPreflight: true, commitment: 'recent' });
}

module.exports = async function () {
  console.log('\n\n*** Test constructor\n');
  const { connection, payer, src, token, tokenProgramId, programId, registers: [pool, treasury, lpt] } = await init();

  try {
    await poolConstructor(pool, treasury, lpt, src, token, tokenProgramId, programId, payer, connection);
  } catch (er) {
    console.log(er)
    // Token or Account is already initialized
    console.log('The token and accound may be created already');
  }
  console.log('Pool info:', await info(pool, connection));
  console.log('Treasury info:', await info(treasury, connection));
  console.log('LPT info:', await info(lpt, connection));
}