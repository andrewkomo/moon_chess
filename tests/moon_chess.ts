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

async function setup_game(program,whitePlayer,blackPlayer,whiteTime,blackTime,whiteBonus,blackBonus) {
  const gameKeypair = anchor.web3.Keypair.generate();
  await program.rpc.setupGame(blackPlayer.publicKey, new anchor.BN(whiteTime), new anchor.BN(blackTime), 
    whiteBonus, blackBonus, {
    accounts: {
      game: gameKeypair.publicKey,
      whitePlayer: whitePlayer.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId
    },
    signers: [gameKeypair]
  });
  return gameKeypair;
}

describe("chess_game", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.local());

  const program = anchor.workspace.ChessGame as Program<ChessGame>;
  const whitePlayer = program.provider.wallet;
  const blackPlayer = anchor.web3.Keypair.generate();

  it("setup_game", async () => {
    const gameKeypair = await setup_game(program,whitePlayer,blackPlayer,100,100,1,1);

    let gameState = await program.account.game.fetch(gameKeypair.publicKey);
    expect(gameState.numMoves).to.equal(0);
  });

  it("play_16_moves_valid", async () => {
    const gameKeypair = await setup_game(program,whitePlayer,blackPlayer,100,100,1,1);
    for (let i=0;i<8;i++) {
      await play(program,gameKeypair.publicKey,whitePlayer,5,1,i,3,i);
      await play(program,gameKeypair.publicKey,blackPlayer,5,6,i,4,i);
      // console.log(i);
    }
    let gameState = await program.account.game.fetch(gameKeypair.publicKey);
    expect(gameState.numMoves).to.equal(16);
    expect(gameState.currBoard.halfMoves).to.equal(0);
  });

  it("play_repitition", async () => {
    const gameKeypair = await setup_game(program,whitePlayer,blackPlayer,100,100,1,1);
    for (let i=0;i<2;i++) {
      await play(program,gameKeypair.publicKey,whitePlayer,1,0,1,2,2);
      await play(program,gameKeypair.publicKey,blackPlayer,1,7,1,5,2);
      await play(program,gameKeypair.publicKey,whitePlayer,1,2,2,0,1);
      await play(program,gameKeypair.publicKey,blackPlayer,1,5,2,7,1);
    }
    let gameState = await program.account.game.fetch(gameKeypair.publicKey);
    expect(gameState.currBoard.halfMoves).to.equal(8);
    expect(Object.keys(gameState.status)[0]).to.equal('drawRepetition');
  });

  it("play_checkmate", async () => {
    const gameKeypair = await setup_game(program,whitePlayer,blackPlayer,100,100,1,1);
    await play(program,gameKeypair.publicKey,whitePlayer,5,1,4,3,4); // 1. e4
    await play(program,gameKeypair.publicKey,blackPlayer,5,6,5,4,5); // 1. f6
    await play(program,gameKeypair.publicKey,whitePlayer,1,0,1,2,2); // 2. Nc3
    await play(program,gameKeypair.publicKey,blackPlayer,5,6,6,4,6); // 2. g5
    await play(program,gameKeypair.publicKey,whitePlayer,3,0,3,4,7); // 3. Qh5#
    let gameState = await program.account.game.fetch(gameKeypair.publicKey);
    expect(Object.keys(gameState.status)[0]).to.equal('whiteWinCheckmate');
  });

  // it("play_evergreen_game", async () => {
  //   const gameKeypair = await setup_game(program,whitePlayer,blackPlayer,100,100,1,1);
  //   await play(program,gameKeypair.publicKey,whitePlayer,5,1,4,3,4); // 1. e4
  //   await play(program,gameKeypair.publicKey,blackPlayer,5,6,4,4,4); // 1. e5
  //   await play(program,gameKeypair.publicKey,whitePlayer,1,0,6,2,5); // 2. Nf3
  //   await play(program,gameKeypair.publicKey,blackPlayer,1,7,1,5,2); // 2. Nc6
  //   await play(program,gameKeypair.publicKey,whitePlayer,2,0,5,3,2); // 3. Bc4
  //   await play(program,gameKeypair.publicKey,blackPlayer,2,7,5,4,2); // 3. Bc5
  //   await play(program,gameKeypair.publicKey,whitePlayer,5,1,1,3,1); // 4. b4
  //   await play(program,gameKeypair.publicKey,blackPlayer,2,4,2,3,1); // 4. Bxb4
  //   await play(program,gameKeypair.publicKey,whitePlayer,5,1,2,2,2); // 5. c3
  //   await play(program,gameKeypair.publicKey,blackPlayer,2,3,1,4,0); // 5. Ba5
  //   let gameState = await program.account.game.fetch(gameKeypair.publicKey);
  //   console.log(gameState.currBoard.pieceBoard);
  //   console.log(gameState.currBoard.whiteBoard);
  // });
});
