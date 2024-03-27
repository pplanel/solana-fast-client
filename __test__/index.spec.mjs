
import test from 'ava';
import { AccountEncoding, CommitmentLevelInternal, QuicClient } from '../index.js';
import { PublicKey } from "@solana/web3.js";
import bs58 from 'bs58';

test('test thread safe callback functions', async (t) => {
  t.timeout(100000)
  const client = QuicClient.new(
    'https://mainnet.helius-rpc.com/?api-key=62ab39d9-db50-4757-a99f-8f360b325ce3',
    'wss://mainnet.helius-rpc.com/?api-key=62ab39d9-db50-4757-a99f-8f360b325ce3'
  );
  await client.connect();

  const accountChangeCallback = (err, data) => {
    console.log("Callend")
    if (err) {
      console.error('Error receiving account change notification:', err);
      t.fail();
      return;
    }

    console.log('Account change notification:', JSON.parse(data).account.data);
    t.pass();
  };

  const pubkey = new PublicKey('675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8');
  const config = {
    filters: [
      { dataSize: 752 },
      {
        memcmp: {
          offset: 432,
          bytes: 'So11111111111111111111111111111111111111112'
        }
      },
      {
        memcmp: {
          offset: 560,
          bytes: 'srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX'
        }
      },
      { memcmp: { offset: 0, bytes: '21D35quxec7' } }
    ],
    accountConfig: {
      encoding: AccountEncoding.JsonParsed,
      dataSlice: { offset: 0, length: 100 },
      commitment: CommitmentLevelInternal.Finalized,
    }
  };

  await client.onProgramAccountChange(pubkey.toString(), config, accountChangeCallback);
  console.log(pubkey.toString(), config, accountChangeCallback);
});

// test.before(async () => {
//     payer = web3.Keypair.generate();
//     toAccount = web3.Keypair.generate();
//
//     const connection = new web3.Connection('http://127.0.0.1:8899', 'confirmed');
//
//     let airdropSignature = await connection.requestAirdrop(
//         payer.publicKey,
//         web3.LAMPORTS_PER_SOL,
//     );
//
//     await connection.confirmTransaction(airdropSignature);
// });
//
// test('benchmark send quic transaction', async (t) => {
//     console.time('Quic Transaction');
//
//     const connection = new web3.Connection('http://127.0.0.1:8899', 'confirmed');
//
//     let recentBlockhash = await connection.getRecentBlockhash();
//
//     let manualTransaction = new web3.Transaction({
//         recentBlockhash: recentBlockhash.blockhash,
//         feePayer: payer.publicKey,
//     }).add(
//         web3.SystemProgram.transfer({
//             fromPubkey: payer.publicKey,
//             toPubkey: toAccount.publicKey,
//             lamports: 1000, // Adjust lamports as needed
//         }),
//     );
//
//     let transactionBuffer = manualTransaction.serializeMessage();
//     let signature = nacl.sign.detached(transactionBuffer, payer.secretKey);
//     let signatureBuffer = Buffer.from(signature.buffer);
//
//     manualTransaction.addSignature(payer.publicKey, signatureBuffer);
//
//     const a = manualTransaction.serialize();
//
//     const quic_client = QuicClient.new('http://127.0.0.1:8899', 'ws://127.0.0.1:8900');
//     await quic_client.connect()
//
//     const ret = await quic_client.sendTransaction(a)
//     console.log('Quic transaction sent: ', ret)
//
//     console.timeEnd('Quic Transaction');
//     t.is(typeof ret, 'string')
// });
//
// test('benchmark send rpc transaction', async (t) => {
//     console.time('RPC Transaction');
//
//     const connection = new Connection('http://127.0.0.1:8899', 'confirmed');
//
//     let recentBlockhash = await connection.getRecentBlockhash();
//
//     let transaction = new Transaction({
//         recentBlockhash: recentBlockhash.blockhash
//     }).add(
//         SystemProgram.transfer({
//             fromPubkey: payer.publicKey,
//             toPubkey: toAccount.publicKey,
//             lamports: web3.LAMPORTS_PER_SOL / 100,
//         }),
//     );
//
//     transaction.sign(payer);
//
//     let ret = await connection.sendTransaction(transaction, [payer]);
//     await connection.confirmTransaction(ret);
//
//     console.log('RPC transaction sent: ', ret);
//
//     console.timeEnd('RPC Transaction');
//     t.is(typeof ret, 'string');
// });
