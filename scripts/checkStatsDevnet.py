from erdpy.contracts import SmartContract
from erdpy.proxy.core import ElrondProxy
from erdpy.accounts import Address
import json

def decodeStack(encodedNfts):
  result = []
  for elemHex in encodedNfts:
    nameLenght = int(elemHex[0:8], 16) * 2
    nameHex = elemHex[8:8+nameLenght]
    name = bytes.fromhex(nameHex).decode('utf-8')
    nonce = int(elemHex[8+nameLenght:8+nameLenght+16], 16)
    result.append('str:' + name)
    result.append(nonce)
  return result

NEXT_BATTLE_ID = 2
scAddress = Address('erd1qqqqqqqqqqqqqpgqnv9zr6wxyqlfax06zap2k4vkwnqckwww46lqw84ayu')
proxy = ElrondProxy('https://devnet-gateway.elrond.com')

sc = SmartContract(scAddress)

stakedNfts = []

data = sc.query(proxy, "getFirstStack", [NEXT_BATTLE_ID])
firstStackHex = list(map(lambda x: x.hex, data))
stakedNfts.extend(decodeStack(firstStackHex))

data = sc.query(proxy, "getSecondStack", [NEXT_BATTLE_ID])
secondStackHex = list(map(lambda x: x.hex, data))
stakedNfts.extend(decodeStack(secondStackHex))

data = sc.query(proxy, "getStatsForNft", stakedNfts)

stats = []

cursor = 1
currentNft = {}
for elem in data:
  match cursor:
    case 1:
      currentNft['collection'] = bytes.fromhex(elem.hex).decode('utf-8')
      cursor += 1
    case 2:
      currentNft['nonce'] = elem.number
      cursor += 1
    case 3:
      encoded = elem.hex
      nftStats = {}
      nftStats['win'] = int(encoded[0:16], 16)
      nftStats['loss'] = int(encoded[16:32], 16)
      nftStats['owner'] = Address(encoded[32:]).bech32()
      currentNft['stats'] = nftStats
      cursor = 1
      stats.append(currentNft)
      currentNft = {}

for token in stats:
  data = sc.query(proxy, "getTokenAttributes", ['str:' + token['collection'], token['nonce']])[0].hex
  attributes = {}
  attributes['power'] = int(data[0:4], 16)
  attributes['heart'] = int(data[4:8], 16)
  attributes['ram'] = int(data[8:16], 16)
  token['attributes'] = attributes

stats.sort(key=lambda x: x['stats']['win'], reverse=True)

with open('scripts/stats.json', 'w') as outfile:
  json.dump(stats, outfile, indent=2)