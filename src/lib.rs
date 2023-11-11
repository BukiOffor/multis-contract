#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_std::*;
use core::fmt::Debug;


#[derive(Serialize, SchemaType, Debug, PartialEq, Eq)]
pub struct State{
    transactions: collections::BTreeMap<u8,Proposal>,
    admins: Vec<Address>,
    //transactions: Vec<Proposal>
}

pub trait IsOwner {
    fn is_owner(&self, host: &Host<State>, sender: &Address) -> bool {
       host.state().admins.contains(sender) 
    }

    fn voters(&self, host: &Host<State>) -> usize {
        host.state().admins.len()
    }

}

// proposal <amount, address>
// approve 
#[derive(Serialize, SchemaType, Debug, PartialEq, Eq)]
pub struct Proposal {
    index: u32,
    amount: Amount,
    receiptient: Address,
    voted: Vec<Address>,
    approvals: u8,
    fufilled: bool,
    owner: Address
}

impl IsOwner for Proposal{}

impl Proposal {

    pub fn new(
        index:u32, amount: Amount, receiptient: Address,
        approvals:u8, owner:Address, 
    ) -> Self {
        let voted = Vec::new();
        Proposal{index,amount,receiptient,voted,approvals,fufilled:false,owner}
    }

    pub fn vote(&mut self,ctx: &ReceiveContext, host: &Host<State>)->Result<bool,Error> {
       if self.voted.contains(&ctx.sender()) {
            return Err(Error::AlreadyVoted)
       }else {
            let sender = ctx.sender().clone();
            self.voted.push(sender);
            self.approvals += 1;
            let num_of_voted = self.voters(host) as u8;
            Ok(self.approvals == num_of_voted)
       }
    }
    
}

impl State {    
    pub fn new()-> Self {
        let transactions = collections::BTreeMap::new();
        let admins = Vec::new();
        State { transactions, admins }
    }
}

/// Your smart contract errors.
#[derive(Debug, PartialEq, Eq, Reject, Serialize, SchemaType)]
pub enum Error {
    /// Failed parsing the parameter.
    #[from(ParseError)]
    ParseParams,
    /// Your error
    YourError,
    AlreadyVoted
}
#[derive(Serialize, SchemaType, Debug, PartialEq, Eq)]
pub struct InitParameter {
    admins : Vec<Address>
}

/// Init function that creates a new smart contract.
#[init(
    contract = "ccd_multisig",
    parameter="InitParameter"
)]
fn init(_ctx: &InitContext, _state_builder: &mut StateBuilder) -> InitResult<State> {
    // Your code
    let state = State::new();
    Ok(state)
}


#[receive( contract = "ccd_multisig", name = "transfer",)]
fn transfer(_ctx: &ReceiveContext, host: &Host<State>)-> ReceiveResult<()> {
    let _balance = host.self_balance();
    //host.invoke_transfer(receiver, amount)
    Ok(())
}

/// This function recieves CCD from anybody
#[receive(contract = "ccd_multisig", name = "insert", payable)]
#[allow(unused_variables)]
fn insert(ctx: &ReceiveContext,host: &Host<State>,amount: Amount
) -> ReceiveResult<()> {
    Ok(())
}
#[receive(contract = "ccd_multisig", name = "create_tx", mutable)]
pub fn create_tx(_ctx: &ReceiveContext,host: &mut Host<State>)-> ReceiveResult<u32>{
    let index = host.state().admins.len() as u32;
    Ok(index)
}

/// View function that returns the content of the state.
#[receive(contract = "ccd_multisig", name = "view", return_value = "State")]
fn view<'b>(_ctx: &ReceiveContext, host: &'b Host<State>) -> ReceiveResult<&'b State> {
    Ok(host.state())
}
