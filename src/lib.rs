#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_std::*;
use core::fmt::Debug;


#[derive(Debug, Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S: HasStateApi = StateApi> {
    transactions: StateMap<u32,Proposal,S>,
    admins: StateBox<Vec<Address>,S>,
}

impl State {
    pub fn voters(&self) -> usize {
        self.admins.len()
    }
    pub fn is_owner(&self, sender: &Address) -> bool {
        self.admins.contains(sender) 
     }
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
    receiptient: AccountAddress,
    voted: Vec<Address>,
    approvals: u8,
    fufilled: bool,
    owner: Address
}

impl IsOwner for Proposal{}

impl Proposal {

    pub fn new(
        index:u32, amount: Amount, receiptient: AccountAddress,
        approvals:u8, owner:Address, 
    ) -> Self {
        let voted = Vec::new();
        Proposal{index,amount,receiptient,voted,approvals,fufilled:false,owner}
    }

    pub fn vote(&mut self,ctx: &ReceiveContext, votes_needed: usize )->Result<bool,Error> {
       if self.voted.contains(&ctx.sender()) {
            return Err(Error::AlreadyVoted)
       }else {
            self.voted.push(ctx.sender());
            self.approvals += 1;
            Ok(self.approvals == votes_needed as u8)
       }
    }
    pub fn approved(&self, votes_needed: usize ) -> Result<bool,Error>{
        let num_of_voted = votes_needed as u8;
        Ok(self.approvals == num_of_voted)
    }
    
}

impl State {    
    pub fn new(state_builder: &mut StateBuilder, admins: Vec<Address> )-> Self {
            State { 
                transactions: state_builder.new_map(), 
                admins: state_builder.new_box(admins) 
            }
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
    AlreadyVoted,
    TransactionHasNotBeenApprovedOrAlreadyFufilled
}
#[derive(Serialize, SchemaType, Debug, PartialEq, Eq)]
pub struct InitParameter {
    admins : Vec<Address>
}
#[derive(Serialize, SchemaType)]
pub struct TxParameter {
    index: u32,
    receiver : AccountAddress,
    amount: Amount,
}

#[derive(Serialize, SchemaType, Debug, PartialEq, Eq)]
pub struct ApproveParameter {
    index: u32,
}

/// Init function that creates a new smart contract.
#[init(
    contract = "ccd_multisig",
    parameter="InitParameter"
)]
fn init(ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State> {
    // Your code
    let param: InitParameter = ctx.parameter_cursor().get()?;
    let admins = param.admins;
    let state = State::new(state_builder,admins);
    Ok(state)
   
}


#[receive( contract = "ccd_multisig", name = "transfer", parameter="TxParameter", mutable)]
fn transfer(ctx: &ReceiveContext, host: &mut Host<State>)-> ReceiveResult<()> {
    let param:TxParameter = ctx.parameter_cursor().get()?;
    let votes_needed = host.state().admins.len();
    let index = param.index;
    let approved = host.state_mut()
        .transactions
        .get(&index).unwrap().approved(votes_needed).unwrap();
    let not_fufilled = host.state_mut()
        .transactions
        .get_mut(&index).unwrap().fufilled == false;
    match (approved,not_fufilled) {
        (true,true) => {
            host.state_mut()
            .transactions
            .get_mut(&index).unwrap().fufilled = true;
            let response = host.invoke_transfer(&param.receiver, param.amount);
            match response{
                Ok(()) => Ok(()),
                Err(_) => bail!()
            }
        },
        _ => bail!()
    }
    
}

/// This function recieves CCD from anybody
#[receive(contract = "ccd_multisig", name = "insert", payable)]
#[allow(unused_variables)]
fn insert(ctx: &ReceiveContext,host: &Host<State>,amount: Amount
) -> ReceiveResult<()> {
    Ok(())
}

/// initialises a new transaction pending approval
#[receive(contract = "ccd_multisig", name = "create_tx", parameter="TxParameter", mutable)]
pub fn create_tx(ctx: &ReceiveContext,host: &mut Host<State>)-> ReceiveResult<u32>{
    let index = host.state().admins.len() as u32;
    let param:TxParameter = ctx.parameter_cursor().get()?;
    let proposal = Proposal::new(index,param.amount,param.receiver,0,ctx.sender());
    host.state_mut().transactions.insert(index, proposal);
    Ok(index)
}

#[receive(contract = "ccd_multisig", name = "approve", parameter="ApproveParameter", mutable)]
pub fn approve(ctx: &ReceiveContext,host: &mut Host<State>)-> ReceiveResult<bool>{
    let param:ApproveParameter = ctx.parameter_cursor().get()?;
    let index = param.index;
    let votes_needed = host.state().voters();
    let mut proposal = host.state_mut().transactions.get_mut(&index).unwrap();
    ensure_eq!(index,proposal.index);
    let approved = proposal.vote(ctx,votes_needed)?;
    Ok(approved)
}

///View function that returns the content of the state.
#[receive(contract = "ccd_multisig", name = "view", return_value = "Proposal")]
fn view<'a, 'b>(ctx: &'a ReceiveContext, host: &'b Host<State>) ->   ReceiveResult<Proposal> {
    let param:ApproveParameter = ctx.parameter_cursor().get()?;
    let prop = host.state().transactions.get(&param.index).unwrap();
    let mut voted = Vec::new();
    prop.voted.iter().for_each(|i| voted.push(*i));
    let (index,amount,receiptient, approvals,fufilled, owner) = (prop.index,prop.amount,prop.receiptient,prop.approvals,prop.fufilled,prop.owner);
    Ok(Proposal{index,amount,receiptient,voted,approvals,fufilled,owner})

}
