One day timestamp = 86400
Initial timestamp = 5000

first battle => 5000
second battle => 5000 + 86400 = 91400
third battle => 91400 + 86400 = 182800
fourth battle => 182800 + 86400 = 274200 (Sunday)
and so on...

test-status.scen.json

    set 5100
      -> Status = Battle
    Battle 1
      -> Status = Preparation
    set 91400
      -> Status = Battle
    Battle 2
      -> Status = Preparation
    set 274200 ()
      -> Status = Battle
    Battle 3
      -> Status = Battle (we already reached the timestamp of the next battle,
      so there is no preparation phase but the battle starts directly)
    Battle 4
      -> Status = Preparation


first battle
  270 total power winner (200 + 70)
  daily rewards 10 000
    user 1 wins 200 power
      200/270 * 10 000 = 7407
    user 3 wins 70 power
      70/270 * 10 000 = 2592
    user 4 wins 0 power
  7407 + 2592 = 9999 (not 10 000 because of integer division)


Battle 1
  user1 -> 7407
  user3 -> 2592

Claim rewards
  user1 -> 7407

Battle 2
  user3 -> 4117 => total 6709
  user4 -> 5882

Claim rewards
  user3 -> 6709

Battle 3 (old rewards)
  user1 -> 5324
  user2 -> 887
  user3 -> 2839
  user4 -> 946 => total 6828
Battle 3 (new rewards)
  user -> 5389
  user2 -> 1736
  user3 -> 718
  user4 -> 2155 => total 8037
