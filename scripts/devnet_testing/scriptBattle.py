from erdpy.accounts import Account, Address
from erdpy.transactions import Transaction, BunchOfTransactions
from erdpy.proxy.core import ElrondProxy

AMOUNT_OF_TX = 30
SC_ADDRESS = 'erd1qqqqqqqqqqqqqpgqfmrqd9cw95a7reeaxkuwp9ue6hq2xzkv46lqudcwrx'
proxy = ElrondProxy('https://devnet-gateway.elrond.com')
network = proxy.get_network_config()
user = Account(pem_file='user.pem')
user.sync_nonce(proxy)
userNonce = user.nonce

def prepare_tx(args, gas_limit = 20_000_000):
  tx = Transaction()
  tx.nonce = userNonce
  tx.value = "0"
  tx.sender = user.address.bech32()
  tx.receiver = SC_ADDRESS
  tx.gasPrice = network.min_gas_price
  tx.gasLimit = gas_limit
  tx.data = args
  tx.chainID = network.chain_id
  tx.version = network.min_tx_version

  tx.sign(user)

  return tx

txs = BunchOfTransactions()
for i in range(AMOUNT_OF_TX):
  tx = prepare_tx("battle", 600_000_000)
  txs.add_prepared(tx)
  userNonce += 1


[amount, hashes] = txs.send(proxy)
print(f"Sent {amount} transactions with hashes: {hashes}")
