#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::{
		sp_runtime::traits::Hash,
        traits::{ Randomness, Currency, tokens::ExistenceRequirement },
        transactional
    };
    use sp_io::hashing::blake2_128;
	use scale_info::TypeInfo;

	#[cfg(feature = "std")]
    use frame_support::serde::{Deserialize, Serialize};

    // ACTION #1: Write a Struct to hold Kitty information.
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = 
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	
	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}

    // ACTION #2: Enum declaration for Gender.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

    // ACTION #3: Implementation to handle Gender type in Kitty struct.

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types it depends on.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The Currency handler for the Kitties pallet.
        type Currency: Currency<Self::AccountId>;

        // ACTION #5: Specify the type for Randomness we want to specify for runtime.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

        // ACTION #9: Add MaxKittyOwned constant
        #[pallet::constant]
        type MaxKittyOwned: Get<u32>;
    }

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// Handles arithmetic overflow when incrementing the Kitty counter.
        CountForKittiesOverflow,

        /// An account cannot own more Kitties than `MaxKittyCount`.
        ExceedMaxKittyOwned,

        /// Buyer cannot be the owner.
        BuyerIsKittyOwner,

        /// Cannot transfer a kitty to its owner.
        TransferToSelf,

        /// This kitty already exists
        KittyExists,

        /// This kitty doesn't exist
        KittyNotExist,

        /// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.
        NotKittyOwner,

        /// Ensures the Kitty is for sale.
        KittyNotForSale,

        /// Ensures that the buying price is greater than the asking price.
        KittyBidPriceTooLow,

        /// Ensures that an account has enough funds to purchase a Kitty.
        NotEnoughBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config>{

        /// A new Kitty was successfully created. \[sender, kitty_id\]
        Created(T::AccountId, T::Hash),

        /// Kitty price was successfully set. \[sender, kitty_id, new_price\]
        PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),

        /// A Kitty was successfully transferred. \[from, to, kitty_id\]
        Transferred(T::AccountId, T::AccountId, T::Hash),

        /// A Kitty was successfully bought. \[buyer, seller, kitty_id, bid_price\]
        Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
    }

    // ACTION: Storage item to keep a count of all existing Kitties.

    // TODO Part II: Remaining storage items.
	#[pallet::storage]
	#[pallet::getter(fn count_for_kitties)]
	pub(super) type CountForKitties<T: Config> = StorageValue<
        _,
        u64,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub(super) type Kitties<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::Hash,
        Kitty<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn kitties_owned)]
    pub(super) type KittiesOwned<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BoundedVec<T::Hash, T::MaxKittyOwned>,
        ValueQuery,
    >;

    // TODO Part III: Our pallet's genesis configuration.

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        // TODO Part III: create_kitty
        #[pallet::weight(100)]
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::mint(&sender, None, None)?;
            // Logging to the console
            log::info!("A kitty is born with ID: {:?}.", kitty_id);
        
            // ACTION #4: Deposit `Created` event
            Self::deposit_event(Event::Created(sender, kitty_id));
        
            Ok(())
        }

        // TODO Part III: set_price

        // TODO Part III: transfer

        // TODO Part III: buy_kitty

        // TODO Part III: breed_kitty
    }

    // TODO Part II: helper function for Kitty struct

    impl<T: Config> Pallet<T> {
		// TODO Part III: helper functions for dispatchable functions
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;

			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}

		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::extrinsic_index().unwrap_or_default(),
				<frame_system::Pallet<T>>::block_number(),
			);

			payload.using_encoded(blake2_128)
		}

        // TODO: increment_nonce, random_hash, mint, transfer_from
        pub fn mint(
            owner: &T::AccountId,
            dna: Option<[u8; 16]>,
            gender: Option<Gender>,
        ) -> Result<T::Hash, Error<T>> {
            let kitty = Kitty::<T> {
                dna: dna.unwrap_or_else(Self::gen_dna),
                price: None,
                gender: gender.unwrap_or_else(Self::gen_gender),
                owner: owner.clone(),
            };
        
            let kitty_id = T::Hashing::hash_of(&kitty);
        
            // Performs this operation first as it may fail
            let new_cnt = Self::count_for_kitties().checked_add(1)
                .ok_or(<Error<T>>::CountForKittiesOverflow)?;
        
            // Check if the kitty does not already exist in our storage map
            ensure!(Self::kitties(&kitty_id) == None, <Error<T>>::KittyExists);
        
            // Performs this operation first because as it may fail
            <KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| {
                kitty_vec.try_push(kitty_id)
            }).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;
        
            <Kitties<T>>::insert(kitty_id, kitty);
            <CountForKitties<T>>::put(new_cnt);
            Ok(kitty_id)
        }
    }
}