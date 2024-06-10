pub mod candidate;
use core::marker::PhantomData;

pub use candidate::*;

pub mod delay;
pub use delay::*;

pub mod delegate;
pub use delegate::*;

pub mod epoch;
pub use epoch::*;
use sp_core::Get;

pub struct AddGet<T, R> {
	_phantom: PhantomData<(T, R)>,
}
impl<T, R> Get<u32> for AddGet<T, R>
where
	T: Get<u32>,
	R: Get<u32>,
{
	fn get() -> u32 {
		T::get() + R::get()
	}
}

pub type DispatchResultWithValue<T> = Result<T, sp_runtime::DispatchError>;
