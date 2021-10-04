const anchor = require('@project-serum/anchor');
const assert = require("assert");
const {
  TOKEN_PROGRAM_ID,
  sleep,
  getTokenAccount,
  createMint,
  createTokenAccount,
} = require("./utils");

describe('ftr_distributor', () => {
  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const program = anchor.workspace.FtrDistributor;
  const no_of_contracts = new anchor.BN(100);
  let totalNoOfContractsInDistributor = new anchor.BN(100);
  let distributorAccount = null;


  it('Initializes the state-of-the-world', async () => {
    // Add your test here.
    usdcMintToken = await createMint(provider);
    ftrMintToken = await createMint(provider);
    contractMintToken = await createMint(provider);
    usdcMint = usdcMintToken.publicKey;
    ftrMint = ftrMintToken.publicKey;
    contractMint = contractMintToken.publicKey;
    creatorContract = await createTokenAccount(
      provider,
      contractMint,
      provider.wallet.publicKey
    )
    await contractMintToken.mintTo(
      creatorContract,
      provider.wallet.publicKey,
      [],
      no_of_contracts.toString(),
    )
    creators_contract_account = await getTokenAccount(
      provider,
      creatorContract
    )
    assert.ok(creators_contract_account.amount.eq(no_of_contracts));
  });

  it("Initializes the distributor Account", async () => {
    //We use the ftr mint address as the seed, could use something else though
    const [_distributorSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [ftrMint.toBuffer()],
      program.programId
    );
    distributorSigner = _distributorSigner;
    distributorFtr = await createTokenAccount(
      provider,
      ftrMint,
      distributorSigner
    );
    distributorUsdc = await createTokenAccount(
      provider,
      usdcMint,
      distributorSigner
    );
    distributorContract = await createTokenAccount(
      provider,
      contractMint,
      distributorSigner
    );
    distributorAccount = anchor.web3.Keypair.generate();

    priceOfContract = new anchor.BN(100);
    ftrPerContract = new anchor.BN(1);
    noOfContract = new anchor.BN(100);

    await program.rpc.initializeDistributor(
      priceOfContract,
      ftrPerContract,
      noOfContract,
      nonce,
      {
        accounts: {
          distributorAccount: distributorAccount.publicKey,
          distributorSigner,
          distributorFtr,
          distributorContract,
          distributorUsdc,
          creatorContract,
          distributionAuthority: provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [distributorAccount],
        instructions: [
          await program.account.distributorAccount.createInstruction(distributorAccount),
        ],
      }
    );

    distributor_contract_account = await getTokenAccount(
      provider,
      distributorContract,
    )
    creators_contract_account = await getTokenAccount(
      provider,
      creatorContract
    );
    assert.ok(creators_contract_account.amount.eq(new anchor.BN(0)));

  });

  it("Update Distributor Account", async () => {

    priceOfContract = new anchor.BN(250);
    await program.rpc.updateDistributor(
      priceOfContract,
      null,
      {
        accounts: {
          distributorAccount: distributorAccount.publicKey,
          distributionAuthority: provider.wallet.publicKey,
        },
      }
      );
      distributor_account = await program.account.distributorAccount.fetch(distributorAccount.publicKey);
      assert(distributor_account.priceOfContract.eq(priceOfContract));
      assert(distributor_account.ftrPerContract.eq(ftrPerContract));
    });
    
    const userUsdcBalance = new anchor.BN(1500);
    const userFtrBalance = new anchor.BN(25);

    it("Deposit USDC,FTR and get back Contract Tokens", async () => {
      const noOfContractsRequired = new anchor.BN(4);
      const amountInUsdcRequired = new anchor.BN(1000);
      const amountInFtrRequired = new anchor.BN(4);
      
      // Total 100 - 10(after this rpc call) = 90
    userUsdc = await createTokenAccount(
      provider,
      usdcMint,
      provider.wallet.publicKey
    );
    await usdcMintToken.mintTo(
      userUsdc,
      provider.wallet.publicKey,
      [],
      userUsdcBalance.toString(),
    );
    userFtr = await createTokenAccount(
      provider,
      ftrMint,
      provider.wallet.publicKey
    );
    await ftrMintToken.mintTo(
      userFtr,
      provider.wallet.publicKey,
      [],
      userFtrBalance.toString(),
    );
    userContract = await createTokenAccount(
      provider,
      contractMint,
      provider.wallet.publicKey
    );

    await program.rpc.distribute(noOfContractsRequired,
      {
        accounts: {
          distributorAccount: distributorAccount.publicKey,
          distributorSigner,
          distributorUsdc,
          distributorFtr,
          distributorContract,
          userAuthority: provider.wallet.publicKey,
          userUsdc,
          userFtr,
          userContract,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      }
    );
    //Substract no of contracts distributed from the total 
    totalNoOfContractsInDistributor = totalNoOfContractsInDistributor.sub(noOfContractsRequired);
    distributorUsdcAccount = await getTokenAccount(provider, distributorUsdc);
    assert.ok(distributorUsdcAccount.amount.eq(amountInUsdcRequired));
    distributorFtrAccount = await getTokenAccount(provider, distributorFtr);
    assert.ok(distributorFtrAccount.amount.eq(amountInFtrRequired));
    distributorContractAccount = await getTokenAccount(provider, distributorContract);
    assert.ok(distributorContractAccount.amount.eq(totalNoOfContractsInDistributor));
  });

  it("Redeem Contract tokens and get back USDC,FTR", async () => {
    const noOfContractsRedeemed = new anchor.BN(4);
    const amountInUsdcRequired = new anchor.BN(1000);
    const amountInFtrRequired = new anchor.BN(4);

  await program.rpc.redeem(noOfContractsRedeemed,
    {
      accounts: {
        distributorAccount: distributorAccount.publicKey,
        distributorSigner,
        distributorUsdc,
        distributorFtr,
        distributorContract,
        userAuthority: provider.wallet.publicKey,
        userUsdc,
        userFtr,
        userContract,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    }
  );
  //Substract no of contracts distributed from the total 
  totalNoOfContractsInDistributor = totalNoOfContractsInDistributor.add(noOfContractsRedeemed);
  distributorUsdcAccount = await getTokenAccount(provider, distributorUsdc);
  assert.ok(distributorUsdcAccount.amount.eq(new anchor.BN(0)));
  distributorFtrAccount = await getTokenAccount(provider, distributorFtr);
  assert.ok(distributorFtrAccount.amount.eq(new anchor.BN(0)));
  distributorContractAccount = await getTokenAccount(provider, distributorContract);
  assert.ok(distributorContractAccount.amount.eq(totalNoOfContractsInDistributor));
  userUsdcWallet = await getTokenAccount(provider, userUsdc);
  assert.ok(userUsdcWallet.amount.eq(userUsdcBalance));
  userFtrWallet = await getTokenAccount(provider, userFtr);
  assert.ok(userFtrWallet.amount.eq(userFtrBalance));
  userContractWallet = await getTokenAccount(provider, userContract);
  assert.ok(userContractWallet.amount.eq(new anchor.BN(0)));
  debugger;
})
});
