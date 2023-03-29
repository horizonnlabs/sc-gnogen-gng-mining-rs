// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           48
// Async Callback (empty):               1
// Total number of exported functions:  50

#![no_std]
#![feature(alloc_error_handler, lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    gng_minting
    (
        stake
        battle
        claimRewards
        withdraw
        getBattleStatus
        isTodaySpecial
        getAllStakedForAddress
        getPendingRewardsForAddress
        getStatsForAddress
        getGlobalStats
        getGlobalStatsByAddress
        getCurrentBattle
        getStakedForAddress
        getBattleStack
        getUniqueIdBattleStack
        hasBattleStarted
        getTotalNftEngaged
        getTotalBattleWinnerPower
        getRawPendingRewardsForAddress
        getAddresses
        getTotalRewardsForStakers
        getTotalRewardsForOperators
        setBattleToken
        addExtraBattleToken
        removeExtraBattleToken
        setAttributes
        pause
        resume
        depositGng
        addAdmin
        removeAdmin
        setBattleRewardAmount
        setBattleOperatorRewardAmount
        getBattleRewardAmountWithHalving
        getTokenAttributes
        getBattleTokens
        getState
        getNftOwner
        getGngTokenId
        getEmidasTokenId
        getSupremeTokenId
        getGnogonTokenId
        getValidatorV2TokenId
        getDogaTokenId
        getRewardCapacity
        getBattleRewardAmount
        getBattleOperatorRewardAmount
        getFirstBattleTimestamp
    )
}

multiversx_sc_wasm_adapter::empty_callback! {}
