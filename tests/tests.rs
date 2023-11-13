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