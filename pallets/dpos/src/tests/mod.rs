use crate::mock::RuntimeOrigin;

// Short for runtime origin signed
pub fn ros(indx: u64) -> RuntimeOrigin {
	RuntimeOrigin::signed(indx)
}

#[cfg(test)]
mod delay_deregister_candidate;
#[cfg(test)]
mod delegate_candidate;
#[cfg(test)]
mod force_deregister_candidate;
#[cfg(test)]
mod force_undelegate_candidate;
#[cfg(test)]
mod register_as_candidate;
