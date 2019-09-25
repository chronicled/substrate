const { ApiPromise } = require('@polkadot/api');
const testKeyring = require('@polkadot/keyring/testing');

const aliceAddr = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const alicePair = testKeyring.default().getPair(aliceAddr);

/**
 * Sends a transaction ot the chain returning a promise.
 * When the transaction is finalized, resolve to the list of events received.
 * When the transaction fails, reject relaying the error message.
 * @param tx The transaction to sign and await
 * @param pair A keypair with which to sign the transaction
 * @return A promise for the events emitted by the transaction
 */
function signAndAwait(tx, pair) {
  return new Promise((resolve, reject) => {
    //TODO do I need to bind an unsubscribe function here?
    tx.signAndSend(pair, ({events = [], status}) => {
      if (status.isFinalized) {
        resolve(events);
      }
    })
    // Rejection should be relayed automatically, right?
  });
}

/**
 * Interact with blockchain to demonstrate that multi queries are not working
 * properly on doublemaps
 */
async function main () {
  // Create an await for the API
  const api = await ApiPromise.create();

  // Take note of Alice's original balance
  const original = await api.query.balances.freeBalance(aliceAddr);

  // Call the fixed normal 2m
  await signAndAwait(api.tx.templateModule.fixedNormal2M(0), alicePair);
  const after2m = await api.query.balances.freeBalance(aliceAddr);
  console.log(`fixed_normal_2m cost Alice ${original - after2m}. Expected 2million`);

  // Call fixed normal 5
  await signAndAwait(api.tx.templateModule.fixedNormal5(0), alicePair);
  const after5 = await api.query.balances.freeBalance(aliceAddr);
  console.log(`fixed_normal_2m cost Alice ${after2m - after5}. Expected 5`);

  // Call fixed normal 1
  await signAndAwait(api.tx.templateModule.fixedNormal1(0), alicePair);
  const after1 = await api.query.balances.freeBalance(aliceAddr);
  console.log(`fixed_normal_2m cost Alice ${after5 - after1}. Expected 1`);

  // Call custom 5
  await signAndAwait(api.tx.templateModule.custom5(0), alicePair);
  const afterCustom = await api.query.balances.freeBalance(aliceAddr);
  console.log(`fixed_normal_2m cost Alice ${after5 - afterCustom}. Expected 5`);
}

main()
.then(() => process.exit(0))
.catch( e => {
  console.error(e);
  process.exit(1);
});
