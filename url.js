// To connect to a public cluster, set `export LIVE=1` in your
// environment. By default, `LIVE=1` will connect to the devnet cluster.

const { clusterApiUrl } = require('@solana/web3.js');
const dotenv = require('dotenv');

function chooseCluster() {
  dotenv.config();
  if (!process.env.LIVE) return;
  switch (process.env.CLUSTER) {
    case 'devnet':
    case 'testnet':
    case 'mainnet-beta': {
      return process.env.CLUSTER;
    }
  }
  throw 'Unknown cluster "' + process.env.CLUSTER + '", check the .env file';
}

const cluster = chooseCluster();

const url =
  process.env.RPC_URL ||
  (process.env.LIVE ? clusterApiUrl(cluster, false) : 'http://localhost:8899');

const urlTls =
  process.env.RPC_URL ||
  (process.env.LIVE ? clusterApiUrl(cluster, true) : 'http://localhost:8899');

  const walletUrl =
  process.env.WALLET_URL || 'https://solana-example-webwallet.herokuapp.com/';


module.exports = { cluster, url, urlTls, walletUrl }