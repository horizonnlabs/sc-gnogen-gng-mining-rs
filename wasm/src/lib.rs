// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           57
// Async Callback (empty):               1
// Total number of exported functions:  59

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    gng_minting
    (
        init => init
        upgrade => upgrade
        switchMode => switch_mode
        stake => stake
        battle => battle
        claimRewards => claim_rewards
        withdraw => withdraw
        getBattleStatus => get_battle_status
        isTodaySpecial => is_today_special
        isCurrentBattleSpecial => is_current_battle_special
        getAllStakedForAddress => get_all_staked_for_address
        getPendingRewardsForAddress => get_pending_rewards_for_address
        getStatsForAddress => get_stats_for_address
        getGlobalStats => get_global_stats
        getGlobalStatsByAddress => get_global_stats_by_address
        getAmountOfUsers => get_amount_of_users
        getCurrentBattle => current_battle
        getStakedForAddress => staked_for_address
        getBattleStack => battle_stack
        getRemainingNftsInBattle => remaining_nfts_in_battle
        hasBattleStarted => has_battle_started
        getTotalNftEngaged => total_nft_engaged
        getTotalBattleWinnerPower => total_battle_winner_power
        getRawPendingRewardsForAddress => raw_pending_rewards_for_address
        getAddresses => addresses
        getTotalRewardsForStakers => total_rewards_for_stakers
        getTotalRewardsForOperators => total_rewards_for_operators
        getTotalNumberClashesCurrentBattle => total_number_clashes_current_battle
        getPastBattleAmount => past_battle_amount
        setBattleToken => set_battle_token
        addExtraBattleToken => add_extra_battle_token
        removeExtraBattleToken => remove_extra_battle_token
        setAttributes => set_attributes
        pause => pause
        resume => resume
        depositGng => deposit_gng
        addAdmin => add_admin
        removeAdmin => remove_admin
        setBattleRewardAmount => set_battle_reward_amount
        setBattleOperatorRewardAmount => set_battle_operator_reward_amount
        getBattleRewardAmountWithHalving => get_battle_reward_amount_with_halving
        getBattleOperatorRewards => get_battle_operator_rewards
        getTokenAttributes => get_token_attributes
        getBattleTokens => battle_tokens
        getState => state
        getNftOwner => nft_owner
        getGngTokenId => gng_token_id
        getEmidasTokenId => emidas_token_id
        getSupremeTokenId => supreme_token_id
        getGnogonTokenId => gnogon_token_id
        getValidatorV2TokenId => validator_v2_token_id
        getDogaTokenId => doga_token_id
        getRewardCapacity => reward_capacity
        getBattleRewardAmount => battle_reward_amount
        getBattleOperatorRewardAmount => battle_operator_reward_amount
        getFirstBattleTimestamp => first_battle_timestamp
        getFirstBattleTimestampCurrentPeriod => first_battle_timestamp_current_period
        battleMode => battle_mode
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
