import {
  LIQUIDITY_STATE_LAYOUT_V4,
  MAINNET_PROGRAM_ID,
  Token,
  TokenAmount,
} from '@raydium-io/raydium-sdk';
import { QuicClient } from './index.js';
import bs58 from 'bs58';

const RAYDIUM_LIQUIDITY_PROGRAM_ID_V4 = MAINNET_PROGRAM_ID.AmmV4;
const OPENBOOK_PROGRAM_ID = MAINNET_PROGRAM_ID.OPENBOOK_MARKET;

const run = async () => {
  const client = QuicClient.new(
    'https://api.mainnet-beta.solana.com',
    'wss://api.mainnet-beta.solana.com'
  );
  await client.connect();

  const config = {
    filters: [
      { dataSize: LIQUIDITY_STATE_LAYOUT_V4.span },
      {
        memcmp: {
          offset: LIQUIDITY_STATE_LAYOUT_V4.offsetOf('quoteMint'),
          bytes: Token.WSOL.mint.toBase58(),
        },
      },
      {
        memcmp: {
          offset: LIQUIDITY_STATE_LAYOUT_V4.offsetOf('marketProgramId'),
          bytes: OPENBOOK_PROGRAM_ID.toBase58(),
        },
      },
      {
        memcmp: {
          offset: LIQUIDITY_STATE_LAYOUT_V4.offsetOf('status'),
          bytes: bs58.encode([6, 0, 0, 0, 0, 0, 0, 0]),
        },
      },
    ],
    accountConfig: {
      encoding: 'base64',
      dataSlice: { offset: 0, length: 100 },
      commitment: "finalized"
    }
  }

  await client.onProgramAccountChange(
    RAYDIUM_LIQUIDITY_PROGRAM_ID_V4.toString(),
    config,
    (err: any, updatedAccountInfo: any) => {
      if (err) {
        console.error('Error receiving account change notification:', err);
        return;
      }
      const account = JSON.parse(updatedAccountInfo);
      const b = Buffer.from(account.account.data[0], 'base64');
      const poolState = LIQUIDITY_STATE_LAYOUT_V4.decode(b);

      const poolSize = new TokenAmount(Token.WSOL, poolState.swapQuoteInAmount, true);
      console.info(`Processing pool: ${account.pubkey} with ${poolSize.toFixed()} ${Token.WSOL.symbol} in liquidity`);
    }
  );

}

run()
