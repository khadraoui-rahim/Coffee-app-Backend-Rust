use crate::orders::OrderStatus;

/// Service for managing order status transitions
pub struct StatusMachine;

impl StatusMachine {
    /// Check if a status transition is valid
    /// 
    /// # Arguments
    /// * `from` - Current order status
    /// * `to` - Desired new status
    /// 
    /// # Returns
    /// `true` if the transition is valid, `false` otherwise
    /// 
    /// # Valid Transitions
    /// - Pending → Confirmed, Cancelled
    /// - Confirmed → Preparing, Cancelled
    /// - Preparing → Ready, Cancelled
    /// - Ready → Completed, Cancelled
    /// - Completed → Cancelled (refund scenario)
    /// - Cancelled → (no transitions allowed except to itself)
    /// - Any status → Same status (idempotent)
    pub fn is_valid_transition(from: OrderStatus, to: OrderStatus) -> bool {
        // Same status is always valid (idempotent)
        if from == to {
            return true;
        }
        
        match (from, to) {
            // From Pending
            (OrderStatus::Pending, OrderStatus::Confirmed) => true,
            (OrderStatus::Pending, OrderStatus::Cancelled) => true,
            
            // From Confirmed
            (OrderStatus::Confirmed, OrderStatus::Preparing) => true,
            (OrderStatus::Confirmed, OrderStatus::Cancelled) => true,
            
            // From Preparing
            (OrderStatus::Preparing, OrderStatus::Ready) => true,
            (OrderStatus::Preparing, OrderStatus::Cancelled) => true,
            
            // From Ready
            (OrderStatus::Ready, OrderStatus::Completed) => true,
            (OrderStatus::Ready, OrderStatus::Cancelled) => true,
            
            // From Completed
            (OrderStatus::Completed, OrderStatus::Cancelled) => true,
            
            // From Cancelled - no transitions allowed (except to itself, handled above)
            (OrderStatus::Cancelled, _) => false,
            
            // All other transitions are invalid
            _ => false,
        }
    }

