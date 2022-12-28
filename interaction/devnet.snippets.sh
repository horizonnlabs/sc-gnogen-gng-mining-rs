PROXY="https://devnet-gateway.elrond.com"
CHAIN="D"
OWNER="wallet.pem"
CONTRACT="output/gng-minting.wasm"

SC_ADDRESS_V1="erd1qqqqqqqqqqqqqpgq3e2aclafyqmr88s9pphwtwtjdy9szng046lqkzu3ch"
SC_ADDRESS="erd1qqqqqqqqqqqqqpgqp8tdvgkt2kgpjd626ykluvqm3hzdz50s46lq673ssu"

FIRST_BATTLE_TIMESTAMP=1671813000
GNG_TOKEN_ID="str:GNG-b90f5e"

# deploy
# setBattleTokens
# setDailyRewardAmount
# setBaseBattleRewardAmount
# depositGng
# script addAttributes

deploy() {
    erdpy --verbose contract deploy --bytecode="$CONTRACT" --recall-nonce \
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
    erdpy --verbose contract upgrade ${SC_ADDRESS} --bytecode="$CONTRACT" --recall-nonce \
    --pem=${OWNER} \
    --gas-limit=599000000 \
    --proxy=${PROXY} --chain=${CHAIN} \
    --arguments $FIRST_BATTLE_TIMESTAMP $GNG_TOKEN_ID \
    --send --outfile="deploy-devnet.interaction.json" || return

    echo "Smart contract upgraded address: ${ADDRESS}"
}

addAdmin() {
    admin="erd17p8u2hhqn88nhytuy0qm2yd9e2s009tvdd7r06q09wye3azhgz0qndlxr7"

    erdpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=10000000 \
          --function="addAdmin" \
          --arguments $admin \
          --send || return
}

removeAdmin() {
    admin="erd17yva92k3twysqdf4xfw3w0q8fun2z3ltpnkqldj59297mqp9nqjs9qvkwn"

    erdpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=10000000 \
          --function="removeAdmin" \
          --arguments $admin \
          --send || return
}

setBattleToken() {
    erdpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=20000000 \
          --function="setBattleToken" \
          --arguments "str:GNOGENDUP-814470" "str:SUPREMDUP-7d5943" "str:GNOGONDUP-b3f401" "str:VTWODUP-314d2c" "str:GNOGENDUP-a1e28a" \
          --send || return
}

setDailyRewardAmount() {
    erdpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=20000000 \
          --function="setDailyRewardAmount" \
          --arguments 100000000000000000000 \
          --send || return
}

setBaseBattleRewardAmount() {
    erdpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=20000000 \
          --function="setBaseBattleRewardAmount" \
          --arguments 500000000000000000 \
          --send || return
}

setFirstBalleTimestamp() {
    erdpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=20000000 \
          --function="setFirstBalleTimeStamp" \
          --arguments 1671814200 \
          --send || return
}
