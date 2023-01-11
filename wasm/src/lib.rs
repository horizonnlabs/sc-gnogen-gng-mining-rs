// Code generated by the elrond-wasm multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           42
// Async Callback (empty):               1
// Total number of exported functions:  44

#![no_std]

elrond_wasm_node::wasm_endpoints! {
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
        getStatsForNft
        getGlobalStats
        getCurrentBattle
        getStakedForAddress
        getBattleStack
        getUniqueIdBattleStack
        getTotalNftEngaged
        getBattleHistory
        getRawPendingRewardsForAddress
        getAddresses
        setBattleToken
        setAttributes
        pause
        resume
        depositGng
        addAdmin
        removeAdmin
        setDailyRewardAmount
        setBaseBattleRewardAmount
        getDailyRewardAmountWithHalving
        getTokenAttributes
        getBattleTokens
        getState
        getGngTokenId
        getEmidasTokenId
        getSupremeTokenId
        getGnogonTokenId
        getValidatorV2TokenId
        getDogaTokenId
        getRewardCapacity
        getDailyRewardAmount
        getBaseBattleRewardAmount
        getFirstBattleTimestamp
    )
}

elrond_wasm_node::wasm_empty_callback! {}
