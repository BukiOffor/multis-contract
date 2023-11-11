#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_std::*;
use core::fmt::Debug;


#[derive(Serialize, SchemaType)]
pub struct State{
    approvals: collections::BTreeMap<AccountAddress,bool>,
    admins: Vec<AccountAddress>
}


impl State {    
    pub fn new()-> Self {
        let approvals = collections::BTreeMap::new();
        let admins = Vec::new();
        State { approvals, admins }
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
}

/// Init function that creates a new smart contract.
#[init(contract = "ccd_multisig")]
fn init(_ctx: &InitContext, _state_builder: &mut StateBuilder) -> InitResult<State> {
    // Your code
    let state = State::new();

    Ok(state)
}

pub type MyInputType = bool;

/// Receive function. The input parameter is the boolean variable `throw_error`.
///  If `throw_error == true`, the receive function will throw a custom error.
///  If `throw_error == false`, the receive function executes successfully.
#[receive(
    contract = "ccd_multisig",
    name = "receive",
    parameter = "MyInputType",
    error = "Error",
    mutable
)]
fn receive(ctx: &ReceiveContext, _host: &mut Host<State>) -> Result<(), Error> {
    // Your code

    let throw_error = ctx.parameter_cursor().get()?; // Returns Error::ParseError on failure
    if throw_error {
        Err(Error::YourError)
    } else {
        Ok(())
    }
}

/// View function that returns the content of the state.
#[receive(contract = "ccd_multisig", name = "view", return_value = "State")]
fn view<'b>(_ctx: &ReceiveContext, host: &'b Host<State>) -> ReceiveResult<&'b State> {
    Ok(host.state())
}