use crate::mock::RuntimeOrigin;

// Short for runtime origin signed
pub fn ros(indx: u64) -> RuntimeOrigin {
	RuntimeOrigin::signed(indx)
}

#[cfg(test)]
mod test_candidate_bond_less;
#[cfg(test)]
mod test_candidate_bond_more;
#[cfg(test)]
mod test_delay_deregister_candidate;
#[cfg(test)]
mod test_delay_undelegate_candidate;
#[cfg(test)]
mod test_delegate_candidate;
#[cfg(test)]
mod test_force_deregister_candidate;
#[cfg(test)]
mod test_force_undelegate_candidate;
mod test_helpers;
#[cfg(test)]
mod test_register_as_candidate;
#[cfg(test)]
mod test_validator_election;
