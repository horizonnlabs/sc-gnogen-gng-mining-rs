One day timestamp = 86400
Initial timestamp = 5000

first battle => 5000
second battle => 5000 + 86400 = 91400
third battle => 91400 + 86400 = 182800
fourth battle => 182800 + 86400 = 274200
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