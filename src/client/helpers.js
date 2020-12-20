const { PublicKey } = require('@solana/web3.js');
const soproxABI = require('soprox-abi');
const { establishConnection, loadPayer } = require('../../lib/network');
const store = require('../../lib/store');

/**
 * Initialize accounts
 */
const init = async () => {
  const connection = await establishConnection();
  const payer = await loadPayer(connection);
  const src = new PublicKey('E1dewiwqs7zqPPzoctiWgrR7ndoyY64yc2BwU6eYny3Q');
  const token = new PublicKey('EbU12HD74t19Jhe8JfTBPiY3ikcEMNGRv7bNbsPkdhXZ');
  const tokenProgramId = new PublicKey('6D5sUPSzXsLPtDxBpi8fngu83FPk54CvpinX3zzZovTr');
  const program = store.load('program');
  const programId = new PublicKey(program.address);
  const registers = store.load('abi').schema.map(register => {
    register.publicKey = new PublicKey(register.address);
    return register;
  });
  return { connection, payer, src, token, tokenProgramId, programId, registers }
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