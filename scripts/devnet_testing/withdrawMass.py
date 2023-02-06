from erdpy.accounts import Account, Address
from erdpy.transactions import Transaction, BunchOfTransactions
from erdpy.proxy.core import ElrondProxy
from erdpy.contracts import SmartContract

SC_ADDRESS = 'erd1qqqqqqqqqqqqqpgqqt3yvus3er8jd2knhfxcnhfhzpjx7m8w46lqd4cncw'
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

nonces = range(250, 3250)
data_txs = []

while len(nonces) > 0:
  noncesToTransfer = nonces[:150]
  nonces = nonces[150:]
  args = list(map(lambda nonce: ['str:GNOGONDUP-b3f401', nonce], noncesToTransfer))
  flattenedArgs = [item for sublist in args for item in sublist]
  preparedArgs = SmartContract().prepare_execute_transaction_data('withdraw', flattenedArgs)
  data_txs.append(preparedArgs)

txs = BunchOfTransactions()
for data_tx in data_txs:
  tx = prepare_tx(data_tx, 600_000_000)
  txs.add_prepared(tx)
  userNonce += 1


[amount, hashes] = txs.send(proxy)
print(f"Sent {amount} transactions with hashes: {hashes}")
