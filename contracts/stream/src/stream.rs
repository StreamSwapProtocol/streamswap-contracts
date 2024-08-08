use cosmwasm_std::{Decimal, Decimal256, Fraction, Timestamp, Uint256};
use std::ops::Mul;
use streamswap_types::stream::{Status, Stream};

pub fn sync_stream_status(stream: &mut Stream, now: Timestamp) {
    if matches!(
        stream.status_info.status,
        Status::Finalized | Status::Cancelled
    ) {
        return;
    }
    stream.status_info.status = match now {
        _ if now < stream.status_info.bootstrapping_start_time => Status::Waiting,
        _ if now >= stream.status_info.bootstrapping_start_time
            && now < stream.status_info.start_time =>
        {
            Status::Bootstrapping
        }
        _ if now >= stream.status_info.start_time && now < stream.status_info.end_time => {
            Status::Active
        }
        _ if now >= stream.status_info.end_time => Status::Ended,
        _ => stream.status_info.status.clone(),
    };
}

pub fn compute_shares_amount(stream: &Stream, amount_in: Uint256, round_up: bool) -> Uint256 {
    if stream.shares.is_zero() || amount_in.is_zero() {
        return amount_in;
    }
    let shares = stream.shares.mul(amount_in);
    if round_up {
        (shares + stream.in_supply - Uint256::one()) / stream.in_supply
    } else {
        shares / stream.in_supply
    }
}
pub fn update_stream(stream: &mut Stream, now: Timestamp) {
    let diff = calculate_diff(
        stream.status_info.start_time,
        stream.status_info.end_time,
        stream.status_info.last_updated,
        now,
    );

    if !stream.shares.is_zero() && !diff.is_zero() {
        let new_distribution_balance = stream
            .out_remaining
            .multiply_ratio(diff.numerator(), diff.denominator());
        let spent_in = stream
            .in_supply
            .multiply_ratio(diff.numerator(), diff.denominator());

        stream.spent_in += spent_in;
        stream.in_supply -= spent_in;

        if !new_distribution_balance.is_zero() {
            stream.out_remaining -= new_distribution_balance;
            stream.dist_index += Decimal256::from_ratio(new_distribution_balance, stream.shares);
            stream.current_streamed_price =
                Decimal256::from_ratio(spent_in, new_distribution_balance);
        }
    }

    stream.status_info.last_updated = now;
}

fn calculate_diff(
    start_time: Timestamp,
    end_time: Timestamp,
    mut last_updated: Timestamp,
    now: Timestamp,
) -> Decimal {
    // If the stream is not started yet or already ended, return 0
    if now < start_time || last_updated >= end_time {
        return Decimal::zero();
    }
    // If we are here, the stream is active. If the last update time is before the start time,
    // This means stream is updated before start time, in order to calculate the diff, we should
    // set the last updated time to start time.
    // ---Waiting---|---Bootstrapping-(last_updated)--|----(now)--Active---|---Ended---|--Finalized--|
    //              |              Not include here=--|----=We should be updating here
    if last_updated < start_time {
        last_updated = start_time;
    }
    // If the now is greater than end time, we should set the now to end time.
    // ---Waiting---|---Bootstrapping---|---Active----(last updated)---|-----(now)--Ended---|--Finalized--|
    //              |                   |     We should update here=---|-----=Not here
    // That is why we are taking the minimum of now and end time.
    let now = if now > end_time { end_time } else { now };

    let numerator = now.nanos().saturating_sub(last_updated.nanos());
    let denominator = end_time.nanos().saturating_sub(last_updated.nanos());

    if denominator == 0 || numerator == 0 {
        Decimal::zero()
    } else {
        Decimal::from_ratio(numerator, denominator)
    }
}