    /// Attempt to transition from one status to another
    /// 
    /// # Arguments
    /// * `from` - Current order status
    /// * `to` - Desired new status
    /// 
    /// # Returns
    /// `Ok(to)` if the transition is valid, `Err(message)` otherwise
    pub fn transition(from: OrderStatus, to: OrderStatus) -> Result<OrderStatus, String> {
        if Self::is_valid_transition(from, to) {
            Ok(to)
        } else {
            Err(format!(
                "Invalid status transition from {} to {}",
                from, to
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test valid transitions from Pending
    #[test]
    fn test_pending_to_confirmed() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Pending,
            OrderStatus::Confirmed
        ));
    }

    #[test]
    fn test_pending_to_cancelled() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Pending,
            OrderStatus::Cancelled
        ));
    }

    // Test valid transitions from Confirmed
    #[test]
    fn test_confirmed_to_preparing() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Confirmed,
            OrderStatus::Preparing
        ));
    }

    #[test]
    fn test_confirmed_to_cancelled() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Confirmed,
            OrderStatus::Cancelled
        ));
    }

    // Test valid transitions from Preparing
    #[test]
    fn test_preparing_to_ready() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Preparing,
            OrderStatus::Ready
        ));
    }

    #[test]
    fn test_preparing_to_cancelled() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Preparing,
            OrderStatus::Cancelled
        ));
    }

    // Test valid transitions from Ready
    #[test]
    fn test_ready_to_completed() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Ready,
            OrderStatus::Completed
        ));
    }

    #[test]
    fn test_ready_to_cancelled() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Ready,
            OrderStatus::Cancelled
        ));
    }

    // Test valid transition from Completed
    #[test]
    fn test_completed_to_cancelled() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Completed,
            OrderStatus::Cancelled
        ));
    }

    // Test no transitions from Cancelled
    #[test]
    fn test_cancelled_to_pending() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Cancelled,
            OrderStatus::Pending
        ));
    }

    #[test]
    fn test_cancelled_to_confirmed() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Cancelled,
            OrderStatus::Confirmed
        ));
    }

    #[test]
    fn test_cancelled_to_preparing() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Cancelled,
            OrderStatus::Preparing
        ));
    }

    #[test]
    fn test_cancelled_to_ready() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Cancelled,
            OrderStatus::Ready
        ));
    }

    #[test]
    fn test_cancelled_to_completed() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Cancelled,
            OrderStatus::Completed
        ));
    }

    // Test invalid backward transitions
    #[test]
    fn test_confirmed_to_pending() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Confirmed,
            OrderStatus::Pending
        ));
    }

    #[test]
    fn test_preparing_to_pending() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Preparing,
            OrderStatus::Pending
        ));
    }

    #[test]
    fn test_preparing_to_confirmed() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Preparing,
            OrderStatus::Confirmed
        ));
    }

    #[test]
    fn test_ready_to_preparing() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Ready,
            OrderStatus::Preparing
        ));
    }

    #[test]
    fn test_completed_to_ready() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Completed,
            OrderStatus::Ready
        ));
    }

    #[test]
    fn test_completed_to_preparing() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Completed,
            OrderStatus::Preparing
        ));
    }

    #[test]
    fn test_completed_to_confirmed() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Completed,
            OrderStatus::Confirmed
        ));
    }

    #[test]
    fn test_completed_to_pending() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Completed,
            OrderStatus::Pending
        ));
    }

    // Test invalid skip transitions
    #[test]
    fn test_pending_to_preparing() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Pending,
            OrderStatus::Preparing
        ));
    }

    #[test]
    fn test_pending_to_ready() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Pending,
            OrderStatus::Ready
        ));
    }

    #[test]
    fn test_pending_to_completed() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Pending,
            OrderStatus::Completed
        ));
    }

    #[test]
    fn test_confirmed_to_ready() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Confirmed,
            OrderStatus::Ready
        ));
    }

    #[test]
    fn test_confirmed_to_completed() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Confirmed,
            OrderStatus::Completed
        ));
    }

    #[test]
    fn test_preparing_to_completed() {
        assert!(!StatusMachine::is_valid_transition(
            OrderStatus::Preparing,
            OrderStatus::Completed
        ));
    }

    // Test same status transitions (no-op)
    #[test]
    fn test_same_status_pending() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Pending,
            OrderStatus::Pending
        ));
    }

    #[test]
    fn test_same_status_confirmed() {
        assert!(StatusMachine::is_valid_transition(
            OrderStatus::Confirmed,
            OrderStatus::Confirmed
        ));
    }

    // Test transition function with valid transition
    #[test]
    fn test_transition_valid() {
        let result = StatusMachine::transition(OrderStatus::Pending, OrderStatus::Confirmed);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), OrderStatus::Confirmed);
    }

    // Test transition function with invalid transition
    #[test]
    fn test_transition_invalid() {
        let result = StatusMachine::transition(OrderStatus::Pending, OrderStatus::Preparing);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid status transition"));
    }

    // Test transition function from cancelled
    #[test]
    fn test_transition_from_cancelled() {
        let result = StatusMachine::transition(OrderStatus::Cancelled, OrderStatus::Pending);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Helper to generate OrderStatus
    fn order_status_strategy() -> impl Strategy<Value = OrderStatus> {
        prop_oneof![
            Just(OrderStatus::Pending),
            Just(OrderStatus::Confirmed),
            Just(OrderStatus::Preparing),
            Just(OrderStatus::Ready),
            Just(OrderStatus::Completed),
            Just(OrderStatus::Cancelled),
        ]
    }

    /// Property 15: Valid status transitions are allowed
    /// Validates: Requirements 4.3-4.7
    /// Verifies that all valid transitions are accepted
    #[test]
    fn prop_valid_transitions_are_allowed() {
        // Define all valid transitions
        let valid_transitions = vec![
            (OrderStatus::Pending, OrderStatus::Confirmed),
            (OrderStatus::Pending, OrderStatus::Cancelled),
            (OrderStatus::Confirmed, OrderStatus::Preparing),
            (OrderStatus::Confirmed, OrderStatus::Cancelled),
            (OrderStatus::Preparing, OrderStatus::Ready),
            (OrderStatus::Preparing, OrderStatus::Cancelled),
            (OrderStatus::Ready, OrderStatus::Completed),
            (OrderStatus::Ready, OrderStatus::Cancelled),
            (OrderStatus::Completed, OrderStatus::Cancelled),
        ];

        for (from, to) in valid_transitions {
            assert!(
                StatusMachine::is_valid_transition(from, to),
                "Valid transition from {} to {} should be allowed",
                from,
                to
            );
            
            // Also test the transition function
            let result = StatusMachine::transition(from, to);
            assert!(
                result.is_ok(),
                "Transition from {} to {} should succeed",
                from,
                to
            );
            assert_eq!(result.unwrap(), to);
        }
    }

    /// Property 16: Invalid status transitions are rejected
    /// Validates: Requirements 4.8
    /// Verifies that invalid transitions are rejected
    #[test]
    fn prop_invalid_transitions_are_rejected() {
        // Define some invalid transitions
        let invalid_transitions = vec![
            // Backward transitions
            (OrderStatus::Confirmed, OrderStatus::Pending),
            (OrderStatus::Preparing, OrderStatus::Confirmed),
            (OrderStatus::Ready, OrderStatus::Preparing),
            (OrderStatus::Completed, OrderStatus::Ready),
            // Skip transitions
            (OrderStatus::Pending, OrderStatus::Preparing),
            (OrderStatus::Pending, OrderStatus::Ready),
            (OrderStatus::Pending, OrderStatus::Completed),
            (OrderStatus::Confirmed, OrderStatus::Ready),
            (OrderStatus::Confirmed, OrderStatus::Completed),
            (OrderStatus::Preparing, OrderStatus::Completed),
            // From cancelled
            (OrderStatus::Cancelled, OrderStatus::Pending),
            (OrderStatus::Cancelled, OrderStatus::Confirmed),
            (OrderStatus::Cancelled, OrderStatus::Preparing),
            (OrderStatus::Cancelled, OrderStatus::Ready),
            (OrderStatus::Cancelled, OrderStatus::Completed),
        ];

        for (from, to) in invalid_transitions {
            assert!(
                !StatusMachine::is_valid_transition(from, to),
                "Invalid transition from {} to {} should be rejected",
                from,
                to
            );
            
            // Also test the transition function
            let result = StatusMachine::transition(from, to);
            assert!(
                result.is_err(),
                "Transition from {} to {} should fail",
                from,
                to
            );
        }
    }

    /// Property: Same status transitions are always valid (idempotent)
    /// Verifies that transitioning to the same status is always allowed
    #[test]
    fn prop_same_status_is_valid() {
        proptest!(|(status in order_status_strategy())| {
            prop_assert!(
                StatusMachine::is_valid_transition(status, status),
                "Transition from {} to {} (same status) should be valid",
                status,
                status
            );
        });
    }

    /// Property: Cancelled is a terminal state
    /// Verifies that no transitions are allowed from Cancelled
    #[test]
    fn prop_cancelled_is_terminal() {
        proptest!(|(to_status in order_status_strategy())| {
            if to_status != OrderStatus::Cancelled {
                prop_assert!(
                    !StatusMachine::is_valid_transition(OrderStatus::Cancelled, to_status),
                    "No transition should be allowed from Cancelled to {}",
                    to_status
                );
            }
        });
    }

    /// Property: Cancelled can be reached from any state
    /// Verifies that any status can transition to Cancelled
    #[test]
    fn prop_can_always_cancel() {
        proptest!(|(from_status in order_status_strategy())| {
            // Exception: Cancelled to Cancelled is allowed (same status)
            // but we're testing the general rule that you can cancel from any state
            if from_status != OrderStatus::Cancelled {
                prop_assert!(
                    StatusMachine::is_valid_transition(from_status, OrderStatus::Cancelled),
                    "Transition from {} to Cancelled should always be valid",
                    from_status
                );
            }
        });
    }

    /// Property: Transition function consistency
    /// Verifies that transition() and is_valid_transition() are consistent
    #[test]
    fn prop_transition_consistency() {
        proptest!(|(
            from in order_status_strategy(),
            to in order_status_strategy()
        )| {
            let is_valid = StatusMachine::is_valid_transition(from, to);
            let transition_result = StatusMachine::transition(from, to);
            
            if is_valid {
                prop_assert!(
                    transition_result.is_ok(),
                    "If is_valid_transition returns true, transition should succeed"
                );
                prop_assert_eq!(transition_result.unwrap(), to);
            } else {
                prop_assert!(
                    transition_result.is_err(),
                    "If is_valid_transition returns false, transition should fail"
                );
            }
        });
    }
}
