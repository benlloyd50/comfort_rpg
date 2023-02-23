# Planning
## Goals
- create a cozy rpg game
- not very story driven more so environment driven
- explore places like caves, forests, towns, and cities

## Design Specs
- Grid based movement (probably not gonna worry about subpixel movement)
- 8x8 sprites (exceptions on epic stuff like multi block sprites)
- ECS (bevy most likely)

## Gameplay Systems (what's needed)
- Inventory
- Tilemap
- Building
- Movement
- Monster AI

## Features (what's fun?)
- exploration in a procedurally generated environment
- destructable terrain
- building
- inspectable text on random things (almost everything)

## Prototype
- [X] Generate World
- [X] Move Player around world
- [X] Gather resources
- [X] Craft resources into deployables
- [ ] Build with the deployables

## Inventory
- will probably need some more safety features like enforcing size when adding items

## Crafting System
- At it's root we are checking to see if a player has the items needed, removing them, and giving them a new one
- Define recipes in a json similar to how items are defined
  - Recipes will probably have an id, components, and amt given
- Need some sort of menu, probably similar to the inventory menu
  - Player will attempt to craft an item by navigating the menu and then some fn will run

## Ideas
- A flowing ocean with animated tiles?
- Fishing
- Mining ideas, multi block hit so you can hit multiple blocks
- Certain terrain could require upgraded materials
- Ascii Graphics *Toggable*
