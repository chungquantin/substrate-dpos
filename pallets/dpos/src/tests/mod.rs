use crate::mock::RuntimeOrigin;

// Short for runtime origin signed
pub fn ros(indx: u64) -> RuntimeOrigin {
	RuntimeOrigin::signed(indx)
}

#[cfg(test)]
mod delegate_candidate;
#[cfg(test)]
mod deregister_candidate;
#[cfg(test)]
mod register_as_candidate;
#[cfg(test)]
mod undelegate_candidate;
