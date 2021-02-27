const soproxABI = require('soprox-abi');
const { createAccount, fromAddress, SPLT } = require('senswapjs');
const { establishConnection, loadPayer } = require('../../lib/network');
const store = require('../../lib/store');

/**
 * Initialize accounts
 */
const init = async () => {
  const connection = await establishConnection();
  const payer = await loadPayer(connection);
  const splt = new SPLT();
  const mint = createAccount();
  const src = createAccount();
  await splt.initializeMint(9, null, mint, payer);
  await splt.initializeAccount(src, mint.publicKey.toBase58(), payer);
  await splt.mintTo(8000000000000000000n, mint.publicKey.toBase58(), src.publicKey.toBase58(), payer);
  const data = await splt.getAccountData(src.publicKey.toBase58());
  console.log(data);
  const spltProgramId = splt.spltProgramId;
  const programId = fromAddress(store.load('program').address);
  const registers = store.load('abi').map(register => {
    register.publicKey = fromAddress(register.address);
    return register;
  });
  return { connection, payer, src, mint, spltProgramId, programId, registers }
}

/**
 * Account info
 */
const info = async (register, connection) => {
  const { data } = await connection.getAccountInfo(register.publicKey);
  if (!data) throw new Error('Cannot find data of', register.address);
  const layout = new soproxABI.struct(register.schema);
  layout.fromBuffer(data);
  return layout.value;
}

module.exports = { init, info }