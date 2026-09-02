# Bartend
Desktop GUI bartending app that tracks your inventory, recipes, and what potential ingredients can be used in a recipe.

Customize how you want to categorize your ingredients and build recipes off of the categories you define.

Keeps track of how much of an ingredient should be in store, within reason. Variance in use can not be practically considered.

No account, No networking, fully private. Data is stored in a single SQLite db file, making transfers easy.  

## Installation Instructions
Download from linked release, or:

## Cargo Installation
cargo install --git https://github.com/Bill-Figliolini/Bartend

## Build Instructions
git clone https://github.com/Bill-Figliolini/Bartend
cd Bartend
cargo build --release

## Config location
~/.config/Bartend/

## Default DB Location
~/.local/share/Bartend/

## Development Status
The current work as complete as far as the initial goal of solving my gripes with other recipe-tracking solutions. There are various small features that I would like for polish reasons; Theme switching being a small one, as is configuring some manner of sharing. But I would consider my time better spent taking the lessons from making this application  and applying them to something new, that does not overlook the value of making sure to write tests from the start and having a CI system to enforce that.

## Known Issues
 * Attempting to use an ingredient twice in a serving can result in it bottoming out at 0
 * Tables are not presently filterable 

## License
MIT
