import csv
import time
from erdpy.accounts import Address, Account
from erdpy.contracts import SmartContract
from erdpy.proxy.core import ElrondProxy
from erdpy.transactions import Transaction

AMOUNT_IN_ONE_TX = 300
SC_ADDRESS='erd1qqqqqqqqqqqqqpgqp8tdvgkt2kgpjd626ykluvqm3hzdz50s46lq673ssu'
proxy = ElrondProxy('https://devnet-gateway.elrond.com')
network = proxy.get_network_config()
user = Account(pem_file='wallet.pem')
user.sync_nonce(proxy)
userNonce = user.nonce

def send_tx(args, gas_limit = 20_000_000):
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
  tx_hash = tx.send(proxy)

  return tx_hash


all_metadatas = []

def fromStringIdToTokenIdAndNonce(id):
  parts = id.split('-')
  return (f'{parts[0]}-{parts[1]}', int(parts[2], 16))

with open('./scripts/data_devnet/emidas.csv') as csvfile:
  reader = csv.DictReader(csvfile)
  for row in reader:
    tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
    power = int(row['ATTRIBUTE:POWER'])
    all_metadatas.append([f'str:{tokenId}', nonce, power, 0, 0])

with open('./scripts/data_devnet/gnogons.csv') as csvfile:
  reader = csv.DictReader(csvfile)
  for row in reader:
    tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
    power = int(row['ATTRIBUTE:POWER'])
    heart = int(row['ATTRIBUTE:HEART'])
    all_metadatas.append([f'str:{tokenId}', nonce, power, heart, 0])

with open('./scripts/data_devnet/validators.csv') as csvfile:
  reader = csv.DictReader(csvfile)
  for row in reader:
    tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
    power = int(row['ATTRIBUTE:POWER'])
    ram = int(row['ATTRIBUTE:RAM'])
    all_metadatas.append([f'str:{tokenId}', nonce, power, 0, ram])

with open('./scripts/data_devnet/doga.csv') as csvfile:
  reader = csv.DictReader(csvfile)
  for row in reader:
    tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
    power = int(row['ATTRIBUTE:POWER'])
    all_metadatas.append([f'str:{tokenId}', nonce, power, 0, 0])

with open('./scripts/data_devnet/supreme.csv') as csvfile:
  reader = csv.DictReader(csvfile)
  for row in reader:
    tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
    power = int(row['ATTRIBUTE:POWER'])
    all_metadatas.append([f'str:{tokenId}', nonce, power, 0, 0])

amountSent = 0
totalToSend = len(all_metadatas)

while len(all_metadatas) > 0:
  currentBatch = all_metadatas[:AMOUNT_IN_ONE_TX]
  currentFlattenBatch = [item for sublist in currentBatch for item in sublist]

  args = SmartContract().prepare_execute_transaction_data("setAttributes", currentFlattenBatch)
  all_metadatas = all_metadatas[AMOUNT_IN_ONE_TX:]
  tx_hash = send_tx(args, gas_limit = 600_000_000)
  amountSent += AMOUNT_IN_ONE_TX
  print(f"Sent {amountSent}/{totalToSend}. Tx hash: {tx_hash}")
  time.sleep(0.2)
  userNonce += 1

