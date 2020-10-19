const { sendAndConfirmTransaction: realSendAndConfirmTransaction } = require('@solana/web3.js');
const YAML = require('json-to-pretty-yaml');

let notify = () => undefined;

function onTransaction(callback) {
  notify = callback;
}

async function sendAndConfirmTransaction(title, connection, transaction, ...signers) {
  const when = Date.now();

  const signature = await realSendAndConfirmTransaction(
    connection,
    transaction,
    signers,
    {
      skipPreflight: true,
      commitment: 'recent',
    },
  );

  const body = {
    time: new Date(when).toString(),
    signature,
    instructions: transaction.instructions.map(i => {
      return {
        keys: i.keys.map(keyObj => keyObj.pubkey.toBase58()),
        programId: i.programId.toBase58(),
        data: '0x' + i.data.toString('hex'),
      };
    }),
  };

  notify(title, YAML.stringify(body).replace(/"/g, ''));
}

module.exports = { onTransaction, sendAndConfirmTransaction }