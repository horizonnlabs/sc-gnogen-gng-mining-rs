import base64
import csv
import json

import requests

EMIDAS_TOKEN_ID = 'XEMIDAS-7966ea'
GNOGON_TOKEN_ID = 'XGNOGONS-0a6d4c'
VALIDATOR_TOKEN_ID = 'XVTWO-fcc831'
SC_ADDRESS = 'erd1qqqqqqqqqqqqqpgqunfdvkfvux3025m9kzsx6e7n5peg07lmm8qsj6sshf'
TX_AMOUNT = 5

def fromStringIdToTokenIdAndNonce(id):
  parts = id.split('-')
  return (f'{parts[0]}-{parts[1]}', int(parts[2], 16))

def decodeTokenStruct(base64Token):
  hexToken = base64.b64decode(base64Token).hex()
  tokenIdLen = int(hexToken[0:8], 16)
  hexTokenId = hexToken[8:8 + tokenIdLen * 2]
  tokenId = bytes.fromhex(hexTokenId).decode('utf-8')
  nonce = int(hexToken[8 + tokenIdLen * 2:8 + tokenIdLen * 2 + 16], 16)
  return {'tokenId': tokenId, 'nonce': nonce}

def base64ToString(base64String):
  return base64.b64decode(base64String).decode('utf-8')

def base64ToInt(base64String):
  return int(base64.b64decode(base64String).hex(), 16)

def loadAttributes():
  attributes = []
  with open('./scripts/data_devnet/emidas.csv') as csvfile:
    reader = csv.DictReader(csvfile)
    for row in reader:
      tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
      power = int(row['ATTRIBUTE:POWER'])
      attributes.append({'tokenId': tokenId, 'nonce': nonce, 'power': power, 'heart': 0, 'ram': 0})

  with open('./scripts/data_devnet/gnogons.csv') as csvfile:
    reader = csv.DictReader(csvfile)
    for row in reader:
      tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
      power = int(row['ATTRIBUTE:POWER'])
      heart = int(row['ATTRIBUTE:HEART'])
      attributes.append({'tokenId': tokenId, 'nonce': nonce, 'power': power, 'heart': heart, 'ram': 0})

  with open('./scripts/data_devnet/validators.csv') as csvfile:
    reader = csv.DictReader(csvfile)
    for row in reader:
      tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
      power = int(row['ATTRIBUTE:POWER'])
      ram = int(row['ATTRIBUTE:RAM'])
      attributes.append({'tokenId': tokenId, 'nonce': nonce, 'power': power, 'heart': 0, 'ram': ram})

  with open('./scripts/data_devnet/doga.csv') as csvfile:
    reader = csv.DictReader(csvfile)
    for row in reader:
      tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
      power = int(row['ATTRIBUTE:POWER'])
      attributes.append({'tokenId': tokenId, 'nonce': nonce, 'power': power, 'heart': 0, 'ram': 0})

  with open('./scripts/data_devnet/supreme.csv') as csvfile:
    reader = csv.DictReader(csvfile)
    for row in reader:
      tokenId, nonce = fromStringIdToTokenIdAndNonce(row['TOKEN'])
      power = int(row['ATTRIBUTE:POWER'])
      attributes.append({'tokenId': tokenId, 'nonce': nonce, 'power': power, 'heart': 0, 'ram': 0})

  return attributes

def loadEvents(scAddress, txAmount):
  clashes = []
  url = f'https://devnet-api.multiversx.com/transactions?size={txAmount}&receiver={scAddress}&function=battle&withLogs=true&status=success'
  response = requests.get(url).json()
  for tx in response:
    events = tx['logs']['events']
    for event in events:
      if event['identifier'] != 'battle':
        continue
      topics = event['topics']
      eventType = base64ToString(topics[0])
      if eventType != 'clash':
        print(eventType)
        continue
      battleId = base64ToInt(topics[1])
      winner = decodeTokenStruct(topics[2])
      loser = decodeTokenStruct(topics[3])
      isDraw = topics[4] != ''
      clashes.append({'winner': winner, 'loser': loser, 'isDraw': isDraw, 'battleId': battleId})
  return clashes

def checkClashResult(firstTokenAttributes, secondTokenAttributes, isDraw):
  if firstTokenAttributes['power'] > secondTokenAttributes['power']:
    return not isDraw
  elif firstTokenAttributes['power'] < secondTokenAttributes['power']:
    return False
  else:
    if firstTokenAttributes['tokenId'] == EMIDAS_TOKEN_ID and secondTokenAttributes['tokenId'] != EMIDAS_TOKEN_ID:
      return not isDraw
    if firstTokenAttributes['tokenId'] != EMIDAS_TOKEN_ID and secondTokenAttributes['tokenId'] == EMIDAS_TOKEN_ID:
      return False
    if firstTokenAttributes['tokenId'] == GNOGON_TOKEN_ID and secondTokenAttributes['tokenId'] == VALIDATOR_TOKEN_ID:
      return not isDraw
    if firstTokenAttributes['tokenId'] == VALIDATOR_TOKEN_ID and secondTokenAttributes['tokenId'] == GNOGON_TOKEN_ID:
      return False
    if firstTokenAttributes['tokenId'] == GNOGON_TOKEN_ID and secondTokenAttributes['tokenId'] == GNOGON_TOKEN_ID:
      if firstTokenAttributes['heart'] > secondTokenAttributes['heart']:
        return not isDraw
      if firstTokenAttributes['heart'] < secondTokenAttributes['heart']:
        return False
    if firstTokenAttributes['tokenId'] == VALIDATOR_TOKEN_ID and secondTokenAttributes['tokenId'] == VALIDATOR_TOKEN_ID:
      if firstTokenAttributes['ram'] > secondTokenAttributes['ram']:
        return not isDraw
      if firstTokenAttributes['ram'] < secondTokenAttributes['ram']:
        return False
  return isDraw


attributes = loadAttributes()
events = loadEvents(SC_ADDRESS, TX_AMOUNT)

logs = []
errors = []
tiebreakedAmount = 0
for event in events:
  winner = event['winner']
  loser = event['loser']
  isDraw = event['isDraw']
  winnerAttributes = next((x for x in attributes if x['tokenId'] == winner['tokenId'] and x['nonce'] == winner['nonce']), None)
  loserAttributes = next((x for x in attributes if x['tokenId'] == loser['tokenId'] and x['nonce'] == loser['nonce']), None)
  if winnerAttributes is None or loserAttributes is None:
    print('Error: attributes not found')
    continue
  log = {'winner': winnerAttributes, 'loser': loserAttributes, 'isDraw': isDraw}
  if winnerAttributes['power'] == loserAttributes['power']:
    tiebreakedAmount += 1
    print(f'Winner: {winnerAttributes}')
    print(f'Loser: {loserAttributes}')
    print()
  if checkClashResult(winnerAttributes, loserAttributes, isDraw) == True:
    log['isCorrect'] = True
  else:
    log['isCorrect'] = False
    errors.append(log)
  logs.append(log)

if len(errors) != 0:
  print('Errors found')

with open('./scripts/eventsLogs/logs.json', 'w') as outfile:
  json.dump(logs, outfile)
with open('./scripts/eventsLogs/errors.json', 'w') as outfile:
  json.dump(errors, outfile)

print(f'Total clashes: {len(events)}')
print(f'Total tiebreaked: {tiebreakedAmount}')