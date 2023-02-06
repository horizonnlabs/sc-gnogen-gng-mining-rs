import sys
from erdpy.accounts import Account, Address
from erdpy.transactions import Transaction, BunchOfTransactions
from erdpy.proxy.core import ElrondProxy
from erdpy.contracts import SmartContract
import requests

SC_ADDRESS = 'erd1qqqqqqqqqqqqqpgqfmrqd9cw95a7reeaxkuwp9ue6hq2xzkv46lqudcwrx'
ACCOUNT = 'erd12fyncyqdnj3vslakxq08cslemt7jdg2t6tnchwgq5dnnes996qdqrl5ft2'
sc_address_hex = Address(SC_ADDRESS).hex()
proxy = ElrondProxy('https://devnet-gateway.elrond.com')
network = proxy.get_network_config()
user = Account(pem_file='user.pem')
user.sync_nonce(proxy)
userNonce = user.nonce

def get_tokens_list(token_id):
  response = requests.get(f'https://devnet-api.multiversx.com/accounts/{ACCOUNT}/collections/{token_id}?size=5000').json()
  tokens = [] #[[token['collection'], token['nonce']] for token in response]
  print(response)
  return tokens

def prepare_tx(args, gas_limit = 20_000_000):
  tx = Transaction()
  tx.nonce = userNonce
  tx.value = "0"
  tx.sender = user.address.bech32()
  tx.receiver = user.address.bech32()
  tx.gasPrice = network.min_gas_price
  tx.gasLimit = gas_limit
  tx.data = args
  tx.chainID = network.chain_id
  tx.version = network.min_tx_version

  tx.sign(user)

  return tx

# tokens = get_tokens_list('GNOGONDUP-b3f401')

#MultiESDTNFTTransfer

nonces = range(250, 3250)
data_txs = []

while len(nonces) > 0:
  noncesToTransfer = nonces[:150]
  nonces = nonces[150:]
  payments = list(map(lambda nonce: ['str:GNOGONDUP-b3f401', nonce, 1], noncesToTransfer))
  flattened_payments = [item for sublist in payments for item in sublist]
  args = [SC_ADDRESS, 150] + flattened_payments + ["str:stake"]
  preparedArgs = SmartContract().prepare_execute_transaction_data('MultiESDTNFTTransfer', args)
  data_txs.append(preparedArgs)

txs = BunchOfTransactions()
for data_tx in data_txs:
  tx = prepare_tx(data_tx, 600_000_000)
  txs.add_prepared(tx)
  userNonce += 1


[amount, hashes] = txs.send(proxy)
print(f"Sent {amount} transactions with hashes: {hashes}")
