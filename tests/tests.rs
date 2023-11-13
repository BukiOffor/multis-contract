use concordium_smart_contract_testing::*;
use ccd_multisig::*;
use std::str::FromStr;


/// A test account.
const ALICE: AccountAddress = AccountAddress([0u8; 32]);
const ALICE_ADDR: Address = Address::Account(ALICE);
const BOB: AccountAddress = AccountAddress([1u8; 32]);
const BOB_ADDR: Address = Address::Account(BOB);
const MIKE: AccountAddress = AccountAddress([2u8; 32]);
const MIKE_ADDR: Address = Address::Account(MIKE);
const SETH: AccountAddress = AccountAddress([3u8; 32]);
const SETH_ADDR: Address = Address::Account(SETH);


/// The initial balance of the ALICE test account.
const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(100000_000);

/// A [`Signer`] with one set of keys, used for signing transactions.
const SIGNER: Signer = Signer::with_one_key();




/// Test that invoking the `receive` endpoint with the `false` parameter
/// succeeds in updating the contract.
#[test]
fn test_can_send_ccd_to_contract() {
    let (mut chain, init) = initialize();
    // Update the contract via the `receive` entrypoint with the parameter `false`.
    chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10_000), UpdateContractPayload {
            address:      init.contract_address,
            amount:       Amount::from_ccd(50_000),
            receive_name: OwnedReceiveName::new_unchecked("ccd_multisig.insert".to_string()),
            message:      OwnedParameter::empty()
        })
        .unwrap();
}

#[test]
fn test_creating_a_transaction(){
    let (mut chain, init) = initialize();
    let param = TxParameter{
        index: 0,
        amount:Amount::from_ccd(100_000),
        receiver: BOB
    };
    // Update the contract via the `receive` entrypoint with the parameter `false`.
    chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10_000), UpdateContractPayload {
            address:      init.contract_address,
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("ccd_multisig.create_tx".to_string()),
            message:      OwnedParameter::from_serial(&param).unwrap()
        })
        .unwrap();
    
}
#[test]
fn test_is_address(){
    let address = Address::from_str("45FWHaAQz44w5VrcrX7XUeNHGwTvPHWRZGSUsdekqyw44Tz2iu").unwrap();
    assert_eq!(address.is_account(), true);
}

#[test]
fn test_is_acc_address(){
    let address = AccountAddress::from_str("45FWHaAQz44w5VrcrX7XUeNHGwTvPHWRZGSUsdekqyw44Tz2iu").unwrap();
    println!("{}", address)
    //assert_eq!(address.is_account(), true);
}



#[test]
fn test_view_proposal(){
   todo!()
}


#[test]
fn test_approving_a_transaction(){
    todo!()
}


#[test]
fn test_should_fail_if_none_admin_tries_to_approve_transaction(){
    todo!()    
}


#[test]
fn test_excute_transaction_if_approved(){
    todo!()

}

#[test]
fn test_should_fail_if_transaction_is_not_approved(){
    todo!()

}



#[test]
fn test_should_fail_if_admin_tries_to_approve_twice(){
    todo!()
}


// cargo run -- --node http://node.testnet.concordium.com:20000 --account ~/3UsPQ4MxhGNLEbYac53H7C2JHzE3Xe41zrgCdLVrp5vphx4YSe.export --module ~/ccd-multisig/concordium-out/module.wasm.v1

//DEPLOYING CONTRACT
// 19f525965028ce5d5b1956118da1a4d74577381451cdd839df1b329661803d0b <-- module ref
// c14afeeba9a260ef2365138ad0b10b27e39667e5588ae99df216efd9002d0660 <-- deployment tx-hash

// INITIALIZING CONTRACT
// 96051b1d4fab7b5c0d2d160715a3468a87b49673ab43f3f51d8d5a7bd7d35073 <-- tx-hash

// UPDATING CONTRACT 
// 5f9730f97be67c7e3f2b0365e80780357a419280b0e6dc2208441377a93bb710 <-- tx-hash






/// Helper method for initializing the contract.
///
/// Does the following:
///  - Creates the [`Chain`]
///  - Creates one account, `Alice` with `10_000` CCD as the initial balance.
///  - Initializes the contract.
///  - Returns the [`Chain`] and the [`ContractInitSuccess`]
fn initialize() -> (Chain, ContractInitSuccess) {
    // Initialize the test chain.
    let mut chain = Chain::new();

    // Create the test account.
    chain.create_account(Account::new(ALICE, ACC_INITIAL_BALANCE));
    chain.create_account(Account::new(BOB, ACC_INITIAL_BALANCE));
    chain.create_account(Account::new(MIKE, ACC_INITIAL_BALANCE));
    chain.create_account(Account::new(SETH, ACC_INITIAL_BALANCE));




    // Load the module.
    let module = module_load_v1("./concordium-out/module.wasm.v1").expect("Module exists at path");
    // Deploy the module.
    let deployment = chain.module_deploy_v1(SIGNER, ALICE, module).expect("Deploy valid module");
    let mut param = InitParameter{admins:Vec::new()};
    param.admins.push(ALICE_ADDR);
    param.admins.push(BOB_ADDR);
    param.admins.push(MIKE_ADDR);

    // Initialize the contract.
    let init = chain
        .contract_init(SIGNER, ALICE, Energy::from(10_000), InitContractPayload {
            amount:    Amount::zero(),
            mod_ref:   deployment.module_reference,
            init_name: OwnedContractName::new_unchecked("init_ccd_multisig".to_string()),
            param:     OwnedParameter::from_serial(&param).unwrap(),
        })
        .expect("Initializing contract");

    (chain, init)
}