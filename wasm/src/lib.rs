////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    gng_minting
    (
        addAdmin
        battle
        claimRewards
        depositGng
        getAddresses
        getAllStakedForAddress
        getBaseBattleRewardAmount
        getBattleHistory
        getBattleStack
        getBattleStatus
        getBattleTokens
        getCurrentBattle
        getDailyRewardAmount
        getDailyRewardAmountWithHalving
        getDogaTokenId
        getEmidasTokenId
        getFirstBattleTimestamp
        getGlobalStats
        getGngTokenId
        getGnogonTokenId
        getPendingRewardsForAddress
        getRawPendingRewardsForAddress
        getRewardCapacity
        getStakedForAddress
        getState
        getStatsForAddress
        getStatsForNft
        getSupremeTokenId
        getTokenAttributes
        getTotalNftEngaged
        getUniqueIdBattleStack
        getValidatorV2TokenId
        isTodayASunday
        pause
        removeAdmin
        resume
        setAttributes
        setBaseBattleRewardAmount
        setBattleToken
        setDailyRewardAmount
        stake
        withdraw
    )
}

elrond_wasm_node::wasm_empty_callback! {}
