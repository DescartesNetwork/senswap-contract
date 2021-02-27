const {
  sendAndConfirmTransaction, TransactionInstruction, Transaction,
  Account, PublicKey, SYSVAR_RENT_PUBKEY,
} = require('@solana/web3.js');
const soproxABI = require('soprox-abi');
const { init, info } = require('./helpers');

/**
 * Pool constructor
 */
const initialePool = async (pool, treasury, lpt, srcPublicKey, mintPublicKey, spltProgramId, programId, payer, connection) => {
  console.log('Pool constructor at', pool.publicKey.toBase58());
  console.log('Treasury constructor at', treasury.publicKey.toBase58());
  const seeds = [pool.publicKey.toBuffer()];
  const treasurerPublicKey = await PublicKey.createProgramAddress(seeds, programId);
  console.log('Token Owner constructor at', treasurerPublicKey.toBase58());
  const schema = [
    { key: 'code', type: 'u8' },
    { key: 'reserve', type: 'u64' },
    { key: 'lpt', type: 'u64' },
  ];
  const layout = new soproxABI.struct(schema, {
    code: 0, reserve: 5000000000000n, lpt: 1000000000000n
  });
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: pool.publicKey, isSigner: true, isWritable: true },
      { pubkey: treasury.publicKey, isSigner: true, isWritable: true },
      { pubkey: lpt.publicKey, isSigner: true, isWritable: true },
      { pubkey: srcPublicKey, isSigner: false, isWritable: true },
      { pubkey: mintPublicKey, isSigner: false, isWritable: false },
      { pubkey: treasurerPublicKey, isSigner: false, isWritable: false },
      { pubkey: spltProgramId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
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
  const { connection, payer, src, mint, spltProgramId, programId, registers: [pool, treasury, lpt] } = await init();
  const srcPublicKey = src.publicKey;
  const mintPublicKey = mint.publicKey;
  try {
    await initialePool(pool, treasury, lpt, srcPublicKey, mintPublicKey, spltProgramId, programId, payer, connection);
  } catch (er) {
    console.log(er)
    // Token or Account is already initialized
    console.log('The mint and accound may be created already');
  }
  console.log('Pool info:', await info(pool, connection));
  console.log('Treasury info:', await info(treasury, connection));
  console.log('LPT info:', await info(lpt, connection));
}