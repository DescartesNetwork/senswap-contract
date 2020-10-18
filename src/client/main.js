import { establishConnection, establishPayer, loadProgram, sayHello, reportHellos } from './hello_world';

async function main() {
  console.log("Let's say hello to a Solana account...");
  await establishConnection(); // Establish connection to the cluster
  await establishPayer(); // Determine who pays for the fees
  await loadProgram(); // Load the program if not already loaded
  await sayHello(); // Say hello to an account
  await reportHellos(); // Find out how many times that account has been greeted
  console.log('Success');
}

main().then(process.exit).catch(err => {
  console.error(err);
  process.exit(1);
});
