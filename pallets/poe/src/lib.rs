#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;


	#[pallet::config]
	pub trait Config: frame_system::Config {
        #[pallet::constant]
        type MaxClaimLength: Get<u32>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}
	
   
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


	#[pallet::storage]
	pub(super) type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8,T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimedTransfered(T::AccountId,T::AccountId,Vec<u8>),
    }

	#[pallet::error]
	pub enum Error<T> {
        ProofAlreadyExist,
        ClaimTooLong,
        ClaimNotExist,
		NotClaimOwner,
		NoSuchProof,
	
    }

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			//校验发送方是一个签名的
			let sender = ensure_signed(origin)?;
			let bounded_claim=BoundedVec::<u8,T::MaxClaimLength>::try_from(claim.clone()).map_err(|_|Error::<T>::ClaimTooLong)?;
            ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);
			//存储
			Proofs::<T>::insert(
				&bounded_claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number())
			);
			//
			Self::deposit_event(Event::ClaimCreated(sender, claim));
			Ok(().into())
		}

		#[pallet::weight(0)]
	
		pub fn revoke_claim(origin: OriginFor<T>, claim:  Vec<u8>) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let sender = ensure_signed(origin)?;
			let bounded_claim=BoundedVec::<u8,T::MaxClaimLength>::try_from(claim.clone()).map_err(|_|Error::<T>::ClaimTooLong)?;
            


			// Get owner of the claim, if none return an error.
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			// Verify that sender of the current call is the claim owner.
			ensure!(sender == owner, Error::<T>::NotClaimOwner);

			// Remove claim from storage.
			Proofs::<T>::remove(&bounded_claim);

			// Emit an event that the claim was erased.
			Self::deposit_event(Event::ClaimRevoked(sender, claim));
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn transfer_claim(origin: OriginFor<T>, dest: T::AccountId ,claim: Vec<u8>) -> DispatchResultWithPostInfo {
			//校验发送方是一个签名的
			let sender = ensure_signed(origin)?;
			let bounded_claim=BoundedVec::<u8,T::MaxClaimLength>::try_from(claim.clone()).map_err(|_|Error::<T>::ClaimTooLong)?;
            
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			// Verify that sender of the current call is the claim owner.
			ensure!(sender == owner, Error::<T>::NotClaimOwner);
			Proofs::<T>::remove(&bounded_claim);
			Proofs::<T>::insert(
				&bounded_claim,
				(dest.clone(), frame_system::Pallet::<T>::block_number())
			);
			//
			Self::deposit_event(Event::ClaimedTransfered(sender,dest,claim));
			Ok(().into())
		}

   

		

	}
}