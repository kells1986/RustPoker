- [x] Polish Game Logic

Make sure that the game logic is correct and extensible to multiplayer games, covering all possible scenarios with and variations of the games that might happen.

Deal with corner cases like Heads up games, splitting pots at show down, handling all-ins, etc.

This should be built on top of a solid foundation of unit tests.

Once unit tests are in place, documentation should be updated in [AGENTS.md](AGENTS.md) with details about how to run the tests.

Any changes to the game logic and underlying logic should be reflected in the [ARCHITECTURE.md](ARCHITECTURE.md) file.

- [x] Create an AGENTS.md file in src

Document the logic that's currently in place in the various files. Explain the basic structure and workflows. Make it easy for an AI Agent to understand the codebase and pick up where it left off in a new context window


- [x] Add Simple Strategy Module

Allow agent players to be added to the game. We should be able to specify a number of seats at the table, the table stakes, big blind, small blind, initial stack size of each player, etc and either have all players play against each other, or have a single human player play against a number of agents.
