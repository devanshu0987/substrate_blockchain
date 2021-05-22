#![cfg_attr(not(feature = "std"), no_std)]

//! Simple Token and Resource 
//! Balance
//! 1. set total supply
//! 2. establish ownership upon configuration of circulating tokens
//! 3. coordinate token transfers with the runtime functions
//! Resource
//! Set who owns which resource or rather which resource is owned by whom
//! In this system, there is only 1 owner
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use sp_std::vec::Vec;
use frame_support::sp_runtime::print;

#[cfg(test)]
mod tests;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
	trait Store for Module<T: Config> as Token {
		pub Balances get(fn get_balance): map hasher(blake2_128_concat) T::AccountId => u64;
		// we store resourceHash - accountId mappings here
		pub ResourceOwnership get(fn get_resource_ownership): map hasher(blake2_128_concat) Vec<u8> => T::AccountId;

		pub TotalSupply get(fn total_supply): u64 = 21000000;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Config>::AccountId,
	{
		/// Token was initialized by user
		Initialized(AccountId),
		/// Tokens successfully transferred between users
		Transfer(AccountId, AccountId, u64), // (from, to, value)
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Attempted to register account again.
		SenderAlreadyRegistered,
		/// Attempted to transfer more funds than were available
		InsufficientFunds,
		/// Resource not present
		ResourceNotPresent,
		SenderDoesNotOwnsResource,
		ReceiverNotRegistered,
		SenderNotRegistered,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Initialize the token
		/// transfers the total_supply amout to the caller
		#[weight = 10_000]
		fn register_account(origin) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			print("Inside register_account");
			//print(origin);
			//print(sender);
			//// we search the map to see if we already have registered it. if yes, then we
			// return already registered. else we just set the initial supply
			let status = <Balances<T>>::contains_key(&sender);
			print(status);
			ensure!(!status, <Error<T>>::SenderAlreadyRegistered);

			<Balances<T>>::insert(&sender, Self::total_supply());
			print("Post Balance insert");
			Ok(())
		}

		/// Initialize the token
		/// transfers the total_supply amout to the caller
		#[weight = 10_000]
		fn register_resource(origin, resource_hash: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			print("Inside register_resource");
			//print(origin);
			//print(sender);
			//print(resource_hash);
			// we search the balances map to see if we have registered the account or not
			// if yes, then only we go forward
			// return already registered. else we just set the initial supply
			let status = <Balances<T>>::contains_key(&sender);
			print(status);
			ensure!(status, <Error<T>>::SenderNotRegistered);

			// if account is present, then we register the resource to this person
			<ResourceOwnership<T>>::insert(resource_hash, &sender);
			print("Post Resource insert");
			Ok(())
		}

		/// Transfer tokens from one account to another
		#[weight = 10_000]
		pub fn transfer_balance(_origin, to: T::AccountId, value: u64) -> DispatchResult {
			let sender = ensure_signed(_origin)?;
			let sender_balance = Self::get_balance(&sender);
			let receiver_balance = Self::get_balance(&to);

			// Calculate new balances
			let updated_from_balance = sender_balance.checked_sub(value).ok_or(<Error<T>>::InsufficientFunds)?;
			let updated_to_balance = receiver_balance.checked_add(value).expect("Entire supply fits in u64; qed");

			// Write new balances to storage
			<Balances<T>>::insert(&sender, updated_from_balance);
			<Balances<T>>::insert(&to, updated_to_balance);

			Self::deposit_event(RawEvent::Transfer(sender, to, value));
			Ok(())
		}

		/// Transfer tokens from to to
		#[weight = 10_000]
		pub fn transfer_balance_from_to(_origin, from:T::AccountId, to: T::AccountId, value: u64) -> DispatchResult {
			// need to check what is origin? I think it just checks security
			// but I guess, it means that anyone can call this function and say, bro send money
			// we need to restrict it to tresury account, to and from somehow
			let _sender = ensure_signed(_origin)?;
			let sender_balance = Self::get_balance(&from);
			let receiver_balance = Self::get_balance(&to);

			// Calculate new balances
			let updated_from_balance = sender_balance.checked_sub(value).ok_or(<Error<T>>::InsufficientFunds)?;
			let updated_to_balance = receiver_balance.checked_add(value).expect("Entire supply fits in u64; qed");

			// Write new balances to storage
			<Balances<T>>::insert(&from, updated_from_balance);
			<Balances<T>>::insert(&to, updated_to_balance);

			Self::deposit_event(RawEvent::Transfer(from, to, value));
			Ok(())
		}

		/// Transfer resource from one account to another
		/// This is only possible if the origin owns the resource
		/// And we have transfered the tokens from newOwner to old owner
		#[weight = 10_000]
		pub fn transfer_resource(_origin, to: T::AccountId, resource_hash: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(_origin)?;
			// we can check if both exist in balances
			// we can check if owner exist in resource ownership
			let status = <ResourceOwnership<T>>::contains_key(&resource_hash);
			print(status);
			ensure!(status, <Error<T>>::ResourceNotPresent);

			let owner = Self::get_resource_ownership(&resource_hash);
			ensure!(sender == owner, <Error<T>>::SenderDoesNotOwnsResource);

			// then we check if new owner is present in balances because they should also exist in the system already
			let status = <Balances<T>>::contains_key(&to);
			print(status);
			ensure!(status, <Error<T>>::ReceiverNotRegistered);
			let status = <Balances<T>>::contains_key(&sender);
			print(status);
			ensure!(status, <Error<T>>::ReceiverNotRegistered);	

			// Write new ownership details to storage
			<ResourceOwnership<T>>::insert(resource_hash, &to);
			print("Post Resource transfer");
			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
    pub fn get_value() -> u32 {
        // just comment
		1
    }

	pub fn resource_exist(resource_hash : &Vec<u8>) -> bool {
		<ResourceOwnership<T>>::contains_key(&resource_hash)
	}
}
