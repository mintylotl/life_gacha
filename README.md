### Work in Progress.

## LifeGacha
LifeGacha is essentially just turning your rest and rewards in life from a certainty into a gamble.
You have to gamble for the chance of Mythic, S, A or B rank drops which each give unique rewards or currency
that can be used in the store.

## Purpose
Just made to make the grind less boring.
Mundane tasks are wrapped in a layer of irresistable pulling. "Just one more pull..."

## Layout
#### gacha_protocol
a crate / library which contains the core "slot machine" logic and droprates.
acts as the core library for the project's gacha logic.
has relevant datatypes inside.

#### gacha
the binary which orchestrates the entire program. It handles user persistence with rusqlite, serves json over axum for the UI,
and overall is just the central piece of the whole program. Runs as a daemon, waits for requests from UI and responds / serves
requests. Also handles earning of currency, pull logic etc. It calls gacha_protocol for the rates.

#### gacha_ui
a Combination of vite, tailwind and tauri. The frontend is jsut that, a frontend for the game.
