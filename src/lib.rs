#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_std::*;
use core::fmt::Debug;


#[derive(Debug, Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S: HasStateApi = StateApi> {
    pub transactions: StateMap<u32,Proposal,S>,
    pub admins: StateBox<Vec<Address>,S>,
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
    pub index: u32,
    pub amount: Amount,
    pub receiptient: AccountAddress,
    pub voted: Vec<Address>,
    pub approvals: u8,
    pub fufilled: bool,
    pub owner: Address
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

    pub fn approve(&mut self,ctx: &ReceiveContext, votes_needed: usize )->Result<bool,Error> {
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
    pub admins : Vec<Address>
}
#[derive(Serialize, SchemaType)]
pub struct TxParameter {
    pub index: u32,
    pub receiver : AccountAddress,
    pub amount: Amount,
}

impl TxParameter {
    pub fn default() -> Self {
        TxParameter { index: 0, receiver: AccountAddress([0u8; 32]), amount: (Amount { micro_ccd: 0 }) }
    }
    pub fn new(index:u32, receiver:AccountAddress,amount:u64) -> Self {
        TxParameter { index, receiver, amount: Amount { micro_ccd: amount } }
    }
}

#[derive(Serialize, SchemaType, Debug, PartialEq, Eq)]
pub struct ApproveParameter {
    pub index: u32,
}

impl ApproveParameter {
    pub fn new(index: u32) -> Self {
        Self {index}
    }
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


#[receive( contract = "ccd_multisig", name = "transfer", parameter="ApproveParameter", mutable)]
fn transfer(ctx: &ReceiveContext, host: &mut Host<State>)-> ReceiveResult<()> {
    let param:ApproveParameter = ctx.parameter_cursor().get()?;
    let votes_needed = host.state().admins.len();
    let index = param.index;
    let approved = host.state_mut()
        .transactions
        .get(&index).unwrap().approved(votes_needed).unwrap();
    let not_fufilled = host.state_mut()
        .transactions
        .get_mut(&index).unwrap().fufilled == false;
    let amount = host.state_mut()
        .transactions
        .get(&index).unwrap().amount;
    let receiptient = host.state_mut()
        .transactions
        .get(&index).unwrap().receiptient;
    if host.self_balance() < amount {
        bail!()
    }
    match (approved,not_fufilled) {
        (true,true) => {
            host.state_mut()
            .transactions
            .get_mut(&index).unwrap().fufilled = true;
            let response = host.invoke_transfer(&receiptient, amount);
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
    if host.state().is_owner(&ctx.sender()){
        let votes_needed = host.state().voters();
        let mut proposal = host.state_mut().transactions.get_mut(&index)
            .expect("The key does not exist");
        ensure_eq!(index,proposal.index);
        let approved = proposal.approve(ctx,votes_needed)?;
        Ok(approved)
    }else{
        bail!()
    }
    
}

///View function that returns the content of the state.
#[receive(contract = "ccd_multisig", name = "view",parameter="ApproveParameter",return_value = "Proposal")]
fn view<'a, 'b>(ctx: &'a ReceiveContext, host: &'b Host<State>) ->   ReceiveResult<Proposal> {
    let param:ApproveParameter = ctx.parameter_cursor().get()?;
    let prop = host.state().transactions.get(&param.index).unwrap();
    let mut voted = Vec::new();
    prop.voted.iter().for_each(|i| voted.push(*i));
    let (index,amount,receiptient, approvals,fufilled, owner) = (prop.index,prop.amount,prop.receiptient,prop.approvals,prop.fufilled,prop.owner);
    Ok(Proposal{index,amount,receiptient,voted,approvals,fufilled,owner})

}

#[receive(contract = "ccd_multisig", name = "get_admins",parameter="ApproveParameter",return_value = "Proposal")]
fn get_admins<'a, 'b>(_ctx: &'a ReceiveContext, host: &'b Host<State>) ->   ReceiveResult<Vec<Address>> {
    let admins = host.state().admins.clone();
    let mut voted = Vec::new();
    admins.iter().for_each(|admin| voted.push(*admin));
    Ok(voted)
}
