PROXY="https://devnet-gateway.elrond.com"
CHAIN="D"
OWNER="gnogen_collections.pem"
CONTRACT="output/gng-minting.wasm"

SC_ADDRESS="erd1qqqqqqqqqqqqqpgqunfdvkfvux3025m9kzsx6e7n5peg07lmm8qsj6sshf"

FIRST_BATTLE_TIMESTAMP=1675368000
GNG_TOKEN_ID="str:XGNG-04bd9e"

# deploy
# setBattleTokens
# setBattleRewardAmount (GNG amount to distribute for one battle)
# setBattleOperatorRewardAmount (GNG amount to distribute for one battle for operators)
# depositGng (add admin)
# script addAttributes
# resume

# For reward operators
#    2M GNG during 10 years
#    547.945205479 GNG to distribute for 1 day (amount to set in setBattleOperatorRewardAmount and to not forget decimals)
#    -> 547945205479000000000

# For reward stakers
#     388,500,000 GNG the first year
#     1064383.56164 GNG to distribute for 1 day (amount to set in setBattleRewardAmount and to not forget decimals)
#     -> 1064383561640000000000000



deploy() {
    mxpy --verbose contract deploy --bytecode="$CONTRACT" --recall-nonce \
        --pem=$OWNER \
        --gas-limit=599000000 \
        --arguments $FIRST_BATTLE_TIMESTAMP $GNG_TOKEN_ID \
        --proxy=$PROXY --chain=$CHAIN \
        --outfile="deploy-devnet.interaction.json" --send || return

    TRANSACTION=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['contractAddress']")

    erdpy data store --key=address-devnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-devnet --value=${TRANSACTION}

    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    mxpy --verbose contract upgrade ${SC_ADDRESS} --bytecode="$CONTRACT" --recall-nonce \
    --pem=${OWNER} \
    --gas-limit=599000000 \
    --proxy=${PROXY} --chain=${CHAIN} \
    --arguments $FIRST_BATTLE_TIMESTAMP $GNG_TOKEN_ID \
    --send --outfile="deploy-devnet.interaction.json" || return

    echo "Smart contract upgraded address: ${ADDRESS}"
}

addAdmin() {
    admin="erd1hfz5y8k3htdl56f7wpeu5xpzax02p77av8s84yu0khtela03m8qs8gp5zd"

    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=10000000 \
          --function="addAdmin" \
          --arguments $admin \
          --send || return
}

removeAdmin() {
    admin="erd17p8u2hhqn88nhytuy0qm2yd9e2s009tvdd7r06q09wye3azhgz0qndlxr7"

    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=10000000 \
          --function="removeAdmin" \
          --arguments $admin \
          --send || return
}

setBattleToken() {
    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=20000000 \
          --function="setBattleToken" \
          --arguments "str:XEMIDAS-7966ea" "str:XSUPREMES-40d4d0" "str:XGNOGONS-0a6d4c" "str:XVTWO-fcc831" "str:XDOGA-9b8ee8" \
          --send || return
}

setBattleRewardAmount() {
    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=20000000 \
          --function="setBattleRewardAmount" \
          --arguments 1064383561640000000000000 \
          --send || return
}

setBattleOperatorRewardAmount() {
    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=20000000 \
          --function="setBattleOperatorRewardAmount" \
          --arguments 547945205479000000000 \
          --send || return
}

setFirstBattleTimestamp() {
    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=20000000 \
          --function="setTimestamp" \
          --arguments 1673512034 \
          --send || return
}
