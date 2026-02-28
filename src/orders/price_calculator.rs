use rust_decimal::Decimal;

/// Service for calculating order prices and subtotals
pub struct PriceCalculator;

impl PriceCalculator {
    /// Calculate subtotal for an order item
    /// 
    /// # Arguments
    /// * `quantity` - Number of items ordered
    /// * `price_snapshot` - Price per item at time of order
    /// 
    /// # Returns
    /// Subtotal as Decimal (quantity * price_snapshot)
    pub fn calculate_subtotal(quantity: i32, price_snapshot: Decimal) -> Decimal {
        Decimal::from(quantity) * price_snapshot
    }

    /// Calculate total price for an order
    /// 
    /// # Arguments
    /// * `subtotals` - Slice of subtotals for all order items
    /// 
    /// # Returns
    /// Total price as Decimal (sum of all subtotals)
    pub fn calculate_total(subtotals: &[Decimal]) -> Decimal {
        subtotals.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_subtotal_basic() {
        let quantity = 2;
        let price = dec!(4.50);
        let subtotal = PriceCalculator::calculate_subtotal(quantity, price);
        assert_eq!(subtotal, dec!(9.00));
    }

    #[test]
    fn test_calculate_subtotal_single_item() {
        let quantity = 1;
        let price = dec!(3.75);
        let subtotal = PriceCalculator::calculate_subtotal(quantity, price);
        assert_eq!(subtotal, dec!(3.75));
    }

    #[test]
    fn test_calculate_subtotal_large_quantity() {
        let quantity = 100;
        let price = dec!(2.50);
        let subtotal = PriceCalculator::calculate_subtotal(quantity, price);
        assert_eq!(subtotal, dec!(250.00));
    }

    #[test]
    fn test_calculate_total_single_item() {
        let subtotals = vec![dec!(10.00)];
        let total = PriceCalculator::calculate_total(&subtotals);
        assert_eq!(total, dec!(10.00));
    }

    #[test]
    fn test_calculate_total_multiple_items() {
        let subtotals = vec![dec!(10.00), dec!(5.50), dec!(3.25)];
        let total = PriceCalculator::calculate_total(&subtotals);
        assert_eq!(total, dec!(18.75));
    }

    #[test]
    fn test_calculate_total_empty() {
        let subtotals: Vec<Decimal> = vec![];
        let total = PriceCalculator::calculate_total(&subtotals);
        assert_eq!(total, dec!(0.00));
    }

    #[test]
    fn test_decimal_precision() {
        let quantity = 3;
        let price = dec!(4.33);
        let subtotal = PriceCalculator::calculate_subtotal(quantity, price);
        assert_eq!(subtotal, dec!(12.99));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use rust_decimal::Decimal;

    /// Property 9: Subtotal calculation invariant
    /// Validates: Requirements 2.2
    /// Verifies that subtotal = quantity * price for all valid inputs
    #[test]
    fn prop_subtotal_calculation_invariant() {
        proptest!(|(
            quantity in 1i32..=1000,
            price_cents in 1u32..=10000u32
        )| {
            let price = Decimal::from(price_cents) / Decimal::from(100);
            let subtotal = PriceCalculator::calculate_subtotal(quantity, price);
            let expected = Decimal::from(quantity) * price;
            prop_assert_eq!(subtotal, expected);
        });
    }

    /// Property 12: Total price calculation invariant
    /// Validates: Requirements 3.1
    /// Verifies that total = sum of subtotals for all valid inputs
    #[test]
    fn prop_total_calculation_invariant() {
        proptest!(|(
            subtotals_cents in prop::collection::vec(1u32..=100000u32, 1..=20)
        )| {
            let subtotals: Vec<Decimal> = subtotals_cents
                .iter()
                .map(|&cents| Decimal::from(cents) / Decimal::from(100))
                .collect();
            
            let total = PriceCalculator::calculate_total(&subtotals);
            let expected: Decimal = subtotals.iter().sum();
            
            prop_assert_eq!(total, expected);
        });
    }

    /// Property 21: Order totals are non-negative
    /// Validates: Requirements 6.5
    /// Verifies that calculated totals are always >= 0
    #[test]
    fn prop_totals_are_non_negative() {
        proptest!(|(
            quantities in prop::collection::vec(1i32..=100, 1..=10),
            prices_cents in prop::collection::vec(1u32..=10000u32, 1..=10)
        )| {
            // Ensure we have matching quantities and prices
            let count = quantities.len().min(prices_cents.len());
            let quantities = &quantities[..count];
            let prices_cents = &prices_cents[..count];
            
            let subtotals: Vec<Decimal> = quantities
                .iter()
                .zip(prices_cents.iter())
                .map(|(&qty, &price_cents)| {
                    let price = Decimal::from(price_cents) / Decimal::from(100);
                    PriceCalculator::calculate_subtotal(qty, price)
                })
                .collect();
            
            let total = PriceCalculator::calculate_total(&subtotals);
            
            prop_assert!(total >= Decimal::ZERO, "Total must be non-negative, got: {}", total);
        });
    }

    /// Additional property: Subtotals are non-negative
    /// Verifies that individual subtotals are always >= 0
    #[test]
    fn prop_subtotals_are_non_negative() {
        proptest!(|(
            quantity in 1i32..=1000,
            price_cents in 1u32..=10000u32
        )| {
            let price = Decimal::from(price_cents) / Decimal::from(100);
            let subtotal = PriceCalculator::calculate_subtotal(quantity, price);
            
            prop_assert!(subtotal >= Decimal::ZERO, "Subtotal must be non-negative, got: {}", subtotal);
        });
    }

    /// Additional property: Total with single item equals that item's subtotal
    /// Verifies consistency between subtotal and total calculations
    #[test]
    fn prop_single_item_total_equals_subtotal() {
        proptest!(|(
            quantity in 1i32..=100,
            price_cents in 1u32..=10000u32
        )| {
            let price = Decimal::from(price_cents) / Decimal::from(100);
            let subtotal = PriceCalculator::calculate_subtotal(quantity, price);
            let total = PriceCalculator::calculate_total(&[subtotal]);
            
            prop_assert_eq!(total, subtotal);
        });
    }

    /// Additional property: Order of subtotals doesn't affect total (commutative)
    /// Verifies that addition is commutative
    #[test]
    fn prop_total_is_commutative() {
        proptest!(|(
            subtotals_cents in prop::collection::vec(1u32..=10000u32, 2..=10)
        )| {
            let subtotals: Vec<Decimal> = subtotals_cents
                .iter()
                .map(|&cents| Decimal::from(cents) / Decimal::from(100))
                .collect();
            
            let total1 = PriceCalculator::calculate_total(&subtotals);
            
            // Reverse the order
            let mut reversed = subtotals.clone();
            reversed.reverse();
            let total2 = PriceCalculator::calculate_total(&reversed);
            
            prop_assert_eq!(total1, total2, "Total should be same regardless of order");
        });
    }
}
