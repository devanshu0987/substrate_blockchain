#![cfg_attr(not(feature = "std"), no_std)]

//! Auction Pallet
//! Register resource and accept bids
use frame_support::codec::{Decode, Encode};
use frame_support::sp_runtime::print;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

type AuctionDataOf<T> = AuctionData<<T as frame_system::Config>::AccountId>;

pub trait Config: frame_system::Config + pallet_basic_token_and_resource::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

#[derive(Encode, Decode, Clone, PartialEq)]
pub enum AuctionState {
	Open,
	Finished,
}

impl Default for AuctionState {
	fn default() -> AuctionState {
		AuctionState::Open
	}
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct BidData<AccountId> {
	bid_owner: AccountId,
	bid_amount: u64,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct AuctionData<AccountId> {
	resource_hash: Vec<u8>,
	state: AuctionState,
	initial_owner: Option<AccountId>,
	final_owner: Option<AccountId>,
	max_bid_owner: AccountId,
	max_bid: u64,
	// not good, but meh how to do that map thing. I dont know
	bid_list: Vec<BidData<AccountId>>,
}

decl_storage! {
	trait Store for Module<T: Config> as Auction {
		// stores open auctions to be returned
		// not waiting for iterable storage map query to be answered in discord
		pub OpenAuctions get(fn get_open_auctions): Vec<u64> = Vec::with_capacity(20);
		// reverse map of resource_hash and auctionId. Helps in quick find
		// we maintian that one resource will only be owned by one person and only 1 instance of auction will exist for it
		pub ResourceAuctionIdMapping get(fn get_mapping): map hasher(blake2_128_concat) Vec<u8> => u64;

		// auctionId and auction data mapping. We only maintain open auctions here.
		pub AuctionList get(fn get_auction_data): map hasher(blake2_128_concat) u64 => AuctionDataOf<T>;
		// auctionId
		pub NextAuctionId : u64 = 10;
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
		/// Auction already exist for this resource
		AuctionAlreadyExist,
		/// Auction does not exist for this resource
		AuctionDoesNotExist,
		/// Bid rejected because bid amount is less than current max bid
		BidRejected,
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

		/// Register resource for auction
		#[weight = 10_000]
		fn open_auction_for_resource(origin, resource_hash: Vec<u8>, initial_bid: u64) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			print("Inside open_auction_for_resource");
			// the origin should own this resource before opening auction for it
			let status = pallet_basic_token_and_resource::Module::<T>::resource_exist(&resource_hash);
			// I dont know if this Error will work or not?
			ensure!(status, <Error<T>>::ResourceNotPresent);
			print("Resource present");
			let owner = pallet_basic_token_and_resource::Module::<T>::get_resource_ownership(&resource_hash);
			ensure!(sender == owner, <Error<T>>::SenderDoesNotOwnsResource);
			print("Sender owns resource");
			// check reverse map
			let status = <ResourceAuctionIdMapping>::contains_key(&resource_hash);
			ensure!(!status, <Error<T>>::AuctionAlreadyExist);
			<ResourceAuctionIdMapping>::insert(&resource_hash, <NextAuctionId>::get());
			print("Sanity checks work");
			// Increment the id value
			let auction_id = <NextAuctionId>::get();
			let new_auction_id = auction_id.checked_add(1).expect("Entire increment fits in u64; qed");

			// this maintains the current set of open auctions
			let mut auctions = OpenAuctions::get();
			auctions.push(auction_id);
			<OpenAuctions>::put(auctions);

			// increment the auction id
			<NextAuctionId>::put(new_auction_id);

			let initial_auction_data = AuctionData {
				resource_hash : resource_hash,
				state : AuctionState::Open,
				initial_owner : Some(sender.clone()),
				final_owner : None,
				max_bid_owner : sender.clone(),
				max_bid : initial_bid,
				bid_list : Vec::with_capacity(20)
			};
			<AuctionList<T>>::insert(&auction_id, &initial_auction_data);
			print("Auction data inserted");
			Ok(())
		}

		#[weight = 10_000]
		fn finish_auction_for_resource(origin, auction_id: u64, resource_hash: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			print("Inside finish_auction_for_resource");
			// the origin should own this resource before closing auction for it
			let status = pallet_basic_token_and_resource::Module::<T>::resource_exist(&resource_hash);
			ensure!(status, <Error<T>>::ResourceNotPresent);
			print("Resource present");
			let owner = pallet_basic_token_and_resource::Module::<T>::get_resource_ownership(&resource_hash);
			ensure!(sender == owner, <Error<T>>::SenderDoesNotOwnsResource);
			print("Sender owns Resource");
			// check reverse map
			let status = <ResourceAuctionIdMapping>::contains_key(&resource_hash);
			ensure!(status, <Error<T>>::AuctionDoesNotExist);
			print("Auction exist");
			let auction_id_stored = Self::get_mapping(&resource_hash);
			ensure!(auction_id == auction_id_stored, <Error<T>>::AuctionDoesNotExist);
			// once you are here, you just have to manipulate the data
			let saved_auction = Self::get_auction_data(&auction_id);

			// modify the state
			print("Modify state");
			let mut copy = saved_auction.clone();
			copy.state = AuctionState::Finished;
			// set new owner
			copy.final_owner = Some(copy.max_bid_owner.clone());
			// insert back into the map
			<AuctionList<T>>::insert(&auction_id, &copy);
			// remove this from mapping because the auction is not valid anymore for the resourceHash
			// once finished, we can again create a new auction for this resource but that will be by the new owner
			<ResourceAuctionIdMapping>::remove(&resource_hash);

			// this maintains the current set of open auctions. We need to remove this auction id from open
			let mut index = 1000;
			let mut auctions = OpenAuctions::get();
			for (pos, item) in auctions.iter().enumerate()
			{
				if item == &auction_id_stored
				{
					index = pos;
					break;
				}
			}

			if index != 1000
			{
				// lets remove this from the auctions vector
				auctions.remove(index);
				// if you are here, then index can be only 1 thing
				<OpenAuctions>::put(auctions);
			}

			// if finished, then transfer money from bidder to owner.
			// due to my hack, the money is already present in the alice account
			// we need to return back the money to people other than the max bidder

			// if finshed, then we need to return back the money as well to the people.
			// we need to transfer resource from owner to bidder
			for (_pos, item) in copy.bid_list.iter().enumerate()
			{
				// we reutrn money for all bidders except max_bid_owner
				if item.bid_owner != saved_auction.max_bid_owner
				{
					let _res = pallet_basic_token_and_resource::Module::<T>::transfer_balance(
						origin.clone(), item.bid_owner.clone(), item.bid_amount);
					break;
				}
			}

			// change ownership for resource here
			let to = saved_auction.max_bid_owner;
			let _response = pallet_basic_token_and_resource::Module::<T>::transfer_resource(
				origin, to, saved_auction.resource_hash);
			print("Resource Transfer finished");
			Ok(())
		}

		#[weight = 10_000]
		fn bid_for_resource(origin, auction_id: u64, resource_hash: Vec<u8>, bid: u64) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			print("Inside bid_for_resource");

			// check reverse map
			let status = <ResourceAuctionIdMapping>::contains_key(&resource_hash);
			print(status);
			ensure!(status, <Error<T>>::AuctionDoesNotExist);

			let auction_id_stored = Self::get_mapping(&resource_hash);
			print("Auction id stored");
			print(auction_id_stored);

			print("Auction id input");
			print(auction_id);
			ensure!(auction_id == auction_id_stored, <Error<T>>::AuctionDoesNotExist);
			print("Auction exist");
			// once you are here, you just have to manipulate the data
			let saved_auction = Self::get_auction_data(&auction_id);
			print("We got auction data");

			// modify the state
			let mut copy = saved_auction.clone();
			// we modify 3 things
			// first check if we can accept bid or not. if total bid_amount is greater than the current max
			// search in vector to get previous bid amount from this account
			let mut previous_bid_amount = 0;
			// not expecting we will have 1K bids
			let mut index = 1000;
			for (pos, item) in copy.bid_list.iter().enumerate()
			{
				if item.bid_owner == sender
				{
					previous_bid_amount = item.bid_amount;
					index = pos;
					break;
				}
			}
			print("New bid amount");
			let new_bid_amount = previous_bid_amount + bid;
			print(new_bid_amount);
			ensure!(new_bid_amount > copy.max_bid, <Error<T>>::BidRejected);

			print("Bid accepted");
			// if you are here, then bid will be accepted. Hence modify stuff
			copy.max_bid_owner = sender.clone();
			copy.max_bid = new_bid_amount;
			if index != 1000
			{
				// which means we got a hit, so modify it
				copy.bid_list[index].bid_amount = new_bid_amount;
			}
			else
			{
				let bid = BidData
				{
					bid_owner: sender.clone(),
					bid_amount: new_bid_amount
				};
				copy.bid_list.push(bid);
			}

			// insert back into the map
			<AuctionList<T>>::insert(&auction_id, &copy);

			// if finished, then transfer bid_amount from bidder to Tresury.
			// I will register treasury account from backend when the backend starts.
			// which means the address will just be there, isnt it.
			// For example, alice is the treasury for us
			// we sent to alice, and then alice has to send back to me automatically
			// or I could hack it in this way : I send to the owner and then they dont have access
			// they cant get money out of it. So in that way it works
			// Lets test that hypothesis
			// _origin is the same origin
			// (_origin, to: T::AccountId, value: u64)
			// to : we can get it by current owner of the auction
			let to = saved_auction.initial_owner.unwrap();
			print("Tranfer balance");
			let _res = pallet_basic_token_and_resource::Module::<T>::transfer_balance(origin, to, bid);

			// I am not sure what we should do here with the res, but lets test this functionality
			// once
			print("Finish");
			Ok(())
		}
	}
}
