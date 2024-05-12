# Overview
*TODO
___
## Spawn viewer
### Description
*TODO
### Features
- [x] Show spawns for NpcId
- [x] Show spawns in selected region
- [x] Draw polygon and display in:
  - [x] Spawn format 
  - [x] Zone format 
  - [x] Custom format
- [ ] Draw multiple polygons
- [ ] Show spawn walk paths
- [ ] Map layers _(for dungeons/towers)_
- [ ] Z coord from geodata
___
## Dat Editor
### Description
*TODO
### Entities
- [x] Skill _(skillsoundsource,msconditiondata, skillname, skillgrp, skillsoundgrp )_
- [x] Npc _(npcgrp, additionalnpcgrpparts, npcname-ru, mobskillanimgrp)_
- [x] Quest _(questname)_
- [x] Recipe _(recipe)_
- [x] Items _(additionalitemgrp, itemstatdata, item_baseinfo, itemname)_
    - [x] Weapon _(weapongrp)_
    - [x] Armor _(armorgrp)_
    - [x] Etc _(etcitemgrp)_
- [x] Item Set _(setitemgrp)_
- [x] Hunting Zone _(huntingzone)_
- [ ] Weapon Enchant Effect
- [ ] Armor Enchant Effect
- [ ] Ensoul
- [ ] Instant Zone
- [ ] Daily Missions
- [x] Map Regions _(zonename, minimapregion)_
- [x] Raid Info _(raiddata)_
- [ ] Lifestone Options
### Features
- [x] .dat enc/dec, ser/de
- [x] Autosave opened tabs _(to .asave file, Bincode format)_
- [x] Import/Export for Entities _(in Ron format)_ 
- [ ] String dats editor _(npcstring, systring, etc)_
- [ ] Graph based quest step editor
- [x] Modified status for opened Entities
- [x] Delete Entity
- [x] Modified/deleted status in catalogs
- [x] In app logs 
- [x] Search history
___
### Dev TODO
- [x] Parallel save to .dat
- [ ] Parallel load from .dat
- [ ] Verbose errors
- [ ] Get rid of openssl dependency
---