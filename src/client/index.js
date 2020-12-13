const testConstructor = require('./constructor.test');

const main = async () => {
  await testConstructor();
}

try {
  main();
}
catch (er) {
  console.error(er);
}