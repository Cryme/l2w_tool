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
- [ ] Hunting Zone
- [ ] Weapon Enchant Effect
- [ ] Armor Enchant Effect
- [ ] Ensoul
- [ ] Instant Zone
- [ ] Daily Missions
- [ ] Map
- [ ] Minimap
- [ ] Lifestone Options
### Features
- [x] .dat ser/de
- [x] Autosave opened tabs
- [x] Clipboard Copy/Paste for Entities _(Ron)_ 
- [ ] String dats editor _(npcstring, systring, etc)_
- [ ] Graph based quest step editor
- [ ] Tab modified status
- [x] In app logs 
___
### Dev TODO
- [x] Parallel save to .dat
- [ ] Parallel load from .dat
- [ ] Verbose errors
- [ ] Get rid of openssl dependency
---