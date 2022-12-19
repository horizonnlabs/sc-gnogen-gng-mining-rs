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
        getAllStakedForAddress
        getBaseBattleRewardAmount
        getBattleHistory
        getBattleStatus
        getBattleTokens
        getCurrentBattle
        getDailyRewardAmount
        getDailyRewardAmountWithHalving
        getDogaTokenId
        getEmidasTokenId
        getFirstBattleTimestamp
        getFirstStack
        getGngTokenId
        getGnogonTokenId
        getLastStakedId
        getPendingRewardsForAddress
        getRawPendingRewardsForAddress
        getRewardCapacity
        getSecondStack
        getStakedForAddress
        getState
        getStatsForAddress
        getStatsForNft
        getSupremeTokenId
        getTokenAttributes
        getTotalNftEngaged
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
