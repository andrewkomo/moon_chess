import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ChessGame } from "../target/types/chess_game";
import { expect } from 'chai';

async function play(program, game, player, piece, from_rank, from_col, to_rank, to_col) {
  await program.rpc.play((piece << 12)+(from_rank << 9)+(from_col << 6)+(to_rank << 3)+(to_col), {
    accounts: {
      player: player.publicKey,
      game
    },
    signers: player instanceof (anchor.Wallet as any) ? [] : [player]
  });
}

describe("chess_game", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.local());

  const program = anchor.workspace.ChessGame as Program<ChessGame>;
  const gameKeypair = anchor.web3.Keypair.generate();
  const whitePlayer = program.provider.wallet;
  const blackPlayer = anchor.web3.Keypair.generate();

  it("setup_game", async () => {
    await program.rpc.setupGame(blackPlayer.publicKey, new anchor.BN(100), new anchor.BN(100), 1, 1, {
      accounts: {
        game: gameKeypair.publicKey,
        whitePlayer: whitePlayer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      },
      signers: [gameKeypair]
    });

    let gameState = await program.account.game.fetch(gameKeypair.publicKey);
    expect(gameState.numMoves).to.equal(0);
    console.log(gameState);
  });

  it("play_16_moves_valid", async () => {
    for (let i=0;i<16;i++) {
      await play(program,gameKeypair.publicKey,whitePlayer,5,1,i,3,i);
      console.log(i)
      await play(program,gameKeypair.publicKey,blackPlayer,5,6,i,4,i);
      console.log(i);
    }
  });
});
